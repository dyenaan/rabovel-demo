"use client";

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";

import { PageContainer } from "@/components/layout/page-container";
import { LoadingState } from "@/components/shared/loading-state";
import { PageHeader } from "@/components/shared/page-header";
import { RedemptionForm } from "@/features/primary-market/components/redemption-form";

function RedeemContent() {
  const searchParams = useSearchParams();
  const assetId = searchParams.get("assetId") ?? undefined;

  return (
    <div className="max-w-xl">
      <RedemptionForm defaultAssetId={assetId} />
    </div>
  );
}

export default function RedeemPage() {
  return (
    <PageContainer>
      <PageHeader title="Redeem" description="Submit a redemption request for an existing holding." />
      <Suspense fallback={<LoadingState />}>
        <RedeemContent />
      </Suspense>
    </PageContainer>
  );
}
