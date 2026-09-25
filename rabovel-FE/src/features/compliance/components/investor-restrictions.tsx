import { ShieldAlert } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { mockInvestors } from "@/mocks/investors.mock";

export function InvestorRestrictions() {
  const restricted = mockInvestors.filter(
    (i) => i.eligibilityStatus === "RESTRICTED" || i.accountStatus === "SUSPENDED",
  );

  return (
    <Card>
      <CardHeader>
        <CardTitle>Investor Restrictions</CardTitle>
      </CardHeader>
      <CardContent className="pb-6">
        {restricted.length === 0 ? (
          <EmptyState icon={ShieldAlert} title="No restricted investors" />
        ) : (
          <ul className="divide-y">
            {restricted.map((investor) => (
              <li key={investor.investorId} className="flex items-center justify-between py-3 first:pt-0 last:pb-0">
                <div>
                  <p className="text-sm font-medium text-foreground">{investor.fullName}</p>
                  <p className="text-xs text-muted-foreground">{investor.jurisdiction}</p>
                </div>
                <div className="flex gap-2">
                  <StatusBadge status={investor.eligibilityStatus} />
                  <StatusBadge status={investor.accountStatus} />
                </div>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
