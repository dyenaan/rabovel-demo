export const queryKeys = {
  assets: {
    all: ["assets"] as const,
    list: (filters?: Record<string, unknown>) =>
      ["assets", "list", filters] as const,
    detail: (assetId: string) => ["assets", "detail", assetId] as const,
    navHistory: (assetId: string) => ["assets", "nav-history", assetId] as const,
  },
  markets: {
    all: ["markets"] as const,
    list: (filters?: Record<string, unknown>) =>
      ["markets", "list", filters] as const,
    detail: (marketId: string) => ["markets", "detail", marketId] as const,
    orderBook: (marketId: string) => ["markets", "order-book", marketId] as const,
    priceHistory: (marketId: string, range: string) =>
      ["markets", "price-history", marketId, range] as const,
    recentTrades: (marketId: string) => ["markets", "recent-trades", marketId] as const,
  },
  portfolio: {
    all: ["portfolio"] as const,
    summary: () => ["portfolio", "summary"] as const,
    holdings: () => ["portfolio", "holdings"] as const,
    performance: (range: string) => ["portfolio", "performance", range] as const,
    activity: () => ["portfolio", "activity"] as const,
  },
  orders: {
    all: ["orders"] as const,
    list: (filters?: Record<string, unknown>) =>
      ["orders", "list", filters] as const,
    detail: (orderId: string) => ["orders", "detail", orderId] as const,
  },
  trades: {
    all: ["trades"] as const,
    list: (filters?: Record<string, unknown>) =>
      ["trades", "list", filters] as const,
  },
  settlements: {
    all: ["settlements"] as const,
    list: (filters?: Record<string, unknown>) =>
      ["settlements", "list", filters] as const,
    detail: (settlementId: string) => ["settlements", "detail", settlementId] as const,
  },
  custody: {
    all: ["custody"] as const,
    wallets: () => ["custody", "wallets"] as const,
  },
  investors: {
    all: ["investors"] as const,
    list: (filters?: Record<string, unknown>) =>
      ["investors", "list", filters] as const,
    detail: (investorId: string) => ["investors", "detail", investorId] as const,
  },
  compliance: {
    all: ["compliance"] as const,
    queue: () => ["compliance", "queue"] as const,
    cases: (filters?: Record<string, unknown>) =>
      ["compliance", "cases", filters] as const,
  },
  reconciliation: {
    all: ["reconciliation"] as const,
    list: (filters?: Record<string, unknown>) =>
      ["reconciliation", "list", filters] as const,
  },
  reserves: {
    all: ["reserves"] as const,
    coverage: () => ["reserves", "coverage"] as const,
    attestations: () => ["reserves", "attestations"] as const,
  },
  admin: {
    dashboard: () => ["admin", "dashboard"] as const,
  },
} as const;
