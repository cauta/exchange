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

  const columns = useMemo<ColumnDef<Balance>[]>(
    () => [
      {
        accessorKey: "token_ticker",
        header: "Token",
        cell: ({ row }) => <div className="font-semibold text-foreground">{row.getValue("token_ticker")}</div>,
        size: 100,
      },
      {
        accessorKey: "available",
        header: "Available",
        cell: ({ row }) => {
          const balance = row.original;
          const available = balance.amountValue - balance.lockedValue;
          return <div className="text-right font-mono text-sm">{available.toFixed(8)}</div>;
        },
        size: 150,
      },
      {
        accessorKey: "lockedDisplay",
        header: "In Orders",
        cell: ({ row }) => (
          <div className="text-right font-mono text-sm text-muted-foreground">{row.getValue("lockedDisplay")}</div>
        ),
        size: 150,
      },
      {
        accessorKey: "amountDisplay",
        header: "Total",
        cell: ({ row }) => (
          <div className="text-right font-mono text-sm font-semibold text-foreground">
            {row.getValue("amountDisplay")}
          </div>
        ),
        size: 150,
      },
    ],
    []
  );

  if (!isAuthenticated || !userAddress) {
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">Connect your wallet to view balances</p>
      </div>
    );
  }

  if (balances.length === 0) {
    return (
      <div className="p-8 text-center">
        <p className="text-muted-foreground text-sm">
          No balances found. Use the faucet button in the top bar to get tokens!
        </p>
      </div>
    );
  }

  return <DataTable columns={columns} data={balances} emptyMessage="No balances found" />;
}
