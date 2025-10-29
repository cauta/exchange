/**
 * Exchange SDK - TypeScript client for the exchange API
 *
 * @example
 * ```typescript
 * import { ExchangeClient } from '@exchange/sdk';
 *
 * const client = new ExchangeClient({
 *   restUrl: 'http://localhost:8888',
 *   wsUrl: 'ws://localhost:8888/ws',
 * });
 *
 * // REST API
 * const markets = await client.rest.getMarkets();
 *
 * // WebSocket
 * client.ws.connect();
 * client.ws.subscribe('Trades', { marketId: 'BTC/USDC' });
 * client.ws.on('trade', (message) => {
 *   console.log('New trade:', message.trade);
 * });
 * ```
 */

import { RestClient } from './rest';
import type { RestClientConfig } from './rest';
import { WebSocketClient } from './websocket';
import type { WebSocketClientConfig } from './websocket';

export { RestClient } from './rest';
export type { RestClientConfig } from './rest';

export { WebSocketClient } from './websocket';
export type { WebSocketClientConfig } from './websocket';

export {
  SdkError,
  ApiError,
  WebSocketError,
  ValidationError,
} from './errors';

// Export types
export type {
  Market,
  Token,
  Order,
  Trade,
  Balance,
  Side,
  OrderType,
  OrderStatus,
} from './rest';

export type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  MessageHandler,
  OrderbookLevel,
} from './types/websocket';

// Re-export generated types for advanced usage
export type { components } from './types/generated';

/**
 * Main Exchange SDK client
 */
export interface ExchangeClientConfig {
  restUrl: string;
  wsUrl: string;
  restTimeout?: number;
  wsReconnectDelays?: number[];
  wsPingInterval?: number;
}

export class ExchangeClient {
  public readonly rest: RestClient;
  public readonly ws: WebSocketClient;

  constructor(config: ExchangeClientConfig) {
    this.rest = new RestClient({
      baseUrl: config.restUrl,
      timeout: config.restTimeout,
    });

    this.ws = new WebSocketClient({
      url: config.wsUrl,
      reconnectDelays: config.wsReconnectDelays,
      pingInterval: config.wsPingInterval,
    });
  }

  /**
   * Disconnect all connections
   */
  disconnect(): void {
    this.ws.disconnect();
  }
}
