import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { MarketsTable } from "@/features/markets/components/markets-table";

export const metadata: Metadata = { title: "Markets" };

export default function MarketsPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Markets"
        description="Browse simulated NGX equities and demo markets. Prices are fixtures, not live exchange quotes."
      />
      <MarketsTable />
    </PageContainer>
  );
}
