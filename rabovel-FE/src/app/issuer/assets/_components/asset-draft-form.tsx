"use client";

import { FormEvent, useState } from "react";
import { useRouter } from "next/navigation";
import { ArrowLeft, Check, Copy, ImagePlus, Save } from "lucide-react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { createAssetDraft, updateAssetDraft, uploadAssetImage, type AssetDraftSubmission } from "@/features/auth/api/backend-auth";
import { useAuthStore } from "@/stores/auth-store";

export const DISCLOSURE = "Prototype / simulated asset / not an offering / not SEC-approved";
export const defaultAssetDraft: AssetDraftSubmission = {
  instrument_code: "RABO-NG-TELCO", name: "Rabovel Nigeria Telco", ticker: "RABO-NG-TELCO", market: "NGX", share_class: "Ordinary", asset_type: "equity", decimals: 0,
  authorized_units: "1000000", settlement_currency: "CNGN", representation: "1 token = 1 simulated beneficial entitlement to 1 share",
  rights_description: "Simulated ordinary-share entitlement for the Rabovel prototype.", disclosure: DISCLOSURE,
  metadata: { name: "Rabovel Nigeria Telco", symbol: "RABO-NG-TELCO", description: "A simulated Nigerian telecom equity used only in the Rabovel demo.", image_uri: null, external_url: null, metadata_uri: null, additional_metadata: [] },
};

