export type SubscriptionStatus =
  | "SUBMITTED"
  | "UNDER_REVIEW"
  | "RESERVE_CONFIRMED"
  | "ISSUED"
  | "REJECTED";

export type Subscription = {
  subscriptionId: string;
  assetId: string;
  assetName: string;
  amount: string;
  currency: string;
  status: SubscriptionStatus;
  submittedAt: string;
};

export type RedemptionStatus = "SUBMITTED" | "UNDER_REVIEW" | "APPROVED" | "SETTLED" | "REJECTED";

export type Redemption = {
  redemptionId: string;
  assetId: string;
  assetName: string;
  quantity: string;
  status: RedemptionStatus;
  submittedAt: string;
};

export type ReserveStatus = {
  assetId: string;
  totalIssued: string;
  totalReserved: string;
  coveragePercent: string;
};
