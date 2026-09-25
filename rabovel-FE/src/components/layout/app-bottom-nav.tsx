"use client";

import { usePathname } from "next/navigation";
import { Building2, LayoutDashboard, MoreHorizontal, Repeat, Wallet } from "lucide-react";

import { isNavPathActive } from "@/lib/utils";
import { useSidebarStore } from "@/stores/sidebar-store";
import { BottomNavBar, type BottomNavLinkItem } from "./bottom-nav-bar";

const PRIMARY_ITEMS: BottomNavLinkItem[] = [
  { label: "Home", href: "/dashboard", icon: LayoutDashboard },
  { label: "Assets", href: "/assets", icon: Building2 },
  { label: "Trade", href: "/trade", icon: Repeat },
  { label: "Wallet", href: "/wallet", icon: Wallet },
];

export function AppBottomNav() {
  const pathname = usePathname();
  const setMobileOpen = useSidebarStore((s) => s.setMobileOpen);

  const isPrimaryActive = PRIMARY_ITEMS.some((item) => isNavPathActive(pathname, item.href));

  return (
    <BottomNavBar
      items={PRIMARY_ITEMS}
      more={{
        label: "More",
        icon: MoreHorizontal,
        isActive: !isPrimaryActive,
        onClick: () => setMobileOpen(true),
      }}
    />
  );
}
