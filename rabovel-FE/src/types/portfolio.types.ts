export type Holding = {
  assetId: string;
  assetName: string;
  symbol: string;
  quantity: string;
  averageCost: string;
  marketValue: string;
  costBasis: string;
  unrealizedPnl: string;
  unrealizedPnlPercent: string;
  allocationPercent: string;
};

export type PortfolioSummary = {
  totalValue: string;
  totalCostBasis: string;
  totalUnrealizedPnl: string;
  totalUnrealizedPnlPercent: string;
  cashBalance: string;
  investedValue: string;
  ytdIncome: string;
};

export type PerformancePoint = {
  date: string;
  portfolioValue: string;
  benchmarkValue?: string;
};

export type ActivityEvent = {
  eventId: string;
  type:
    | "ORDER_PLACED"
    | "ORDER_FILLED"
    | "SETTLEMENT_UPDATE"
    | "SUBSCRIPTION"
    | "REDEMPTION"
    | "DEPOSIT"
    | "WITHDRAWAL"
    | "KYC_UPDATE"
    | "DOCUMENT_PUBLISHED";
  title: string;
  description: string;
  timestamp: string;
};
