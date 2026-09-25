export type OrderSide = "BUY" | "SELL";

export type OrderType = "MARKET" | "LIMIT" | "STOP_LIMIT";

export type OrderStatus =
  | "CREATED"
  | "RISK_ACCEPTED"
  | "OPEN"
  | "PARTIALLY_FILLED"
  | "FILLED"
  | "HALTED"
  | "REJECTED"
  | "CANCELLED"
  | "EXPIRED";

export const OPEN_ORDER_STATUSES: OrderStatus[] = [
  "CREATED",
  "RISK_ACCEPTED",
  "OPEN",
  "PARTIALLY_FILLED",
];

export const CLOSED_ORDER_STATUSES: OrderStatus[] = [
  "FILLED",
  "HALTED",
  "REJECTED",
  "CANCELLED",
  "EXPIRED",
];

export type Order = {
  orderId: string;
  clientOrderId: string;
  marketId: string;
  marketSymbol: string;

  side: OrderSide;
  orderType: OrderType;

  limitPrice?: string;
  stopPrice?: string;

  quantity: string;
  executedQuantity: string;
  remainingQuantity: string;
  averageFillPrice?: string;

  status: OrderStatus;

  rejectionReason?: string;

  createdAt: string;
  updatedAt: string;
};
