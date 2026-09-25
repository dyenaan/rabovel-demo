"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Banknote,
  Building2,
  FileSearch,
  GitCompareArrows,
  Landmark,
  LayoutDashboard,
  ListOrdered,
  Repeat,
  ScanEye,
  Settings,
  ShieldCheck,
  Users,
  Vault,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Logo } from "@/components/layout/logo";
import { cn, isNavPathActive } from "@/lib/utils";

type NavItem = { label: string; href: string; icon: LucideIcon };

const ADMIN_NAV: NavItem[] = [
  { label: "Dashboard", href: "/admin", icon: LayoutDashboard },
  { label: "Investors", href: "/admin/investors", icon: Users },
  { label: "Assets", href: "/admin/assets", icon: Building2 },
  { label: "Issuance", href: "/admin/issuance", icon: Landmark },
  { label: "Compliance", href: "/admin/compliance", icon: ScanEye },
  { label: "Orders", href: "/admin/orders", icon: ListOrdered },
  { label: "Trades", href: "/admin/trades", icon: Repeat },
  { label: "Settlements", href: "/admin/settlements", icon: ShieldCheck },
  { label: "Reconciliation", href: "/admin/reconciliation", icon: GitCompareArrows },
  { label: "Proof of Reserves", href: "/admin/reserves", icon: Banknote },
  { label: "Custody", href: "/admin/custody", icon: Vault },
  { label: "Audit Logs", href: "/admin/audit", icon: FileSearch },
  { label: "Settings", href: "/admin/settings", icon: Settings },
];

export function AdminSidebar() {
  const pathname = usePathname();

  return (
    <aside className="hidden w-64 shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground md:flex">
      <div className="flex h-16 items-center gap-2 border-b border-sidebar-border px-4">
        <Logo href="/admin" />
        <span className="rounded-md bg-sidebar-accent px-1.5 py-0.5 text-[10px] font-medium text-sidebar-accent-foreground">
          ADMIN
        </span>
      </div>
      <nav className="flex-1 space-y-0.5 overflow-y-auto px-3 py-4">
        {ADMIN_NAV.map((item) => {
          const isActive = isNavPathActive(pathname, item.href, item.href === "/admin");
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex items-center gap-3 rounded-md px-2.5 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-sidebar-accent text-sidebar-accent-foreground"
                  : "text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
              )}
            >
              <item.icon className="size-4 shrink-0" aria-hidden="true" />
              {item.label}
            </Link>
          );
        })}
      </nav>
    </aside>
  );
}

export { ADMIN_NAV };
