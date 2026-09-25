import type { Order } from "@/types";
import { Decimal } from "@/lib/formatters";

function buildOrder(partial: {
  orderId: string;
  marketId: string;
  marketSymbol: string;
  side: Order["side"];
  orderType: Order["orderType"];
  limitPrice?: string;
  quantity: string;
  executedQuantity: string;
  status: Order["status"];
  daysAgo: number;
}): Order {
  const remaining = new Decimal(partial.quantity)
    .minus(partial.executedQuantity)
    .toFixed(4);
  const createdAt = new Date();
  createdAt.setDate(createdAt.getDate() - partial.daysAgo);
  const updatedAt = new Date(createdAt);
  updatedAt.setMinutes(updatedAt.getMinutes() + 45);

  return {
    orderId: partial.orderId,
    clientOrderId: `cli_${partial.orderId}`,
    marketId: partial.marketId,
    marketSymbol: partial.marketSymbol,
    side: partial.side,
    orderType: partial.orderType,
    limitPrice: partial.limitPrice,
    quantity: partial.quantity,
    executedQuantity: partial.executedQuantity,
    remainingQuantity: remaining,
    averageFillPrice:
      partial.executedQuantity !== "0" ? partial.limitPrice : undefined,
    status: partial.status,
    createdAt: createdAt.toISOString(),
    updatedAt: updatedAt.toISOString(),
  };
}

export const mockOrders: Order[] = [
  buildOrder({
    orderId: "ord_1001",
    marketId: "mkt_rtf_usd",
    marketSymbol: "RTF/NGN",
    side: "BUY",
    orderType: "LIMIT",
    limitPrice: "100.40",
    quantity: "5000",
    executedQuantity: "5000",
    status: "FILLED",
    daysAgo: 12,
  }),
  buildOrder({
    orderId: "ord_1002",
    marketId: "mkt_gif_usd",
    marketSymbol: "GIF/NGN",
    side: "BUY",
    orderType: "LIMIT",
    limitPrice: "104.80",
    quantity: "1200",
    executedQuantity: "640",
    status: "PARTIALLY_FILLED",
    daysAgo: 1,
  }),
  buildOrder({
    orderId: "ord_1003",
    marketId: "mkt_pref_usd",
    marketSymbol: "PREF/NGN",
    side: "SELL",
    orderType: "LIMIT",
    limitPrice: "97.90",
    quantity: "800",
    executedQuantity: "0",
    status: "OPEN",
    daysAgo: 0,
  }),
  buildOrder({
    orderId: "ord_1004",
    marketId: "mkt_pco_usd",
    marketSymbol: "PCO/NGN",
    side: "BUY",
    orderType: "MARKET",
    quantity: "300",
    executedQuantity: "300",
    status: "FILLED",
    daysAgo: 4,
  }),
  buildOrder({
    orderId: "ord_1005",
    marketId: "mkt_crf_usd",
    marketSymbol: "CRF/NGN",
    side: "SELL",
    orderType: "STOP_LIMIT",
    limitPrice: "2380.00",
    quantity: "50",
    executedQuantity: "0",
    status: "OPEN",
    daysAgo: 2,
  }),
  buildOrder({
    orderId: "ord_1006",
    marketId: "mkt_rtf_usd",
    marketSymbol: "RTF/NGN",
    side: "SELL",
    orderType: "LIMIT",
    limitPrice: "100.55",
    quantity: "2000",
    executedQuantity: "0",
    status: "CANCELLED",
    daysAgo: 6,
  }),
  buildOrder({
    orderId: "ord_1007",
    marketId: "mkt_gif_usd",
    marketSymbol: "GIF/NGN",
    side: "BUY",
    orderType: "LIMIT",
    limitPrice: "103.00",
    quantity: "500",
    executedQuantity: "0",
    status: "REJECTED",
    daysAgo: 9,
  }),
];
