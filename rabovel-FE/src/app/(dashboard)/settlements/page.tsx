import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { SettlementsTable } from "@/features/settlements/components/settlements-table";

export const metadata: Metadata = { title: "Settlements" };

export default function SettlementsPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Settlements"
        description="Track asset and cash leg settlement through to on-chain finality."
      />
      <SettlementsTable />
    </PageContainer>
  );
}
