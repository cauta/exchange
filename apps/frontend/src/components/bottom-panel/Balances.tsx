"use client";

import { useMemo } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { useExchangeStore } from "@/lib/store";
import { useBalances } from "@/lib/hooks";
import { DataTable } from "@/components/ui/data-table";
import type { Balance } from "@/lib/types/exchange";

export function Balances() {
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const balances = useBalances();
  const tokens = useExchangeStore((state) => state.tokens);

  const columns = useMemo<ColumnDef<Balance>[]>(
    () => [
      {
        accessorKey: "token_ticker",
        header: "Asset",
        cell: ({ row }) => <div className="font-semibold text-foreground">{row.getValue("token_ticker")}</div>,
        size: 100,
      },
      {
        accessorKey: "available",
        header: () => <div className="text-right">Available</div>,
        cell: ({ row }) => {
          const balance = row.original;
          const available = balance.amountValue - balance.lockedValue;
          const token = tokens.find((t) => t.ticker === balance.token_ticker);
          const decimals = token?.decimals ?? 8;
          return <div className="text-right font-mono text-sm">{available.toFixed(decimals)}</div>;
        },
        size: 150,
      },
      {
        accessorKey: "lockedDisplay",
        header: () => <div className="text-right">In Orders</div>,
        cell: ({ row }) => (
          <div className="text-right font-mono text-sm text-muted-foreground">{row.getValue("lockedDisplay")}</div>
        ),
        size: 150,
      },
      {
        accessorKey: "amountDisplay",
        header: () => <div className="text-right">Total</div>,
        cell: ({ row }) => (
          <div className="text-right font-mono text-sm font-semibold text-foreground">
            {row.getValue("amountDisplay")}
          </div>
        ),
        size: 150,
      },
    ],
    [tokens]
  );

  if (!isAuthenticated || !userAddress) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-muted-foreground text-sm">Connect your wallet to view balances</p>
      </div>
    );
  }

  if (balances.length === 0) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-muted-foreground text-sm">
          No balances found. Use the faucet button in the top bar to get tokens!
        </p>
      </div>
    );
  }

  return (
    <div className="h-full">
      <DataTable columns={columns} data={balances} emptyMessage="No balances found" />
    </div>
  );
}
