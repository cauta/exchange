/**
 * Hook for managing user orders with WebSocket subscriptions
 */

import { useEffect } from "react";
import { useExchangeStore } from "../store";
import { useExchangeClient } from "./useExchangeClient";

/**
 * Hook that fetches initial orders via REST and subscribes to WebSocket updates
 * Orders are stored in Zustand store and automatically updated via WebSocket
 */
export function useOrders() {
  const client = useExchangeClient();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const setOrders = useExchangeStore((state) => state.setOrders);
  const updateOrder = useExchangeStore((state) => state.updateOrder);
  const orders = useExchangeStore((state) => state.orders);

  useEffect(() => {
    if (!userAddress || !isAuthenticated) {
      setOrders([]);
      return;
    }

    // Fetch initial orders via REST (returns enhanced orders from SDK)
    const fetchInitialOrders = async () => {
      try {
        const result = await client.getOrders(userAddress, selectedMarketId || undefined);
        setOrders(result);
      } catch (err) {
        console.error("Failed to fetch initial orders:", err);
        setOrders([]);
      }
    };

    fetchInitialOrders();

    // Subscribe to WebSocket order updates
    const unsubscribe = client.onUserOrders(userAddress, (orderUpdate) => {
      updateOrder(orderUpdate.order_id, orderUpdate.status, orderUpdate.filled_size);
    });

    return () => {
      unsubscribe();
    };
  }, [userAddress, isAuthenticated, selectedMarketId, client, setOrders, updateOrder]);

  // Filter orders by selected market if market is selected
  return selectedMarketId ? orders.filter((o) => o.market_id === selectedMarketId) : orders;
}
