import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { OrdersTable } from "@/features/orders/components/orders-table";
import { mockOrders } from "@/mocks/orders.mock";

export const metadata: Metadata = { title: "Orders — Admin" };

export default function AdminOrdersPage() {
  return (
    <PageContainer>
      <PageHeader title="Orders" description="Platform-wide order activity across all investors." />
      <OrdersTable orders={mockOrders} />
    </PageContainer>
  );
}
