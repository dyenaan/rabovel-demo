import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { EligibilityTable } from "./eligibility-table";
import { InvestorRestrictions } from "./investor-restrictions";
import { KycReviewQueue } from "./kyc-review-queue";

export function ComplianceDashboard() {
  return (
    <div className="space-y-6">
      <Tabs defaultValue="queue">
        <TabsList>
          <TabsTrigger value="queue">KYC Review Queue</TabsTrigger>
          <TabsTrigger value="eligibility">Eligibility</TabsTrigger>
        </TabsList>
        <TabsContent value="queue" className="pt-4">
          <KycReviewQueue />
        </TabsContent>
        <TabsContent value="eligibility" className="pt-4">
          <EligibilityTable />
        </TabsContent>
      </Tabs>

      <InvestorRestrictions />
    </div>
  );
}
