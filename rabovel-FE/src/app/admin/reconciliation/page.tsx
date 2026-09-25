import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { ReconciliationDashboard } from "@/features/reconciliation/components/reconciliation-dashboard";

export const metadata: Metadata = { title: "Reconciliation — Admin" };

export default function AdminReconciliationPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Reconciliation"
        description="Compare internal ledger, custody, and on-chain balances across all assets."
      />
      <ReconciliationDashboard />
    </PageContainer>
  );
}
