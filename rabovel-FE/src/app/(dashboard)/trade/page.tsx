"use client";

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";

import { PageContainer } from "@/components/layout/page-container";
import { LoadingState } from "@/components/shared/loading-state";
import { TradingTerminal } from "@/features/trading/components/trading-terminal";
import { useTradingPreferencesStore } from "@/stores/trading-preferences-store";

function TradePageContent() {
  const searchParams = useSearchParams();
  const selectedMarketId = useTradingPreferencesStore((s) => s.selectedMarketId);
  const marketId = searchParams.get("market") ?? selectedMarketId;

  return <TradingTerminal marketId={marketId} />;
}

export default function TradePage() {
  return (
    <PageContainer className="max-w-none">
      <Suspense fallback={<LoadingState label="Loading trading terminal…" />}>
        <TradePageContent />
      </Suspense>
    </PageContainer>
  );
}
