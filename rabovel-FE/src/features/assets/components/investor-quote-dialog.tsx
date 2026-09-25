"use client";

import { FormEvent, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { InvestorCatalogAsset } from "@/types";
import { useInvestorQuote } from "../hooks/use-assets";

type Props = {
  asset: InvestorCatalogAsset;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  paymentDecimals: number;
};

export function InvestorQuoteDialog({ asset, open, onOpenChange, paymentDecimals }: Props) {
  const [quantity, setQuantity] = useState("1");
  const quote = useInvestorQuote();

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setQuantity("1");
      quote.reset();
    }
    onOpenChange(nextOpen);
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!/^\d+$/.test(quantity) || quantity === "0") return;
    quote.mutate({ assetId: asset.asset_id, quantity });
  }

  const validQuantity = /^\d+$/.test(quantity) && quantity !== "0";

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Buy {asset.ticker}</DialogTitle>
          <DialogDescription>
            Request a short-lived quote against the live issuer inventory and your cNGN balance.
          </DialogDescription>
        </DialogHeader>

        {quote.data ? (
          <div className="space-y-4">
            <div className="grid gap-3 rounded-lg border p-4 text-sm sm:grid-cols-2">
              <QuoteValue label="Units" value={quote.data.quantity} />
              <QuoteValue
                label="Price per unit"
                value={`${formatBaseUnits(quote.data.price_per_unit, paymentDecimals)} cNGN`}
              />
              <QuoteValue
                label={`Fee (${quote.data.fee_bps / 100}%)`}
                value={`${formatBaseUnits(quote.data.fee_amount, paymentDecimals)} cNGN`}
              />
              <QuoteValue
                label="Total payment"
                value={`${formatBaseUnits(quote.data.total_payment, paymentDecimals)} cNGN`}
              />
            </div>
            <p className="text-xs text-muted-foreground">
              Quote expires {new Date(quote.data.expires_at * 1000).toLocaleTimeString()}. This
              quote validates funds and inventory; settlement confirmation is not enabled yet.
            </p>
            <DialogFooter>
              <Button variant="outline" onClick={() => quote.reset()}>
                Change quantity
              </Button>
              <Button disabled>Confirm purchase (coming next)</Button>
            </DialogFooter>
          </div>
        ) : (
          <form className="space-y-4" onSubmit={submit}>
            <div className="space-y-2">
              <Label htmlFor={`quantity-${asset.asset_id}`}>Whole units</Label>
              <Input
                id={`quantity-${asset.asset_id}`}
                inputMode="numeric"
                min="1"
                max={asset.issuer_inventory}
                step="1"
                type="number"
                value={quantity}
                onChange={(event) => setQuantity(event.target.value)}
                aria-invalid={!validQuantity}
              />
              <p className="text-xs text-muted-foreground">
                {Number(asset.issuer_inventory).toLocaleString()} units currently available.
              </p>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={!validQuantity || quote.isPending}>
                {quote.isPending ? "Checking funds…" : "Review quote"}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}

function QuoteValue({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="mt-1 font-medium tabular-nums">{value}</p>
    </div>
  );
}

function formatBaseUnits(value: string, decimals: number) {
  const padded = value.padStart(decimals + 1, "0");
  const whole = decimals ? padded.slice(0, -decimals) : padded;
  const fraction = decimals ? padded.slice(-decimals).replace(/0+$/, "") : "";
  return `${whole}${fraction ? `.${fraction}` : ""}`;
}
