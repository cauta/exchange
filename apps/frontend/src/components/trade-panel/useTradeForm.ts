import { useState, useCallback, useMemo } from "react";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import { getDecimalPlaces } from "@/lib/format";
import type { OrderSide, OrderType, TradeFormData, TradeFormErrors, OrderEstimate, TradeFormProps } from "./types";

export function useTradeForm(params: TradeFormProps | null) {
  const client = useExchangeClient();
  const [formData, setFormData] = useState<TradeFormData>({
    side: "buy",
    orderType: "limit",
    price: "",
    size: "",
  });
  const [errors, setErrors] = useState<TradeFormErrors>({});
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState<string | null>(null);

  // Extract params with safe defaults
  const selectedMarket = params?.selectedMarket;
  const baseToken = params?.baseToken;
  const quoteToken = params?.quoteToken;
  const availableBase = params?.availableBase ?? 0;
  const availableQuote = params?.availableQuote ?? 0;
  const bestBid = params?.bestBid ?? null;
  const bestAsk = params?.bestAsk ?? null;
  const lastTradePrice = params?.lastTradePrice ?? null;

  // Calculate decimal places
  const priceDecimals =
    selectedMarket && quoteToken ? getDecimalPlaces(selectedMarket.tick_size, quoteToken.decimals) : 2;
  const sizeDecimals = selectedMarket && baseToken ? getDecimalPlaces(selectedMarket.lot_size, baseToken.decimals) : 2;

  // Validate form
  const validateForm = useCallback((): { isValid: boolean; errors: TradeFormErrors } => {
    const newErrors: TradeFormErrors = {};

    // Can't validate without market/token data
    if (!selectedMarket || !baseToken || !quoteToken) {
      newErrors.general = "Market data not loaded";
      return { isValid: false, errors: newErrors };
    }

    // Validate price for limit orders
    if (formData.orderType === "limit") {
      const priceNum = parseFloat(formData.price);
      if (!formData.price.trim()) {
        newErrors.price = "Price is required for limit orders";
      } else if (isNaN(priceNum) || priceNum <= 0) {
        newErrors.price = "Invalid price";
      }
    }

    // Validate size
    const sizeNum = parseFloat(formData.size);
    if (!formData.size.trim()) {
      newErrors.size = "Size is required";
    } else if (isNaN(sizeNum) || sizeNum <= 0) {
      newErrors.size = "Invalid size";
    } else {
      // Check if user has enough balance
      if (formData.side === "buy") {
        const priceNum = formData.orderType === "limit" ? parseFloat(formData.price) : bestAsk || lastTradePrice || 0;
        const requiredQuote = sizeNum * priceNum;
        if (requiredQuote > availableQuote) {
          newErrors.size = `Insufficient ${quoteToken.ticker} balance`;
        }
      } else {
        if (sizeNum > availableBase) {
          newErrors.size = `Insufficient ${baseToken.ticker} balance`;
        }
      }
    }

    return {
      isValid: Object.keys(newErrors).length === 0,
      errors: newErrors,
    };
  }, [formData, availableBase, availableQuote, baseToken, quoteToken, bestAsk, lastTradePrice, selectedMarket]);

  // Calculate order estimate
  const estimate = useMemo((): OrderEstimate | null => {
    if (!selectedMarket) return null;

    const priceNum = parseFloat(formData.price);
    const sizeNum = parseFloat(formData.size);

    let effectivePrice = 0;
    if (formData.orderType === "limit") {
      effectivePrice = priceNum;
    } else {
      effectivePrice = (formData.side === "buy" ? bestAsk : bestBid) || lastTradePrice || 0;
    }

    if (isNaN(effectivePrice) || effectivePrice <= 0 || isNaN(sizeNum) || sizeNum <= 0) {
      return null;
    }

    const total = effectivePrice * sizeNum;
    const feeBps = formData.orderType === "market" ? selectedMarket.taker_fee_bps : selectedMarket.maker_fee_bps;
    const fee = (total * Math.abs(feeBps)) / 10000;
    const finalAmount = formData.side === "buy" ? total + fee : total - fee;

    return {
      price: effectivePrice,
      size: sizeNum,
      total,
      fee,
      finalAmount,
    };
  }, [formData, selectedMarket, bestBid, bestAsk, lastTradePrice]);

  // Handle form submission
  const handleSubmit = useCallback(
    async (userAddress: string | null, isAuthenticated: boolean) => {
      setErrors({});
      setSuccess(null);

      // Check market data
      if (!selectedMarket) {
        setErrors({ general: "Market data not loaded" });
        return;
      }

      // Check authentication
      if (!isAuthenticated || !userAddress) {
        setErrors({ general: "Please connect your wallet first" });
        return;
      }

      // Validate form
      const validation = validateForm();
      if (!validation.isValid) {
        setErrors(validation.errors);
        return;
      }

      setLoading(true);

      try {
        const finalPrice = formData.orderType === "limit" ? parseFloat(formData.price) : 0;
        const finalSize = parseFloat(formData.size);

        // For demo purposes, using a simple signature
        // In production, this would use Turnkey to sign
        const signature = `${userAddress}:${Date.now()}`;

        // Use SDK's placeOrderDecimal - it handles conversion to atoms and rounding
        const result = await client.rest.placeOrderDecimal({
          userAddress,
          marketId: selectedMarket.id,
          side: formData.side,
          orderType: formData.orderType,
          priceDecimal: finalPrice.toString(),
          sizeDecimal: finalSize.toString(),
          signature,
        });

        const successMessage = `Order placed! ${
          result.trades.length > 0 ? `Filled ${result.trades.length} trade(s)` : "Order in book"
        }`;
        setSuccess(successMessage);

        // Clear form
        setFormData((prev) => ({
          ...prev,
          price: "",
          size: "",
        }));

        // Auto-clear success message after 3 seconds
        setTimeout(() => setSuccess(null), 3000);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : "Failed to place order";
        setErrors({ general: errorMessage });
      } finally {
        setLoading(false);
      }
    },
    [formData, validateForm, client, selectedMarket]
  );

  // Update form fields
  const updateField = useCallback(
    <K extends keyof TradeFormData>(field: K, value: TradeFormData[K]) => {
      setFormData((prev) => ({ ...prev, [field]: value }));
      // Clear error for this field when it's updated
      if (errors[field as keyof TradeFormErrors]) {
        setErrors((prev) => {
          const newErrors = { ...prev };
          delete newErrors[field as keyof TradeFormErrors];
          return newErrors;
        });
      }
    },
    [errors]
  );

  return {
    formData,
    updateField,
    errors,
    loading,
    success,
    estimate,
    priceDecimals,
    sizeDecimals,
    handleSubmit,
  };
}
