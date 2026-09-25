import { BarChart3, ShieldCheck, Zap } from "lucide-react";

import { Logo } from "@/components/layout/logo";
import { ThemeToggle } from "@/components/layout/theme-toggle";
import { APP_DESCRIPTION } from "@/lib/constants";

const HIGHLIGHTS = [
  {
    icon: ShieldCheck,
    title: "Regulated & compliant",
    description: "KYC/AML-verified onboarding and bank-grade custody for every account.",
  },
  {
    icon: Zap,
    title: "Real-time settlement",
    description: "Trades clear and settle on-chain in seconds, not days.",
  },
  {
    icon: BarChart3,
    title: "Full portfolio visibility",
    description: "Track holdings, orders, and proof of reserves from one dashboard.",
  },
];

export default function AuthLayout({ children }: LayoutProps<"/">) {
  return (
    <div className="flex min-h-screen">
      <aside className="relative hidden w-full max-w-md flex-col justify-between overflow-hidden bg-primary px-10 py-12 text-primary-foreground lg:flex xl:max-w-lg">
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 opacity-[0.07] [background-image:radial-gradient(circle_at_1px_1px,white_1px,transparent_0)] [background-size:24px_24px]"
        />

        <Logo className="relative brightness-0 invert" />

        <div className="relative space-y-10">
          <h1 className="text-balance text-2xl font-semibold leading-snug">{APP_DESCRIPTION}</h1>
          <ul className="space-y-6">
            {HIGHLIGHTS.map(({ icon: Icon, title, description }) => (
              <li key={title} className="flex gap-3">
                <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary-foreground/15">
                  <Icon className="size-4.5" aria-hidden="true" />
                </span>
                <div>
                  <p className="text-sm font-medium">{title}</p>
                  <p className="mt-0.5 text-sm text-primary-foreground/70">{description}</p>
                </div>
              </li>
            ))}
          </ul>
        </div>

        <p className="relative text-xs text-primary-foreground/60">
          &copy; {new Date().getFullYear()} Rabovel. All rights reserved.
        </p>
      </aside>

      <main className="flex flex-1 flex-col bg-secondary/30">
        <header className="flex items-center justify-between px-4 py-4 sm:px-6 lg:justify-end">
          <Logo className="lg:hidden" />
          <ThemeToggle />
        </header>
        <div className="flex flex-1 items-center justify-center px-4 pb-12">
          <div className="w-full max-w-md">{children}</div>
        </div>
      </main>
    </div>
  );
}
