import { AssetDraftForm, defaultAssetDraft } from "../_components/asset-draft-form";

export default function NewIssuerAssetPage() {
  return <AssetDraftForm initialValue={defaultAssetDraft} />;
}
