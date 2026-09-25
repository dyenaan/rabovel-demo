import { notFound } from "next/navigation";
import { Users } from "lucide-react";

import { PageContainer } from "@/components/layout/page-container";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { mockComplianceCases } from "@/mocks/compliance.mock";
import { mockInvestors } from "@/mocks/investors.mock";
import { formatDate } from "@/lib/formatters";

export default async function AdminInvestorDetailPage(props: PageProps<"/admin/investors/[investorId]">) {
  const { investorId } = await props.params;
  const investor = mockInvestors.find((i) => i.investorId === investorId);

  if (!investor) notFound();

  const complianceCase = mockComplianceCases.find((c) => c.investorId === investorId);

  return (
    <PageContainer>
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-foreground">{investor.fullName}</h1>
        <p className="text-sm text-muted-foreground">{investor.email}</p>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Account Overview</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 pb-6 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Investor ID</span>
              <span className="font-mono text-xs">{investor.investorId}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Account Type</span>
              <span>{investor.accountType}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Jurisdiction</span>
              <span>{investor.jurisdiction}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">KYC Status</span>
              <StatusBadge status={investor.kycStatus} />
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Eligibility</span>
              <StatusBadge status={investor.eligibilityStatus} />
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Account Status</span>
              <StatusBadge status={investor.accountStatus} />
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Joined</span>
              <span>{formatDate(investor.createdAt)}</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Compliance Case</CardTitle>
          </CardHeader>
          <CardContent className="pb-6">
            {complianceCase ? (
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Status</span>
                  <StatusBadge status={complianceCase.status} />
                </div>
                {complianceCase.reason && (
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Reason</span>
                    <span>{complianceCase.reason.replaceAll("_", " ")}</span>
                  </div>
                )}
                {complianceCase.notes && (
                  <p className="rounded-md bg-muted/40 p-3 text-xs text-muted-foreground">{complianceCase.notes}</p>
                )}
              </div>
            ) : (
              <EmptyState icon={Users} title="No open compliance case" />
            )}
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
