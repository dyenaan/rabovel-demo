import type { Market, OrderBookLevel, OrderBookSnapshot, PricePoint, PublicTrade } from "@/types";

export const mockMarkets: Market[] = [
  {
    marketId: "mkt_rtf_usd",
    symbol: "RTF/NGN",
    assetName: "Rabovel Treasury Fund",
    baseAsset: "RTF",
    quoteAsset: "NGN",
    lastPrice: "100.41",
    priceChange24h: "0.03",
    priceChangePercent24h: "0.03",
    bid: "100.40",
    ask: "100.43",
    high24h: "100.44",
    low24h: "100.38",
    volume24h: "3820450",
    status: "OPEN",
  },
  {
    marketId: "mkt_gif_usd",
    symbol: "GIF/NGN",
    assetName: "Global Infrastructure Fund",
    baseAsset: "GIF",
    quoteAsset: "NGN",
    lastPrice: "105.20",
    priceChange24h: "1.15",
    priceChangePercent24h: "1.11",
    bid: "105.10",
    ask: "105.35",
    high24h: "105.60",
    low24h: "103.90",
    volume24h: "1245800",
    status: "OPEN",
  },
  {
    marketId: "mkt_pref_usd",
    symbol: "PREF/NGN",
    assetName: "Prime Real Estate Fund",
    baseAsset: "PREF",
    quoteAsset: "NGN",
    lastPrice: "97.60",
    priceChange24h: "-0.85",
    priceChangePercent24h: "-0.86",
    bid: "97.45",
    ask: "97.70",
    high24h: "98.60",
    low24h: "97.20",
    volume24h: "982300",
    status: "OPEN",
  },
  {
    marketId: "mkt_pco_usd",
    symbol: "PCO/NGN",
    assetName: "Private Credit Opportunities",
    baseAsset: "PCO",
    quoteAsset: "NGN",
    lastPrice: "101.75",
    priceChange24h: "0.20",
    priceChangePercent24h: "0.20",
    bid: "101.60",
    ask: "101.90",
    high24h: "101.95",
    low24h: "101.40",
    volume24h: "445600",
    status: "OPEN",
  },
  {
    marketId: "mkt_crf_usd",
    symbol: "CRF/NGN",
    assetName: "Commodity Reserve Fund",
    baseAsset: "CRF",
    quoteAsset: "NGN",
    lastPrice: "2415.80",
    priceChange24h: "-12.40",
    priceChangePercent24h: "-0.51",
    bid: "2414.90",
    ask: "2417.10",
    high24h: "2431.00",
    low24h: "2408.50",
    volume24h: "6120000",
    status: "OPEN",
  },
  {
    marketId: "mkt_pge_usd",
    symbol: "PGE/NGN",
    assetName: "Pre-IPO Growth Equity Fund",
    baseAsset: "PGE",
    quoteAsset: "NGN",
    lastPrice: "0",
    status: "PRE_OPEN",
  },
];

function seededNoise(seed: number): number {
  return (((seed * 9301 + 49297) % 233280) / 233280) - 0.5;
}

function buildOrderBookSide(
  basePrice: number,
  direction: 1 | -1,
  levels = 12,
): OrderBookLevel[] {
  let cumulative = 0;
  return Array.from({ length: levels }, (_, i) => {
    const price = basePrice + direction * (i + 1) * basePrice * 0.0006;
    const size = 500 + Math.abs(seededNoise(i * direction)) * 4000;
    cumulative += size;
    return {
      price: price.toFixed(2),
      size: size.toFixed(2),
      total: cumulative.toFixed(2),
    };
  });
}

export function getMockOrderBook(marketId: string): OrderBookSnapshot {
  const market = mockMarkets.find((m) => m.marketId === marketId) ?? mockMarkets[0];
  const base = Number(market.lastPrice) || 100;
  return {
    marketId: market.marketId,
    bids: buildOrderBookSide(base, -1),
    asks: buildOrderBookSide(base, 1).reverse(),
    updatedAt: new Date().toISOString(),
  };
}

export function getMockPriceHistory(marketId: string, points = 120): PricePoint[] {
  const market = mockMarkets.find((m) => m.marketId === marketId) ?? mockMarkets[0];
  const base = Number(market.lastPrice) || 100;
  let last = base * 0.96;

  return Array.from({ length: points }, (_, i) => {
    const timestamp = new Date();
    timestamp.setHours(timestamp.getHours() - (points - i));
    const change = seededNoise(i) * base * 0.01;
    const open = last;
    const close = Math.max(open + change, base * 0.5);
    const high = Math.max(open, close) + Math.abs(seededNoise(i + 1)) * base * 0.003;
    const low = Math.min(open, close) - Math.abs(seededNoise(i + 2)) * base * 0.003;
    last = close;
    return {
      timestamp: timestamp.toISOString(),
      open: open.toFixed(2),
      high: high.toFixed(2),
      low: low.toFixed(2),
      close: close.toFixed(2),
      volume: (1000 + Math.abs(seededNoise(i + 3)) * 20000).toFixed(0),
    };
  });
}

export function getMockRecentTrades(marketId: string, count = 25): PublicTrade[] {
  const market = mockMarkets.find((m) => m.marketId === marketId) ?? mockMarkets[0];
  const base = Number(market.lastPrice) || 100;

  return Array.from({ length: count }, (_, i) => {
    const timestamp = new Date();
    timestamp.setSeconds(timestamp.getSeconds() - i * 47);
    const price = base + seededNoise(i) * base * 0.002;
    return {
      tradeId: `ptrade_${marketId}_${i}`,
      marketId,
      price: price.toFixed(2),
      quantity: (10 + Math.abs(seededNoise(i + 5)) * 500).toFixed(2),
      side: i % 2 === 0 ? "BUY" : "SELL",
      executedAt: timestamp.toISOString(),
    };
  });
}
