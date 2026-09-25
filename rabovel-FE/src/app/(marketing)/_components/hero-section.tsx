"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { ArrowRight } from "lucide-react";

import { Button } from "@/components/ui/button";

const stats = [
  { label: "Assets Under Tokenization", value: "₦1.9T+" },
  { label: "Institutional Investors", value: "140+" },
  { label: "Settlement Networks", value: "6" },
];

export function HeroSection() {
  return (
    <section className="relative overflow-hidden border-b bg-gradient-to-b from-secondary/60 to-background">
      <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 lg:px-8 lg:py-28">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, ease: "easeOut" }}
          className="mx-auto max-w-3xl text-center"
        >
          <span className="inline-flex items-center rounded-full border bg-background px-3 py-1 text-xs font-medium text-muted-foreground">
            Institutional infrastructure for tokenized real-world assets
          </span>
          <h1 className="mt-6 text-4xl font-semibold tracking-tight text-balance text-foreground sm:text-5xl">
            Issue, trade, and settle real-world assets on infrastructure built for institutions
          </h1>
          <p className="mt-6 text-lg text-muted-foreground text-balance">
            Rabovel connects primary issuance, secondary market trading, custody, and compliance
            in a single platform — so tokenized treasuries, credit, real estate, and infrastructure
            assets move with the rigor institutional capital requires.
          </p>
          <div className="mt-10 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <Button size="lg" asChild>
              <Link href="/register">
                Request Institutional Access
                <ArrowRight className="size-4" />
              </Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="/how-it-works">See How It Works</Link>
            </Button>
          </div>
        </motion.div>

        <div className="mx-auto mt-16 grid max-w-2xl grid-cols-3 gap-6 border-t pt-8">
          {stats.map((stat) => (
            <div key={stat.label} className="text-center">
              <p className="text-2xl font-semibold text-foreground font-tabular">{stat.value}</p>
              <p className="mt-1 text-xs text-muted-foreground">{stat.label}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
