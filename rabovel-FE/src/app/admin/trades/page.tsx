import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { TradesTable } from "@/features/trades/components/trades-table";

export const metadata: Metadata = { title: "Trades — Admin" };

export default function AdminTradesPage() {
  return (
    <PageContainer>
      <PageHeader title="Trades" description="Platform-wide executed trades and settlement linkage." />
      <TradesTable />
    </PageContainer>
  );
}
