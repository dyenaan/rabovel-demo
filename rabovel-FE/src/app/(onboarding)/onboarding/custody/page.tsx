"use client";

import { useRouter } from "next/navigation";
import { Lock, Wallet } from "lucide-react";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { OnboardingProgress } from "@/features/onboarding/components/onboarding-progress";
import { cn } from "@/lib/utils";
import { useOnboardingStore } from "@/stores/onboarding-store";

const OPTIONS = [
  {
    value: "CUSTODIAL" as const,
    icon: Lock,
    title: "Institutional Custody",
    description: "Rabovel's custody partner holds your assets. Recommended for most institutional investors.",
  },
  {
    value: "SELF_CUSTODY" as const,
    icon: Wallet,
    title: "Self Custody",
    description: "Bind your own multi-sig or hardware wallet. Requires additional verification.",
  },
];

export default function CustodyStepPage() {
  const router = useRouter();
  const { custodyChoice, setField } = useOnboardingStore();

  return (
    <div className="space-y-8">
      <OnboardingProgress current="custody" />

      <Card>
        <CardHeader>
          <CardTitle>Custody Preference</CardTitle>
          <CardDescription>Choose how your tokenized assets will be held.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 pb-6">
          {OPTIONS.map((option) => (
            <button
              key={option.value}
              type="button"
              onClick={() => setField("custodyChoice", option.value)}
              className={cn(
                "flex w-full items-start gap-3 rounded-md border p-4 text-left transition-colors",
                custodyChoice === option.value ? "border-primary bg-primary/5" : "hover:bg-muted/40",
              )}
            >
              <option.icon className="mt-0.5 size-5 text-muted-foreground" />
              <div>
                <p className="text-sm font-medium text-foreground">{option.title}</p>
                <p className="text-sm text-muted-foreground">{option.description}</p>
              </div>
            </button>
          ))}
        </CardContent>
      </Card>

      <div className="flex justify-between">
        <Button variant="outline" onClick={() => router.push("/onboarding/eligibility")}>
          Back
        </Button>
        <Button disabled={!custodyChoice} onClick={() => router.push("/onboarding/review")}>
          Continue
        </Button>
      </div>
    </div>
  );
}
