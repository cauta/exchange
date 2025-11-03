"use client";

import { useState, useEffect } from "react";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import type { Order } from "@/lib/types/exchange";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

export function RecentOrders() {
  const client = useExchangeClient();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);

  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!userAddress || !isAuthenticated || !selectedMarketId) {
      setOrders([]);
      return;
    }

    const fetchOrders = async () => {
      setLoading(true);
      try {
        const result = await client.getOrders(userAddress, selectedMarketId);
        setOrders(result);
      } catch (err) {
        console.error("Failed to fetch orders:", err);
        setOrders([]);
      } finally {
        setLoading(false);
      }
    };

    fetchOrders();
    const interval = setInterval(fetchOrders, 2000); // Refresh every 2 seconds

    return () => clearInterval(interval);
  }, [userAddress, isAuthenticated, selectedMarketId, client]);

  if (!selectedMarketId || !selectedMarket) {
    return <p className="text-muted-foreground text-sm">Select a market to view orders</p>;
  }

  return (
    <div>
      <div className="overflow-auto max-h-80">
        {loading && !orders.length ? (
          <p className="text-muted-foreground text-sm">Loading orders...</p>
        ) : !isAuthenticated || !userAddress ? (
          <p className="text-muted-foreground text-sm">Connect your wallet to view orders</p>
        ) : orders.length === 0 ? (
          <p className="text-muted-foreground text-sm">No orders found</p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Side</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Price</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Filled</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Time</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {orders.map((order) => (
                <TableRow key={order.id}>
                  <TableCell>
                    <span className={`font-bold ${order.side === "buy" ? "text-green-500" : "text-red-500"}`}>
                      {order.side === "buy" ? "Buy" : "Sell"}
                    </span>
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {order.order_type === "limit" ? "Limit" : "Market"}
                  </TableCell>
                  <TableCell className="font-mono">{order.priceDisplay}</TableCell>
                  <TableCell className="font-mono text-muted-foreground">{order.sizeDisplay}</TableCell>
                  <TableCell className="font-mono text-muted-foreground">{order.filledDisplay}</TableCell>
                  <TableCell>
                    <span
                      className={`text-xs px-2 py-1 font-semibold uppercase tracking-wide ${
                        order.status === "filled"
                          ? "bg-green-500/10 text-green-500 border border-green-500/20"
                          : order.status === "partially_filled"
                            ? "bg-yellow-500/10 text-yellow-500 border border-yellow-500/20"
                            : order.status === "cancelled"
                              ? "bg-muted text-muted-foreground border border-border"
                              : "bg-blue-500/10 text-blue-500 border border-blue-500/20"
                      }`}
                    >
                      {order.status.replace("_", " ")}
                    </span>
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {order.created_at.toLocaleTimeString()}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  );
}
