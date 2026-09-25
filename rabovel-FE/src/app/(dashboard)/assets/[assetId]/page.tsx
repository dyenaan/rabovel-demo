import { PageContainer } from "@/components/layout/page-container";
import { AssetDetailView } from "@/features/assets/components/asset-detail-view";

export default async function AssetDetailPage(props: PageProps<"/assets/[assetId]">) {
  const { assetId } = await props.params;

  return (
    <PageContainer>
      <AssetDetailView assetId={assetId} />
    </PageContainer>
  );
}
