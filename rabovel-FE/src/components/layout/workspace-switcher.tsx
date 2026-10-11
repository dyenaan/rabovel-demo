"use client";

import Link from "next/link";
import { Building2, ChevronDown } from "lucide-react";
import { usePathname } from "next/navigation";

import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { useAuth } from "@/features/auth/hooks/use-auth";
import { workspacesForUser } from "@/features/auth/role-access";

export function WorkspaceSwitcher() {
  const { user } = useAuth();
  const pathname = usePathname();
  const workspaces = workspacesForUser(user);
  if (!workspaces.length) return null;
  const active = workspaces.find((workspace) => workspace.href === "/dashboard"
    ? !pathname.startsWith("/issuer") && !pathname.startsWith("/admin")
    : pathname === workspace.href || pathname.startsWith(`${workspace.href}/`));

  if (workspaces.length === 1) {
    return <Button asChild variant="outline" size="sm"><Link href={workspaces[0].href}><Building2 className="size-4" /><span className="sm:hidden">{workspaces[0].label.replace(" workspace", "")}</span><span className="hidden sm:inline">{workspaces[0].label}</span></Link></Button>;
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="sm" aria-label="Switch workspace"><Building2 className="size-4" /><span className="sm:hidden">{active?.label.replace(" workspace", "") ?? "Workspace"}</span><span className="hidden sm:inline">{active?.label ?? "Workspace"}</span><ChevronDown className="size-4" /></Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        {workspaces.map((workspace) => <DropdownMenuItem key={workspace.href} asChild><Link href={workspace.href} aria-current={active?.href === workspace.href ? "page" : undefined}>{workspace.label}</Link></DropdownMenuItem>)}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
