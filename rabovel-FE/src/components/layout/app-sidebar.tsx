"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  BarChart3,
  Building2,
  ChevronsLeft,
  ChevronsRight,
  FileText,
  Landmark,
  LayoutDashboard,
  ListOrdered,
  Repeat,
  ShieldCheck,
  Wallet,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Logo } from "@/components/layout/logo";
import { Button } from "@/components/ui/button";
import { cn, isNavPathActive } from "@/lib/utils";
import { useSidebarStore } from "@/stores/sidebar-store";

type NavItem = { label: string; href: string; icon: LucideIcon };
type NavGroup = { title: string; items: NavItem[] };

const NAV_GROUPS: NavGroup[] = [
  {
    title: "Overview",
    items: [{ label: "Dashboard", href: "/dashboard", icon: LayoutDashboard }],
  },
  {
    title: "Invest",
    items: [
      { label: "Assets", href: "/assets", icon: Building2 },
      { label: "Primary Market", href: "/primary-market", icon: Landmark },
    ],
  },
  {
    title: "Trade",
    items: [
      { label: "Markets", href: "/markets", icon: Repeat },
      { label: "Trade", href: "/trade", icon: BarChart3 },
      { label: "Orders", href: "/orders", icon: ListOrdered },
      { label: "Trades", href: "/trades", icon: Repeat },
      { label: "Settlements", href: "/settlements", icon: ShieldCheck },
    ],
  },
  {
    title: "Account",
    items: [
      { label: "Wallet", href: "/wallet", icon: Wallet },
      { label: "Documents", href: "/documents", icon: FileText },
      { label: "Security", href: "/security", icon: ShieldCheck },
    ],
  },
];

export function AppSidebar({ className }: { className?: string }) {
  const pathname = usePathname();
  const isCollapsed = useSidebarStore((s) => s.isCollapsed);
  const toggleCollapsed = useSidebarStore((s) => s.toggleCollapsed);

  return (
    <aside
      className={cn(
        "hidden shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground md:flex",
        isCollapsed ? "w-16" : "w-64",
        "transition-[width] duration-200",
        className,
      )}
    >
      <div className="flex h-16 items-center justify-between border-b border-sidebar-border px-4">
        {!isCollapsed && <Logo href="/dashboard" />}
        <Button
          variant="ghost"
          size="icon"
          className="ml-auto"
          onClick={toggleCollapsed}
          aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {isCollapsed ? <ChevronsRight className="size-4" /> : <ChevronsLeft className="size-4" />}
        </Button>
      </div>

      <nav className="flex-1 space-y-6 overflow-y-auto px-3 py-4">
        {NAV_GROUPS.map((group) => (
          <div key={group.title}>
            {!isCollapsed && (
              <p className="px-2 pb-1.5 text-xs font-medium tracking-wide text-sidebar-foreground/50 uppercase">
                {group.title}
              </p>
            )}
            <div className="space-y-0.5">
              {group.items.map((item) => {
                const isActive = isNavPathActive(pathname, item.href);
                return (
                  <Link
                    key={item.href}
                    href={item.href}
                    title={isCollapsed ? item.label : undefined}
                    className={cn(
                      "flex items-center gap-3 rounded-md px-2.5 py-2 text-sm font-medium transition-colors",
                      isActive
                        ? "bg-sidebar-accent text-sidebar-accent-foreground"
                        : "text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                      isCollapsed && "justify-center px-0",
                    )}
                  >
                    <item.icon className="size-4 shrink-0" aria-hidden="true" />
                    {!isCollapsed && <span>{item.label}</span>}
                  </Link>
                );
              })}
            </div>
          </div>
        ))}
      </nav>
    </aside>
  );
}

export { NAV_GROUPS };
