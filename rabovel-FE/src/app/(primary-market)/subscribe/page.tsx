"use client";

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";

import { PageContainer } from "@/components/layout/page-container";
import { LoadingState } from "@/components/shared/loading-state";
import { PageHeader } from "@/components/shared/page-header";
import { SubscriptionForm } from "@/features/primary-market/components/subscription-form";

function SubscribeContent() {
  const searchParams = useSearchParams();
  const assetId = searchParams.get("assetId") ?? undefined;

  return (
    <div className="max-w-xl">
      <SubscriptionForm defaultAssetId={assetId} />
    </div>
  );
}

export default function SubscribePage() {
  return (
    <PageContainer>
      <PageHeader title="Subscribe" description="Submit a subscription request for a tokenized asset." />
      <Suspense fallback={<LoadingState />}>
        <SubscribeContent />
      </Suspense>
    </PageContainer>
  );
}
