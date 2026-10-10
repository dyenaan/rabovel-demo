"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { ArrowRight, CheckCircle2, CircleDashed, ShieldCheck } from "lucide-react";

import { Button } from "@/components/ui/button";

const stats = [
  { label: "Assets Under Tokenization", value: "₦1.9T+" },
  { label: "Institutional Investors", value: "140+" },
  { label: "Settlement Networks", value: "6" },
];

const pipeline = [
  { label: "KYC", state: "Complete" },
  { label: "Issuer onboarding", state: "Approved" },
  { label: "Settlement", state: "Live" },
];

export function HeroSection() {
  return (
    <section className="relative overflow-hidden border-b bg-gradient-to-b from-secondary/70 via-background to-background">
      <motion.div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.8 }}
      >
        <div className="animate-float absolute -left-12 top-12 h-48 w-48 rounded-full bg-primary/10 blur-3xl" />
        <div className="animate-float-slow absolute right-8 top-20 h-64 w-64 rounded-full bg-violet-500/10 blur-3xl" />
        <div className="animate-float absolute bottom-0 left-1/3 h-56 w-56 rounded-full bg-cyan-500/10 blur-3xl" />
      </motion.div>

      <div className="relative mx-auto max-w-6xl px-4 py-20 sm:px-6 lg:px-8 lg:py-28">
        <motion.div
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.55, ease: "easeOut" }}
          className="mx-auto max-w-3xl text-center"
        >
          <motion.span
            initial={{ opacity: 0, scale: 0.96 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: 0.1, duration: 0.35 }}
            className="inline-flex items-center rounded-full border border-primary/15 bg-background/80 px-3 py-1 text-xs font-medium text-muted-foreground shadow-sm backdrop-blur-sm"
          >
            Institutional infrastructure for tokenized real-world assets
          </motion.span>

          <h1 className="mt-6 text-4xl font-semibold tracking-tight text-balance text-foreground sm:text-5xl">
            Issue, trade, and settle real-world assets with institutional clarity
          </h1>
          <p className="mt-6 text-lg text-muted-foreground text-balance">
            Rabovel connects primary issuance, secondary market trading, custody, and compliance
            in one operating layer — so capital moves with confidence from onboarding to settlement.
          </p>

          <div className="mt-10 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <motion.div whileHover={{ y: -2, scale: 1.01 }} whileTap={{ scale: 0.99 }}>
              <Button size="lg" asChild>
                <Link href="/register">
                  Request Institutional Access
                  <ArrowRight className="size-4" />
                </Link>
              </Button>
            </motion.div>
            <motion.div whileHover={{ y: -2, scale: 1.01 }} whileTap={{ scale: 0.99 }}>
              <Button size="lg" variant="outline" asChild>
                <Link href="/how-it-works">See How It Works</Link>
              </Button>
            </motion.div>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.15, duration: 0.55 }}
          className="mx-auto mt-16 max-w-4xl"
        >
          <div className="relative overflow-hidden rounded-2xl border border-border/80 bg-card/80 p-5 shadow-[0_20px_80px_rgba(88,86,200,0.12)] backdrop-blur-sm">
            <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent" />
            <div className="flex flex-col gap-5 lg:flex-row lg:items-center lg:justify-between">
              <div className="flex items-center gap-3">
                <div className="flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
                  <ShieldCheck className="size-5" />
                </div>
                <div>
                  <p className="text-sm font-medium text-foreground">Operating status</p>
                  <p className="text-xs text-muted-foreground">Institutional pipeline is active</p>
                </div>
              </div>
              <div className="flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/10 px-3 py-1.5 text-xs font-medium text-emerald-600 dark:text-emerald-400">
                <CircleDashed className="size-3.5 animate-spin" />
                Real-time monitoring
              </div>
            </div>

            <div className="mt-6 grid gap-4 md:grid-cols-3">
              {pipeline.map((item, index) => (
                <motion.div
                  key={item.label}
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.2 + index * 0.1, duration: 0.35 }}
                  className="rounded-xl border border-border bg-background/70 p-4"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-xs font-medium text-muted-foreground">{item.label}</span>
                    <CheckCircle2 className="size-4 text-emerald-500" />
                  </div>
                  <p className="mt-3 text-lg font-semibold text-foreground">{item.state}</p>
                  <div className="mt-3 h-1.5 rounded-full bg-muted">
                    <div className="h-full rounded-full bg-gradient-to-r from-primary to-violet-500" style={{ width: `${68 + index * 12}%` }} />
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        </motion.div>

        <div className="mx-auto mt-16 grid max-w-2xl grid-cols-3 gap-6 border-t border-border/70 pt-8">
          {stats.map((stat) => (
            <motion.div
              key={stat.label}
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.35 }}
              className="text-center"
            >
              <p className="text-2xl font-semibold text-foreground font-tabular">{stat.value}</p>
              <p className="mt-1 text-xs text-muted-foreground">{stat.label}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
