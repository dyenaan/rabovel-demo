import { AttestationTimeline } from "./attestation-timeline";
import { ReserveAssetsTable } from "./reserve-assets-table";
import { ReserveCoverage } from "./reserve-coverage";

export function ProofOfReservesDashboard() {
  return (
    <div className="space-y-6">
      <ReserveCoverage />
      <div className="grid gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2">
          <ReserveAssetsTable />
        </div>
        <AttestationTimeline />
      </div>
    </div>
  );
}
