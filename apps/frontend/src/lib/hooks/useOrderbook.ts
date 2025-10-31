/**
 * Hook for subscribing to orderbook updates
 */

import { useEffect } from "react";
import { useExchangeStore, selectOrderbookBids, selectOrderbookAsks } from "../store";
import { getWebSocketManager } from "../websocket";
import type { OrderbookSnapshotMessage, OrderbookUpdateMessage } from "../types/websocket";
import type { ServerMessage } from "../types/websocket";

export function useOrderbook(marketId: string | null) {
  const updateOrderbook = useExchangeStore((state) => state.updateOrderbook);
  const setOrderbookLoading = useExchangeStore((state) => state.setOrderbookLoading);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);

  useEffect(() => {
    if (!marketId) return;

    const ws = getWebSocketManager();
    setOrderbookLoading(true);

    // Handler for orderbook messages
    const handleOrderbookSnapshot = (message: any) => {
      console.log('[useOrderbook] Snapshot received:', {
        messageMarketId: message.orderbook?.market_id,
        hookMarketId: marketId,
        matches: message.orderbook?.market_id === marketId,
        bidsCount: message.orderbook?.bids?.length,
        asksCount: message.orderbook?.asks?.length,
      });

      if (message.orderbook && message.orderbook.market_id === marketId) {
        updateOrderbook(message.orderbook.market_id, message.orderbook.bids, message.orderbook.asks);
      }
    };

    const handleOrderbookUpdate = (message: any) => {
      console.log('[useOrderbook] Update received:', {
        messageMarketId: message.orderbook?.market_id,
        hookMarketId: marketId,
        matches: message.orderbook?.market_id === marketId,
        bidsCount: message.orderbook?.bids?.length,
        asksCount: message.orderbook?.asks?.length,
      });

      if (message.orderbook && message.orderbook.market_id === marketId) {
        updateOrderbook(message.orderbook.market_id, message.orderbook.bids, message.orderbook.asks);
      }
    };

    // Register handlers
    ws.on("orderbook_snapshot", handleOrderbookSnapshot as (message: ServerMessage) => void);
    ws.on("orderbook_update", handleOrderbookUpdate as (message: ServerMessage) => void);

    // Subscribe to orderbook
    ws.subscribe("orderbook", marketId);

    // Cleanup
    return () => {
      ws.off("orderbook_snapshot", handleOrderbookSnapshot as (message: ServerMessage) => void);
      ws.off("orderbook_update", handleOrderbookUpdate as (message: ServerMessage) => void);
      ws.unsubscribe("orderbook", marketId);
    };
  }, [marketId, updateOrderbook, setOrderbookLoading]);

  return {
    bids,
    asks,
  };
}
