import { PageContainer } from "@/components/layout/page-container";
import { SettlementDetailView } from "@/features/settlements/components/settlement-detail-view";

export default async function SettlementDetailPage(props: PageProps<"/settlements/[settlementId]">) {
  const { settlementId } = await props.params;

  return (
    <PageContainer>
      <SettlementDetailView settlementId={settlementId} />
    </PageContainer>
  );
}
