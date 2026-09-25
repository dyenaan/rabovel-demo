import { Card, CardContent } from "@/components/ui/card";
import { FinancialMetric } from "@/components/financial/financial-metric";
import { mockReconciliationRecords } from "@/mocks/compliance.mock";
import { ReconciliationTable } from "./reconciliation-table";

export function ReconciliationDashboard() {
  const reconciled = mockReconciliationRecords.filter((r) => r.status === "RECONCILED").length;
  const mismatches = mockReconciliationRecords.filter((r) => r.status === "MISMATCH").length;
  const underReview = mockReconciliationRecords.filter((r) => r.status === "UNDER_REVIEW").length;

  return (
    <div className="space-y-6">
      <div className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Reconciled" value={reconciled} />
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Mismatches" value={mismatches} />
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Under Review" value={underReview} />
          </CardContent>
        </Card>
      </div>
      <ReconciliationTable />
    </div>
  );
}
