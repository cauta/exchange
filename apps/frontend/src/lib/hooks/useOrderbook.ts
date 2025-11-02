/**
 * Hook for subscribing to orderbook updates
 */

import { useEffect } from "react";
import { useExchangeStore, selectOrderbookBids, selectOrderbookAsks } from "../store";
import { getExchangeClient } from "../api";

export function useOrderbook(marketId: string | null) {
  const updateOrderbook = useExchangeStore((state) => state.updateOrderbook);
  const setOrderbookLoading = useExchangeStore((state) => state.setOrderbookLoading);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);

  useEffect(() => {
    if (!marketId) return;

    const client = getExchangeClient();
    setOrderbookLoading(true);

    // Subscribe to orderbook updates using SDK convenience method
    const unsubscribe = client.onOrderbook(marketId, ({ bids, asks }) => {
      updateOrderbook(marketId, bids, asks);
    });

    // Cleanup
    return unsubscribe;
  }, [marketId, updateOrderbook, setOrderbookLoading]);

  return {
    bids,
    asks,
  };
}
