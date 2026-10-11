"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuth } from "../hooks/use-auth";
import { homeForRole } from "../role-access";
import { ShieldAlert } from "lucide-react";

import { PageContainer } from "@/components/layout/page-container";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/shared/empty-state";

export function AccessRestricted({ area = "this section", requiredRole }: { area?: string; requiredRole?: string }) {
  const router = useRouter();
  const { user, logout } = useAuth();
  const destination = user ? homeForRole(user.role) : "/login";
  return (
    <PageContainer>
      <EmptyState
        icon={ShieldAlert}
        title="You don't have access to this area"
        description={requiredRole ? `${area} requires an ${requiredRole} account. Sign in with that account to continue.` : "Your account does not have access to this section. Return to your workspace or sign in with a different account."}
        action={
          <div className="flex flex-wrap justify-center gap-2"><Button asChild>
            <Link href={destination}>{user?.role === "ISSUER" ? "Go to issuer workspace" : "Go to your workspace"}</Link>
          </Button><Button variant="outline" onClick={async () => { await logout(); router.push("/login"); }}>Sign in with another account</Button></div>
        }
      />
    </PageContainer>
  );
}
