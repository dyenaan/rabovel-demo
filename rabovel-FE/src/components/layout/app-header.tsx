"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { Bell, LogOut, Settings, ShieldCheck, User, Wifi, WifiOff } from "lucide-react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useAuth } from "@/features/auth/hooks/use-auth";
import { useConnectionStatus } from "@/lib/websocket/hooks";
import { formatRelativeTime } from "@/lib/formatters";
import { mockActivity } from "@/mocks/portfolio.mock";
import { ThemeToggle } from "./theme-toggle";

function initials(name: string) {
  return name
    .split(" ")
    .map((part) => part[0])
    .slice(0, 2)
    .join("")
    .toUpperCase();
}

export function AppHeader() {
  const router = useRouter();
  const { user, logout } = useAuth();
  const connectionStatus = useConnectionStatus();
  const isLive = connectionStatus === "open";

  async function handleLogout() {
    await logout();
    router.push("/login");
  }

  return (
    <header className="sticky top-0 z-30 flex h-16 items-center gap-3 border-b bg-background px-4 sm:px-6">
      <div className="flex-1" />

      <Tooltip>
        <TooltipTrigger asChild>
          <span className="hidden items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs text-muted-foreground sm:inline-flex">
            {isLive ? (
              <Wifi className="size-3.5 text-success" aria-hidden="true" />
            ) : (
              <WifiOff className="size-3.5 text-muted-foreground" aria-hidden="true" />
            )}
            {isLive ? "Live" : "Connecting…"}
          </span>
        </TooltipTrigger>
        <TooltipContent>Real-time market data connection</TooltipContent>
      </Tooltip>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="relative" aria-label="Notifications">
            <Bell className="size-4" />
            <span className="absolute top-1.5 right-1.5 size-1.5 rounded-full bg-primary" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-80">
          <DropdownMenuLabel>Recent activity</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {mockActivity.slice(0, 4).map((event) => (
            <DropdownMenuItem key={event.eventId} className="flex-col items-start gap-0.5">
              <span className="text-sm font-medium">{event.title}</span>
              <span className="text-xs text-muted-foreground">
                {formatRelativeTime(event.timestamp)}
              </span>
            </DropdownMenuItem>
          ))}
          <DropdownMenuSeparator />
          <DropdownMenuItem asChild>
            <Link href="/dashboard">View all activity</Link>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <ThemeToggle />

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" className="gap-2 px-2">
            <Avatar className="size-7">
              <AvatarFallback>{user ? initials(user.name) : <User className="size-4" />}</AvatarFallback>
            </Avatar>
            <div className="hidden text-left sm:block">
              <p className="text-sm leading-tight font-medium">{user?.name ?? "Guest"}</p>
              <p className="text-xs leading-tight text-muted-foreground">
                <Badge variant="muted" className="px-1.5 py-0 text-[10px]">
                  {user?.role ?? "—"}
                </Badge>
              </p>
            </div>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuLabel>{user?.email}</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem asChild>
            <Link href="/security">
              <ShieldCheck className="size-4" />
              Security
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link href="/documents">
              <Settings className="size-4" />
              Account settings
            </Link>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem variant="destructive" onSelect={handleLogout}>
            <LogOut className="size-4" />
            Log out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </header>
  );
}
