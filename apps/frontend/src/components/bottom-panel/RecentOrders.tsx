"use client";

import { useMemo, useState } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { useOrders } from "@/lib/hooks";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import type { Order } from "@/lib/types/exchange";
import { DataTable } from "@/components/ui/data-table";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";

export function RecentOrders() {
  const client = useExchangeClient();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const orders = useOrders();
  const [cancellingOrders, setCancellingOrders] = useState<Set<string>>(new Set());
  const [cancellingAll, setCancellingAll] = useState(false);

  const handleCancelOrder = async (orderId: string) => {
    if (!userAddress) return;

    setCancellingOrders((prev) => new Set(prev).add(orderId));
    try {
      // TODO: Get signature from wallet
      await client.cancelOrder({
        userAddress,
        orderId,
        signature: "0x", // Placeholder - need wallet integration
      });
    } catch (err) {
      console.error("Failed to cancel order:", err);
    } finally {
      setCancellingOrders((prev) => {
        const next = new Set(prev);
        next.delete(orderId);
        return next;
      });
    }
  };

  const handleCancelAll = async () => {
    if (!userAddress) return;

    setCancellingAll(true);
    try {
      // TODO: Get signature from wallet
      await client.cancelAllOrders({
        userAddress,
        marketId: selectedMarketId || undefined,
        signature: "0x", // Placeholder - need wallet integration
      });
    } catch (err) {
      console.error("Failed to cancel all orders:", err);
    } finally {
      setCancellingAll(false);
    }
  };

  const columns = useMemo<ColumnDef<Order>[]>(
    () => [
      {
        accessorKey: "market_id",
        header: "Market",
        cell: ({ row }) => <div className="font-semibold text-foreground">{row.getValue("market_id")}</div>,
        size: 100,
      },
      {
        accessorKey: "side",
        header: "Side",
        cell: ({ row }) => (
          <span className={`font-bold ${row.getValue("side") === "buy" ? "text-green-500" : "text-red-500"}`}>
            {row.getValue("side") === "buy" ? "Buy" : "Sell"}
          </span>
        ),
        size: 70,
      },
      {
        accessorKey: "order_type",
        header: "Type",
        cell: ({ row }) => (
          <span className="text-muted-foreground">{row.getValue("order_type") === "limit" ? "Limit" : "Market"}</span>
        ),
        size: 70,
      },
      {
        accessorKey: "priceDisplay",
        header: "Price",
        cell: ({ row }) => <div className="font-mono text-sm">{row.getValue("priceDisplay")}</div>,
        size: 100,
      },
      {
        accessorKey: "sizeDisplay",
        header: "Size",
        cell: ({ row }) => <div className="font-mono text-sm text-muted-foreground">{row.getValue("sizeDisplay")}</div>,
        size: 100,
      },
      {
        id: "filled",
        header: "Filled",
        cell: ({ row }) => {
          const order = row.original;
          const filledPercent = order.sizeValue > 0 ? (order.filledValue / order.sizeValue) * 100 : 0;
          return <div className="font-mono text-sm text-muted-foreground">{filledPercent.toFixed(1)}%</div>;
        },
        size: 80,
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
                      : status === "pending"
                        ? "bg-blue-500/10 text-blue-500 border border-blue-500/20"
                        : "bg-gray-500/10 text-gray-500 border border-gray-500/20"
              }`}
            >
              {status === "pending" ? "open" : status.replace("_", " ")}
            </span>
          );
        },
        size: 110,
      },
      {
        accessorKey: "created_at",
        header: "Time",
        cell: ({ row }) => (
          <div className="text-muted-foreground text-xs">
            {(row.getValue("created_at") as Date).toLocaleTimeString()}
          </div>
        ),
        size: 90,
      },
      {
        id: "actions",
        header: "",
        cell: ({ row }) => {
          const order = row.original;
          const canCancel = order.status === "pending" || order.status === "partially_filled";
          if (!canCancel) return null;

          const isCancelling = cancellingOrders.has(order.id);
          return (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleCancelOrder(order.id)}
              disabled={isCancelling}
              className="h-7 w-7 p-0 text-muted-foreground hover:text-red-500"
            >
              <X className="h-4 w-4" />
            </Button>
          );
        },
        size: 50,
      },
    ],
    [cancellingOrders]
  );

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-muted-foreground text-sm">Select a market to view orders</p>
      </div>
    );
  }

  if (!isAuthenticated || !userAddress) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-muted-foreground text-sm">Connect your wallet to view orders</p>
      </div>
    );
  }

  const hasOpenOrders = orders.some((o) => o.status === "pending" || o.status === "partially_filled");

  return (
    <div className="h-full">
      <DataTable
        columns={columns}
        data={orders}
        emptyMessage="No orders found"
        headerAction={
          hasOpenOrders ? (
            <Button
              variant="outline"
              size="sm"
              onClick={handleCancelAll}
              disabled={cancellingAll}
              className="text-red-500 hover:text-red-600 hover:bg-red-500/10 h-7"
            >
              {cancellingAll ? "Cancelling..." : "Cancel All"}
            </Button>
          ) : null
        }
      />
    </div>
  );
}
