import type { ComplianceStatus } from "./investor.types";

export type ComplianceHoldReason =
  | "SANCTIONS_SCREEN"
  | "DOCUMENT_EXPIRED"
  | "SOURCE_OF_FUNDS"
  | "JURISDICTION_RESTRICTION"
  | "MANUAL_REVIEW";

export type ComplianceCase = {
  caseId: string;
  investorId: string;
  investorName: string;
  status: ComplianceStatus;
  reason?: ComplianceHoldReason;
  assignedTo?: string;
  openedAt: string;
  updatedAt: string;
  notes?: string;
};

export type { ComplianceStatus };

export type ReconciliationStatus =
  | "RECONCILED"
  | "PENDING"
  | "MISMATCH"
  | "UNDER_REVIEW";

export type ReconciliationRecord = {
  recordId: string;
  assetName: string;
  ledgerBalance: string;
  custodyBalance: string;
  onChainBalance: string;
  status: ReconciliationStatus;
  lastCheckedAt: string;
};
