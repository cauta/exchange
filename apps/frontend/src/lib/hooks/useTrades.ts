/**
 * Hook for subscribing to trade updates
 */

import { useEffect } from "react";
import { useExchangeStore, selectRecentTrades } from "../store";
import { getExchangeClient } from "../api";

export function useTrades(marketId: string | null) {
  const addTrade = useExchangeStore((state) => state.addTrade);
  const trades = useExchangeStore(selectRecentTrades);

  useEffect(() => {
    if (!marketId) return;

    const client = getExchangeClient();

    // Subscribe to trade updates using SDK convenience method
    const unsubscribe = client.onTrades(marketId, (trade) => {
      addTrade({
        id: trade.id,
        market_id: trade.market_id,
        buyer_address: trade.buyer_address,
        seller_address: trade.seller_address,
        price: trade.price,
        size: trade.size,
        buyer_fee: "0", // Not included in WebSocket message
        seller_fee: "0", // Not included in WebSocket message
        timestamp: trade.timestamp,
      });
    });

    // Cleanup
    return unsubscribe;
  }, [marketId, addTrade]);

  return trades;
}
