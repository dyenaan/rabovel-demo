"use client";

import { useEffect, useState } from "react";
import type { Market, OrderBookSnapshot, PublicTrade } from "@/types";
import { channels } from "./subscriptions";
import type { ConnectionStatus } from "./types";
import { getWebSocketManager } from "./websocket-manager";

export function useConnectionStatus(): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>("idle");

  useEffect(() => {
    return getWebSocketManager().onStatusChange(setStatus);
  }, []);

  return status;
}

export function useMarketStream(marketId: string, initial?: Market) {
  const [market, setMarket] = useState<Market | undefined>(initial);

  useEffect(() => {
    return getWebSocketManager().subscribe<Market>(channels.market(marketId), (message) => {
      setMarket(message.payload);
    });
  }, [marketId]);

  return market;
}

export function useOrderBookStream(marketId: string, initial?: OrderBookSnapshot) {
  const [orderBook, setOrderBook] = useState<OrderBookSnapshot | undefined>(initial);

  useEffect(() => {
    return getWebSocketManager().subscribe<OrderBookSnapshot>(
      channels.orderBook(marketId),
      (message) => setOrderBook(message.payload),
    );
  }, [marketId]);

  return orderBook;
}

export function useTradeStream(marketId: string, initial: PublicTrade[] = []) {
  const [trades, setTrades] = useState<PublicTrade[]>(initial);

  useEffect(() => {
    return getWebSocketManager().subscribe<PublicTrade>(channels.trades(marketId), (message) => {
      setTrades((prev) => [message.payload, ...prev].slice(0, 50));
    });
  }, [marketId]);

  return trades;
}
