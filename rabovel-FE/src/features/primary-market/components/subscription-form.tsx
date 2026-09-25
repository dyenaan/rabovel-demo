"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
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
import { formatMoney } from "@/lib/formatters";
import { useAssets } from "@/features/assets/hooks/use-assets";
import { useSubmitSubscription } from "../hooks/use-primary-market";
import { subscriptionSchema, type SubscriptionFormValues } from "../schemas/subscription.schema";

export function SubscriptionForm({ defaultAssetId }: { defaultAssetId?: string }) {
  const { data: assets } = useAssets();
  const submitSubscription = useSubmitSubscription();

  const form = useForm<SubscriptionFormValues>({
    resolver: zodResolver(subscriptionSchema),
    defaultValues: {
      assetId: defaultAssetId ?? "",
      amount: "",
      acknowledgeRisk: false as unknown as true,
    },
  });

  const selectedAsset = assets?.find((a) => a.assetId === form.watch("assetId"));
  const amount = form.watch("amount");
  const belowMinimum =
    selectedAsset?.minimumInvestment && amount
      ? Number(amount) < Number(selectedAsset.minimumInvestment)
      : false;

  function onSubmit(values: SubscriptionFormValues) {
    submitSubscription.mutate(
      { assetId: values.assetId, amount: values.amount, idempotencyKey: createIdempotencyKey() },
      { onSuccess: () => form.reset({ assetId: values.assetId, amount: "", acknowledgeRisk: false as unknown as true }) },
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Subscribe</CardTitle>
      </CardHeader>
      <CardContent className="pb-6">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="assetId"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Asset</FormLabel>
                  <Select value={field.value} onValueChange={field.onChange}>
                    <FormControl>
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="Select an asset" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {(assets ?? [])
                        .filter((a) => a.lifecycleStatus === "SUBSCRIPTION_OPEN" || a.lifecycleStatus === "ACTIVE")
                        .map((asset) => (
                          <SelectItem key={asset.assetId} value={asset.assetId}>
                            {asset.name} ({asset.symbol})
                          </SelectItem>
                        ))}
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            {selectedAsset && (
              <div className="rounded-md bg-muted/40 p-3 text-xs text-muted-foreground">
                Minimum investment: {formatMoney(selectedAsset.minimumInvestment)} · NAV: {formatMoney(selectedAsset.nav)}
              </div>
            )}

            <FormField
              control={form.control}
              name="amount"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Subscription Amount (NGN)</FormLabel>
                  <FormControl>
                    <Input inputMode="decimal" placeholder="0.00" {...field} />
                  </FormControl>
                  {belowMinimum && (
                    <p className="text-xs text-warning-foreground">
                      Amount is below the minimum investment for this asset.
                    </p>
                  )}
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="acknowledgeRisk"
              render={({ field }) => (
                <FormItem>
                  <div className="flex items-start gap-2">
                    <FormControl>
                      <Checkbox checked={field.value} onCheckedChange={field.onChange} />
                    </FormControl>
                    <FormLabel className="text-sm font-normal text-muted-foreground">
                      I have reviewed the offering documents and understand the risks of this investment.
                    </FormLabel>
                  </div>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Button type="submit" className="w-full" disabled={submitSubscription.isPending}>
              {submitSubscription.isPending ? "Submitting…" : "Submit Subscription"}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}
