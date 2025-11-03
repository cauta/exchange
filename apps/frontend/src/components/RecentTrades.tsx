"use client";

import { useState, useEffect, useMemo } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import type { Trade } from "@/lib/types/exchange";
import { DataTable } from "@/components/ui/data-table";

type EnhancedTrade = Trade & {
  side: "buy" | "sell";
};

export function RecentTrades() {
  const client = useExchangeClient();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);

  const [trades, setTrades] = useState<EnhancedTrade[]>([]);
  const [loading, setLoading] = useState(false);

  const baseToken = tokens.find((t) => t.ticker === selectedMarket?.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket?.quote_ticker);

  const columns = useMemo<ColumnDef<EnhancedTrade>[]>(
    () => [
      {
        accessorKey: "priceDisplay",
        header: () => <div className="flex justify-end w-full">Price ({quoteToken?.ticker})</div>,
        cell: ({ row }) => {
          const side = row.getValue("side") as string;
          return (
            <div className={`font-mono text-sm font-semibold flex justify-end w-full ${side === "buy" ? "text-green-500" : "text-red-500"}`}>
              {row.getValue("priceDisplay")}
            </div>
          );
        },
        size: 120,
      },
      {
        accessorKey: "sizeDisplay",
        header: () => <div className="flex justify-end w-full">Size ({baseToken?.ticker})</div>,
        cell: ({ row }) => <div className="font-mono text-sm text-muted-foreground flex justify-end w-full">{row.getValue("sizeDisplay")}</div>,
        size: 120,
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
        size: 100,
      },
      {
        accessorKey: "timestamp",
        header: "Time",
        cell: ({ row }) => (
          <div className="text-muted-foreground text-xs">
            {(row.getValue("timestamp") as Date).toLocaleTimeString()}
          </div>
        ),
        size: 100,
      },
    ],
    [baseToken?.ticker, quoteToken?.ticker]
  );

  useEffect(() => {
    if (!userAddress || !isAuthenticated || !selectedMarketId) {
      setTrades([]);
      return;
    }

    const fetchTrades = async () => {
      setLoading(true);
      try {
        const result = await client.getTrades(userAddress, selectedMarketId);
        // Add side information to each trade
        const enhancedTrades: EnhancedTrade[] = result.slice(0, 50).map((trade) => ({
          ...trade,
          side: trade.buyer_address === userAddress ? "buy" : "sell",
        }));
        setTrades(enhancedTrades);
      } catch (err) {
        console.error("Failed to fetch trades:", err);
        setTrades([]);
      } finally {
        setLoading(false);
      }
    };

    fetchTrades();
    const interval = setInterval(fetchTrades, 2000); // Refresh every 2 seconds

    return () => clearInterval(interval);
  }, [userAddress, isAuthenticated, selectedMarketId, client]);

  if (!selectedMarketId || !selectedMarket || !baseToken || !quoteToken) {
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">Select a market to view trades</p>
      </div>
    );
  }

  if (loading && !trades.length) {
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">Loading trades...</p>
      </div>
    );
  }

  if (!isAuthenticated || !userAddress) {
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">Connect your wallet to view your trades</p>
      </div>
    );
  }

  return <DataTable columns={columns} data={trades} emptyMessage="No trades found" />;
}
