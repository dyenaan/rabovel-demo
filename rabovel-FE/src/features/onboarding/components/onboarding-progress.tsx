import { Check } from "lucide-react";

import { cn } from "@/lib/utils";

const STEPS = [
  { key: "identity", label: "Identity" },
  { key: "eligibility", label: "Eligibility" },
  { key: "custody", label: "Custody" },
  { key: "review", label: "Review" },
] as const;

export type OnboardingStepKey = (typeof STEPS)[number]["key"];

export function OnboardingProgress({ current }: { current: OnboardingStepKey }) {
  const currentIndex = STEPS.findIndex((s) => s.key === current);

  return (
    <div className="flex items-center">
      {STEPS.map((step, index) => (
        <div key={step.key} className="flex flex-1 items-center last:flex-none">
          <div className="flex flex-col items-center gap-1.5">
            <div
              className={cn(
                "flex size-8 items-center justify-center rounded-full border text-xs font-medium",
                index < currentIndex
                  ? "border-success bg-success text-success-foreground"
                  : index === currentIndex
                    ? "border-primary bg-primary text-primary-foreground"
                    : "border-border bg-background text-muted-foreground",
              )}
            >
              {index < currentIndex ? <Check className="size-4" /> : index + 1}
            </div>
            <span
              className={cn(
                "text-xs whitespace-nowrap",
                index <= currentIndex ? "font-medium text-foreground" : "text-muted-foreground",
              )}
            >
              {step.label}
            </span>
          </div>
          {index < STEPS.length - 1 && (
            <div className={cn("mx-2 h-px flex-1", index < currentIndex ? "bg-success" : "bg-border")} />
          )}
        </div>
      ))}
    </div>
  );
}
