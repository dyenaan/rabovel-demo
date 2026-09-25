import type { Metadata } from "next";
import { FileCheck2, Lock, ScanEye } from "lucide-react";

export const metadata: Metadata = { title: "About" };

export default function AboutPage() {
  return (
    <div className="mx-auto max-w-4xl px-4 py-16 sm:px-6 lg:px-8">
      <h1 className="text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">
        Institutional infrastructure for tokenized assets
      </h1>
      <p className="mt-6 text-muted-foreground">
        Rabovel was built for asset managers, custodians, and institutional investors who need
        the efficiency of on-chain settlement without compromising on the compliance, custody, and
        audit standards required by regulated capital. We are not a retail exchange, and we do not
        offer speculative crypto products — every asset on Rabovel represents a real, off-chain
        claim on treasuries, credit, real estate, infrastructure, or commodities.
      </p>

      <div id="compliance" className="mt-16 flex gap-4 scroll-mt-24">
        <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
          <ScanEye className="size-5" aria-hidden="true" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-foreground">Compliance-first design</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Every account is screened for identity, accreditation, and jurisdictional eligibility
            before it can transact. Compliance holds can pause settlement independently of trade
            execution, and every decision is auditable.
          </p>
        </div>
      </div>

      <div id="custody" className="mt-10 flex gap-4 scroll-mt-24">
        <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
          <Lock className="size-5" aria-hidden="true" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-foreground">Institutional custody</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Assets are held through regulated custody partners across multiple blockchain
            networks. Investors never manage private keys directly, and every wallet binding is
            independently verified.
          </p>
        </div>
      </div>

      <div className="mt-10 flex gap-4">
        <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
          <FileCheck2 className="size-5" aria-hidden="true" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-foreground">Continuous reconciliation</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Ledger balances, custody balances, and on-chain balances are reconciled continuously,
            with mismatches surfaced to operations before they ever reach investors.
          </p>
        </div>
      </div>
    </div>
  );
}
