import type { Settlement } from "@/types";

function daysAgo(n: number, hoursOffset = 0): string {
  const date = new Date();
  date.setDate(date.getDate() - n);
  date.setHours(date.getHours() + hoursOffset);
  return date.toISOString();
}

export const mockSettlements: Settlement[] = [
  {
    settlementId: "stl_5001",
    tradeId: "trd_9001",
    assetLeg: { type: "ASSET", assetOrCurrency: "RTF", amount: "5000" },
    cashLeg: { type: "CASH", assetOrCurrency: "NGN", amount: "502000.00" },
    chainId: "eip155:1",
    custodyRoute: "Fireblocks — Institutional Vault",
    finalityPolicy: "12 confirmations",
    transactionHash:
      "0x7c4a8d09ca3762af61e59520943dc26494f8941b1188379ac25b7a11e5b3f2a",
    status: "SETTLED",
    createdAt: daysAgo(12),
    updatedAt: daysAgo(12, 1),
  },
  {
    settlementId: "stl_5002",
    tradeId: "trd_9002",
    assetLeg: { type: "ASSET", assetOrCurrency: "GIF", amount: "640" },
    cashLeg: { type: "CASH", assetOrCurrency: "NGN", amount: "67040.00" },
    chainId: "eip155:1",
    custodyRoute: "Fireblocks — Institutional Vault",
    finalityPolicy: "12 confirmations",
    transactionHash:
      "0x1f9e2b3a4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f",
    status: "SIGNED",
    createdAt: daysAgo(1),
    updatedAt: daysAgo(1, 1),
  },
  {
    settlementId: "stl_5003",
    tradeId: "trd_9003",
    assetLeg: { type: "ASSET", assetOrCurrency: "PCO", amount: "300" },
    cashLeg: { type: "CASH", assetOrCurrency: "NGN", amount: "30510.00" },
    chainId: "eip155:1",
    custodyRoute: "Anchorage Digital",
    finalityPolicy: "Safe head",
    transactionHash:
      "0x9a8b7c6d5e4f30291807f6e5d4c3b2a190807f6e5d4c3b2a190807f6e5d4c3b",
    status: "FINALIZED",
    createdAt: daysAgo(4),
    updatedAt: daysAgo(4, 2),
  },
  {
    settlementId: "stl_5004",
    tradeId: "trd_9004",
    assetLeg: { type: "ASSET", assetOrCurrency: "PREF", amount: "420" },
    cashLeg: { type: "CASH", assetOrCurrency: "NGN", amount: "41202.00" },
    custodyRoute: "Fireblocks — Institutional Vault",
    status: "COMPLIANCE_HOLD",
    createdAt: daysAgo(15),
    updatedAt: daysAgo(14),
  },
  {
    settlementId: "stl_5005",
    tradeId: "trd_9005",
    assetLeg: { type: "ASSET", assetOrCurrency: "CRF", amount: "18" },
    cashLeg: { type: "CASH", assetOrCurrency: "NGN", amount: "43365.60" },
    chainId: "eip155:8453",
    custodyRoute: "Anchorage Digital",
    finalityPolicy: "Finalized epoch",
    transactionHash:
      "0x2b1a0c9d8e7f605142332415f6e7d8c9b0a19283746551627384f5e6d7c8b9a",
    status: "RECONCILED",
    createdAt: daysAgo(20),
    updatedAt: daysAgo(19),
  },
  {
    settlementId: "stl_5006",
    tradeId: "trd_9006",
    assetLeg: { type: "ASSET", assetOrCurrency: "GIF", amount: "150" },
    cashLeg: { type: "CASH", assetOrCurrency: "NGN", amount: "15780.00" },
    chainId: "eip155:1",
    custodyRoute: "Fireblocks — Institutional Vault",
    finalityPolicy: "12 confirmations",
    status: "AWAITING_SIGNATURE",
    createdAt: daysAgo(0),
    updatedAt: daysAgo(0),
  },
];
