import Link from "next/link";
import { ShieldAlert } from "lucide-react";

import { PageContainer } from "@/components/layout/page-container";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/shared/empty-state";

export function AccessRestricted() {
  return (
    <PageContainer>
      <EmptyState
        icon={ShieldAlert}
        title="You don't have access to this area"
        description="This section is restricted to admin, compliance, and operations roles."
        action={
          <Button asChild>
            <Link href="/dashboard">Back to Dashboard</Link>
          </Button>
        }
      />
    </PageContainer>
  );
}
