# Exchange TypeScript SDK

Type-safe TypeScript SDK for the exchange API, with types auto-generated from OpenAPI.

## Installation

```bash
bun add @exchange/sdk
```

## Quick Start

```typescript
import { ExchangeClient } from '@exchange/sdk';

const client = new ExchangeClient({
  restUrl: 'http://localhost:8888',
  wsUrl: 'ws://localhost:8888/ws',
});

// REST API
const markets = await client.rest.getMarkets();
const order = await client.rest.placeOrder({
  userAddress: 'alice',
  marketId: 'BTC/USDC',
  side: 'Buy',
  orderType: 'Limit',
  price: '100000000000',
  size: '1000000',
  signature: 'sig...',
});

// WebSocket
client.ws.connect();
client.ws.subscribe('Trades', { marketId: 'BTC/USDC' });

const unsubscribe = client.ws.on('trade', (message) => {
  console.log('New trade:', message.trade);
});

// Cleanup
unsubscribe();
client.disconnect();
```

## Features

- ✅ **Type-safe**: All types generated from OpenAPI schema
- ✅ **Auto-reconnect**: WebSocket auto-reconnects with exponential backoff
- ✅ **Error handling**: Typed errors for better debugging
- ✅ **Message queuing**: WebSocket messages queued when disconnected
- ✅ **Ping/pong**: Automatic keep-alive
- ✅ **Clean API**: Simple, intuitive interface

## Development

```bash
# Generate types from OpenAPI
bun run generate

# Build
bun run build

# Type check
bun run typecheck
```

## Type Generation

Types are automatically generated from the backend's OpenAPI schema:

```bash
# In backend
cargo run --bin generate-openapi

# In SDK
bun run generate
```

This ensures frontend and backend types are always in sync!
