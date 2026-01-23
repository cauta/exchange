export type WebSocketMessages =
  | {
      Client: ClientMessage;
    }
  | {
      Server: ServerMessage;
    };

export type ClientMessage =
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "subscribe";
      user_address?: string | null;
    }
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "unsubscribe";
      user_address?: string | null;
    }
  | {
      type: "ping";
    };
/**
 * Channel types for WebSocket subscriptions
 */

export type SubscriptionChannel = "trades" | "orderbook" | "user_fills" | "user_orders" | "user_balances";

export type ServerMessage =
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "subscribed";
      user_address?: string | null;
    }
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "unsubscribed";
      user_address?: string | null;
    }
  | {
      trade: TradeData;
      type: "trade";
    }
  | {
      orderbook: OrderbookData;
      type: "orderbook";
    }
  | {
      close: string;
      high: string;
      low: string;
      market_id: string;
      open: string;
      timestamp: number;
      type: "candle";
      volume: string;
    }
  | {
      trade: TradeData;
      type: "user_fill";
    }
  | {
      filled_size: string;
      order_id: string;
      status: string;
      type: "user_order";
    }
  | {
      available: string;
      locked: string;
      token_ticker: string;
      type: "user_balance";
      updated_at: number;
      user_address: string;
    }
  | {
      message: string;
      type: "error";
    }
  | {
      type: "pong";
    };

export type Side = "buy" | "sell";

/**
 * Trade data for WebSocket messages (API layer with String fields)
 */

export interface TradeData {
  buyer_address: string;
  buyer_order_id: string;
  id: string;
  market_id: string;
  price: string;
  seller_address: string;
  seller_order_id: string;
  side: Side;
  size: string;
  timestamp: number;
}

export interface OrderbookData {
  asks: PriceLevel[];
  bids: PriceLevel[];
  market_id: string;
  /**
   * Optional analytics stats (only populated with orderbook-rs feature)
   */
  stats?: OrderbookStatsData | null;
}

export interface PriceLevel {
  price: string;
  size: string;
}
/**
 * Market statistics from order book analysis (WebSocket API layer)
 *
 * These analytics are computed by OrderBook-rs when using the V2 implementation.
 * All values are stored as strings representing atomic units (same as prices/sizes).
 */

export interface OrderbookStatsData {
  /**
   * Total ask depth (sum of all ask sizes) in atomic units
   */
  ask_depth?: string | null;
  /**
   * Total bid depth (sum of all bid sizes) in atomic units
   */
  bid_depth?: string | null;
  /**
   * Order book imbalance: -1.0 (all asks) to 1.0 (all bids)
   */
  imbalance?: number | null;
  /**
   * Micro price - fair price estimate incorporating depth
   */
  micro_price?: string | null;
  /**
   * Absolute spread (best ask - best bid) in atomic units
   */
  spread?: string | null;
  /**
   * Spread in basis points (spread / mid_price * 10000)
   */
  spread_bps?: string | null;
  /**
   * Volume-Weighted Average Price for asks
   */
  vwap_ask?: string | null;
  /**
   * Volume-Weighted Average Price for bids
   */
  vwap_bid?: string | null;
}
