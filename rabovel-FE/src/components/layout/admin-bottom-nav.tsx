"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Building2, LayoutDashboard, MoreHorizontal, ScanEye, Users } from "lucide-react";

import { Logo } from "@/components/layout/logo";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { cn, isNavPathActive } from "@/lib/utils";
import { ADMIN_NAV } from "./admin-sidebar";
import { BottomNavBar, type BottomNavLinkItem } from "./bottom-nav-bar";

const PRIMARY_ITEMS: BottomNavLinkItem[] = [
  { label: "Home", href: "/admin", icon: LayoutDashboard },
  { label: "Investors", href: "/admin/investors", icon: Users },
  { label: "Assets", href: "/admin/assets", icon: Building2 },
  { label: "Compliance", href: "/admin/compliance", icon: ScanEye },
];

export function AdminBottomNav() {
  const pathname = usePathname();
  const [isMoreOpen, setMoreOpen] = useState(false);

  const isPrimaryActive = PRIMARY_ITEMS.some((item) =>
    isNavPathActive(pathname, item.href, item.href === "/admin"),
  );

  return (
    <>
      <BottomNavBar
        items={PRIMARY_ITEMS}
        more={{
          label: "More",
          icon: MoreHorizontal,
          isActive: !isPrimaryActive,
          onClick: () => setMoreOpen(true),
        }}
      />
      <Sheet open={isMoreOpen} onOpenChange={setMoreOpen}>
        <SheetContent side="left" className="w-72 p-0">
          <SheetHeader className="border-b">
            <SheetTitle asChild>
              <Logo href="/admin" />
            </SheetTitle>
          </SheetHeader>
          <nav className="flex-1 space-y-0.5 overflow-y-auto px-3 py-4">
            {ADMIN_NAV.map((item) => {
              const isActive = isNavPathActive(pathname, item.href, item.href === "/admin");
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  onClick={() => setMoreOpen(false)}
                  className={cn(
                    "flex items-center gap-3 rounded-md px-2.5 py-2 text-sm font-medium transition-colors",
                    isActive
                      ? "bg-accent text-accent-foreground"
                      : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                  )}
                >
                  <item.icon className="size-4 shrink-0" aria-hidden="true" />
                  <span>{item.label}</span>
                </Link>
              );
            })}
          </nav>
        </SheetContent>
      </Sheet>
    </>
  );
}
