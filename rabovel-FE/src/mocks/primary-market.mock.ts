import type {
  Redemption,
  ReserveStatus,
  Subscription,
} from "@/features/primary-market/types/primary-market.types";

export const mockSubscriptions: Subscription[] = [
  {
    subscriptionId: "sub_2001",
    assetId: "ast_credit_01",
    assetName: "Private Credit Opportunities",
    amount: "150000.00",
    currency: "NGN",
    status: "UNDER_REVIEW",
    submittedAt: new Date(Date.now() - 1000 * 60 * 60 * 26).toISOString(),
  },
  {
    subscriptionId: "sub_2000",
    assetId: "ast_infra_01",
    assetName: "Global Infrastructure Fund",
    amount: "500000.00",
    currency: "NGN",
    status: "ISSUED",
    submittedAt: new Date(Date.now() - 1000 * 60 * 60 * 24 * 40).toISOString(),
  },
];

export const mockRedemptions: Redemption[] = [
  {
    redemptionId: "red_3001",
    assetId: "ast_treasury_01",
    assetName: "Rabovel Treasury Fund",
    quantity: "2000",
    status: "SETTLED",
    submittedAt: new Date(Date.now() - 1000 * 60 * 60 * 24 * 10).toISOString(),
  },
];

export const mockReserveStatus: ReserveStatus[] = [
  { assetId: "ast_treasury_01", totalIssued: "482500000", totalReserved: "482500000", coveragePercent: "100.00" },
  { assetId: "ast_infra_01", totalIssued: "215000000", totalReserved: "215000000", coveragePercent: "100.00" },
  { assetId: "ast_realestate_01", totalIssued: "340000000", totalReserved: "338100000", coveragePercent: "99.44" },
  { assetId: "ast_credit_01", totalIssued: "612000000", totalReserved: "612000000", coveragePercent: "100.00" },
  { assetId: "ast_commodity_01", totalIssued: "89400000", totalReserved: "89400000", coveragePercent: "100.00" },
];
