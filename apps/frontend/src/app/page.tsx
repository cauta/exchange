"use client";

import { useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { Orderbook } from "@/components/Orderbook";
import { TradingViewChart } from "@/components/TradingViewChart";
import { TradePanel } from "@/components/TradePanel";
import { BottomPanel } from "@/components/BottomPanel";
import { AuthButton } from "@/components/AuthButton";
import { formatPrice, formatSize } from "@/lib/format";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export default function Home() {
  const { markets, isLoading } = useMarkets();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectMarket = useExchangeStore((state) => state.selectMarket);
  const selectedMarket = useExchangeStore((state) =>
    state.markets.find((m) => m.id === selectedMarketId)
  );
  const currentPrice = useExchangeStore((state) => {
    if (state.priceHistory.length > 0) {
      return state.priceHistory[state.priceHistory.length - 1]?.price ?? null;
    }
    return null;
  });

  // Auto-select BTC/USDC market when markets load
  useEffect(() => {
    if (markets.length > 0 && !selectedMarketId) {
      const btcUsdcMarket = markets.find((m) => m.id === "BTC/USDC");
      if (btcUsdcMarket) {
        selectMarket(btcUsdcMarket.id);
      } else {
        selectMarket(markets[0]?.id || "");
      }
    }
  }, [markets, selectedMarketId, selectMarket]);

  return (
    <main className="min-h-screen bg-background text-foreground p-4">
      <div className="max-w-[2000px] mx-auto">
        {/* Header */}
        <div className="mb-4 md:mb-6">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            {/* Logo and Market Info */}
            <div className="flex flex-col sm:flex-row items-start sm:items-center gap-4 sm:gap-6 w-full sm:w-auto">
              <h1 className="text-2xl font-bold tracking-tight">Exchange</h1>

              {/* Market Selector */}
              {isLoading ? (
                <div className="text-muted-foreground text-sm">Loading...</div>
              ) : markets.length === 0 ? (
                <div className="text-muted-foreground text-sm">No markets</div>
              ) : (
                <div className="flex flex-col sm:flex-row items-start sm:items-center gap-4 w-full sm:w-auto">
                  <Select
                    value={selectedMarketId || ""}
                    onValueChange={selectMarket}
                  >
                    <SelectTrigger className="w-full sm:w-[180px] bg-card/100">
                      <SelectValue placeholder="Select market" />
                    </SelectTrigger>
                    <SelectContent className="bg-card/100 backdrop-blur-sm">
                      {markets.map((market) => (
                        <SelectItem key={market.id} value={market.id}>
                          {market.base_ticker}/{market.quote_ticker}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  {/* Market Stats */}
                  {selectedMarket && (
                    <div className="flex items-center gap-4 lg:gap-6 text-xs overflow-x-auto w-full sm:w-auto">
                      <div className="flex flex-col gap-1 shrink-0">
                        <span className="text-muted-foreground font-medium">Price</span>
                        <span className="text-foreground font-mono text-base font-semibold">
                          {currentPrice !== null ? currentPrice.toFixed(2) : "—"} {selectedMarket.quote_ticker}
                        </span>
                      </div>
                      <div className="hidden md:flex flex-col gap-1 shrink-0">
                        <span className="text-muted-foreground font-medium">Tick Size</span>
                        <span className="text-foreground font-mono">
                          {formatPrice(selectedMarket.tick_size, selectedMarket.quote_decimals)} {selectedMarket.quote_ticker}
                        </span>
                      </div>
                      <div className="hidden md:flex flex-col gap-1 shrink-0">
                        <span className="text-muted-foreground font-medium">Lot Size</span>
                        <span className="text-foreground font-mono">
                          {formatSize(selectedMarket.lot_size, selectedMarket.base_decimals)} {selectedMarket.base_ticker}
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* Auth Button */}
            <AuthButton />
          </div>
        </div>

        {/* Main Trading Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-12 gap-4 mb-4">
          {/* Chart - Takes up most of the space */}
          <div className="md:col-span-2 lg:col-span-8 h-[500px] md:h-[600px]">
            <TradingViewChart />
          </div>

          {/* Orderbook with Trades tab */}
          <div className="md:col-span-1 lg:col-span-2 h-[500px] md:h-[600px]">
            <Orderbook />
          </div>

          {/* Trade Panel */}
          <div className="md:col-span-1 lg:col-span-2 h-[500px] md:h-[600px]">
            <TradePanel />
          </div>
        </div>

        {/* Bottom Panel - Balances and Orders */}
        <BottomPanel />
      </div>
    </main>
  );
}
