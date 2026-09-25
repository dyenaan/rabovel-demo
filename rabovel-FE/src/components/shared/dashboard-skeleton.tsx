import { PageContainer } from "@/components/layout/page-container";
import { Skeleton } from "@/components/ui/skeleton";

export function DashboardSkeleton() {
  return (
    <PageContainer>
      <Skeleton className="mb-6 h-8 w-48" />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Skeleton key={i} className="h-24 w-full" />
        ))}
      </div>
      <Skeleton className="mt-6 h-72 w-full" />
    </PageContainer>
  );
}

export function TableSkeleton() {
  return (
    <PageContainer>
      <Skeleton className="mb-6 h-8 w-48" />
      <div className="space-y-2">
        {Array.from({ length: 8 }).map((_, i) => (
          <Skeleton key={i} className="h-10 w-full" />
        ))}
      </div>
    </PageContainer>
  );
}

export function AssetDetailsSkeleton() {
  return (
    <PageContainer>
      <Skeleton className="h-24 w-full" />
      <div className="mt-6 grid gap-4 sm:grid-cols-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <Skeleton key={i} className="h-32 w-full" />
        ))}
      </div>
    </PageContainer>
  );
}

export function TradingTerminalSkeleton() {
  return (
    <PageContainer className="max-w-none">
      <Skeleton className="h-20 w-full" />
      <div className="mt-4 grid gap-4 lg:grid-cols-[1fr_320px]">
        <Skeleton className="h-80 w-full" />
        <Skeleton className="h-80 w-full" />
      </div>
    </PageContainer>
  );
}
