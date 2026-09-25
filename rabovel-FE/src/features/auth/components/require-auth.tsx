"use client";

import { useEffect } from "react";
import { usePathname, useRouter } from "next/navigation";
import type { ReactNode } from "react";

import { LoadingState } from "@/components/shared/loading-state";
import { useAuth } from "../hooks/use-auth";

/**
 * UI-layer guard only — redirects to /login when signed out. The backend
 * remains the source of truth for authorization; never rely on this alone.
 */
export function RequireAuth({ children }: { children: ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      const target = pathname + window.location.search;
      router.replace(`/login?redirect=${encodeURIComponent(target)}`);
    }
  }, [isLoading, isAuthenticated, pathname, router]);

  if (isLoading || !isAuthenticated) {
    return <LoadingState />;
  }

  return <>{children}</>;
}
