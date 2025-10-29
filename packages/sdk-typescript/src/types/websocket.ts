/**
 * WebSocket message types for the exchange
 * These are hand-written since WebSocket isn't part of OpenAPI
 */

import type { components } from './generated';

// Re-export domain types from generated
export type Trade = components['schemas']['Trade'];
export type Order = components['schemas']['Order'];

// Subscription channels
export type SubscriptionChannel = 'Trades' | 'Orderbook' | 'User';

// Client -> Server messages
export type ClientMessage =
  | {
      type: 'subscribe';
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: 'unsubscribe';
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: 'ping';
    };

// Server -> Client messages
export type ServerMessage =
  | {
      type: 'subscribed';
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: 'unsubscribed';
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: 'trade';
      market_id: string;
      trade: Trade;
    }
  | {
      type: 'orderbook_snapshot';
      market_id: string;
      bids: OrderbookLevel[];
      asks: OrderbookLevel[];
    }
  | {
      type: 'orderbook_update';
      market_id: string;
      bids: OrderbookLevel[];
      asks: OrderbookLevel[];
    }
  | {
      type: 'order_placed';
      order: Order;
    }
  | {
      type: 'order_cancelled';
      order_id: string;
    }
  | {
      type: 'pong';
    }
  | {
      type: 'error';
      message: string;
    };

export interface OrderbookLevel {
  price: string;
  size: string;
}

// Message handler type
export type MessageHandler<T extends ServerMessage = ServerMessage> = (message: T) => void;
