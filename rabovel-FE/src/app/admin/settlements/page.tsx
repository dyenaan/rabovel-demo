import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { SettlementsTable } from "@/features/settlements/components/settlements-table";

export const metadata: Metadata = { title: "Settlements — Admin" };

export default function AdminSettlementsPage() {
  return (
    <PageContainer>
      <PageHeader title="Settlements" description="Monitor settlement lifecycle and finality across all trades." />
      <SettlementsTable />
    </PageContainer>
  );
}
