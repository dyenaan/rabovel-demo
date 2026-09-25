"use client";

import { Eye } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Decimal, formatDateTime, formatMoney } from "@/lib/formatters";
import type { ReconciliationRecord } from "@/types";

export function MismatchDetails({ record }: { record: ReconciliationRecord }) {
  const ledgerVsCustody = new Decimal(record.ledgerBalance).minus(record.custodyBalance);
  const custodyVsChain = new Decimal(record.custodyBalance).minus(record.onChainBalance);

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="View details">
          <Eye className="size-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{record.assetName}</DialogTitle>
        </DialogHeader>
        <div className="space-y-3 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Internal Ledger</span>
            <span className="font-tabular">{formatMoney(record.ledgerBalance)}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Custody</span>
            <span className="font-tabular">{formatMoney(record.custodyBalance)}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">On-Chain</span>
            <span className="font-tabular">{formatMoney(record.onChainBalance)}</span>
          </div>
          <div className="border-t pt-3">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Ledger vs. Custody Delta</span>
              <span className={`font-tabular ${ledgerVsCustody.isZero() ? "text-success" : "text-destructive"}`}>
                {formatMoney(ledgerVsCustody.toString())}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Custody vs. On-Chain Delta</span>
              <span className={`font-tabular ${custodyVsChain.isZero() ? "text-success" : "text-destructive"}`}>
                {formatMoney(custodyVsChain.toString())}
              </span>
            </div>
          </div>
          <div className="flex justify-between border-t pt-3">
            <span className="text-muted-foreground">Last Checked</span>
            <span>{formatDateTime(record.lastCheckedAt)}</span>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
