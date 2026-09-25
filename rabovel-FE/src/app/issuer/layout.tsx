"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { Building2, FileCheck2, LogOut } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Logo } from "@/components/layout/logo";
import { AccessRestricted } from "@/features/auth/components/access-restricted";
import { RoleGuard } from "@/features/auth/components/role-guard";
import { useAuth } from "@/features/auth/hooks/use-auth";

export default function IssuerLayout({ children }: LayoutProps<"/issuer">) {
  const router = useRouter();
  const { user, logout } = useAuth();
  return (
    <RoleGuard allow={["ISSUER"]} fallback={<AccessRestricted />}>
      <div className="flex min-h-screen">
        <aside className="hidden w-64 border-r bg-sidebar p-4 md:block">
          <Logo href="/issuer" />
          <nav className="mt-8 space-y-1">
            <Link href="/issuer" className="flex items-center gap-2 rounded-md bg-sidebar-accent p-2 text-sm font-medium"><Building2 className="size-4" />Issuer overview</Link>
            <Link href="/issuer/onboarding" className="flex items-center gap-2 rounded-md p-2 text-sm text-muted-foreground hover:bg-sidebar-accent hover:text-foreground"><FileCheck2 className="size-4" />Organization verification</Link>
          </nav>
        </aside>
        <div className="flex min-w-0 flex-1 flex-col">
          <header className="flex h-16 items-center border-b px-4 sm:px-6"><span className="font-medium md:hidden">Issuer</span><div className="ml-auto flex items-center gap-3"><span className="hidden text-sm text-muted-foreground sm:inline">{user?.email}</span><Button variant="ghost" size="sm" onClick={async () => { await logout(); router.push("/login"); }}><LogOut className="size-4" />Log out</Button></div></header>
          <main className="flex-1 p-4 sm:p-6">{children}</main>
        </div>
      </div>
    </RoleGuard>
  );
}
