"use client";

import { useEffect } from "react";

import { PageContainer } from "@/components/layout/page-container";
import { ErrorState } from "@/components/shared/error-state";

export default function AdminError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <PageContainer>
      <ErrorState
        title="Something went wrong in the admin console"
        message="Please try again. If this persists, contact engineering with the request ID below."
        requestId={error.digest}
        onRetry={reset}
      />
    </PageContainer>
  );
}
