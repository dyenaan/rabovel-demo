import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { AuditLogTable } from "@/features/admin/components/audit-log-table";

export const metadata: Metadata = { title: "Audit Logs — Admin" };

export default function AdminAuditPage() {
  return (
    <PageContainer>
      <PageHeader title="Audit Logs" description="Immutable record of administrative and system actions." />
      <AuditLogTable />
    </PageContainer>
  );
}
