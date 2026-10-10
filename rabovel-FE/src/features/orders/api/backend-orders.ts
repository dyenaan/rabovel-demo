import { env } from "@/lib/env";
import { useAuthStore } from "@/stores/auth-store";
import type { Order, OrderSide, OrderStatus, OrderType } from "@/types";

/** Wire shapes of the rabovel-api order endpoints (snake_case, unwrapped). */
type BackendOrderState = "accepted" | "partially_filled" | "filled" | "cancelled";

export type BackendOrder = {
  order_id: string;
  symbol: string;
  side: "buy" | "sell";
  order_type: "market" | "limit";
  limit_price: string | null;
  quantity: string;
  filled_quantity: string;
  remaining_quantity: string;
  average_price: string | null;
  state: BackendOrderState;
  created_at: number;
  updated_at: number;
};

export type BackendOrderAck = {
  order_id: string;
  accepted_at: number;
  state: BackendOrderState;
  filled_quantity: string;
  resting_quantity: string;
  cancelled_quantity: string;
  average_price: string | null;
  fills: { trade_id: string; quantity: string; price: string; liquidity: "internalized" | "principal" }[];
  unfilled_reason: string | null;
};

const STATUS: Record<BackendOrderState, OrderStatus> = {
  accepted: "OPEN",
  partially_filled: "PARTIALLY_FILLED",
  filled: "FILLED",
  cancelled: "CANCELLED",
};

const UNFILLED_REASON: Record<string, string> = {
  no_reference_price: "No market price is available right now.",
  no_house_quote: "Rabovel is not quoting this side right now.",
  house_risk_limit: "Rabovel's position limit for this market was reached.",
  inventory_limit: "Rabovel's available inventory for this market was reached.",
};

/** Backend symbols are bare tickers; UI markets use `TICKER/NGN`. */
export function toBackendSymbol(marketSymbol: string): string {
  return marketSymbol.split("/")[0].trim().toUpperCase();
}

function toMarketSymbol(symbol: string): string {
  return `${symbol}/NGN`;
}

export function toBackendOrderType(orderType: OrderType): BackendOrder["order_type"] {
  if (orderType === "STOP_LIMIT") {
    throw new Error("Stop-limit orders are not supported yet.");
  }
  return orderType === "MARKET" ? "market" : "limit";
}

export function toBackendSide(side: OrderSide): BackendOrder["side"] {
  return side === "BUY" ? "buy" : "sell";
}

function isoFromSeconds(seconds: number): string {
  return new Date(seconds * 1000).toISOString();
}

export function adaptBackendOrder(order: BackendOrder): Order {
  return {
    orderId: order.order_id,
    clientOrderId: order.order_id,
    marketId: order.symbol,
    marketSymbol: toMarketSymbol(order.symbol),
    side: order.side === "buy" ? "BUY" : "SELL",
    orderType: order.order_type === "market" ? "MARKET" : "LIMIT",
    limitPrice: order.limit_price ?? undefined,
    quantity: order.quantity,
    executedQuantity: order.filled_quantity,
    remainingQuantity: order.remaining_quantity,
    averageFillPrice: order.average_price ?? undefined,
    status: STATUS[order.state],
    createdAt: isoFromSeconds(order.created_at),
    updatedAt: isoFromSeconds(order.updated_at),
  };
}

export function adaptOrderAck(
  ack: BackendOrderAck,
  request: { symbol: string; side: OrderSide; orderType: OrderType; limitPrice?: string; quantity: string; clientOrderId: string },
): Order {
  const at = isoFromSeconds(ack.accepted_at);
  return {
    orderId: ack.order_id,
    clientOrderId: request.clientOrderId,
    marketId: request.symbol,
    marketSymbol: toMarketSymbol(request.symbol),
    side: request.side,
    orderType: request.orderType,
    limitPrice: request.limitPrice,
    quantity: request.quantity,
    executedQuantity: ack.filled_quantity,
    remainingQuantity: ack.resting_quantity,
    averageFillPrice: ack.average_price ?? undefined,
    status: STATUS[ack.state],
    rejectionReason: ack.unfilled_reason
      ? (UNFILLED_REASON[ack.unfilled_reason] ?? ack.unfilled_reason)
      : undefined,
    createdAt: at,
    updatedAt: at,
  };
}

export async function ordersRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const token = useAuthStore.getState().token;
  if (!token) throw new Error("Sign in to trade.");
  const response = await fetch(new URL(path, env.NEXT_PUBLIC_API_BASE_URL), {
    ...init,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
      ...init?.headers,
    },
  });
  if (response.status === 204) return undefined as T;
  const payload = (await response.json().catch(() => null)) as
    | T
    | { error?: { message?: string }; message?: string }
    | null;
  if (!response.ok) {
    const details = payload as { error?: { message?: string }; message?: string } | null;
    throw new Error(details?.error?.message ?? details?.message ?? "The order request failed.");
  }
  return payload as T;
}
