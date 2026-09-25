import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { AdminAssetsTable } from "@/features/admin/components/admin-assets-table";

export const metadata: Metadata = { title: "Assets — Admin" };

export default function AdminAssetsPage() {
  return (
    <PageContainer>
      <PageHeader title="Assets" description="Manage asset lifecycle status across the platform." />
      <AdminAssetsTable />
    </PageContainer>
  );
}
