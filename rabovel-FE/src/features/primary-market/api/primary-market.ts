import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import {
  mockRedemptions,
  mockReserveStatus,
  mockSubscriptions,
} from "@/mocks/primary-market.mock";
import type { Redemption, ReserveStatus, Subscription } from "../types/primary-market.types";

export async function getSubscriptions(): Promise<Subscription[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockSubscriptions;
  }
  const { data } = await apiClient.get<Subscription[]>("/primary-market/subscriptions");
  return data;
}

export async function getRedemptions(): Promise<Redemption[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockRedemptions;
  }
  const { data } = await apiClient.get<Redemption[]>("/primary-market/redemptions");
  return data;
}

export async function getReserveStatus(): Promise<ReserveStatus[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockReserveStatus;
  }
  const { data } = await apiClient.get<ReserveStatus[]>("/primary-market/reserves");
  return data;
}

export async function submitSubscription(input: {
  assetId: string;
  amount: string;
  idempotencyKey: string;
}): Promise<Subscription> {
  if (isMockApiEnabled) {
    await mockDelay(700);
    const subscription: Subscription = {
      subscriptionId: `sub_${Date.now()}`,
      assetId: input.assetId,
      assetName: "Selected Asset",
      amount: input.amount,
      currency: "NGN",
      status: "SUBMITTED",
      submittedAt: new Date().toISOString(),
    };
    mockSubscriptions.unshift(subscription);
    return subscription;
  }
  const { data } = await apiClient.post<Subscription>("/primary-market/subscriptions", input, {
    idempotencyKey: input.idempotencyKey,
  });
  return data;
}

export async function submitRedemption(input: {
  assetId: string;
  quantity: string;
  idempotencyKey: string;
}): Promise<Redemption> {
  if (isMockApiEnabled) {
    await mockDelay(700);
    const redemption: Redemption = {
      redemptionId: `red_${Date.now()}`,
      assetId: input.assetId,
      assetName: "Selected Asset",
      quantity: input.quantity,
      status: "SUBMITTED",
      submittedAt: new Date().toISOString(),
    };
    mockRedemptions.unshift(redemption);
    return redemption;
  }
  const { data } = await apiClient.post<Redemption>("/primary-market/redemptions", input, {
    idempotencyKey: input.idempotencyKey,
  });
  return data;
}
