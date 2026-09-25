"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import type { LucideIcon } from "lucide-react";

import { cn, isNavPathActive } from "@/lib/utils";

export type BottomNavLinkItem = { label: string; href: string; icon: LucideIcon };

export function BottomNavBar({
  items,
  more,
}: {
  items: BottomNavLinkItem[];
  more: { label: string; icon: LucideIcon; isActive: boolean; onClick: () => void };
}) {
  const pathname = usePathname();
  const MoreIcon = more.icon;

  return (
    <nav
      className="fixed inset-x-0 bottom-0 z-40 flex border-t bg-background/95 backdrop-blur-sm md:hidden"
      style={{ paddingBottom: "env(safe-area-inset-bottom)" }}
    >
      {items.map((item) => {
        const isActive = isNavPathActive(pathname, item.href);
        return (
          <Link
            key={item.href}
            href={item.href}
            className={cn(
              "flex flex-1 flex-col items-center gap-1 py-2.5 text-[11px] font-medium transition-colors",
              isActive ? "text-primary" : "text-muted-foreground hover:text-foreground",
            )}
          >
            <item.icon className="size-5" aria-hidden="true" />
            {item.label}
          </Link>
        );
      })}
      <button
        type="button"
        onClick={more.onClick}
        className={cn(
          "flex flex-1 flex-col items-center gap-1 py-2.5 text-[11px] font-medium transition-colors",
          more.isActive ? "text-primary" : "text-muted-foreground hover:text-foreground",
        )}
      >
        <MoreIcon className="size-5" aria-hidden="true" />
        {more.label}
      </button>
    </nav>
  );
}
