export type SettlementStatus =
  | "CREATED"
  | "ASSETS_RESERVED"
  | "ELIGIBILITY_RECHECKED"
  | "READY_TO_BUILD"
  | "SIMULATED"
  | "AWAITING_SIGNATURE"
  | "SIGNED"
  | "SUBMITTED"
  | "INCLUDED"
  | "SAFE"
  | "FINALIZED"
  | "RECONCILED"
  | "SETTLED"
  | "RETRYABLE"
  | "REPLACEMENT_PENDING"
  | "REORGED"
  | "BLOCKHASH_EXPIRED"
  | "COMPLIANCE_HOLD"
  | "CUSTODY_HOLD"
  | "FAILED_MANUAL_REVIEW"
  | "CANCELED_BEFORE_SUBMISSION";

export const SETTLEMENT_HOLD_STATUSES: SettlementStatus[] = [
  "COMPLIANCE_HOLD",
  "CUSTODY_HOLD",
  "FAILED_MANUAL_REVIEW",
];

export const SETTLEMENT_TERMINAL_STATUSES: SettlementStatus[] = [
  "SETTLED",
  "RECONCILED",
  "CANCELED_BEFORE_SUBMISSION",
];

export type SettlementLeg = {
  type: "ASSET" | "CASH";
  assetOrCurrency: string;
  amount: string;
  fromAccount?: string;
  toAccount?: string;
};

export type Settlement = {
  settlementId: string;
  tradeId: string;

  assetLeg: SettlementLeg;
  cashLeg: SettlementLeg;

  chainId?: string;
  custodyRoute?: string;
  finalityPolicy?: string;
  transactionHash?: string;

  status: SettlementStatus;

  createdAt: string;
  updatedAt: string;
};

export type FinalityState =
  | "SUBMITTED"
  | "PRECONFIRMED"
  | "INCLUDED"
  | "SAFE"
  | "FINALIZED"
  | "ORPHANED";
