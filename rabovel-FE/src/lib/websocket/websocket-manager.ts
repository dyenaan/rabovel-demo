import { isMockApiEnabled } from "@/lib/api/client";
import { MockWebSocketTransport } from "./mock-transport";
import { WebSocketClient } from "./websocket-client";

export type Transport = WebSocketClient | MockWebSocketTransport;

let instance: Transport | null = null;

/** Single shared connection for the whole app — every hook subscribes through this. */
export function getWebSocketManager(): Transport {
  if (!instance) {
    instance = isMockApiEnabled ? new MockWebSocketTransport() : new WebSocketClient();
  }
  return instance;
}
