import type { ChannelName } from "./types";

export const channels = {
  market: (marketId: string): ChannelName => `market:${marketId}`,
  orderBook: (marketId: string): ChannelName => `orderbook:${marketId}`,
  trades: (marketId: string): ChannelName => `trades:${marketId}`,
  orders: (investorId: string): ChannelName => `orders:${investorId}`,
  settlements: (investorId: string): ChannelName => `settlements:${investorId}`,
  notifications: (investorId: string): ChannelName => `notifications:${investorId}`,
};
