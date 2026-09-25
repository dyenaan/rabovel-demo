import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { OrderSide, OrderType } from "@/types";

type OrderBookDensity = "compact" | "comfortable";

type TradingPreferencesState = {
  selectedMarketId: string;
  defaultOrderType: OrderType;
  defaultSide: OrderSide;
  orderBookDensity: OrderBookDensity;
  orderBookDepth: number;
  showRecentTradesInline: boolean;
  setSelectedMarketId: (marketId: string) => void;
  setDefaultOrderType: (type: OrderType) => void;
  setDefaultSide: (side: OrderSide) => void;
  setOrderBookDensity: (density: OrderBookDensity) => void;
  setOrderBookDepth: (depth: number) => void;
  toggleRecentTradesInline: () => void;
};

export const useTradingPreferencesStore = create<TradingPreferencesState>()(
  persist(
    (set) => ({
      selectedMarketId: "mkt_rtf_usd",
      defaultOrderType: "LIMIT",
      defaultSide: "BUY",
      orderBookDensity: "comfortable",
      orderBookDepth: 12,
      showRecentTradesInline: true,
      setSelectedMarketId: (marketId) => set({ selectedMarketId: marketId }),
      setDefaultOrderType: (type) => set({ defaultOrderType: type }),
      setDefaultSide: (side) => set({ defaultSide: side }),
      setOrderBookDensity: (density) => set({ orderBookDensity: density }),
      setOrderBookDepth: (depth) => set({ orderBookDepth: depth }),
      toggleRecentTradesInline: () =>
        set((s) => ({ showRecentTradesInline: !s.showRecentTradesInline })),
    }),
    { name: "rabovel-trading-preferences" },
  ),
);