export function AssetDraftForm({ initialValue, assetId }: { initialValue: AssetDraftSubmission; assetId?: string }) {
  const token = useAuthStore((state) => state.token); const router = useRouter();
  const [form, setForm] = useState(initialValue); const [error, setError] = useState<string | null>(null); const [saving, setSaving] = useState(false);
  const [savedAssetId, setSavedAssetId] = useState(assetId);
  const [copied, setCopied] = useState(false);
  const [imageFile, setImageFile] = useState<File | null>(null); const [imagePreview, setImagePreview] = useState<string | null>(form.metadata.image_uri);
  const editing = Boolean(savedAssetId);
  const field = (key: keyof AssetDraftSubmission) => (event: React.ChangeEvent<HTMLInputElement>) => setForm((current) => ({ ...current, [key]: event.target.value }));
  const metadataField = (key: "name" | "description" | "external_url") => (event: React.ChangeEvent<HTMLInputElement>) => setForm((current) => ({ ...current, metadata: { ...current.metadata, [key]: event.target.value || null } }));
  async function submit(event: FormEvent) { event.preventDefault(); setError(null); if (!token) { setError("Sign in again to save this draft."); return; } setSaving(true); let persistedId = savedAssetId; try { const saved = persistedId ? await updateAssetDraft(token, persistedId, form) : await createAssetDraft(token, form); persistedId = saved.asset_id; setSavedAssetId(saved.asset_id); setForm(saved.draft); if (imageFile) { const withImage = await uploadAssetImage(token, saved.asset_id, imageFile); setForm(withImage.draft); setImagePreview(withImage.draft.metadata.image_uri); setImageFile(null); } router.push("/issuer"); } catch (reason) { if (persistedId && !assetId) router.replace(`/issuer/assets/${encodeURIComponent(persistedId)}/edit`); const message = reason instanceof Error ? reason.message : "Could not save the asset draft."; setError(persistedId && imageFile ? `The draft was saved, but its image was not uploaded. ${message}` : message); } finally { setSaving(false); } }
  const metadataDocument = {
    name: form.metadata.name,
    symbol: form.metadata.symbol,
    description: form.metadata.description,
    ...(form.metadata.image_uri ? { image: form.metadata.image_uri } : {}),
    ...(form.metadata.external_url ? { external_url: form.metadata.external_url } : {}),
    properties: {
      ...Object.fromEntries(form.metadata.additional_metadata.filter(({ key }) => !["underlying", "underlying_reference"].includes(key.trim().toLowerCase())).map(({ key, value }) => [key, value])),
      ratio: form.representation,
      asset_id: savedAssetId ?? "assigned-after-save",
      asset_type: form.asset_type,
      market: form.market,
      share_class: form.share_class,
      representation: form.representation,
      rights_description: form.rights_description,
      decimals: form.decimals,
      authorized_units: form.authorized_units,
      settlement_currency: form.settlement_currency,
      disclosure: DISCLOSURE,
      schema_version: "1.0",
    },
  };
  const metadataJson = JSON.stringify(metadataDocument, null, 2);
  async function copyMetadata() { await navigator.clipboard.writeText(metadataJson); setCopied(true); window.setTimeout(() => setCopied(false), 1500); }

  return <div className="mx-auto max-w-4xl space-y-6"><div><Button asChild variant="ghost" className="mb-2 -ml-3"><Link href="/issuer"><ArrowLeft />Issuer overview</Link></Button><p className="text-sm font-medium text-primary">Asset registry</p><h1 className="text-3xl font-semibold tracking-tight">{editing ? "Edit asset draft" : "Create asset draft"}</h1><p className="mt-2 text-muted-foreground">Define the simulated instrument and metadata. Saving does not create a mint or issue inventory.</p></div>
    <form onSubmit={submit} className="space-y-6"><Card><CardHeader><CardTitle>Instrument</CardTitle><CardDescription>Stable identity, classification, and authorized supply.</CardDescription></CardHeader><CardContent className="grid gap-4 sm:grid-cols-2">
      <Field label="Instrument code"><Input value={form.instrument_code} onChange={field("instrument_code")} required /></Field><Field label="Display name"><Input value={form.name} onChange={field("name")} required /></Field>
      <Field label="Ticker"><Input value={form.ticker} onChange={(event) => setForm((current) => ({ ...current, ticker: event.target.value, metadata: { ...current.metadata, symbol: event.target.value } }))} required /></Field><Field label="Market"><Input value={form.market} onChange={field("market")} required /></Field>
      <Field label="Share class"><Input value={form.share_class} onChange={field("share_class")} required /></Field><Field label="Asset type"><Input value={form.asset_type} onChange={field("asset_type")} required /></Field>
      <Field label="Authorized units"><Input inputMode="numeric" pattern="[0-9]+" value={form.authorized_units} onChange={field("authorized_units")} required /></Field><Field label="Decimals"><Input type="number" min={0} max={9} value={form.decimals} onChange={(event) => setForm((current) => ({ ...current, decimals: Number(event.target.value) }))} required /></Field>
      <Field label="Settlement currency"><Input value={form.settlement_currency} onChange={field("settlement_currency")} required /></Field>
      <Field label="Representation" wide><Input value={form.representation} onChange={field("representation")} required /></Field><Field label="Rights description" wide><Input value={form.rights_description} onChange={field("rights_description")} required /></Field>
    </CardContent></Card><Card><CardHeader><CardTitle>Token metadata</CardTitle><CardDescription>Rabovel generates the public document from these validated fields. The backend will publish it to Supabase Storage when the asset is approved.</CardDescription></CardHeader><CardContent className="grid gap-4 sm:grid-cols-2">
      <Field label="Metadata name"><Input value={form.metadata.name} onChange={metadataField("name")} required /></Field><Field label="Metadata symbol"><Input value={form.metadata.symbol} readOnly /></Field>
      <Field label="Asset image" wide><div className="grid gap-4 rounded-lg border border-dashed p-4 sm:grid-cols-[10rem_1fr] sm:items-center"><div className="flex aspect-square items-center justify-center overflow-hidden rounded-md bg-muted bg-cover bg-center" style={imagePreview ? { backgroundImage: `url(${imagePreview})` } : undefined}>{!imagePreview && <ImagePlus className="size-8 text-muted-foreground" />}</div><div className="space-y-3"><Input type="file" accept="image/png,image/jpeg,image/webp" disabled={saving} onChange={(event) => { const file = event.target.files?.[0] ?? null; if (imagePreview?.startsWith("blob:")) URL.revokeObjectURL(imagePreview); setImageFile(file); setImagePreview(file ? URL.createObjectURL(file) : form.metadata.image_uri); }} /><p className="text-xs text-muted-foreground">PNG, JPEG, or WebP up to 5 MB. The selected image uploads when you save the draft.</p></div></div></Field><Field label="External URL (optional)"><Input type="url" value={form.metadata.external_url ?? ""} onChange={metadataField("external_url")} placeholder="https://…" /></Field>
      <Field label="Metadata URI" wide><Input type="url" value={form.metadata.metadata_uri ?? ""} readOnly placeholder="Assigned by the backend after publishing" /><p className="text-xs text-muted-foreground">Planned path: <span className="font-mono">metadata/assets/{assetId ?? "{asset_id}"}/metadata.json</span></p></Field>
    </CardContent></Card><Card className="overflow-hidden"><CardHeader className="flex-row items-start justify-between gap-4 border-b bg-muted/20"><div><CardTitle>Metadata JSON preview</CardTitle><CardDescription className="mt-1">This is the public document that will be uploaded. System-owned fields override custom metadata.</CardDescription></div><Button type="button" size="sm" variant="outline" onClick={() => void copyMetadata()}>{copied ? <Check /> : <Copy />}{copied ? "Copied" : "Copy JSON"}</Button></CardHeader><CardContent className="p-0"><pre className="max-h-[32rem] overflow-auto p-5 text-xs leading-relaxed"><code>{metadataJson}</code></pre></CardContent></Card><Card className="border-amber-500/40 bg-amber-500/5"><CardContent className="pt-6"><p className="font-medium">{DISCLOSURE}</p><p className="mt-1 text-sm text-muted-foreground">This mandatory disclosure is stored with the draft and metadata.</p></CardContent></Card>
    {error && <p className="text-sm text-destructive">{error}</p>}<div className="flex justify-end"><Button disabled={saving} type="submit"><Save />{saving ? "Saving…" : editing ? "Save changes" : "Save draft"}</Button></div></form></div>;
}

function Field({ label, wide, children }: { label: string; wide?: boolean; children: React.ReactNode }) { return <div className={`space-y-2 ${wide ? "sm:col-span-2" : ""}`}><Label>{label}</Label>{children}</div>; }
