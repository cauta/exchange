/**
 * Core exchange types - now imported from SDK
 */

// Re-export SDK types
export type {
  Market,
  Token,
  Side,
  OrderType,
  OrderStatus,
  // Use enhanced types from SDK
  EnhancedTrade as Trade,
  EnhancedOrder as Order,
  EnhancedBalance as Balance,
  EnhancedOrderbookLevel as OrderbookLevel,
} from "@exchange/sdk";

// Import raw OrderbookLevel for WebSocket data (not enhanced)
import type { OrderbookLevel as RawOrderbookLevel } from "@exchange/sdk";

// Orderbook composite type - uses raw data from WebSocket
export interface Orderbook {
  market_id: string;
  bids: RawOrderbookLevel[];
  asks: RawOrderbookLevel[];
  timestamp?: number;
}

// For visualization
export interface PricePoint {
  timestamp: number;
  price: number;
}
