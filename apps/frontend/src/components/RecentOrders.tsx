"use client";

import { useState, useEffect, useMemo } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import type { Order } from "@/lib/types/exchange";
import { DataTable } from "@/components/ui/data-table";

export function RecentOrders() {
  const client = useExchangeClient();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);

  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(false);

  const columns = useMemo<ColumnDef<Order>[]>(
    () => [
      {
        accessorKey: "side",
        header: "Side",
        cell: ({ row }) => (
          <span className={`font-bold ${row.getValue("side") === "buy" ? "text-green-500" : "text-red-500"}`}>
            {row.getValue("side") === "buy" ? "Buy" : "Sell"}
          </span>
        ),
        size: 80,
      },
      {
        accessorKey: "order_type",
        header: "Type",
        cell: ({ row }) => (
          <span className="text-muted-foreground">
            {row.getValue("order_type") === "limit" ? "Limit" : "Market"}
          </span>
        ),
        size: 80,
      },
      {
        accessorKey: "priceDisplay",
        header: "Price",
        cell: ({ row }) => <div className="font-mono">{row.getValue("priceDisplay")}</div>,
        size: 120,
      },
      {
        accessorKey: "sizeDisplay",
        header: "Size",
        cell: ({ row }) => <div className="font-mono text-muted-foreground">{row.getValue("sizeDisplay")}</div>,
        size: 120,
      },
      {
        accessorKey: "filledDisplay",
        header: "Filled",
        cell: ({ row }) => <div className="font-mono text-muted-foreground">{row.getValue("filledDisplay")}</div>,
        size: 120,
      },
      {
        accessorKey: "status",
        header: "Status",
        cell: ({ row }) => {
          const status = row.getValue("status") as string;
          return (
            <span
              className={`text-xs px-2 py-1 font-semibold uppercase tracking-wide rounded ${
                status === "filled"
                  ? "bg-green-500/10 text-green-500 border border-green-500/20"
                  : status === "partially_filled"
                    ? "bg-yellow-500/10 text-yellow-500 border border-yellow-500/20"
                    : status === "cancelled"
                      ? "bg-muted text-muted-foreground border border-border"
                      : "bg-blue-500/10 text-blue-500 border border-blue-500/20"
              }`}
            >
              {status.replace("_", " ")}
            </span>
          );
        },
        size: 120,
      },
      {
        accessorKey: "created_at",
        header: "Time",
        cell: ({ row }) => (
          <div className="text-muted-foreground text-xs">
            {(row.getValue("created_at") as Date).toLocaleTimeString()}
          </div>
        ),
        size: 100,
      },
    ],
    []
  );

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
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">Select a market to view orders</p>
      </div>
    );
  }

  if (loading && !orders.length) {
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">Loading orders...</p>
      </div>
    );
  }

  if (!isAuthenticated || !userAddress) {
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">Connect your wallet to view orders</p>
      </div>
    );
  }

  return <DataTable columns={columns} data={orders} emptyMessage="No orders found" />;
}
