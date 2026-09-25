import { env } from "@/lib/env";
import type { ChannelName, ConnectionStatus, SocketListener, SocketMessage } from "./types";

const MAX_RECONNECT_DELAY_MS = 15_000;
const BASE_RECONNECT_DELAY_MS = 1_000;

/**
 * Thin wrapper around the native WebSocket that owns reconnection, channel
 * subscription bookkeeping (so the same channel is never subscribed twice),
 * and connection-status broadcasting. This is the "real backend" transport —
 * see mock-transport.ts for the transport used while NEXT_PUBLIC_USE_MOCK_API=true.
 */
export class WebSocketClient {
  private socket: WebSocket | null = null;
  private status: ConnectionStatus = "idle";
  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private shouldReconnect = true;

  private readonly channelListeners = new Map<ChannelName, Set<SocketListener>>();
  private readonly statusListeners = new Set<(status: ConnectionStatus) => void>();

  connect() {
    if (this.socket || typeof window === "undefined") return;
    this.shouldReconnect = true;
    this.openSocket();
  }

  disconnect() {
    this.shouldReconnect = false;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.socket?.close();
    this.socket = null;
    this.setStatus("closed");
  }

  subscribe<T>(channel: ChannelName, listener: SocketListener<T>): () => void {
    const isFirstSubscriber = !this.channelListeners.has(channel);
    const listeners = this.channelListeners.get(channel) ?? new Set();
    listeners.add(listener as SocketListener);
    this.channelListeners.set(channel, listeners);

    if (isFirstSubscriber) this.sendSubscription(channel, "subscribe");
    this.connect();

    return () => {
      const current = this.channelListeners.get(channel);
      if (!current) return;
      current.delete(listener as SocketListener);
      if (current.size === 0) {
        this.channelListeners.delete(channel);
        this.sendSubscription(channel, "unsubscribe");
      }
    };
  }

  onStatusChange(listener: (status: ConnectionStatus) => void): () => void {
    this.statusListeners.add(listener);
    listener(this.status);
    return () => this.statusListeners.delete(listener);
  }

  getStatus(): ConnectionStatus {
    return this.status;
  }

  private openSocket() {
    if (!env.NEXT_PUBLIC_WS_URL) {
      this.shouldReconnect = false;
      this.setStatus("closed");
      return;
    }
    this.setStatus(this.reconnectAttempt > 0 ? "reconnecting" : "connecting");

    try {
      this.socket = new WebSocket(env.NEXT_PUBLIC_WS_URL);
    } catch {
      this.setStatus("error");
      this.scheduleReconnect();
      return;
    }

    this.socket.addEventListener("open", () => {
      this.reconnectAttempt = 0;
      this.setStatus("open");
      for (const channel of this.channelListeners.keys()) {
        this.sendSubscription(channel, "subscribe");
      }
    });

    this.socket.addEventListener("message", (event) => {
      try {
        const message = JSON.parse(event.data) as SocketMessage;
        const listeners = this.channelListeners.get(message.channel);
        listeners?.forEach((listener) => listener(message));
      } catch {
        // Ignore malformed frames rather than crashing the socket loop.
      }
    });

    this.socket.addEventListener("close", () => {
      this.socket = null;
      if (this.shouldReconnect) this.scheduleReconnect();
      else this.setStatus("closed");
    });

    this.socket.addEventListener("error", () => {
      this.setStatus("error");
    });
  }

  private sendSubscription(channel: ChannelName, action: "subscribe" | "unsubscribe") {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({ action, channel }));
    }
  }

  private scheduleReconnect() {
    if (!this.shouldReconnect) return;
    this.setStatus("reconnecting");
    const delay = Math.min(
      BASE_RECONNECT_DELAY_MS * 2 ** this.reconnectAttempt,
      MAX_RECONNECT_DELAY_MS,
    );
    this.reconnectAttempt += 1;
    this.reconnectTimer = setTimeout(() => this.openSocket(), delay);
  }

  private setStatus(status: ConnectionStatus) {
    this.status = status;
    this.statusListeners.forEach((listener) => listener(status));
  }
}
