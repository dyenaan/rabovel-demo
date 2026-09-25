import { FileCheck2, GitCompareArrows, Landmark, Lock, ScanEye, Workflow } from "lucide-react";

const pillars = [
  {
    icon: ScanEye,
    title: "Identity & Compliance",
    description: "KYC/AML, accreditation, and jurisdictional eligibility enforced at every transaction boundary.",
  },
  {
    icon: Landmark,
    title: "Primary Issuance",
    description: "Structure and issue tokenized securities with automated subscription and redemption workflows.",
  },
  {
    icon: Workflow,
    title: "Secondary Trading",
    description: "Order-driven markets with transparent order books, matching, and post-trade reporting.",
  },
  {
    icon: Lock,
    title: "Custody & Multi-Chain",
    description: "Institutional custody routes across Ethereum, Base, Polygon, and more — never self-managed keys.",
  },
  {
    icon: GitCompareArrows,
    title: "Settlement Finality",
    description: "Granular finality tracking from submission through safe, finalized, and reconciled states.",
  },
  {
    icon: FileCheck2,
    title: "Reconciliation & Reserves",
    description: "Continuous three-way reconciliation across ledger, custody, and on-chain balances.",
  },
];

export function InfrastructureGrid() {
  return (
    <section className="bg-background">
      <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-semibold tracking-tight text-foreground">
            Built for institutional operations
          </h2>
          <p className="mt-4 text-muted-foreground">
            Every layer of Rabovel is designed for auditors, regulators, and risk teams — not just
            traders.
          </p>
        </div>

        <div className="mt-14 grid gap-x-8 gap-y-10 sm:grid-cols-2 lg:grid-cols-3">
          {pillars.map((pillar) => (
            <div key={pillar.title} className="flex gap-4">
              <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
                <pillar.icon className="size-5" aria-hidden="true" />
              </div>
              <div>
                <h3 className="text-sm font-semibold text-foreground">{pillar.title}</h3>
                <p className="mt-1 text-sm text-muted-foreground">{pillar.description}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
