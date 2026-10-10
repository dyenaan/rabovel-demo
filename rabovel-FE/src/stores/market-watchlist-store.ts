import { create } from "zustand";
import { persist } from "zustand/middleware";
import { mockNgxMarkets } from "@/mocks/markets.mock";

type MarketWatchlistState = {
  marketIds: string[];
  toggleMarket: (marketId: string) => void;
};

export const useMarketWatchlistStore = create<MarketWatchlistState>()(
  persist(
    (set) => ({
      marketIds: mockNgxMarkets.map((market) => market.marketId),
      toggleMarket: (marketId) =>
        set((state) => ({
          marketIds: state.marketIds.includes(marketId)
            ? state.marketIds.filter((id) => id !== marketId)
            : [...state.marketIds, marketId],
        })),
    }),
    {
      name: "rabovel-market-watchlist",
      version: 2,
      migrate: (persistedState) => {
        const previous = (persistedState ?? {}) as { marketIds?: unknown };
        const savedIds = Array.isArray(previous.marketIds)
          ? previous.marketIds.filter((id): id is string => typeof id === "string")
          : [];
        const listedIds = mockNgxMarkets.map((market) => market.marketId);
        const listedIdSet = new Set(listedIds);
        return {
          marketIds: [...new Set([...listedIds, ...savedIds.filter((id) => listedIdSet.has(id))])],
        } as MarketWatchlistState;
      },
      partialize: (state) => ({ marketIds: state.marketIds }),
    },
  ),
);