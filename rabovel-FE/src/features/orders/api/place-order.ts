import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { Decimal, formatPrice } from "@/lib/formatters";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockOrders } from "@/mocks/orders.mock";
import type { Order, OrderSide, OrderType } from "@/types";

export type PlaceOrderInput = {
  marketId: string;
  marketSymbol: string;
  side: OrderSide;
  orderType: OrderType;
  limitPrice?: string;
  quantity: string;
  idempotencyKey: string;
};

export async function placeOrder(input: PlaceOrderInput): Promise<Order> {
  if (isMockApiEnabled) {
    await mockDelay(700);
    const order: Order = {
      orderId: `ord_${Date.now()}`,
      clientOrderId: input.idempotencyKey,
      marketId: input.marketId,
      marketSymbol: input.marketSymbol,
      side: input.side,
      orderType: input.orderType,
      limitPrice: input.limitPrice ? formatPrice(input.limitPrice) : undefined,
      quantity: input.quantity,
      executedQuantity: "0",
      remainingQuantity: input.quantity,
      status: "OPEN",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    mockOrders.unshift(order);
    return order;
  }

  const { data } = await apiClient.post<Order>("/orders", input, {
    idempotencyKey: input.idempotencyKey,
  });
  return data;
}

export async function cancelOrder(orderId: string, idempotencyKey: string): Promise<Order> {
  if (isMockApiEnabled) {
    await mockDelay(500);
    const order = mockOrders.find((o) => o.orderId === orderId);
    if (!order) throw new Error("Order not found");
    order.status = "CANCELLED";
    order.updatedAt = new Date().toISOString();
    return order;
  }

  const { data } = await apiClient.post<Order>(
    `/orders/${orderId}/cancel`,
    {},
    { idempotencyKey },
  );
  return data;
}

export function calculateOrderTotal(quantity: string, price?: string): string {
  if (!price) return "—";
  try {
    return new Decimal(quantity || 0).times(price).toFixed(2);
  } catch {
    return "—";
  }
}
