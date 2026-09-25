import { PageContainer } from "@/components/layout/page-container";
import { MarketDetailView } from "@/features/markets/components/market-detail-view";

export default async function MarketDetailPage(props: PageProps<"/markets/[marketId]">) {
  const { marketId } = await props.params;

  return (
    <PageContainer>
      <MarketDetailView marketId={marketId} />
    </PageContainer>
  );
}
