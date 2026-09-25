import { Card, CardContent } from "@/components/ui/card";
import { FinancialMetric } from "@/components/financial/financial-metric";
import { Money } from "@/components/financial/money";
import { Percentage } from "@/components/financial/percentage";
import { mockReserveStatus } from "@/mocks/primary-market.mock";

export function ReserveCoverage() {
  const totalIssued = mockReserveStatus.reduce((sum, r) => sum + Number(r.totalIssued), 0);
  const totalReserved = mockReserveStatus.reduce((sum, r) => sum + Number(r.totalReserved), 0);
  const coverage = totalIssued === 0 ? 0 : (totalReserved / totalIssued) * 100;

  return (
    <div className="grid gap-4 sm:grid-cols-3">
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric label="Reserve Coverage" value={<Percentage value={coverage} signed={false} />} />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric label="Assets Issued" value={<Money value={totalIssued} compact />} />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric label="Assets Backed" value={<Money value={totalReserved} compact />} />
        </CardContent>
      </Card>
    </div>
  );
}
