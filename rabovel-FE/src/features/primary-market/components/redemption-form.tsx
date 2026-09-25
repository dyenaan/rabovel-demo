"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
import { createIdempotencyKey } from "@/lib/api/client";
import { formatQuantity } from "@/lib/formatters";
import { useHoldings } from "@/features/portfolio/hooks/use-portfolio";
import { useSubmitRedemption } from "../hooks/use-primary-market";
import { redemptionSchema, type RedemptionFormValues } from "../schemas/subscription.schema";

export function RedemptionForm({ defaultAssetId }: { defaultAssetId?: string }) {
  const { data: holdings } = useHoldings();
  const submitRedemption = useSubmitRedemption();

  const form = useForm<RedemptionFormValues>({
    resolver: zodResolver(redemptionSchema),
    defaultValues: { assetId: defaultAssetId ?? "", quantity: "" },
  });

  const selectedHolding = holdings?.find((h) => h.assetId === form.watch("assetId"));
  const quantity = form.watch("quantity");
  const exceedsHolding = selectedHolding && quantity ? Number(quantity) > Number(selectedHolding.quantity) : false;

  function onSubmit(values: RedemptionFormValues) {
    submitRedemption.mutate(
      { assetId: values.assetId, quantity: values.quantity, idempotencyKey: createIdempotencyKey() },
      { onSuccess: () => form.reset({ assetId: values.assetId, quantity: "" }) },
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Redeem</CardTitle>
      </CardHeader>
      <CardContent className="pb-6">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="assetId"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Holding</FormLabel>
                  <Select value={field.value} onValueChange={field.onChange}>
                    <FormControl>
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="Select a holding" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {(holdings ?? []).map((holding) => (
                        <SelectItem key={holding.assetId} value={holding.assetId}>
                          {holding.assetName} ({holding.symbol})
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            {selectedHolding && (
              <div className="rounded-md bg-muted/40 p-3 text-xs text-muted-foreground">
                Available: {formatQuantity(selectedHolding.quantity)} {selectedHolding.symbol}
              </div>
            )}

            <FormField
              control={form.control}
              name="quantity"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Redemption Quantity</FormLabel>
                  <FormControl>
                    <Input inputMode="decimal" placeholder="0.00" {...field} />
                  </FormControl>
                  {exceedsHolding && (
                    <p className="text-xs text-destructive">Quantity exceeds your available holding.</p>
                  )}
                  <FormMessage />
                </FormItem>
              )}
            />

            <Button
              type="submit"
              className="w-full"
              disabled={submitRedemption.isPending || exceedsHolding}
            >
              {submitRedemption.isPending ? "Submitting…" : "Submit Redemption"}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}
