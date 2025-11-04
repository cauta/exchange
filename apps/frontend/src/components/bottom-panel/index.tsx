"use client";

import { Balances } from "./Balances";
import { RecentOrders } from "./RecentOrders";
import { RecentTrades } from "./RecentTrades";
import { Card } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

export function BottomPanel() {
  return (
    <div className="w-full h-[600px]">
      <Card className="py-0 overflow-hidden w-full h-full flex flex-col">
        <Tabs defaultValue="balances" className="flex flex-col h-full">
          <TabsList className="justify-start rounded-none border-b border-border h-auto p-0 bg-card backdrop-blur-sm shrink-0">
            <TabsTrigger
              value="balances"
              className="rounded-none px-4 py-2 text-sm focus-visible:ring-0 focus-visible:ring-offset-0"
            >
              Balances
            </TabsTrigger>
            <TabsTrigger
              value="orders"
              className="rounded-none px-4 py-2 text-sm focus-visible:ring-0 focus-visible:ring-offset-0"
            >
              Orders
            </TabsTrigger>
            <TabsTrigger
              value="trades"
              className="rounded-none px-4 py-2 text-sm focus-visible:ring-0 focus-visible:ring-offset-0"
            >
              Trades
            </TabsTrigger>
          </TabsList>

          <TabsContent value="balances" className="focus-visible:outline-none flex-1 overflow-hidden m-0">
            <Balances />
          </TabsContent>

          <TabsContent value="orders" className="focus-visible:outline-none flex-1 overflow-hidden m-0">
            <RecentOrders />
          </TabsContent>

          <TabsContent value="trades" className="focus-visible:outline-none flex-1 overflow-hidden m-0">
            <RecentTrades />
          </TabsContent>
        </Tabs>
      </Card>
    </div>
  );
}
