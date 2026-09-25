"use client";

import { useState } from "react";
import { Plus, ShieldCheck } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useConnectWallet } from "../hooks/use-wallets";

export function ConnectWalletDialog() {
  const [open, setOpen] = useState(false);
  const connectWallet = useConnectWallet();

  function handleSubmit() {
    connectWallet.mutate(
      undefined,
      {
        onSuccess: () => {
          setOpen(false);
        },
      },
    );
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          <Plus className="size-4" />
          Bind Wallet
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Bind a custody wallet</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 text-sm">
          <div className="flex gap-3 rounded-lg border bg-muted/30 p-4"><ShieldCheck className="mt-0.5 size-5 shrink-0 text-primary" /><div><p className="font-medium">Cryptographic ownership check</p><p className="mt-1 text-muted-foreground">Phantom will connect your Solana address and ask you to sign a one-time Rabovel message. Signing does not submit a transaction or spend SOL.</p></div></div>
          <p className="text-xs text-muted-foreground">
            Rabovel never requests your private key or seed phrase. Reject the request if the message is not from this application.
          </p>
          {connectWallet.error && <p role="alert" className="text-sm text-destructive">{connectWallet.error.message}</p>}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={connectWallet.isPending}>
            {connectWallet.isPending ? "Waiting for Phantom…" : "Connect and verify Phantom"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
