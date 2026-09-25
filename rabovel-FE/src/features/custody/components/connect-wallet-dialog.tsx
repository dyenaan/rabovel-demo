"use client";

import { useState } from "react";
import { Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { Blockchain } from "@/types";
import { useConnectWallet } from "../hooks/use-wallets";

const NETWORKS: Blockchain[] = ["ETHEREUM", "POLYGON", "ARBITRUM", "BASE", "SOLANA", "AVALANCHE"];

export function ConnectWalletDialog() {
  const [open, setOpen] = useState(false);
  const [label, setLabel] = useState("");
  const [address, setAddress] = useState("");
  const [blockchain, setBlockchain] = useState<Blockchain>("ETHEREUM");
  const connectWallet = useConnectWallet();

  function handleSubmit() {
    connectWallet.mutate(
      { label, address, blockchain },
      {
        onSuccess: () => {
          setOpen(false);
          setLabel("");
          setAddress("");
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
        <div className="space-y-4">
          <div className="space-y-1.5">
            <Label>Label</Label>
            <Input value={label} onChange={(e) => setLabel(e.target.value)} placeholder="e.g. Treasury Vault" />
          </div>
          <div className="space-y-1.5">
            <Label>Network</Label>
            <Select value={blockchain} onValueChange={(v) => setBlockchain(v as Blockchain)}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {NETWORKS.map((network) => (
                  <SelectItem key={network} value={network}>
                    {network.charAt(0) + network.slice(1).toLowerCase()}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label>Wallet Address</Label>
            <Input value={address} onChange={(e) => setAddress(e.target.value)} placeholder="0x..." className="font-mono" />
          </div>
          <p className="text-xs text-muted-foreground">
            Verification is required before this wallet can receive settlement. We never request your private key or seed phrase.
          </p>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!label || !address || connectWallet.isPending}>
            {connectWallet.isPending ? "Binding…" : "Bind Wallet"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
