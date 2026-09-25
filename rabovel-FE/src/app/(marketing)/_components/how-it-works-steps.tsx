import { ShieldCheck, TrendingUp, Wallet2 } from "lucide-react";

const steps = [
  {
    icon: ShieldCheck,
    title: "Onboard & verify",
    description:
      "Complete identity verification and eligibility screening once — accreditation, jurisdiction, and investor classification are checked before any capital moves.",
  },
  {
    icon: TrendingUp,
    title: "Access primary & secondary markets",
    description:
      "Subscribe to new issuances at NAV or trade existing positions on regulated secondary markets with live order books and transparent pricing.",
  },
  {
    icon: Wallet2,
    title: "Settle with institutional custody",
    description:
      "Every trade settles through auditable custody routes with on-chain finality tracking — asset and cash legs reconcile automatically.",
  },
];

export function HowItWorksSteps() {
  return (
    <section className="border-b bg-background">
      <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-semibold tracking-tight text-foreground">
            One platform, end to end
          </h2>
          <p className="mt-4 text-muted-foreground">
            Rabovel replaces the patchwork of transfer agents, exchanges, and custodians with a
            single, auditable infrastructure layer.
          </p>
        </div>

        <div className="mt-14 grid gap-8 md:grid-cols-3">
          {steps.map((step, index) => (
            <div key={step.title} className="relative rounded-lg border bg-card p-6">
              <div className="flex items-center gap-3">
                <div className="flex size-10 items-center justify-center rounded-md bg-primary/10 text-primary">
                  <step.icon className="size-5" aria-hidden="true" />
                </div>
                <span className="text-xs font-medium text-muted-foreground">
                  Step {index + 1}
                </span>
              </div>
              <h3 className="mt-4 text-base font-semibold text-foreground">{step.title}</h3>
              <p className="mt-2 text-sm text-muted-foreground">{step.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
