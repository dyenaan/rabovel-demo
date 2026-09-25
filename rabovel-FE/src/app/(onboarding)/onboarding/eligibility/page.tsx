"use client";

import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { OnboardingProgress } from "@/features/onboarding/components/onboarding-progress";
import { useOnboardingStore } from "@/stores/onboarding-store";

export default function EligibilityStepPage() {
  const router = useRouter();
  const { accreditedInvestor, sourceOfFunds, setField } = useOnboardingStore();

  return (
    <div className="space-y-8">
      <OnboardingProgress current="eligibility" />

      <Card>
        <CardHeader>
          <CardTitle>Eligibility Assessment</CardTitle>
          <CardDescription>
            Tokenized securities on Rabovel are offered only to eligible institutional and accredited investors.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 pb-6">
          <div className="flex items-start gap-2 rounded-md border p-3">
            <Checkbox
              checked={accreditedInvestor}
              onCheckedChange={(checked) => setField("accreditedInvestor", checked === true)}
            />
            <Label className="text-sm font-normal">
              I certify that I qualify as an accredited investor or institutional investor under applicable
              securities laws in my jurisdiction.
            </Label>
          </div>
          <div className="space-y-1.5">
            <Label>Primary Source of Funds</Label>
            <Input
              value={sourceOfFunds}
              onChange={(e) => setField("sourceOfFunds", e.target.value)}
              placeholder="e.g. Investment income, business proceeds"
            />
          </div>
        </CardContent>
      </Card>

      <div className="flex justify-between">
        <Button variant="outline" onClick={() => router.push("/onboarding/identity")}>
          Back
        </Button>
        <Button
          disabled={!accreditedInvestor || !sourceOfFunds}
          onClick={() => router.push("/onboarding/custody")}
        >
          Continue
        </Button>
      </div>
    </div>
  );
}
