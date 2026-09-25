"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { mockDelay } from "@/lib/api/mock-delay";
import { OnboardingProgress } from "@/features/onboarding/components/onboarding-progress";
import { useOnboardingStore } from "@/stores/onboarding-store";

function ReviewRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between border-b py-2.5 text-sm last:border-0">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium text-foreground">{value}</span>
    </div>
  );
}

export default function ReviewStepPage() {
  const router = useRouter();
  const store = useOnboardingStore();
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit() {
    setIsSubmitting(true);
    await mockDelay(900);
    setIsSubmitting(false);
    toast.success("Application submitted for review.");
    store.reset();
    router.push("/dashboard");
  }

  return (
    <div className="space-y-8">
      <OnboardingProgress current="review" />

      <Card>
        <CardHeader>
          <CardTitle>Review & Submit</CardTitle>
          <CardDescription>Confirm your details before submitting for compliance review.</CardDescription>
        </CardHeader>
        <CardContent className="pb-6">
          <ReviewRow label="Full Name" value={store.fullName || "—"} />
          <ReviewRow label="Jurisdiction" value={store.jurisdiction || "—"} />
          <ReviewRow label="Identity Document" value={store.identityDocumentUploaded ? "Uploaded" : "Not uploaded"} />
          <ReviewRow label="Accredited Investor" value={store.accreditedInvestor ? "Confirmed" : "Not confirmed"} />
          <ReviewRow label="Source of Funds" value={store.sourceOfFunds || "—"} />
          <ReviewRow
            label="Custody Preference"
            value={store.custodyChoice === "CUSTODIAL" ? "Institutional Custody" : store.custodyChoice === "SELF_CUSTODY" ? "Self Custody" : "—"}
          />
        </CardContent>
      </Card>

      <div className="flex justify-between">
        <Button variant="outline" onClick={() => router.push("/onboarding/custody")}>
          Back
        </Button>
        <Button onClick={handleSubmit} disabled={isSubmitting}>
          {isSubmitting ? "Submitting…" : "Submit Application"}
        </Button>
      </div>
    </div>
  );
}
