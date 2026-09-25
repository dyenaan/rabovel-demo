import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockOrders } from "@/mocks/orders.mock";
import type { Order } from "@/types";

export async function getOrders(): Promise<Order[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockOrders;
  }
  const { data } = await apiClient.get<Order[]>("/orders");
  return data;
}

export async function getOrder(orderId: string): Promise<Order | undefined> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockOrders.find((order) => order.orderId === orderId);
  }
  const { data } = await apiClient.get<Order>(`/orders/${orderId}`);
  return data;
}
