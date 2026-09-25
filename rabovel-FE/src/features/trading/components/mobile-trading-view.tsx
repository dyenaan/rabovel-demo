"use client";

import { useState } from "react";
import { ShoppingCart } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { Market } from "@/types";
import { OrderBook } from "./order-book";
import { OrderForm } from "./order-form";
import { PriceChart } from "./price-chart";
import { RecentTrades } from "./recent-trades";
import { TradingTabs } from "./trading-tabs";

export function MobileTradingView({ market }: { market: Market }) {
  const [orderSheetOpen, setOrderSheetOpen] = useState(false);

  return (
    <div className="space-y-4 pb-20">
      <PriceChart marketId={market.marketId} />

      <Tabs defaultValue="book">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="book">Order Book</TabsTrigger>
          <TabsTrigger value="trades">Trades</TabsTrigger>
          <TabsTrigger value="orders">My Orders</TabsTrigger>
        </TabsList>
        <TabsContent value="book" className="pt-3">
          <OrderBook marketId={market.marketId} />
        </TabsContent>
        <TabsContent value="trades" className="pt-3">
          <RecentTrades marketId={market.marketId} />
        </TabsContent>
        <TabsContent value="orders" className="pt-3">
          <TradingTabs marketId={market.marketId} />
        </TabsContent>
      </Tabs>

      <div className="fixed inset-x-0 bottom-0 z-20 border-t bg-background p-3">
        <Button className="w-full" size="lg" onClick={() => setOrderSheetOpen(true)}>
          <ShoppingCart className="size-4" />
          Place Order
        </Button>
      </div>

      <Sheet open={orderSheetOpen} onOpenChange={setOrderSheetOpen}>
        <SheetContent side="bottom" className="max-h-[85vh] overflow-y-auto rounded-t-xl">
          <SheetHeader>
            <SheetTitle>{market.symbol}</SheetTitle>
          </SheetHeader>
          <div className="px-4 pb-6">
            <OrderForm market={market} />
          </div>
        </SheetContent>
      </Sheet>
    </div>
  );
}
