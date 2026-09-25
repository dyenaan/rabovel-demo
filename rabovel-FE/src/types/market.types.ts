export type MarketStatus = "OPEN" | "CLOSED" | "HALTED" | "PRE_OPEN";

export type Market = {
  marketId: string;
  symbol: string;
  assetName: string;

  baseAsset: string;
  quoteAsset: string;

  lastPrice: string;
  priceChange24h?: string;
  priceChangePercent24h?: string;

  bid?: string;
  ask?: string;

  high24h?: string;
  low24h?: string;
  volume24h?: string;

  status: MarketStatus;
};

export type OrderBookLevel = {
  price: string;
  size: string;
  total: string;
};

export type OrderBookSnapshot = {
  marketId: string;
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
  updatedAt: string;
};

export type PricePoint = {
  timestamp: string;
  open: string;
  high: string;
  low: string;
  close: string;
  volume: string;
};

export type PublicTrade = {
  tradeId: string;
  marketId: string;
  price: string;
  quantity: string;
  side: "BUY" | "SELL";
  executedAt: string;
};
