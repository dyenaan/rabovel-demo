import Link from "next/link";
import { ArrowRight } from "lucide-react";

import { Button } from "@/components/ui/button";

export function CtaSection() {
  return (
    <section className="border-t bg-primary text-primary-foreground">
      <div className="mx-auto max-w-4xl px-4 py-16 text-center sm:px-6 lg:px-8">
        <h2 className="text-3xl font-semibold tracking-tight text-balance">
          Ready to bring your assets on-chain?
        </h2>
        <p className="mx-auto mt-4 max-w-xl text-primary-foreground/80">
          Talk to our team about issuing, trading, or custodying tokenized assets on
          Rabovel&apos;s institutional infrastructure.
        </p>
        <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
          <Button size="lg" variant="secondary" asChild>
            <Link href="/register">
              Request Institutional Access
              <ArrowRight className="size-4" />
            </Link>
          </Button>
          <Button
            size="lg"
            variant="outline"
            className="border-primary-foreground/30 bg-transparent text-primary-foreground hover:bg-primary-foreground/10 hover:text-primary-foreground"
            asChild
          >
            <Link href="/about">Learn about Rabovel</Link>
          </Button>
        </div>
      </div>
    </section>
  );
}
