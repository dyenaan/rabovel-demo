"use client";

import Link from "next/link";
import { ArrowRight, BadgeCheck, ShieldCheck, UserCheck } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { StatusBadge } from "@/components/shared/status-badge";
import { useAuth } from "@/features/auth/hooks/use-auth";
import { mockInvestors } from "@/mocks/investors.mock";

export function ComplianceSummaryCard() {
  const { user } = useAuth();
  const investor = mockInvestors.find((i) => i.investorId === user?.investorId) ?? mockInvestors[0];

  const rows: { icon: LucideIcon; label: string; status: string }[] = [
    { icon: ShieldCheck, label: "KYC Verification", status: investor.kycStatus },
    { icon: BadgeCheck, label: "Eligibility", status: investor.eligibilityStatus },
    { icon: UserCheck, label: "Account Status", status: investor.accountStatus },
  ];

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>Compliance Status</CardTitle>
      </CardHeader>
      <CardContent className="flex h-full flex-col justify-between gap-6 pb-6">
        <div className="space-y-4">
          {rows.map(({ icon: Icon, label, status }) => (
            <div key={label} className="flex items-center justify-between">
              <span className="flex items-center gap-3 text-sm text-foreground">
                <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted">
                  <Icon className="size-4 text-muted-foreground" aria-hidden="true" />
                </span>
                {label}
              </span>
              <StatusBadge status={status} />
            </div>
          ))}
        </div>
        <Button variant="ghost" size="sm" className="w-full justify-between" asChild>
          <Link href="/security">
            Manage security & compliance
            <ArrowRight className="size-4" />
          </Link>
        </Button>
      </CardContent>
    </Card>
  );
}
