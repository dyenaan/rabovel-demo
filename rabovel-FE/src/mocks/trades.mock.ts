import type { Trade } from "@/types";

function daysAgo(n: number): string {
  const date = new Date();
  date.setDate(date.getDate() - n);
  return date.toISOString();
}

export const mockTrades: Trade[] = [
  {
    tradeId: "trd_9001",
    marketId: "mkt_rtf_usd",
    marketSymbol: "RTF/NGN",
    makerOrderId: "ord_2001",
    takerOrderId: "ord_1001",
    side: "BUY",
    executionPrice: "100.40",
    quantity: "5000",
    executedAt: daysAgo(12),
    settlementStatus: "SETTLED",
    settlementId: "stl_5001",
  },
  {
    tradeId: "trd_9002",
    marketId: "mkt_gif_usd",
    marketSymbol: "GIF/NGN",
    makerOrderId: "ord_2002",
    takerOrderId: "ord_1002",
    side: "BUY",
    executionPrice: "104.75",
    quantity: "640",
    executedAt: daysAgo(1),
    settlementStatus: "SIGNED",
    settlementId: "stl_5002",
  },
  {
    tradeId: "trd_9003",
    marketId: "mkt_pco_usd",
    marketSymbol: "PCO/NGN",
    makerOrderId: "ord_2003",
    takerOrderId: "ord_1004",
    side: "BUY",
    executionPrice: "101.70",
    quantity: "300",
    executedAt: daysAgo(4),
    settlementStatus: "FINALIZED",
    settlementId: "stl_5003",
  },
  {
    tradeId: "trd_9004",
    marketId: "mkt_pref_usd",
    marketSymbol: "PREF/NGN",
    makerOrderId: "ord_2004",
    takerOrderId: "ord_1008",
    side: "SELL",
    executionPrice: "98.10",
    quantity: "420",
    executedAt: daysAgo(15),
    settlementStatus: "COMPLIANCE_HOLD",
    settlementId: "stl_5004",
  },
  {
    tradeId: "trd_9005",
    marketId: "mkt_crf_usd",
    marketSymbol: "CRF/NGN",
    makerOrderId: "ord_2005",
    takerOrderId: "ord_1009",
    side: "BUY",
    executionPrice: "2409.20",
    quantity: "18",
    executedAt: daysAgo(20),
    settlementStatus: "RECONCILED",
    settlementId: "stl_5005",
  },
];
