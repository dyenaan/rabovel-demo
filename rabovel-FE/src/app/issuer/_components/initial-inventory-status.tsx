"use client";

import { useEffect, useState } from "react";
import { Check, CheckCircle2, Copy, RefreshCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { fetchInitialInventory, issueInitialInventory, type InitialInventory } from "@/features/auth/api/backend-auth";

export function InitialInventoryStatus({ token, assetId }: { token: string; assetId: string }) {
  const [inventory, setInventory] = useState<InitialInventory | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [issuing, setIssuing] = useState(false);

  async function load() {
    setLoading(true); setError(null);
    try { setInventory(await fetchInitialInventory(token, assetId)); }
    catch (reason) { setError(reason instanceof Error ? reason.message : "Could not read initial inventory."); }
    finally { setLoading(false); }
  }

  async function issue() {
    setIssuing(true); setError(null);
    try { setInventory(await issueInitialInventory(token, assetId)); }
    catch (reason) { setError(reason instanceof Error ? reason.message : "Could not issue initial inventory."); }
    finally { setIssuing(false); }
  }

  useEffect(() => {
    let active = true;
    void fetchInitialInventory(token, assetId)
      .then((result) => { if (active) setInventory(result); })
      .catch((reason: unknown) => { if (active) setError(reason instanceof Error ? reason.message : "Could not read initial inventory."); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [assetId, token]);

  return <div className="mt-4 rounded-md border bg-background p-3 text-xs">
    <div className="flex flex-wrap items-start justify-between gap-2">
      <div><p className="font-medium">Issuer settlement inventory</p><p className="mt-1 text-muted-foreground">The issuer and broker use this same backend-controlled wallet for the demo.</p></div>
      <div className="flex items-center gap-2"><Badge variant={inventory?.issuance_complete ? "success" : "warning"}>{inventory?.issuance_complete ? "Issued" : "Not issued"}</Badge>{!inventory?.issuance_complete && <Button size="sm" disabled={loading || issuing} onClick={() => void issue()}>{issuing && <RefreshCw className="animate-spin" />}{issuing ? "Issuing…" : "Issue initial inventory"}</Button>}<Button size="icon" variant="ghost" disabled={loading || issuing} onClick={() => void load()} aria-label="Refresh inventory"><RefreshCw className={loading ? "animate-spin" : undefined} /></Button></div>
    </div>
    {inventory && <div className="mt-3 grid gap-3 sm:grid-cols-3"><Value label="Authorized units" value={inventory.authorized_units} /><Value label="Mint supply" value={inventory.supply} /><Value label="Issuer inventory" value={inventory.inventory_balance} /><Address label="Settlement wallet" value={inventory.settlement_wallet} /><Address label="Asset token account" value={inventory.token_account} /><div className="sm:col-span-3 flex flex-wrap gap-4 text-muted-foreground"><span className="flex items-center gap-1"><CheckCircle2 className={inventory.wallet_allowlisted ? "size-3.5 text-primary" : "size-3.5"} />Wallet allow-listed</span><span className="flex items-center gap-1"><CheckCircle2 className={inventory.token_account_ready ? "size-3.5 text-primary" : "size-3.5"} />Token account ready</span></div></div>}
    {!inventory && loading && <p className="mt-3 text-muted-foreground">Loading inventory status…</p>}{error && <p role="alert" className="mt-3 text-destructive">{error}</p>}{!inventory?.issuance_complete && !loading && <p className="mt-3 text-muted-foreground">This one-time action allow-lists the settlement wallet, creates its asset token account, and mints the full authorized supply to it.</p>}
  </div>;
}

function Value({ label, value }: { label: string; value: string }) { return <div><p className="text-muted-foreground">{label}</p><p className="mt-1 font-medium">{Number(value).toLocaleString()}</p></div>; }

function Address({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);
  async function copy() { await navigator.clipboard.writeText(value); setCopied(true); window.setTimeout(() => setCopied(false), 1500); }
  return <div className="sm:col-span-3"><p className="text-muted-foreground">{label}</p><div className="mt-1 flex items-start gap-2"><p className="min-w-0 flex-1 break-all font-mono">{value}</p><Button type="button" variant="ghost" size="icon" className="-mt-1 size-7 shrink-0" onClick={() => void copy()} aria-label={`Copy ${label.toLowerCase()}`}>{copied ? <Check className="text-success" /> : <Copy />}</Button></div></div>;
}
