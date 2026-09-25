import Link from "next/link";

import { APP_DESCRIPTION } from "@/lib/constants";
import { Logo } from "./logo";

const footerColumns = [
  {
    title: "Platform",
    links: [
      { label: "How It Works", href: "/how-it-works" },
      { label: "Assets", href: "/assets" },
      { label: "Primary Market", href: "/primary-market" },
      { label: "Markets", href: "/markets" },
    ],
  },
  {
    title: "Institutional",
    links: [
      { label: "About Rabovel", href: "/about" },
      { label: "Custody & Security", href: "/about#custody" },
      { label: "Compliance", href: "/about#compliance" },
    ],
  },
  {
    title: "Account",
    links: [
      { label: "Log in", href: "/login" },
      { label: "Request Access", href: "/register" },
    ],
  },
];

export function MarketingFooter() {
  return (
    <footer className="border-t bg-background">
      <div className="mx-auto max-w-6xl px-4 py-12 sm:px-6 lg:px-8">
        <div className="grid gap-10 md:grid-cols-[2fr_1fr_1fr_1fr]">
          <div className="space-y-3">
            <Logo />
            <p className="max-w-xs text-sm text-muted-foreground">{APP_DESCRIPTION}</p>
          </div>
          {footerColumns.map((column) => (
            <div key={column.title} className="space-y-3">
              <p className="text-sm font-medium text-foreground">{column.title}</p>
              <ul className="space-y-2">
                {column.links.map((link) => (
                  <li key={link.href}>
                    <Link
                      href={link.href}
                      className="text-sm text-muted-foreground hover:text-foreground"
                    >
                      {link.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
        <div className="mt-10 flex flex-col gap-2 border-t pt-6 text-xs text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
          <p>© {new Date().getFullYear()} Rabovel Capital Markets Ltd. All rights reserved.</p>
          <p>Tokenized securities are offered only to eligible institutional and accredited investors.</p>
        </div>
      </div>
    </footer>
  );
}
