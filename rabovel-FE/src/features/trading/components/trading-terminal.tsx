"use client";

import { Card, CardContent } from "@/components/ui/card";
import { ErrorState } from "@/components/shared/error-state";
import { useMarket } from "@/features/markets/hooks/use-markets";
import { useTradingPreferencesStore } from "@/stores/trading-preferences-store";
import { MarketHeader } from "./market-header";
import { MobileTradingView } from "./mobile-trading-view";
import { OrderBook } from "./order-book";
import { OrderForm } from "./order-form";
import { PriceChart } from "./price-chart";
import { RecentTrades } from "./recent-trades";
import { TradingTabs } from "./trading-tabs";

export function TradingTerminal({ marketId }: { marketId: string }) {
  const { data: market, isError, refetch } = useMarket(marketId);
  const setSelectedMarketId = useTradingPreferencesStore((s) => s.setSelectedMarketId);

  function handleMarketChange(nextMarketId: string) {
    setSelectedMarketId(nextMarketId);
    const url = new URL(window.location.href);
    url.searchParams.set("market", nextMarketId);
    window.history.replaceState({}, "", url);
  }

  if (isError) {
    return <ErrorState onRetry={() => refetch()} />;
  }

  return (
    <div className="space-y-4">
      <MarketHeader market={market} onMarketChange={handleMarketChange} />

      {/* Desktop layout */}
      <div className="hidden gap-4 lg:grid lg:grid-cols-[1fr_320px]">
        <Card>
          <CardContent className="pt-6">{market && <PriceChart marketId={market.marketId} />}</CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">{market && <OrderForm market={market} />}</CardContent>
        </Card>

        <Card>
          <CardContent className="pt-4">{market && <OrderBook marketId={market.marketId} />}</CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">{market && <RecentTrades marketId={market.marketId} />}</CardContent>
        </Card>

        <div className="lg:col-span-2">{market && <TradingTabs marketId={market.marketId} />}</div>
      </div>

      {/* Mobile / tablet layout */}
      <div className="lg:hidden">{market && <MobileTradingView market={market} />}</div>
    </div>
  );
}
