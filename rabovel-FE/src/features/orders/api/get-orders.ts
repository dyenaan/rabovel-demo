import { isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockOrders } from "@/mocks/orders.mock";
import type { Order } from "@/types";
import { adaptBackendOrder, ordersRequest, type BackendOrder } from "./backend-orders";

export async function getOrders(): Promise<Order[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockOrders;
  }
  const orders = await ordersRequest<BackendOrder[]>("/orders");
  return orders.map(adaptBackendOrder);
}

export async function getOrder(orderId: string): Promise<Order | undefined> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockOrders.find((order) => order.orderId === orderId);
  }
  const order = await ordersRequest<BackendOrder>(`/orders/${encodeURIComponent(orderId)}`);
  return adaptBackendOrder(order);
}
