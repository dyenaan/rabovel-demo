import type { ActivityEvent, Holding, PerformancePoint, PortfolioSummary } from "@/types";

export const mockPortfolioSummary: PortfolioSummary = {
  totalValue: "4218650.32",
  totalCostBasis: "3980200.00",
  totalUnrealizedPnl: "238450.32",
  totalUnrealizedPnlPercent: "5.99",
  cashBalance: "182400.10",
  investedValue: "4036250.22",
  ytdIncome: "146820.55",
};

export const mockHoldings: Holding[] = [
  {
    assetId: "ast_treasury_01",
    assetName: "Rabovel Treasury Fund",
    symbol: "RTF",
    quantity: "15000",
    averageCost: "100.10",
    marketValue: "1506150.00",
    costBasis: "1501500.00",
    unrealizedPnl: "4650.00",
    unrealizedPnlPercent: "0.31",
    allocationPercent: "35.7",
  },
  {
    assetId: "ast_infra_01",
    assetName: "Global Infrastructure Fund",
    symbol: "GIF",
    quantity: "6200",
    averageCost: "98.40",
    marketValue: "652240.00",
    costBasis: "610080.00",
    unrealizedPnl: "42160.00",
    unrealizedPnlPercent: "6.91",
    allocationPercent: "15.5",
  },
  {
    assetId: "ast_realestate_01",
    assetName: "Prime Real Estate Fund",
    symbol: "PREF",
    quantity: "8100",
    averageCost: "99.85",
    marketValue: "790560.00",
    costBasis: "808785.00",
    unrealizedPnl: "-18225.00",
    unrealizedPnlPercent: "-2.25",
    allocationPercent: "18.7",
  },
  {
    assetId: "ast_credit_01",
    assetName: "Private Credit Opportunities",
    symbol: "PCO",
    quantity: "9000",
    averageCost: "100.50",
    marketValue: "915750.00",
    costBasis: "904500.00",
    unrealizedPnl: "11250.00",
    unrealizedPnlPercent: "1.24",
    allocationPercent: "21.7",
  },
  {
    assetId: "ast_commodity_01",
    assetName: "Commodity Reserve Fund",
    symbol: "CRF",
    quantity: "70",
    averageCost: "2350.00",
    marketValue: "169106.00",
    costBasis: "164500.00",
    unrealizedPnl: "4606.00",
    unrealizedPnlPercent: "2.80",
    allocationPercent: "4.0",
  },
];

function seededNoise(seed: number): number {
  return (((seed * 9301 + 49297) % 233280) / 233280) - 0.5;
}

export const mockPerformanceHistory: PerformancePoint[] = Array.from(
  { length: 180 },
  (_, i) => {
    const date = new Date();
    date.setDate(date.getDate() - (179 - i));
    const base = 3800000;
    const drift = (i / 180) * base * 0.11;
    const noise = seededNoise(i) * base * 0.006;
    const benchmarkDrift = (i / 180) * base * 0.07;
    return {
      date: date.toISOString().slice(0, 10),
      portfolioValue: (base + drift + noise).toFixed(2),
      benchmarkValue: (base + benchmarkDrift + noise * 0.6).toFixed(2),
    };
  },
);

export const mockActivity: ActivityEvent[] = [
  {
    eventId: "act_1",
    type: "ORDER_FILLED",
    title: "Order filled — GIF/NGN",
    description: "640.00 GIF filled at avg price ₦104.75",
    timestamp: new Date(Date.now() - 1000 * 60 * 40).toISOString(),
  },
  {
    eventId: "act_2",
    type: "SETTLEMENT_UPDATE",
    title: "Settlement signed — stl_5002",
    description: "Trade trd_9002 settlement moved to SIGNED",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 5).toISOString(),
  },
  {
    eventId: "act_3",
    type: "SUBSCRIPTION",
    title: "Subscription submitted — Private Credit Opportunities",
    description: "Subscription request for ₦150,000 submitted for review",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 26).toISOString(),
  },
  {
    eventId: "act_4",
    type: "DOCUMENT_PUBLISHED",
    title: "New document published",
    description: "Q2 2025 Reserve Audit Report available for Rabovel Treasury Fund",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 48).toISOString(),
  },
  {
    eventId: "act_5",
    type: "KYC_UPDATE",
    title: "Eligibility re-verified",
    description: "Annual accreditation re-verification completed successfully",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 96).toISOString(),
  },
];
