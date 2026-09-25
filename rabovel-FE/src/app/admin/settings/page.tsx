"use client";

import { useState } from "react";
import { toast } from "sonner";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";

export default function AdminSettingsPage() {
  const [maintenanceMode, setMaintenanceMode] = useState(false);
  const [newIssuanceRequiresReview, setNewIssuanceRequiresReview] = useState(true);
  const [autoReconciliation, setAutoReconciliation] = useState(true);

  return (
    <PageContainer>
      <PageHeader title="Platform Settings" description="Operational controls for the Rabovel platform." />

      <Card className="max-w-2xl">
        <CardHeader>
          <CardTitle>Operations</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 pb-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-foreground">Maintenance mode</p>
              <p className="text-xs text-muted-foreground">Temporarily disable new order placement platform-wide.</p>
            </div>
            <Switch checked={maintenanceMode} onCheckedChange={setMaintenanceMode} />
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-foreground">New issuance requires review</p>
              <p className="text-xs text-muted-foreground">Draft issuances must be approved before going live.</p>
            </div>
            <Switch checked={newIssuanceRequiresReview} onCheckedChange={setNewIssuanceRequiresReview} />
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-foreground">Automatic reconciliation</p>
              <p className="text-xs text-muted-foreground">Run ledger/custody/on-chain reconciliation every 15 minutes.</p>
            </div>
            <Switch checked={autoReconciliation} onCheckedChange={setAutoReconciliation} />
          </div>
          <div className="pt-2">
            <Button onClick={() => toast.success("Settings saved.")}>Save Changes</Button>
          </div>
        </CardContent>
      </Card>
    </PageContainer>
  );
}
