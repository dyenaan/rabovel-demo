import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { ComplianceDashboard } from "@/features/compliance/components/compliance-dashboard";

export const metadata: Metadata = { title: "Compliance — Admin" };

export default function AdminCompliancePage() {
  return (
    <PageContainer>
      <PageHeader title="Compliance" description="Review KYC cases, eligibility, and investor restrictions." />
      <ComplianceDashboard />
    </PageContainer>
  );
}
