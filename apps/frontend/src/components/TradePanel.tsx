"use client";

import { useState, useEffect } from "react";
import { useExchangeStore, selectSelectedMarket, selectOrderbookBids, selectOrderbookAsks } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { Balance } from "@exchange/sdk";
import {
  toRawValue,
  toDisplayValue,
  roundToTickSize,
  roundToLotSize,
  getDecimalPlaces,
  formatNumberWithCommas,
} from "@/lib/format";

export function TradePanel() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const recentTrades = useExchangeStore((state) => state.recentTrades);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);

  const [side, setSide] = useState<"buy" | "sell">("buy");
  const [orderType, setOrderType] = useState<"limit" | "market">("limit");
  const [price, setPrice] = useState("");
  const [size, setSize] = useState("");
  const [balances, setBalances] = useState<Balance[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  if (!selectedMarketId || !selectedMarket) {
    return (
      <Card className="h-full min-h-[400px]">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Select a market to trade</p>
        </CardContent>
      </Card>
    );
  }

  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <Card className="h-full min-h-[400px]">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Loading token information...</p>
        </CardContent>
      </Card>
    );
  }

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

      const client = getExchangeClient();

      // For demo purposes, using a simple signature
      // In production, this would be a proper cryptographic signature
      const signature = `${userAddress}:${Date.now()}`;

      const result = await client.placeOrder({
        userAddress,
        marketId: selectedMarketId,
        side: side === "buy" ? "buy" : "sell",
        orderType: orderType === "limit" ? "limit" : "market",
        price: orderType === "limit" ? price : "0",
        size,
        signature,
      });

      setSuccess(`Order placed successfully! Order ID: ${result.order.id.slice(0, 8)}...`);
      setPrice("");
      setSize("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to place order");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="h-full">
      <CardContent className="p-6">
        {/* Buy/Sell Tabs */}
        <div className="grid grid-cols-2 gap-2 mb-6">
          <Button
            onClick={() => setSide("buy")}
            variant={side === "buy" ? "default" : "outline"}
            className={side === "buy" ? "bg-green-600 hover:bg-green-700 text-white" : ""}
          >
            Buy
          </Button>
          <Button
            onClick={() => setSide("sell")}
            variant={side === "sell" ? "default" : "outline"}
            className={side === "sell" ? "bg-red-600 hover:bg-red-700 text-white" : ""}
          >
            Sell
          </Button>
        </div>

        {/* Order Type */}
        <div className="mb-6">
          <Label className="mb-2 block text-sm font-medium">Order Type</Label>
          <div className="grid grid-cols-2 gap-2">
            <Button
              onClick={() => setOrderType("limit")}
              variant="outline"
              size="sm"
              className={orderType === "limit" ? "bg-primary text-primary-foreground hover:bg-primary/90" : ""}
            >
              Limit
            </Button>
            <Button
              onClick={() => setOrderType("market")}
              variant="outline"
              size="sm"
              className={orderType === "market" ? "bg-primary text-primary-foreground hover:bg-primary/90" : ""}
            >
              Market
            </Button>
          </div>
        </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        {/* Wallet Connection Status */}
        {!isAuthenticated && (
          <div className="bg-yellow-500/10 border border-yellow-500/50 p-3 text-yellow-500 text-sm">
            Connect your wallet to start trading
          </div>
        )}

        {/* Price - Only for limit orders */}
        {orderType === "limit" && (
          <div className="space-y-2">
            <Label className="text-sm font-medium">Price ({quoteToken.ticker})</Label>
            <Input
              type="text"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              placeholder="0.00"
            />
          </div>
        )}

        {/* Size */}
        <div className="space-y-2">
          <Label className="text-sm font-medium">Size ({baseToken.ticker})</Label>
          <Input
            type="text"
            value={size}
            onChange={(e) => setSize(e.target.value)}
            placeholder="0.00"
          />
        </div>

        {/* Total - Only for limit orders */}
        {orderType === "limit" && price && size && (
          <div className="bg-muted border border-border p-3">
            <div className="flex justify-between items-center text-sm">
              <span className="text-muted-foreground">Total</span>
              <span className="text-foreground font-medium">
                {(parseFloat(price) * parseFloat(size)).toFixed(quoteToken.decimals)}{" "}
                {quoteToken.ticker}
              </span>
            </div>
          </div>
        )}

        {/* Error/Success Messages */}
        {error && (
          <div className="bg-red-500/10 border border-red-500/50 p-3 text-red-500 text-sm">
            {error}
          </div>
        )}
        {success && (
          <div className="bg-green-500/10 border border-green-500/50 p-3 text-green-500 text-sm">
            {success}
          </div>
        )}

        {/* Submit Button */}
        <Button
          type="submit"
          disabled={loading || !isAuthenticated}
          className={`w-full ${
            side === "buy"
              ? "bg-green-600 hover:bg-green-700 text-white"
              : "bg-red-600 hover:bg-red-700 text-white"
          }`}
        >
          {loading ? "Placing Order..." : !isAuthenticated ? "Connect Wallet" : `${side === "buy" ? "Buy" : "Sell"} ${baseToken.ticker}`}
        </Button>
      </form>
      </CardContent>
    </Card>
  );
}
