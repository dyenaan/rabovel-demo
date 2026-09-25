import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { ProofOfReservesDashboard } from "@/features/reserves/components/proof-of-reserves-dashboard";

export const metadata: Metadata = { title: "Proof of Reserves — Admin" };

export default function AdminReservesPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Proof of Reserves"
        description="Independently attested reserve coverage for every tokenized asset."
      />
      <ProofOfReservesDashboard />
    </PageContainer>
  );
}
