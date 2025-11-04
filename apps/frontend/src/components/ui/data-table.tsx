"use client";

import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  SortingState,
  useReactTable,
} from "@tanstack/react-table";
import { useState } from "react";
import { TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  emptyMessage?: string;
  headerAction?: React.ReactNode;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  emptyMessage = "No data available.",
  headerAction,
}: DataTableProps<TData, TValue>) {
  const [sorting, setSorting] = useState<SortingState>([]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    state: {
      sorting,
    },
  });

  return (
    <div className="h-full overflow-auto relative">
      <table className="w-full caption-bottom text-sm">
        <TableHeader className="sticky top-0 bg-card z-10 border-b border-border/50">
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id} className="border-none hover:bg-transparent relative">
              {headerGroup.headers.map((header) => {
                return (
                  <TableHead
                    key={header.id}
                    className="font-semibold text-foreground px-3"
                    style={{ width: header.getSize() !== 150 ? header.getSize() : undefined }}
                  >
                    {header.isPlaceholder ? null : (
                      <div
                        className={
                          header.column.getCanSort()
                            ? "cursor-pointer select-none flex items-center justify-between gap-1 hover:text-primary transition-colors w-full"
                            : "w-full"
                        }
                        onClick={header.column.getToggleSortingHandler()}
                      >
                        <div className="flex-1">
                          {flexRender(header.column.columnDef.header, header.getContext())}
                        </div>
                        {header.column.getCanSort() && (
                          <span className="text-muted-foreground text-xs shrink-0">
                            {{
                              asc: " ↑",
                              desc: " ↓",
                            }[header.column.getIsSorted() as string] ?? " ↕"}
                          </span>
                        )}
                      </div>
                    )}
                  </TableHead>
                );
              })}
              {headerAction && (
                <div className="absolute right-3 top-0 h-full flex items-center">
                  {headerAction}
                </div>
              )}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && "selected"}
                className="border-border/30 hover:bg-primary/5 transition-colors"
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id} className="px-3">{flexRender(cell.column.columnDef.cell, cell.getContext())}</TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                <p className="text-muted-foreground text-sm">{emptyMessage}</p>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </table>
    </div>
  );
}
