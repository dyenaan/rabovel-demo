import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { InvestorsTable } from "@/features/admin/components/investors-table";

export const metadata: Metadata = { title: "Investors — Admin" };

export default function AdminInvestorsPage() {
  return (
    <PageContainer>
      <PageHeader title="Investors" description="Review KYC status, eligibility, and account standing." />
      <InvestorsTable />
    </PageContainer>
  );
}
