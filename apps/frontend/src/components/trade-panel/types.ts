import type { Token, Market } from "@/lib/types/openapi";

export type OrderSide = "buy" | "sell";
export type OrderType = "limit" | "market";

export interface TradeFormData {
  side: OrderSide;
  orderType: OrderType;
  price: string;
  size: string;
}

export interface TradeFormErrors {
  price?: string;
  size?: string;
  general?: string;
}

export interface OrderEstimate {
  price: number;
  size: number;
  total: number;
  fee: number;
  finalAmount: number;
}

export interface TradeFormProps {
  selectedMarket: Market;
  baseToken: Token;
  quoteToken: Token;
  availableBase: number;
  availableQuote: number;
  bestBid: number | null;
  bestAsk: number | null;
  lastTradePrice: number | null;
}
