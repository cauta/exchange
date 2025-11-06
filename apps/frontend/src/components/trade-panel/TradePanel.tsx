"use client";

import { useEffect } from "react";
import { useExchangeStore, selectSelectedMarket, selectOrderbookBids, selectOrderbookAsks } from "@/lib/store";
import { useUserBalances } from "@/lib/hooks";
import { Card, CardContent } from "@/components/ui/card";
import { roundToTickSize, getDecimalPlaces } from "@/lib/format";
import { OrderTypeSelector } from "./OrderTypeSelector";
import { SideSelector } from "./SideSelector";
import { PriceInput } from "./PriceInput";
import { SizeInput } from "./SizeInput";
import { OrderSummary } from "./OrderSummary";
import { SubmitButton } from "./SubmitButton";
import { useTradeForm } from "./useTradeForm";

export function TradePanel() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const recentTrades = useExchangeStore((state) => state.recentTrades);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);
  const selectedPrice = useExchangeStore((state) => state.selectedPrice);
  const setSelectedPrice = useExchangeStore((state) => state.setSelectedPrice);
  const balances = useUserBalances();

  // Look up tokens for the selected market using O(1) Record access
  const baseToken = selectedMarket ? tokens[selectedMarket.base_ticker] : undefined;
  const quoteToken = selectedMarket ? tokens[selectedMarket.quote_ticker] : undefined;

  // Get user balances for base and quote tokens
  const baseBalance = balances.find((b) => b.token_ticker === baseToken?.ticker);
  const quoteBalance = balances.find((b) => b.token_ticker === quoteToken?.ticker);

  // Calculate available balances (total - locked in orders)
  const availableBase = baseBalance ? baseBalance.amountValue - baseBalance.lockedValue : 0;
  const availableQuote = quoteBalance ? quoteBalance.amountValue - quoteBalance.lockedValue : 0;

  // Get price helpers
  const lastTradePrice = recentTrades.length > 0 && recentTrades[0] ? recentTrades[0].priceValue : null;
  const bestBid = bids.length > 0 && bids[0] ? bids[0].priceValue : null;
  const bestAsk = asks.length > 0 && asks[0] ? asks[0].priceValue : null;

  // Use the trade form hook - MUST be called before any early returns
  const formHookParams =
    selectedMarket && baseToken && quoteToken
      ? {
          selectedMarket,
          baseToken,
          quoteToken,
          availableBase,
          availableQuote,
          bestBid,
          bestAsk,
          lastTradePrice,
        }
      : null;

  const { formData, updateField, errors, loading, success, estimate, priceDecimals, handleSubmit } =
    useTradeForm(formHookParams);

  // Handle price selection from orderbook - MUST be before early returns
  useEffect(() => {
    if (selectedPrice !== null && selectedMarket && baseToken && quoteToken) {
      // Auto-switch to limit order if currently on market order
      if (formData.orderType === "market") {
        updateField("orderType", "limit");
      }

      // Round price to tick size and set it
      const priceDecimals = getDecimalPlaces(selectedMarket.tick_size, quoteToken.decimals);
      const rounded = roundToTickSize(selectedPrice, selectedMarket.tick_size, quoteToken.decimals);
      updateField("price", rounded.toFixed(priceDecimals));

      // Clear the selected price from store
      setSelectedPrice(null);
    }
  }, [selectedPrice, selectedMarket, baseToken, quoteToken, formData.orderType, setSelectedPrice, updateField]);

  // Early returns for loading states - AFTER all hooks
  if (!selectedMarketId || !selectedMarket) {
    return (
      <Card className="h-full min-h-[400px]">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Select a market to trade</p>
        </CardContent>
      </Card>
    );
  }

  if (!baseToken || !quoteToken) {
    return (
      <Card className="h-full min-h-[400px]">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Loading token information...</p>
        </CardContent>
      </Card>
    );
  }

  // Calculate fee bps based on order type
  const feeBps = formData.orderType === "market" ? selectedMarket.taker_fee_bps : selectedMarket.maker_fee_bps;

  // Get current price for size calculations
  const currentPrice =
    formData.orderType === "limit"
      ? parseFloat(formData.price) || null
      : (formData.side === "buy" ? bestAsk : bestBid) || lastTradePrice;

  return (
    <Card className="h-full flex flex-col gap-0 py-0 overflow-hidden border-border/40 bg-card min-w-0">
      <OrderTypeSelector value={formData.orderType} onChange={(value) => updateField("orderType", value)} />

      <form
        onSubmit={(e) => {
          e.preventDefault();
          handleSubmit(userAddress, isAuthenticated);
        }}
        className="flex-1 flex flex-col min-h-0"
      >
        <CardContent className="p-3 space-y-3 flex-1 overflow-y-auto">
          {/* Buy/Sell Buttons */}
          <SideSelector value={formData.side} onChange={(value) => updateField("side", value)} />

          {/* Price - Only for limit orders */}
          {formData.orderType === "limit" && (
            <PriceInput
              value={formData.price}
              onChange={(value) => updateField("price", value)}
              market={selectedMarket}
              quoteToken={quoteToken}
              error={errors.price}
            />
          )}

          {/* Size */}
          <SizeInput
            value={formData.size}
            onChange={(value) => updateField("size", value)}
            market={selectedMarket}
            baseToken={baseToken}
            quoteToken={quoteToken}
            side={formData.side}
            availableBase={availableBase}
            availableQuote={availableQuote}
            currentPrice={currentPrice}
            isAuthenticated={isAuthenticated}
            error={errors.size}
          />

          {/* Error/Success Messages */}
          {errors.general && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-md p-2 text-red-600 text-xs font-medium">
              {errors.general}
            </div>
          )}
          {success && (
            <div className="bg-green-500/10 border border-green-500/30 rounded-md p-2 text-green-600 text-xs font-medium">
              {success}
            </div>
          )}
        </CardContent>

        {/* Bottom section with summary and button */}
        <div className="border-t border-border/40 bg-muted/10 p-3 space-y-3 mt-auto">
          {/* Estimated total and fees */}
          <OrderSummary
            estimate={estimate}
            side={formData.side}
            quoteToken={quoteToken}
            priceDecimals={priceDecimals}
            feeBps={feeBps}
          />

          {/* Submit Button */}
          <SubmitButton
            side={formData.side}
            baseToken={baseToken}
            isAuthenticated={isAuthenticated}
            loading={loading}
          />
        </div>
      </form>
    </Card>
  );
}
