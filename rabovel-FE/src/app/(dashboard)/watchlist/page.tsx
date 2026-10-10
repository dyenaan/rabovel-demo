import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { MarketsTable } from "@/features/markets/components/markets-table";

export const metadata: Metadata = { title: "Watchlist" };

export default function WatchlistPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Watchlist"
        description="146 NGX-listed equities modeled as minted and tradable. Prices, volumes, backing, and mint status are simulated; no underlying shares or real tokens are held."
      />
      <MarketsTable watchlistOnly />
    </PageContainer>
  );
}