import { Check, TriangleAlert } from "lucide-react";

import { cn } from "@/lib/utils";
import { SETTLEMENT_HOLD_STATUSES, type SettlementStatus } from "@/types";

const HAPPY_PATH: SettlementStatus[] = [
  "CREATED",
  "ASSETS_RESERVED",
  "ELIGIBILITY_RECHECKED",
  "READY_TO_BUILD",
  "SIMULATED",
  "AWAITING_SIGNATURE",
  "SIGNED",
  "SUBMITTED",
  "INCLUDED",
  "SAFE",
  "FINALIZED",
  "RECONCILED",
  "SETTLED",
];

const LABELS: Record<SettlementStatus, string> = {
  CREATED: "Created",
  ASSETS_RESERVED: "Assets Reserved",
  ELIGIBILITY_RECHECKED: "Eligibility Rechecked",
  READY_TO_BUILD: "Ready to Build",
  SIMULATED: "Simulated",
  AWAITING_SIGNATURE: "Awaiting Signature",
  SIGNED: "Signed",
  SUBMITTED: "Submitted",
  INCLUDED: "Included",
  SAFE: "Safe",
  FINALIZED: "Finalized",
  RECONCILED: "Reconciled",
  SETTLED: "Settled",
  RETRYABLE: "Retryable",
  REPLACEMENT_PENDING: "Replacement Pending",
  REORGED: "Reorged",
  BLOCKHASH_EXPIRED: "Blockhash Expired",
  COMPLIANCE_HOLD: "Compliance Hold",
  CUSTODY_HOLD: "Custody Hold",
  FAILED_MANUAL_REVIEW: "Failed Manual Review",
  CANCELED_BEFORE_SUBMISSION: "Canceled Before Submission",
};

export function SettlementTimeline({ status }: { status: SettlementStatus }) {
  const isHold = SETTLEMENT_HOLD_STATUSES.includes(status);
  const currentIndex = HAPPY_PATH.indexOf(status);

  return (
    <div className="space-y-1">
      {HAPPY_PATH.map((step, index) => {
        const isDone = currentIndex >= 0 ? index < currentIndex : false;
        const isCurrent = index === currentIndex;

        return (
          <div key={step} className="flex items-center gap-3">
            <div
              className={cn(
                "flex size-5 shrink-0 items-center justify-center rounded-full border text-[10px]",
                isDone
                  ? "border-success bg-success text-success-foreground"
                  : isCurrent
                    ? "border-primary bg-primary text-primary-foreground"
                    : "border-border bg-background text-transparent",
              )}
            >
              {isDone && <Check className="size-3" />}
            </div>
            <span
              className={cn(
                "text-sm",
                isDone || isCurrent ? "text-foreground font-medium" : "text-muted-foreground",
              )}
            >
              {LABELS[step]}
            </span>
          </div>
        );
      })}

      {isHold && (
        <div className="mt-3 flex items-center gap-3 rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2">
          <TriangleAlert className="size-4 shrink-0 text-destructive" />
          <span className="text-sm font-medium text-destructive">{LABELS[status]}</span>
        </div>
      )}
    </div>
  );
}
