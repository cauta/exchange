"use client";

import { useMemo } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { useUserTrades } from "@/lib/hooks";
import type { Trade } from "@/lib/types/exchange";
import { DataTable } from "@/components/ui/data-table";

type EnhancedTrade = Trade & {
  side: "buy" | "sell";
};

export function RecentTrades() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const userTrades = useUserTrades();

  // Add side information to each trade
  const trades: EnhancedTrade[] = useMemo(
    () =>
      userTrades.map((trade) => ({
        ...trade,
        side: trade.buyer_address === userAddress ? "buy" : "sell",
      })),
    [userTrades, userAddress]
  );

  const columns = useMemo<ColumnDef<EnhancedTrade>[]>(
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
        cell: ({ row }) => {
          const side = row.getValue("side") as string;
          return (
            <span
              className={`text-xs px-2 py-1 font-semibold uppercase tracking-wide rounded ${
                side === "buy"
                  ? "bg-green-500/10 text-green-500 border border-green-500/20"
                  : "bg-red-500/10 text-red-500 border border-red-500/20"
              }`}
            >
              {side === "buy" ? "Buy" : "Sell"}
            </span>
          );
        },
        size: 90,
      },
      {
        accessorKey: "priceDisplay",
        header: "Price",
        cell: ({ row }) => {
          const side = row.getValue("side") as string;
          return (
            <div className={`font-mono text-sm font-semibold ${side === "buy" ? "text-green-500" : "text-red-500"}`}>
              {row.getValue("priceDisplay")}
            </div>
          );
        },
        size: 110,
      },
      {
        accessorKey: "sizeDisplay",
        header: "Size",
        cell: ({ row }) => <div className="font-mono text-sm text-muted-foreground">{row.getValue("sizeDisplay")}</div>,
        size: 110,
      },
      {
        accessorKey: "timestamp",
        header: "Time",
        cell: ({ row }) => (
          <div className="text-muted-foreground text-xs">
            {(row.getValue("timestamp") as Date).toLocaleTimeString()}
          </div>
        ),
        size: 90,
      },
    ],
    []
  );

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-muted-foreground text-sm">Select a market to view trades</p>
      </div>
    );
  }

  if (!isAuthenticated || !userAddress) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-muted-foreground text-sm">Connect your wallet to view your trades</p>
      </div>
    );
  }

  return (
    <div className="h-full">
      <DataTable columns={columns} data={trades} emptyMessage="No trades found" />
    </div>
  );
}
