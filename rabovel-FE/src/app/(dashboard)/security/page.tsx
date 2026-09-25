"use client";

import { useState } from "react";
import { toast } from "sonner";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { StatusBadge } from "@/components/shared/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { useAuth } from "@/features/auth/hooks/use-auth";
import { mockInvestors } from "@/mocks/investors.mock";
import { useNotificationPreferencesStore } from "@/stores/notification-preferences-store";

const PREFERENCE_LABELS: Record<string, string> = {
  emailAlerts: "Email alerts",
  orderFillAlerts: "Order fill notifications",
  settlementAlerts: "Settlement status updates",
  complianceAlerts: "Compliance & KYC updates",
  marketingUpdates: "Product updates & marketing",
};

export default function SecurityPage() {
  const { user } = useAuth();
  const investor = mockInvestors.find((i) => i.investorId === user?.investorId) ?? mockInvestors[0];
  const [twoFactorEnabled, setTwoFactorEnabled] = useState(true);
  const preferences = useNotificationPreferencesStore();

  return (
    <PageContainer>
      <PageHeader title="Security" description="Manage account security, compliance status, and notification preferences." />

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Compliance Status</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 pb-6">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">KYC Verification</span>
              <StatusBadge status={investor.kycStatus} />
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Eligibility</span>
              <StatusBadge status={investor.eligibilityStatus} />
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Account Status</span>
              <StatusBadge status={investor.accountStatus} />
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Jurisdiction</span>
              <span>{investor.jurisdiction}</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Account Security</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 pb-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-foreground">Two-factor authentication</p>
                <p className="text-xs text-muted-foreground">Require a verification code at login.</p>
              </div>
              <Switch
                checked={twoFactorEnabled}
                onCheckedChange={(checked) => {
                  setTwoFactorEnabled(checked);
                  toast.success(checked ? "Two-factor authentication enabled." : "Two-factor authentication disabled.");
                }}
              />
            </div>
            <Separator />
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-foreground">Password</p>
                <p className="text-xs text-muted-foreground">Last changed 4 months ago.</p>
              </div>
              <Button variant="outline" size="sm">
                Change password
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Notification Preferences</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 pb-6">
            {Object.entries(PREFERENCE_LABELS).map(([key, label]) => (
              <div key={key} className="flex items-center justify-between">
                <p className="text-sm text-foreground">{label}</p>
                <Switch
                  checked={preferences[key as Parameters<typeof preferences.setPreference>[0]]}
                  onCheckedChange={(checked) =>
                    preferences.setPreference(key as Parameters<typeof preferences.setPreference>[0], checked)
                  }
                />
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
