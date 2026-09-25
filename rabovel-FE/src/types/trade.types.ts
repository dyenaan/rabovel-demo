import type { SettlementStatus } from "./settlement.types";

export type Trade = {
  tradeId: string;
  marketId: string;
  marketSymbol: string;

  makerOrderId: string;
  takerOrderId: string;

  side: "BUY" | "SELL";
  executionPrice: string;
  quantity: string;

  executedAt: string;

  /**
   * Trade execution is a separate concept from settlement — a trade can be
   * executed while its settlement is still pending. Never merge these.
   */
  settlementStatus: SettlementStatus;
  settlementId?: string;
};
