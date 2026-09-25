"use client";

import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { mockDelay } from "@/lib/api/mock-delay";
import type { AssetClass } from "@/types";

const ASSET_CLASSES: AssetClass[] = [
  "TREASURY",
  "PRIVATE_CREDIT",
  "REAL_ESTATE",
  "INFRASTRUCTURE",
  "COMMODITY",
  "EQUITY",
];

export function IssuanceForm() {
  const [name, setName] = useState("");
  const [symbol, setSymbol] = useState("");
  const [assetClass, setAssetClass] = useState<AssetClass>("TREASURY");
  const [minimumInvestment, setMinimumInvestment] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit() {
    setIsSubmitting(true);
    await mockDelay(800);
    setIsSubmitting(false);
    toast.success(`${name || "New asset"} created as draft, pending approval.`);
    setName("");
    setSymbol("");
    setMinimumInvestment("");
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>New Issuance</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4 pb-6">
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-1.5">
            <Label>Asset Name</Label>
            <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. Rabovel Green Bond Fund" />
          </div>
          <div className="space-y-1.5">
            <Label>Symbol</Label>
            <Input value={symbol} onChange={(e) => setSymbol(e.target.value.toUpperCase())} placeholder="e.g. RGB" />
          </div>
          <div className="space-y-1.5">
            <Label>Asset Class</Label>
            <Select value={assetClass} onValueChange={(v) => setAssetClass(v as AssetClass)}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {ASSET_CLASSES.map((cls) => (
                  <SelectItem key={cls} value={cls}>
                    {cls.replaceAll("_", " ")}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label>Minimum Investment (NGN)</Label>
            <Input
              inputMode="decimal"
              value={minimumInvestment}
              onChange={(e) => setMinimumInvestment(e.target.value)}
              placeholder="10000"
            />
          </div>
        </div>
        <Button onClick={handleSubmit} disabled={!name || !symbol || isSubmitting}>
          {isSubmitting ? "Creating…" : "Create Draft Issuance"}
        </Button>
      </CardContent>
    </Card>
  );
}
