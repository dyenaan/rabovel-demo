import { PageContainer } from "@/components/layout/page-container";
import { OrderDetailView } from "@/features/orders/components/order-detail-view";

export default async function OrderDetailPage(props: PageProps<"/orders/[orderId]">) {
  const { orderId } = await props.params;

  return (
    <PageContainer>
      <OrderDetailView orderId={orderId} />
    </PageContainer>
  );
}
