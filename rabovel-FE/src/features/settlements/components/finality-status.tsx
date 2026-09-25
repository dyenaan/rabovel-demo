import { Check, CircleDashed, GitBranch, Loader2, ShieldCheck, TriangleAlert } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { FinalityState } from "@/types";

const FINALITY_STEPS: FinalityState[] = ["SUBMITTED", "INCLUDED", "SAFE", "FINALIZED"];

const FINALITY_META: Record<
  FinalityState,
  { label: string; description: string; icon: LucideIcon; variant: "info" | "success" | "destructive" | "muted" }
> = {
  SUBMITTED: {
    label: "Submitted",
    description: "The transaction has been broadcast to the network and is awaiting inclusion in a block.",
    icon: Loader2,
    variant: "muted",
  },
  PRECONFIRMED: {
    label: "Preconfirmed",
    description: "A validator or sequencer has committed to including this transaction, ahead of on-chain inclusion.",
    icon: CircleDashed,
    variant: "info",
  },
  INCLUDED: {
    label: "Included",
    description: "The transaction has been included in a block, but the block is not yet considered safe from reorganization.",
    icon: GitBranch,
    variant: "info",
  },
  SAFE: {
    label: "Safe",
    description: "The block is protected by enough subsequent confirmations that a reorg is highly unlikely.",
    icon: ShieldCheck,
    variant: "info",
  },
  FINALIZED: {
    label: "Finalized",
    description: "The transaction is finalized under the network's consensus rules and cannot be reverted.",
    icon: Check,
    variant: "success",
  },
  ORPHANED: {
    label: "Orphaned",
    description: "The block containing this transaction was reorganized out of the canonical chain. Settlement will be retried.",
    icon: TriangleAlert,
    variant: "destructive",
  },
};

export function FinalityStatus({ state }: { state: FinalityState }) {
  const meta = FINALITY_META[state];
  const stepIndex = FINALITY_STEPS.indexOf(state);

  return (
    <div className="space-y-3">
      {stepIndex >= 0 && (
        <div className="flex items-center gap-1.5">
          {FINALITY_STEPS.map((step, index) => (
            <div key={step} className="flex flex-1 items-center gap-1.5">
              <div
                className={cn(
                  "h-1.5 flex-1 rounded-full",
                  index <= stepIndex ? "bg-primary" : "bg-muted",
                )}
              />
            </div>
          ))}
        </div>
      )}
      <div className="flex items-start gap-2.5">
        <Badge variant={meta.variant} className="gap-1.5 py-1">
          <meta.icon className={cn("size-3.5", state === "SUBMITTED" && "animate-spin")} />
          {meta.label}
        </Badge>
      </div>
      <p className="text-sm text-muted-foreground">{meta.description}</p>
    </div>
  );
}
