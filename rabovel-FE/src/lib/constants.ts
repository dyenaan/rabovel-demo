export const APP_NAME = "Rabovel";

export const APP_DESCRIPTION =
  "Institutional infrastructure for tokenized real-world assets — primary issuance, secondary trading, settlement, and custody in one platform.";

export const NAV_LINKS = {
  marketing: [
    { label: "How It Works", href: "/how-it-works" },
    { label: "Assets", href: "/assets" },
    { label: "About", href: "/about" },
  ],
  investor: [
    { label: "Dashboard", href: "/dashboard" },
    { label: "Assets", href: "/assets" },
    { label: "Primary Market", href: "/primary-market" },
    { label: "Markets", href: "/markets" },
    { label: "Trade", href: "/trade" },
    { label: "Orders", href: "/orders" },
    { label: "Trades", href: "/trades" },
    { label: "Settlements", href: "/settlements" },
    { label: "Wallet", href: "/wallet" },
    { label: "Documents", href: "/documents" },
    { label: "Security", href: "/security" },
  ],
  admin: [
    { label: "Dashboard", href: "/admin" },
    { label: "Investors", href: "/admin/investors" },
    { label: "Assets", href: "/admin/assets" },
    { label: "Issuance", href: "/admin/issuance" },
    { label: "Compliance", href: "/admin/compliance" },
    { label: "Orders", href: "/admin/orders" },
    { label: "Trades", href: "/admin/trades" },
    { label: "Settlements", href: "/admin/settlements" },
    { label: "Reconciliation", href: "/admin/reconciliation" },
    { label: "Proof of Reserves", href: "/admin/reserves" },
    { label: "Custody", href: "/admin/custody" },
    { label: "Audit Logs", href: "/admin/audit" },
    { label: "Settings", href: "/admin/settings" },
  ],
} as const;

export const QUERY_STALE_TIME = {
  realtime: 5_000,
  short: 30_000,
  medium: 60_000,
  long: 5 * 60_000,
} as const;
