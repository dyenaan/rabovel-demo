import type { FinalityState, SettlementStatus } from "@/types";

export function deriveFinalityState(status: SettlementStatus): FinalityState | null {
  switch (status) {
    case "SUBMITTED":
    case "RETRYABLE":
    case "REPLACEMENT_PENDING":
    case "BLOCKHASH_EXPIRED":
      return "SUBMITTED";
    case "INCLUDED":
      return "INCLUDED";
    case "SAFE":
      return "SAFE";
    case "FINALIZED":
    case "RECONCILED":
    case "SETTLED":
      return "FINALIZED";
    case "REORGED":
      return "ORPHANED";
    default:
      // Pre-submission lifecycle stages have no on-chain finality yet.
      return null;
  }
}
