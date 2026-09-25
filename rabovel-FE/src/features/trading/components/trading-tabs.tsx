"use client";

import { useMemo } from "react";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { OrdersTable } from "@/features/orders/components/orders-table";
import { useOrders } from "@/features/orders/hooks/use-orders";
import { TradesTable } from "@/features/trades/components/trades-table";
import { OPEN_ORDER_STATUSES } from "@/types";

export function TradingTabs({ marketId }: { marketId: string }) {
  const { data: orders, isPending } = useOrders();

  const openOrders = useMemo(
    () => (orders ?? []).filter((o) => o.marketId === marketId && OPEN_ORDER_STATUSES.includes(o.status)),
    [orders, marketId],
  );
  const marketOrders = useMemo(
    () => (orders ?? []).filter((o) => o.marketId === marketId),
    [orders, marketId],
  );

  return (
    <Tabs defaultValue="open-orders">
      <TabsList>
        <TabsTrigger value="open-orders">Open Orders ({openOrders.length})</TabsTrigger>
        <TabsTrigger value="order-history">Order History</TabsTrigger>
        <TabsTrigger value="trade-history">Trade History</TabsTrigger>
      </TabsList>
      <TabsContent value="open-orders" className="pt-3">
        <OrdersTable orders={openOrders} isLoading={isPending} />
      </TabsContent>
      <TabsContent value="order-history" className="pt-3">
        <OrdersTable orders={marketOrders} isLoading={isPending} />
      </TabsContent>
      <TabsContent value="trade-history" className="pt-3">
        <TradesTable />
      </TabsContent>
    </Tabs>
  );
}
