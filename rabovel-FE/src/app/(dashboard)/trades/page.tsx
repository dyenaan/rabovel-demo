import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { TradesTable } from "@/features/trades/components/trades-table";

export const metadata: Metadata = { title: "Trades" };

export default function TradesPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Trades"
        description="Executed trades and their independent settlement status."
      />
      <TradesTable />
    </PageContainer>
  );
}
