"use client";

import { FormEvent, useState } from "react";
import { CheckCircle2, FileCheck2, Rocket } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { publishAssetListing, submitAssetBacking, type AssetDraft } from "@/features/auth/api/backend-auth";

export function BackingListingStatus({ token, asset, onChange }: { token: string; asset: AssetDraft; onChange: (asset: AssetDraft) => void }) {
  const [summary, setSummary] = useState("");
  const [file, setFile] = useState<File | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const backing = asset.draft.backing;
  async function submit(event: FormEvent<HTMLFormElement>) { event.preventDefault(); if (!file) return; setBusy(true); setError(null); try { onChange(await submitAssetBacking(token, asset.asset_id, summary, file)); } catch (reason) { setError(reason instanceof Error ? reason.message : "Could not verify backing."); } finally { setBusy(false); } }
  async function makeLive() { setBusy(true); setError(null); try { const updated = await publishAssetListing(token, asset.asset_id); onChange(updated); window.dispatchEvent(new CustomEvent("rabovel:listing-published", { detail: updated })); } catch (reason) { setError(reason instanceof Error ? reason.message : "Could not publish the listing."); } finally { setBusy(false); } }
  return <div className="mt-4 rounded-md border bg-background p-4 text-sm"><div className="flex flex-wrap items-start justify-between gap-3"><div><p className="font-medium">Backing & listing</p><p className="mt-1 text-xs text-muted-foreground">Demo verification records the selected document metadata; the file is not uploaded.</p></div><Badge variant={asset.draft.listing_status === "live" ? "success" : backing ? "warning" : "secondary"}>{asset.draft.listing_status === "live" ? "Live" : backing ? "Ready to list" : "Backing required"}</Badge></div>
    {backing ? <div className="mt-4 space-y-3"><div className="rounded-md bg-muted/40 p-3"><p className="flex items-center gap-2 font-medium"><FileCheck2 className="size-4 text-primary" />{backing.document_name}</p><p className="mt-2 text-muted-foreground">{backing.summary}</p><p className="mt-2 flex items-center gap-1 text-xs text-primary"><CheckCircle2 className="size-3.5" />Instant demo verification passed</p></div>{asset.draft.listing_status !== "live" && <Button onClick={() => void makeLive()} disabled={busy}><Rocket />{busy ? "Publishing…" : "Make listing live"}</Button>}</div> : <form className="mt-4 space-y-3" onSubmit={submit}><div className="space-y-2"><Label htmlFor={`backing-summary-${asset.asset_id}`}>Backing summary</Label><textarea id={`backing-summary-${asset.asset_id}`} className="min-h-20 w-full rounded-md border bg-transparent px-3 py-2 text-sm" minLength={10} maxLength={500} required value={summary} onChange={(event) => setSummary(event.target.value)} placeholder="One issued token represents one simulated ordinary share held by the issuer." /></div><div className="space-y-2"><Label htmlFor={`backing-file-${asset.asset_id}`}>Evidence document</Label><Input id={`backing-file-${asset.asset_id}`} type="file" accept="application/pdf,image/png,image/jpeg" required onChange={(event) => setFile(event.target.files?.[0] ?? null)} /><p className="text-xs text-muted-foreground">PDF, PNG, or JPEG up to 10 MB.</p></div><Button type="submit" disabled={busy || !file || summary.trim().length < 10}>{busy ? "Verifying…" : "Submit and verify backing"}</Button></form>}
    {error && <p role="alert" className="mt-3 text-destructive">{error}</p>}</div>;
}
