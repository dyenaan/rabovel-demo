import { Badge, type badgeVariants } from "@/components/ui/badge";
import type { VariantProps } from "class-variance-authority";
import type {
  AccountStatus,
  ComplianceStatus,
  EligibilityStatus,
  KycStatus,
  MarketStatus,
  OrderStatus,
  ReconciliationStatus,
  SettlementStatus,
} from "@/types";

type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];

const STATUS_VARIANTS: Record<string, BadgeVariant> = {
  // Orders
  CREATED: "muted",
  RISK_ACCEPTED: "info",
  OPEN: "info",
  PARTIALLY_FILLED: "warning",
  FILLED: "success",
  HALTED: "warning",
  REJECTED: "destructive",
  CANCELLED: "muted",
  EXPIRED: "muted",

  // Settlement
  ASSETS_RESERVED: "info",
  ELIGIBILITY_RECHECKED: "info",
  READY_TO_BUILD: "info",
  SIMULATED: "info",
  AWAITING_SIGNATURE: "warning",
  SIGNED: "info",
  SUBMITTED: "info",
  INCLUDED: "info",
  SAFE: "info",
  FINALIZED: "success",
  RECONCILED: "success",
  SETTLED: "success",
  RETRYABLE: "warning",
  REPLACEMENT_PENDING: "warning",
  REORGED: "destructive",
  BLOCKHASH_EXPIRED: "destructive",
  COMPLIANCE_HOLD: "destructive",
  CUSTODY_HOLD: "destructive",
  FAILED_MANUAL_REVIEW: "destructive",
  CANCELED_BEFORE_SUBMISSION: "muted",

  // KYC / Eligibility / Compliance / Account
  NOT_STARTED: "muted",
  PENDING: "warning",
  VERIFIED: "success",
  REQUIRES_ACTION: "warning",
  RESTRICTED: "destructive",
  ELIGIBLE: "success",
  INELIGIBLE: "destructive",
  NOT_ASSESSED: "muted",
  PENDING_REVIEW: "warning",
  ACTIVE: "success",
  SUSPENDED: "destructive",
  CLOSED: "muted",

  // Market
  PRE_OPEN: "info",

  // Reconciliation
  MISMATCH: "destructive",
  UNDER_REVIEW: "warning",
};

type AnyStatus =
  | OrderStatus
  | SettlementStatus
  | KycStatus
  | EligibilityStatus
  | AccountStatus
  | ComplianceStatus
  | MarketStatus
  | ReconciliationStatus
  | string;

export function StatusBadge({
  status,
  label,
}: {
  status: AnyStatus;
  label?: string;
}) {
  const variant = STATUS_VARIANTS[status] ?? "muted";
  const display = label ?? status.replaceAll("_", " ");

  return (
    <Badge variant={variant} className="font-medium capitalize">
      {display.toLowerCase()}
    </Badge>
  );
}
