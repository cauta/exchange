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
    // Note: WebSocket trades are raw and need to be enhanced manually
    // For now, we create minimal enhanced trades with placeholder display values
    const unsubscribe = client.onTrades(marketId, (trade) => {
      addTrade({
        id: trade.id,
        market_id: trade.market_id,
        buyer_address: trade.buyer_address,
        seller_address: trade.seller_address,
        buyer_order_id: "", // Not in WebSocket message
        seller_order_id: "", // Not in WebSocket message
        price: trade.price,
        size: trade.size,
        timestamp: new Date(trade.timestamp),
        // Placeholder enhanced fields - ideally these would be computed
        priceDisplay: trade.price,
        sizeDisplay: trade.size,
        priceValue: parseFloat(trade.price),
        sizeValue: parseFloat(trade.size),
      });
    });

    // Cleanup
    return unsubscribe;
  }, [marketId, addTrade]);

  return trades;
}
