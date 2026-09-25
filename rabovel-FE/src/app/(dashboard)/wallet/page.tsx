import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { ConnectWalletDialog } from "@/features/custody/components/connect-wallet-dialog";
import { CustodyOverview } from "@/features/custody/components/custody-overview";
import { WalletBindingList } from "@/features/custody/components/wallet-binding-list";
import { InvestorCngnCard } from "@/features/custody/components/investor-cngn-card";

export const metadata: Metadata = { title: "Wallet" };

export default function WalletPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Wallet & Custody"
        description="Manage the custody wallets bound to your account for settlement."
        actions={<ConnectWalletDialog />}
      />
      <div className="space-y-6">
        <CustodyOverview />
        <WalletBindingList />
        <InvestorCngnCard />
      </div>
    </PageContainer>
  );
}
