import type { Metadata } from "next";
import { CheckCircle2 } from "lucide-react";

export const metadata: Metadata = { title: "How It Works" };

const lifecycle = [
  {
    phase: "Onboarding",
    title: "Verify identity and eligibility",
    points: [
      "Institutional or individual account setup with jurisdiction screening",
      "Document-based identity verification (KYC/AML)",
      "Accreditation and eligibility assessment against each asset's requirements",
      "Custody wallet binding for on-chain settlement",
    ],
  },
  {
    phase: "Primary Market",
    title: "Subscribe to new issuances",
    points: [
      "Review prospectus, term sheet, and NAV history before committing capital",
      "Submit subscription requests denominated in your settlement currency",
      "Track reserve status and issuance confirmation through to token delivery",
    ],
  },
  {
    phase: "Secondary Market",
    title: "Trade with transparent price discovery",
    points: [
      "Place market, limit, or stop-limit orders against live order books",
      "Track order status from creation through partial or full execution",
      "Monitor open orders, order history, and trade history in one terminal",
    ],
  },
  {
    phase: "Settlement & Custody",
    title: "Settle with auditable finality",
    points: [
      "Asset and cash legs settle independently and reconcile automatically",
      "Blockchain finality tracked from submission through finalized and reconciled",
      "All positions held in institutional custody — never self-managed private keys",
    ],
  },
];

export default function HowItWorksPage() {
  return (
    <div className="mx-auto max-w-4xl px-4 py-16 sm:px-6 lg:px-8">
      <div className="text-center">
        <h1 className="text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">
          How Rabovel works
        </h1>
        <p className="mt-4 text-muted-foreground">
          A single, auditable lifecycle from onboarding to settlement — built for institutional
          capital moving into tokenized real-world assets.
        </p>
      </div>

      <div className="mt-16 space-y-12">
        {lifecycle.map((stage, index) => (
          <div key={stage.phase} className="grid gap-6 sm:grid-cols-[auto_1fr]">
            <div className="flex sm:flex-col sm:items-center">
              <span className="flex size-9 items-center justify-center rounded-full border bg-secondary text-sm font-semibold text-foreground">
                {index + 1}
              </span>
            </div>
            <div>
              <p className="text-xs font-medium tracking-wide text-primary uppercase">
                {stage.phase}
              </p>
              <h2 className="mt-1 text-xl font-semibold text-foreground">{stage.title}</h2>
              <ul className="mt-4 space-y-2">
                {stage.points.map((point) => (
                  <li key={point} className="flex items-start gap-2 text-sm text-muted-foreground">
                    <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-success" aria-hidden="true" />
                    <span>{point}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
