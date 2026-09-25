"use client";

import { useMemo } from "react";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { OrdersTable } from "@/features/orders/components/orders-table";
import { useOrders } from "@/features/orders/hooks/use-orders";
import { CLOSED_ORDER_STATUSES, OPEN_ORDER_STATUSES } from "@/types";

export default function OrdersPage() {
  const { data, isPending } = useOrders();

  const openOrders = useMemo(
    () => (data ?? []).filter((o) => OPEN_ORDER_STATUSES.includes(o.status)),
    [data],
  );
  const orderHistory = useMemo(
    () => (data ?? []).filter((o) => CLOSED_ORDER_STATUSES.includes(o.status)),
    [data],
  );

  return (
    <PageContainer>
      <PageHeader title="Orders" description="Manage your open orders and review order history." />

      <Tabs defaultValue="open">
        <TabsList>
          <TabsTrigger value="open">Open ({openOrders.length})</TabsTrigger>
          <TabsTrigger value="history">History</TabsTrigger>
          <TabsTrigger value="all">All</TabsTrigger>
        </TabsList>
        <TabsContent value="open" className="pt-4">
          <OrdersTable orders={openOrders} isLoading={isPending} />
        </TabsContent>
        <TabsContent value="history" className="pt-4">
          <OrdersTable orders={orderHistory} isLoading={isPending} />
        </TabsContent>
        <TabsContent value="all" className="pt-4">
          <OrdersTable orders={data ?? []} isLoading={isPending} />
        </TabsContent>
      </Tabs>
    </PageContainer>
  );
}
