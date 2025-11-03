"use client";

import { useState, useEffect } from "react";
import { useExchangeStore, selectSelectedMarket, selectOrderbookBids, selectOrderbookAsks } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import { useBalances } from "@/lib/hooks";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  toRawValue,
  toDisplayValue,
  roundToTickSize,
  roundToLotSize,
  getDecimalPlaces,
  formatNumberWithCommas,
} from "@/lib/format";

export function TradePanel() {
  const client = useExchangeClient();
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
  const balances = useBalances();

  const [side, setSide] = useState<"buy" | "sell">("buy");
  const [orderType, setOrderType] = useState<"limit" | "market">("limit");
  const [price, setPrice] = useState("");
  const [size, setSize] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Look up tokens for the selected market
  const baseToken = selectedMarket ? tokens.find((t) => t.ticker === selectedMarket.base_ticker) : undefined;
  const quoteToken = selectedMarket ? tokens.find((t) => t.ticker === selectedMarket.quote_ticker) : undefined;

  // Handle price selection from orderbook
  useEffect(() => {
    if (selectedPrice !== null && selectedMarket && baseToken && quoteToken) {
      // Auto-switch to limit order if currently on market order
      if (orderType === "market") {
        setOrderType("limit");
      }

      // Round price to tick size and set it
      const priceDecimals = getDecimalPlaces(selectedMarket.tick_size, quoteToken.decimals);
      const rounded = roundToTickSize(selectedPrice, selectedMarket.tick_size, quoteToken.decimals);
      setPrice(rounded.toFixed(priceDecimals));

      // Clear the selected price from store
      setSelectedPrice(null);
    }
  }, [selectedPrice, selectedMarket, baseToken, quoteToken, orderType, setSelectedPrice]);

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

  // Get user balances for base and quote tokens
  const baseBalance = balances.find((b) => b.token_ticker === baseToken.ticker);
  const quoteBalance = balances.find((b) => b.token_ticker === quoteToken.ticker);

  // Calculate available balances (total - locked in orders) - use enhanced values!
  const availableBase = baseBalance ? baseBalance.amountValue - baseBalance.lockedValue : 0;
  const availableQuote = quoteBalance ? quoteBalance.amountValue - quoteBalance.lockedValue : 0;

  // Get price helpers - recentTrades now has priceValue!
  const lastTradePrice = recentTrades.length > 0 && recentTrades[0] ? recentTrades[0].priceValue : null;
  const bestBid = bids.length > 0 && bids[0] ? toDisplayValue(bids[0].price, quoteToken.decimals) : null;
  const bestAsk = asks.length > 0 && asks[0] ? toDisplayValue(asks[0].price, quoteToken.decimals) : null;

  // Calculate decimal places based on tick/lot sizes
  const priceDecimals = getDecimalPlaces(selectedMarket.tick_size, quoteToken.decimals);
  const sizeDecimals = getDecimalPlaces(selectedMarket.lot_size, baseToken.decimals);

  // Helper function for quick size selection
  const setPercentageSize = (percentage: number) => {
    // Calculate max size based on available balance
    let maxSize = 0;

    if (side === "buy") {
      // For buy: limited by quote balance / price
      const currentPrice = parseFloat(price) || lastTradePrice || bestAsk || 1;
      maxSize = availableQuote / currentPrice;
    } else {
      // For sell: limited by base balance
      maxSize = availableBase;
    }

    const targetSize = maxSize * (percentage / 100);
    const rounded = roundToLotSize(targetSize, selectedMarket.lot_size, baseToken.decimals);
    setSize(rounded.toFixed(sizeDecimals));
  };

  const handlePriceChange = (value: string) => {
    setPrice(value);
  };

  const handlePriceBlur = () => {
    if (price && orderType === "limit") {
      const numPrice = parseFloat(price);
      if (!isNaN(numPrice)) {
        const rounded = roundToTickSize(numPrice, selectedMarket.tick_size, quoteToken.decimals);
        setPrice(rounded.toFixed(priceDecimals));
      }
    }
  };

  const handleSizeChange = (value: string) => {
    setSize(value);
  };

  const handleSizeBlur = () => {
    if (size) {
      const numSize = parseFloat(size);
      if (!isNaN(numSize)) {
        const rounded = roundToLotSize(numSize, selectedMarket.lot_size, baseToken.decimals);
        setSize(rounded.toFixed(sizeDecimals));
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setLoading(true);

    try {
      if (!isAuthenticated || !userAddress) {
        throw new Error("Please connect your wallet first");
      }
      if (!price.trim() && orderType === "limit") {
        throw new Error("Price is required for limit orders");
      }
      if (!size.trim()) {
        throw new Error("Size is required");
      }

      // Parse and round values
      let finalPrice = orderType === "limit" ? parseFloat(price) : 0;
      let finalSize = parseFloat(size);

      if (isNaN(finalSize) || finalSize <= 0) {
        throw new Error("Invalid size");
      }

      if (orderType === "limit" && (isNaN(finalPrice) || finalPrice <= 0)) {
        throw new Error("Invalid price");
      }

      // Round to tick/lot sizes
      if (orderType === "limit") {
        finalPrice = roundToTickSize(finalPrice, selectedMarket.tick_size, quoteToken.decimals);
      }
      finalSize = roundToLotSize(finalSize, selectedMarket.lot_size, baseToken.decimals);

      // Convert to raw values for API
      const rawPrice = toRawValue(finalPrice, quoteToken.decimals);
      const rawSize = toRawValue(finalSize, baseToken.decimals);

      // Check minimum size
      const minSize = BigInt(selectedMarket.min_size);
      if (BigInt(rawSize) < minSize) {
        const minSizeDisplay = toDisplayValue(selectedMarket.min_size, baseToken.decimals);
        throw new Error(`Size must be at least ${minSizeDisplay} ${baseToken.ticker}`);
      }

      // For demo purposes, using a simple signature
      // In production, this would use Turnkey to sign
      const signature = `${userAddress}:${Date.now()}`;

      const result = await client.rest.placeOrder({
        userAddress,
        marketId: selectedMarketId,
        side: side,
        orderType: orderType,
        price: rawPrice,
        size: rawSize,
        signature,
      });

      setSuccess(
        `Order placed! ${result.trades.length > 0 ? `Filled ${result.trades.length} trade(s)` : "Order in book"}`
      );
      setPrice("");
      setSize("");

      // Auto-clear success message after 3 seconds
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to place order");
    } finally {
      setLoading(false);
    }
  };

  // Calculate estimated total and fees
  const estimatedPrice = parseFloat(price) || (orderType === "market" ? (side === "buy" ? bestAsk : bestBid) || 0 : 0);
  const estimatedSize = parseFloat(size) || 0;
  const estimatedTotal = estimatedPrice * estimatedSize;
  const feeBps = orderType === "market" ? selectedMarket.taker_fee_bps : selectedMarket.maker_fee_bps;
  const estimatedFee = (estimatedTotal * Math.abs(feeBps)) / 10000;

  return (
    <Card className="h-full flex flex-col gap-0 py-0 overflow-hidden shadow-lg border-border/50 bg-gradient-to-b from-card to-card/80">
      <Tabs
        value={orderType}
        onValueChange={(v) => setOrderType(v as "limit" | "market")}
        className="flex-1 flex flex-col"
      >
        <TabsList className="w-full justify-start rounded-none border-b border-border/50 h-auto p-0 bg-gradient-to-b from-muted/30 to-muted/50 backdrop-blur-sm shrink-0">
          <TabsTrigger
            value="limit"
            className="flex-1 rounded-none data-[state=active]:bg-card/80 data-[state=active]:shadow-sm transition-all"
          >
            Limit
          </TabsTrigger>
          <TabsTrigger
            value="market"
            className="flex-1 rounded-none data-[state=active]:bg-card/80 data-[state=active]:shadow-sm transition-all"
          >
            Market
          </TabsTrigger>
        </TabsList>

        <CardContent className="p-4 space-y-4 flex-1">
          {/* Buy/Sell Buttons */}
          <div className="grid grid-cols-2 gap-3">
            <Button
              onClick={() => setSide("buy")}
              variant={side === "buy" ? "default" : "outline"}
              className={
                side === "buy"
                  ? "bg-gradient-to-br from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white shadow-lg shadow-green-600/20 border-green-500/50"
                  : "hover:bg-green-600/10 border-green-600/20 hover:border-green-600/40"
              }
              size="lg"
            >
              Buy
            </Button>
            <Button
              onClick={() => setSide("sell")}
              variant={side === "sell" ? "default" : "outline"}
              className={
                side === "sell"
                  ? "bg-gradient-to-br from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white shadow-lg shadow-red-600/20 border-red-500/50"
                  : "hover:bg-red-600/10 border-red-600/20 hover:border-red-600/40"
              }
              size="lg"
            >
              Sell
            </Button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Wallet Connection Status */}
            {!isAuthenticated && (
              <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3 text-yellow-600 text-xs text-center">
                Connect your wallet to start trading
              </div>
            )}

            {/* Price - Only for limit orders */}
            {orderType === "limit" && (
              <div className="space-y-2">
                <Label className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                  Price ({quoteToken.ticker})
                </Label>
                <Input
                  type="number"
                  value={price}
                  onChange={(e) => handlePriceChange(e.target.value)}
                  onBlur={handlePriceBlur}
                  placeholder="0.00"
                  step={toDisplayValue(selectedMarket.tick_size, quoteToken.decimals)}
                  className="font-mono h-11 text-base border-border/50 focus:border-primary/50 focus:ring-primary/20 bg-muted/30"
                />
              </div>
            )}

            {/* Size */}
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <Label className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                  Size ({baseToken.ticker})
                </Label>
                {isAuthenticated && (
                  <span className="text-xs text-muted-foreground font-medium">
                    Available: {formatNumberWithCommas(side === "buy" ? availableQuote : availableBase, 4)}{" "}
                    {side === "buy" ? quoteToken.ticker : baseToken.ticker}
                  </span>
                )}
              </div>
              <Input
                type="number"
                value={size}
                onChange={(e) => handleSizeChange(e.target.value)}
                onBlur={handleSizeBlur}
                placeholder="0.00"
                step={toDisplayValue(selectedMarket.lot_size, baseToken.decimals)}
                className="font-mono h-11 text-base border-border/50 focus:border-primary/50 focus:ring-primary/20 bg-muted/30"
              />

              {/* Percentage buttons */}
              <div className="grid grid-cols-4 gap-2">
                {[25, 50, 75, 100].map((pct) => (
                  <Button
                    key={pct}
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setPercentageSize(pct)}
                    disabled={!isAuthenticated}
                    className="h-8 text-xs font-semibold hover:bg-primary/10 hover:border-primary/40 hover:text-primary transition-all"
                  >
                    {pct}%
                  </Button>
                ))}
              </div>
            </div>

            {/* Estimated total and fees */}
            {estimatedSize > 0 && estimatedPrice > 0 && (
              <div className="space-y-2 bg-gradient-to-br from-muted/40 to-muted/60 border border-border/50 rounded-lg p-4 text-xs shadow-inner">
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground font-medium">Total</span>
                  <span className="font-mono font-semibold text-sm">
                    {formatNumberWithCommas(estimatedTotal, Math.min(priceDecimals, 4))} {quoteToken.ticker}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground font-medium">
                    Est. Fee ({(Math.abs(feeBps) / 100).toFixed(2)}%)
                  </span>
                  <span className="font-mono text-muted-foreground">
                    {formatNumberWithCommas(estimatedFee, Math.min(priceDecimals, 4))} {quoteToken.ticker}
                  </span>
                </div>
                <div className="flex justify-between items-center pt-2 border-t border-border/50">
                  <span className={`font-semibold ${side === "buy" ? "text-green-600" : "text-red-600"}`}>
                    {side === "buy" ? "Total Cost" : "You Receive"}
                  </span>
                  <span className={`font-mono font-bold text-sm ${side === "buy" ? "text-green-600" : "text-red-600"}`}>
                    {formatNumberWithCommas(
                      side === "buy" ? estimatedTotal + estimatedFee : estimatedTotal - estimatedFee,
                      Math.min(priceDecimals, 4)
                    )}{" "}
                    {quoteToken.ticker}
                  </span>
                </div>
              </div>
            )}

            {/* Error/Success Messages */}
            {error && (
              <div className="bg-red-500/10 border border-red-500/40 rounded-lg p-3 text-red-600 text-xs font-medium shadow-sm">
                {error}
              </div>
            )}
            {success && (
              <div className="bg-green-500/10 border border-green-500/40 rounded-lg p-3 text-green-600 text-xs font-medium shadow-sm">
                {success}
              </div>
            )}

            {/* Submit Button */}
            <Button
              type="submit"
              disabled={loading || !isAuthenticated}
              size="lg"
              className={`w-full font-bold text-base h-12 shadow-lg transition-all ${
                side === "buy"
                  ? "bg-gradient-to-br from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white shadow-green-600/30 hover:shadow-green-600/50"
                  : "bg-gradient-to-br from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white shadow-red-600/30 hover:shadow-red-600/50"
              }`}
            >
              {loading
                ? "Placing Order..."
                : !isAuthenticated
                  ? "Connect Wallet"
                  : `${side === "buy" ? "Buy" : "Sell"} ${baseToken.ticker}`}
            </Button>
          </form>
        </CardContent>
      </Tabs>
    </Card>
  );
}
