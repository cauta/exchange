import { useState, useCallback } from "react";
import { useExchangeStore } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";

interface TradeFormData {
  side: "buy" | "sell";
  orderType: "limit" | "market";
  price: string;
  size: string;
}

interface UseTradeFormSubmitParams {
  selectedMarket: any;
  baseToken: any;
  quoteToken: any;
  availableBase: number;
  availableQuote: number;
  bestBid: number | null;
  bestAsk: number | null;
  lastTradePrice: number | null;
  onSuccess?: () => void;
}

export function useTradeFormSubmit({
  selectedMarket,
  baseToken,
  quoteToken,
  availableBase,
  availableQuote,
  bestBid,
  bestAsk,
  lastTradePrice,
  onSuccess,
}: UseTradeFormSubmitParams) {
  const client = useExchangeClient();
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const userAddress = useExchangeStore((state) => state.userAddress);

  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const submitOrder = useCallback(
    async (data: TradeFormData) => {
      setError(null);
      setSuccess(null);

      // Check market data
      if (!selectedMarket || !baseToken || !quoteToken) {
        setError("Market data not loaded");
        return;
      }

      // Check authentication
      if (!isAuthenticated || !userAddress) {
        setError("Please connect your wallet first");
        return;
      }

      // Simple validation
      if (data.orderType === "limit" && (!data.price.trim() || parseFloat(data.price) <= 0)) {
        setError("Invalid price");
        return;
      }

      if (!data.size.trim() || parseFloat(data.size) <= 0) {
        setError("Invalid size");
        return;
      }

      // Check balance
      const sizeNum = parseFloat(data.size);
      if (data.side === "buy") {
        const priceNum = data.orderType === "limit" ? parseFloat(data.price) : bestAsk || lastTradePrice || 0;
        const requiredQuote = sizeNum * priceNum;
        if (requiredQuote > availableQuote) {
          setError(`Insufficient ${quoteToken.ticker} balance`);
          return;
        }
      } else {
        if (sizeNum > availableBase) {
          setError(`Insufficient ${baseToken.ticker} balance`);
          return;
        }
      }

      setLoading(true);

      try {
        const finalPrice = data.orderType === "limit" ? parseFloat(data.price) : 0;
        const finalSize = parseFloat(data.size);

        // For demo purposes, using a simple signature
        const signature = `${userAddress}:${Date.now()}`;

        // Use SDK's placeOrderDecimal - handles conversion and rounding
        const result = await client.rest.placeOrderDecimal({
          userAddress,
          marketId: selectedMarket.id,
          side: data.side,
          orderType: data.orderType,
          priceDecimal: finalPrice.toString(),
          sizeDecimal: finalSize.toString(),
          signature,
        });

        const successMessage = `Order placed! ${
          result.trades.length > 0 ? `Filled ${result.trades.length} trade(s)` : "Order in book"
        }`;
        setSuccess(successMessage);

        // Call onSuccess callback
        onSuccess?.();

        // Auto-clear success message after 3 seconds
        setTimeout(() => setSuccess(null), 3000);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : "Failed to place order";
        setError(errorMessage);
      } finally {
        setLoading(false);
      }
    },
    [
      selectedMarket,
      baseToken,
      quoteToken,
      availableBase,
      availableQuote,
      bestBid,
      bestAsk,
      lastTradePrice,
      isAuthenticated,
      userAddress,
      client,
      onSuccess,
    ]
  );

  return {
    submitOrder,
    loading,
    success,
    error,
  };
}
