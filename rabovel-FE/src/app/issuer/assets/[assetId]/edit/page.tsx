"use client";

import { use, useEffect, useState } from "react";
import { LoadingState } from "@/components/shared/loading-state";
import { fetchIssuerAsset, type AssetDraft } from "@/features/auth/api/backend-auth";
import { useAuthStore } from "@/stores/auth-store";
import { AssetDraftForm } from "../../_components/asset-draft-form";

export default function EditIssuerAssetPage({ params }: { params: Promise<{ assetId: string }> }) {
  const { assetId } = use(params); const token = useAuthStore((state) => state.token);
  const [asset, setAsset] = useState<AssetDraft | null>(null); const [error, setError] = useState<string | null>(null);
  useEffect(() => { if (token) void fetchIssuerAsset(token, assetId).then(setAsset).catch((reason: unknown) => setError(reason instanceof Error ? reason.message : "Could not load this draft.")); }, [assetId, token]);
  if (error) return <p className="mx-auto max-w-4xl text-sm text-destructive">{error}</p>;
  if (!asset) return <LoadingState />;
  return <AssetDraftForm assetId={asset.asset_id} initialValue={asset.draft} />;
}
