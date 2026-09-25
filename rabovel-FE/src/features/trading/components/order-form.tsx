"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { CheckCircle2, XCircle } from "lucide-react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { createIdempotencyKey } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { formatMoney } from "@/lib/formatters";
import { useAuth } from "@/features/auth/hooks/use-auth";
import { calculateOrderTotal } from "@/features/orders/api/place-order";
import { usePlaceOrder } from "@/features/orders/hooks/use-place-order";
import { usePortfolioSummary } from "@/features/portfolio/hooks/use-portfolio";
import { mockInvestors } from "@/mocks/investors.mock";
import type { Market } from "@/types";
import { orderFormSchema, type OrderFormValues } from "../schemas/order.schema";

function ValidationRow({ ok, label }: { ok: boolean; label: string }) {
  return (
    <div className={cn("flex items-center gap-2 text-xs", ok ? "text-success" : "text-muted-foreground")}>
      {ok ? <CheckCircle2 className="size-3.5" /> : <XCircle className="size-3.5" />}
      {label}
    </div>
  );
}

export function OrderForm({ market }: { market: Market }) {
  const { user } = useAuth();
  const { data: portfolio } = usePortfolioSummary();
  const placeOrder = usePlaceOrder();

  const investor = mockInvestors.find((i) => i.investorId === user?.investorId) ?? mockInvestors[0];
  const isEligible = investor.eligibilityStatus === "ELIGIBLE";
  const isMarketOpen = market.status === "OPEN";

  const form = useForm<OrderFormValues>({
    resolver: zodResolver(orderFormSchema),
    defaultValues: { side: "BUY", orderType: "LIMIT", limitPrice: market.lastPrice, quantity: "" },
  });

  useEffect(() => {
    form.setValue("limitPrice", market.lastPrice);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [market.marketId]);

  const side = form.watch("side");
  const orderType = form.watch("orderType");
  const quantity = form.watch("quantity");
  const limitPrice = form.watch("limitPrice");

  const total = calculateOrderTotal(quantity, orderType === "MARKET" ? market.lastPrice : limitPrice);
  const hasSufficientBalance =
    side === "SELL" || (portfolio && total !== "—" && Number(portfolio.cashBalance) >= Number(total));

  function onSubmit(values: OrderFormValues) {
    placeOrder.mutate(
      {
        marketId: market.marketId,
        marketSymbol: market.symbol,
        side: values.side,
        orderType: values.orderType,
        limitPrice: values.orderType === "MARKET" ? undefined : values.limitPrice,
        quantity: values.quantity,
        idempotencyKey: createIdempotencyKey(),
      },
      { onSuccess: () => form.reset({ side: values.side, orderType: values.orderType, limitPrice: market.lastPrice, quantity: "" }) },
    );
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="side"
          render={({ field }) => (
            <Tabs value={field.value} onValueChange={field.onChange}>
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="BUY" className="data-[state=active]:bg-success data-[state=active]:text-success-foreground">
                  Buy
                </TabsTrigger>
                <TabsTrigger value="SELL" className="data-[state=active]:bg-danger data-[state=active]:text-danger-foreground">
                  Sell
                </TabsTrigger>
              </TabsList>
            </Tabs>
          )}
        />

        <FormField
          control={form.control}
          name="orderType"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Order Type</FormLabel>
              <Select value={field.value} onValueChange={field.onChange}>
                <FormControl>
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  <SelectItem value="MARKET">Market</SelectItem>
                  <SelectItem value="LIMIT">Limit</SelectItem>
                  <SelectItem value="STOP_LIMIT">Stop Limit</SelectItem>
                </SelectContent>
              </Select>
            </FormItem>
          )}
        />

        {orderType !== "MARKET" && (
          <FormField
            control={form.control}
            name="limitPrice"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Price</FormLabel>
                <FormControl>
                  <Input inputMode="decimal" placeholder="0.00" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        )}

        <FormField
          control={form.control}
          name="quantity"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Quantity</FormLabel>
              <FormControl>
                <Input inputMode="decimal" placeholder="0.00" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <div className="flex items-center justify-between rounded-md bg-muted/40 px-3 py-2 text-sm">
          <span className="text-muted-foreground">Estimated Total</span>
          <span className="font-tabular font-medium">{total === "—" ? "—" : formatMoney(total)}</span>
        </div>

        <div className="space-y-1.5 border-t pt-3">
          <ValidationRow ok={isEligible} label="Eligible to trade" />
          <ValidationRow ok={!!hasSufficientBalance} label="Sufficient balance" />
          <ValidationRow ok={isMarketOpen} label="Market open" />
        </div>

        <Button
          type="submit"
          className={cn("w-full", side === "BUY" ? "bg-success hover:bg-success/90" : "bg-danger hover:bg-danger/90")}
          disabled={!isEligible || !isMarketOpen || !hasSufficientBalance || placeOrder.isPending}
        >
          {placeOrder.isPending ? "Placing order…" : `${side === "BUY" ? "Buy" : "Sell"} ${market.baseAsset}`}
        </Button>
      </form>
    </Form>
  );
}
