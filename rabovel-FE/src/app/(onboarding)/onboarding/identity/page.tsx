"use client";

import { useRouter } from "next/navigation";
import { UploadCloud } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { OnboardingProgress } from "@/features/onboarding/components/onboarding-progress";
import { useOnboardingStore } from "@/stores/onboarding-store";

export default function IdentityStepPage() {
  const router = useRouter();
  const { fullName, jurisdiction, identityDocumentUploaded, setField } = useOnboardingStore();

  return (
    <div className="space-y-8">
      <OnboardingProgress current="identity" />

      <Card>
        <CardHeader>
          <CardTitle>Identity Verification</CardTitle>
          <CardDescription>Tell us who you are so we can verify your identity.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 pb-6">
          <div className="space-y-1.5">
            <Label>Full Legal Name</Label>
            <Input value={fullName} onChange={(e) => setField("fullName", e.target.value)} />
          </div>
          <div className="space-y-1.5">
            <Label>Jurisdiction of Residence</Label>
            <Input
              value={jurisdiction}
              onChange={(e) => setField("jurisdiction", e.target.value)}
              placeholder="e.g. United States"
            />
          </div>
          <div className="space-y-1.5">
            <Label>Government-Issued ID</Label>
            <button
              type="button"
              onClick={() => setField("identityDocumentUploaded", true)}
              className="flex w-full flex-col items-center gap-2 rounded-md border border-dashed p-6 text-center hover:bg-muted/40"
            >
              <UploadCloud className="size-6 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">
                {identityDocumentUploaded ? "passport.pdf uploaded" : "Click to upload a passport or national ID"}
              </span>
            </button>
          </div>
        </CardContent>
      </Card>

      <div className="flex justify-end">
        <Button
          disabled={!fullName || !jurisdiction || !identityDocumentUploaded}
          onClick={() => router.push("/onboarding/eligibility")}
        >
          Continue
        </Button>
      </div>
    </div>
  );
}
