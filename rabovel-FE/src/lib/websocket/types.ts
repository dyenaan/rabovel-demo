export type ConnectionStatus =
  | "idle"
  | "connecting"
  | "open"
  | "reconnecting"
  | "closed"
  | "error";

export type ChannelName =
  | `market:${string}`
  | `orderbook:${string}`
  | `trades:${string}`
  | `orders:${string}`
  | `settlements:${string}`
  | `notifications:${string}`;

export type SocketMessage<T = unknown> = {
  channel: ChannelName;
  type: string;
  payload: T;
  timestamp: string;
};

export type SocketListener<T = unknown> = (message: SocketMessage<T>) => void;
