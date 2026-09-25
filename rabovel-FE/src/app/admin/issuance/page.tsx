import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { StatusBadge } from "@/components/shared/status-badge";
import { IssuanceForm } from "@/features/admin/components/issuance-form";
import { mockAssets } from "@/mocks/assets.mock";

export const metadata: Metadata = { title: "Issuance — Admin" };

export default function AdminIssuancePage() {
  return (
    <PageContainer>
      <PageHeader title="Issuance" description="Create new tokenized offerings and track pipeline status." />

      <div className="grid gap-6 lg:grid-cols-2">
        <IssuanceForm />

        <Card>
          <CardHeader>
            <CardTitle>Issuance Pipeline</CardTitle>
          </CardHeader>
          <CardContent className="pb-6">
            <ul className="divide-y">
              {mockAssets.map((asset) => (
                <li key={asset.assetId} className="flex items-center justify-between py-3 first:pt-0 last:pb-0">
                  <div>
                    <p className="text-sm font-medium text-foreground">{asset.name}</p>
                    <Badge variant="secondary" className="mt-1 text-[10px]">
                      {asset.assetClass.replaceAll("_", " ")}
                    </Badge>
                  </div>
                  <StatusBadge status={asset.lifecycleStatus} />
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
