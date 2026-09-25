"use client";

import { ListOrdered } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { ErrorState } from "@/components/shared/error-state";
import { LoadingState } from "@/components/shared/loading-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatDateTime, formatPrice } from "@/lib/formatters";
import { useOrder } from "../hooks/use-orders";
import { OrderProgress } from "./order-progress";

export function OrderDetailView({ orderId }: { orderId: string }) {
  const { data: order, isPending, isError, refetch } = useOrder(orderId);

  if (isPending) return <LoadingState label="Loading order…" />;
  if (isError) return <ErrorState onRetry={() => refetch()} />;
  if (!order) return <EmptyState icon={ListOrdered} title="Order not found" />;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center gap-3">
        <h1 className="text-xl font-semibold text-foreground">{order.marketSymbol}</h1>
        <Badge variant={order.side === "BUY" ? "success" : "destructive"}>{order.side}</Badge>
        <StatusBadge status={order.status} />
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Fill Progress</CardTitle>
          </CardHeader>
          <CardContent className="pb-6">
            <OrderProgress order={order} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Order Details</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 pb-6 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Order ID</span>
              <span className="font-mono text-xs">{order.orderId}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Order Type</span>
              <span>{order.orderType}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Limit Price</span>
              <span className="font-tabular">
                {order.limitPrice ? formatPrice(order.limitPrice) : "Market"}
              </span>
            </div>
            {order.averageFillPrice && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Avg. Fill Price</span>
                <span className="font-tabular">{formatPrice(order.averageFillPrice)}</span>
              </div>
            )}
            <div className="flex justify-between">
              <span className="text-muted-foreground">Created</span>
              <span>{formatDateTime(order.createdAt)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Updated</span>
              <span>{formatDateTime(order.updatedAt)}</span>
            </div>
            {order.rejectionReason && (
              <div className="rounded-md bg-destructive/10 p-2 text-xs text-destructive">
                {order.rejectionReason}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
