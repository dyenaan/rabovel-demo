import { getMockOrderBook, getMockRecentTrades, mockMarkets } from "@/mocks/markets.mock";
import type { ChannelName, ConnectionStatus, SocketListener, SocketMessage } from "./types";

/**
 * Simulates the real-time channels described in the websocket architecture
 * (market, orderbook, trades, settlements) using the mock data generators, so
 * the trading UI can be built and demoed against realistic streaming behavior
 * before a backend exists. Swapped out for WebSocketClient once
 * NEXT_PUBLIC_USE_MOCK_API=false.
 */
export class MockWebSocketTransport {
  private status: ConnectionStatus = "idle";
  private readonly channelListeners = new Map<ChannelName, Set<SocketListener>>();
  private readonly statusListeners = new Set<(status: ConnectionStatus) => void>();
  private readonly intervals = new Map<ChannelName, ReturnType<typeof setInterval>>();

  connect() {
    if (this.status === "open") return;
    this.setStatus("connecting");
    setTimeout(() => this.setStatus("open"), 250);
  }

  disconnect() {
    this.intervals.forEach((interval) => clearInterval(interval));
    this.intervals.clear();
    this.setStatus("closed");
  }

  subscribe<T>(channel: ChannelName, listener: SocketListener<T>): () => void {
    const listeners = this.channelListeners.get(channel) ?? new Set();
    listeners.add(listener as SocketListener);
    this.channelListeners.set(channel, listeners);
    this.connect();
    this.ensureSimulation(channel);

    return () => {
      const current = this.channelListeners.get(channel);
      current?.delete(listener as SocketListener);
      if (current && current.size === 0) {
        this.channelListeners.delete(channel);
        const interval = this.intervals.get(channel);
        if (interval) clearInterval(interval);
        this.intervals.delete(channel);
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

  private ensureSimulation(channel: ChannelName) {
    if (this.intervals.has(channel)) return;

    const [kind, id] = channel.split(":") as [string, string];
    const tick = () => this.emitTick(channel, kind, id);
    const interval = setInterval(tick, kind === "orderbook" ? 2500 : 3500);
    this.intervals.set(channel, interval);
  }

  private emitTick(channel: ChannelName, kind: string, id: string) {
    const listeners = this.channelListeners.get(channel);
    if (!listeners || listeners.size === 0) return;

    let message: SocketMessage | null = null;

    if (kind === "market") {
      const market = mockMarkets.find((m) => m.marketId === id);
      if (!market) return;
      const drift = (Math.random() - 0.5) * Number(market.lastPrice || 100) * 0.001;
      const nextPrice = (Number(market.lastPrice || 100) + drift).toFixed(2);
      message = {
        channel,
        type: "price.tick",
        payload: { ...market, lastPrice: nextPrice },
        timestamp: new Date().toISOString(),
      };
    } else if (kind === "orderbook") {
      message = {
        channel,
        type: "orderbook.snapshot",
        payload: getMockOrderBook(id),
        timestamp: new Date().toISOString(),
      };
    } else if (kind === "trades") {
      const [latest] = getMockRecentTrades(id, 1);
      message = {
        channel,
        type: "trade.print",
        payload: { ...latest, executedAt: new Date().toISOString() },
        timestamp: new Date().toISOString(),
      };
    }

    if (message) listeners.forEach((listener) => listener(message as SocketMessage));
  }

  private setStatus(status: ConnectionStatus) {
    this.status = status;
    this.statusListeners.forEach((listener) => listener(status));
  }
}
