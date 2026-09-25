"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { ArrowRight, Building2, CheckCircle2, Coins, FileCheck2, Pencil, Send } from "lucide-react";

import { LoadingState } from "@/components/shared/loading-state";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { fetchAssetSetup, fetchIssuerAssets, fetchIssuerOverview, startAssetSetup, submitAssetForDemoReview, type AssetDraft, type AssetSetupOperation, type IssuerOverview } from "@/features/auth/api/backend-auth";
import { useAuthStore } from "@/stores/auth-store";
import { AssetLifecycle, lifecycleStep } from "./_components/asset-lifecycle";
import { PaymentAssetStatusCard } from "./_components/payment-asset-status";
import { InitialInventoryStatus } from "./_components/initial-inventory-status";

export default function IssuerPage() {
  const token = useAuthStore((state) => state.token);
  const user = useAuthStore((state) => state.user);
  const [overview, setOverview] = useState<IssuerOverview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [assets, setAssets] = useState<AssetDraft[]>([]);
  const [advancingId, setAdvancingId] = useState<string | null>(null);
  const [setupByAsset, setSetupByAsset] = useState<Record<string, AssetSetupOperation>>({});
  const [startingSetupId, setStartingSetupId] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    void fetchIssuerOverview(token).then(setOverview).catch((reason: unknown) =>
      setError(reason instanceof Error ? reason.message : "Access denied."),
    );
  }, [token]);

  useEffect(() => {
    if (!token || overview?.account_status !== "approved") return;
    void fetchIssuerAssets(token).then(setAssets).catch(() => undefined);
  }, [token, overview?.account_status]);

  useEffect(() => {
    if (!token) return;
    const eligible = assets.filter((asset) => ["approved_for_setup", "minted"].includes(asset.status));
    void Promise.all(eligible.map(async (asset) => {
      try { return await fetchAssetSetup(token, asset.asset_id); } catch { return null; }
    })).then((operations) => setSetupByAsset(Object.fromEntries(operations.filter((operation): operation is AssetSetupOperation => Boolean(operation)).map((operation) => [operation.asset_id, operation]))));
  }, [token, assets]);

  async function submitForDemoApproval(assetId: string) {
    if (!token) return;
    setAdvancingId(assetId); setError(null);
    try { const updated = await submitAssetForDemoReview(token, assetId); setAssets((current) => current.map((asset) => asset.asset_id === assetId ? updated : asset)); }
    catch (reason) { setError(reason instanceof Error ? reason.message : "Could not advance this asset."); }
    finally { setAdvancingId(null); }
  }

  async function beginMintSetup(assetId: string) {
    if (!token) return;
    setStartingSetupId(assetId); setError(null);
    try {
      const operation = await startAssetSetup(token, assetId);
      setSetupByAsset((current) => ({ ...current, [assetId]: operation }));
      const refreshed = await fetchIssuerAssets(token);
      setAssets(refreshed);
    } catch (reason) { setError(reason instanceof Error ? reason.message : "Could not start mint setup."); }
    finally { setStartingSetupId(null); }
  }

  return (
    <div className="mx-auto max-w-5xl space-y-6">
      <div>
        <p className="text-sm font-medium text-primary">Issuer workspace</p>
        <h1 className="text-3xl font-semibold tracking-tight">Welcome, {user?.name}</h1>
        <p className="mt-2 text-muted-foreground">Set up your organization before creating an offering.</p>
      </div>

      {error ? (
        <Card><CardContent className="pt-6 text-sm text-destructive">Access denied: {error}</CardContent></Card>
      ) : !overview ? (
        <LoadingState />
      ) : overview.account_status !== "approved" ? (
        <Card className="overflow-hidden">
          <CardHeader className="border-b bg-muted/30">
            <div className="flex items-start justify-between gap-4">
              <div>
                <CardTitle className="flex items-center gap-2"><FileCheck2 className="size-5 text-primary" />Verify your organization</CardTitle>
                <CardDescription className="mt-2">Complete the mock KYB check to unlock issuer tools.</CardDescription>
              </div>
              <Badge variant="warning">Action required</Badge>
            </div>
          </CardHeader>
          <CardContent className="grid gap-6 pt-6 md:grid-cols-[1fr_auto] md:items-center">
            <div className="space-y-3 text-sm text-muted-foreground">
              <p>We’ll collect your legal entity details, authorized representative, and one registration document.</p>
              <div className="flex flex-wrap gap-x-6 gap-y-2 text-foreground">
                <span className="flex items-center gap-2"><CheckCircle2 className="size-4 text-primary" />About 3 minutes</span>
                <span className="flex items-center gap-2"><CheckCircle2 className="size-4 text-primary" />Instant mock approval</span>
              </div>
            </div>
            <Button asChild><Link href="/issuer/onboarding">Start verification <ArrowRight className="size-4" /></Link></Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-6 md:grid-cols-2">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between gap-3">
                <CardTitle className="flex items-center gap-2"><Building2 className="size-5" />Organization</CardTitle>
                <Badge variant="success"><CheckCircle2 />Verified</Badge>
              </div>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <p className="text-lg font-semibold">{overview.organization?.legal_name}</p>
              <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2">
                <dt className="text-muted-foreground">Type</dt><dd>{overview.organization?.organization_type}</dd>
                <dt className="text-muted-foreground">Jurisdiction</dt><dd>{overview.organization?.jurisdiction}</dd>
                <dt className="text-muted-foreground">Registration</dt><dd>{overview.organization?.registration_number}</dd>
                <dt className="text-muted-foreground">Representative</dt><dd>{overview.organization?.representative_name}</dd>
              </dl>
            </CardContent>
          </Card>
          <Card>
            <CardHeader><CardTitle>Ready to issue</CardTitle><CardDescription>Your organization passed the mock KYB check.</CardDescription></CardHeader>
            <CardContent className="space-y-4 text-sm text-muted-foreground">
              <p>Create a simulated instrument draft with its supply, representation, and token metadata. No mint is created at this stage.</p>
              <Button asChild className="w-full sm:w-auto"><Link href="/issuer/assets/new">Create an asset</Link></Button>
            </CardContent>
          </Card>
          <Card className="md:col-span-2"><CardHeader><CardTitle>Path to market</CardTitle><CardDescription>Issuer submission receives instant demo approval. Mint setup and initial inventory issuance are available; investor distribution and listing remain separate stages.</CardDescription></CardHeader><CardContent><AssetLifecycle /></CardContent></Card>
          <PaymentAssetStatusCard />
          {assets.length > 0 && <Card className="md:col-span-2"><CardHeader><CardTitle>Issuer assets</CardTitle><CardDescription>Edit and approve drafts, then start the configured Token-2022 and ACL setup.</CardDescription></CardHeader><CardContent className="space-y-3">{assets.map((asset) => { const setup = setupByAsset[asset.asset_id]; return <div key={asset.asset_id} className="rounded-lg border bg-muted/10 p-4"><div className="flex flex-wrap items-center justify-between gap-3"><div><p className="font-medium">{asset.draft.name}</p><p className="text-sm text-muted-foreground">{asset.draft.ticker} · {asset.draft.market} · {Number(asset.draft.authorized_units).toLocaleString()} authorized units</p></div><div className="flex flex-wrap items-center gap-2"><Badge variant={["approved_for_setup", "minted"].includes(asset.status) ? "success" : "secondary"}>{asset.status.replaceAll("_", " ")}</Badge>{asset.status === "draft" && <><Button asChild size="sm" variant="outline"><Link href={`/issuer/assets/${asset.asset_id}/edit`}><Pencil />Edit draft</Link></Button><Button size="sm" disabled={advancingId === asset.asset_id} onClick={() => void submitForDemoApproval(asset.asset_id)}><Send />{advancingId === asset.asset_id ? "Approving…" : "Approve draft"}</Button></>}{asset.status === "approved_for_setup" && !setup && <Button size="sm" disabled={startingSetupId === asset.asset_id} onClick={() => void beginMintSetup(asset.asset_id)}><Coins />{startingSetupId === asset.asset_id ? "Setting up…" : "Start mint setup"}</Button>}</div></div>{asset.status === "draft" && <p className="mt-3 text-xs text-muted-foreground">Approval publishes the metadata automatically. If publishing fails, the asset stays editable and you can retry.</p>}{setup && <div className="mt-4 rounded-md border bg-background p-3 text-xs"><div className="flex flex-wrap items-center justify-between gap-2"><p className="font-medium">Mint & ACL setup · {setup.network}</p><Badge variant={setup.status === "confirmed" ? "success" : setup.status === "failed" || setup.status === "reconciliation_required" ? "destructive" : "secondary"}>{setup.status.replaceAll("_", " ")}</Badge></div><p className="mt-2 break-all text-muted-foreground">Mint: {setup.mint_address}</p>{setup.stages.length > 0 && <ul className="mt-3 grid gap-1 sm:grid-cols-2">{setup.stages.map((stage) => <li key={stage.stage} className="flex items-center gap-2"><CheckCircle2 className="size-3.5 text-primary" />{stage.stage.replaceAll("_", " ")}</li>)}</ul>}{setup.error && <p className="mt-3 text-destructive">{setup.error}</p>}</div>}{setup?.status === "confirmed" && token && <InitialInventoryStatus token={token} assetId={asset.asset_id} />}<div className="mt-5 border-t pt-5"><AssetLifecycle current={lifecycleStep(asset.status)} /></div></div>; })}</CardContent></Card>}
        </div>
      )}
    </div>
  );
}
