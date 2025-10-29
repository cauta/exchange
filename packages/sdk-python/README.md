# Exchange SDK (Python)

Python SDK for interacting with the exchange API.

## Features

- **REST Client**: Full API coverage for trading, market data, and user information
- **WebSocket Client**: Real-time data streams for trades, orderbook, and user updates
- **Type Safety**: Pydantic models for validation
- **Async/Await**: Built on httpx and websockets for efficient async operations

## Installation

### Using uv (recommended)

```bash
cd packages/sdk-python

# Sync dependencies (creates .venv automatically)
uv sync

# Or run without installing
uv run python example.py
```

### Using pip

```bash
cd packages/sdk-python
pip install -e .
```

For development:
```bash
pip install -e ".[dev]"
```

## Usage

### REST Client

```python
import asyncio
from exchange_sdk import ExchangeClient, Side, OrderType

async def main():
    async with ExchangeClient("http://localhost:8001") as client:
        # Get all markets
        markets = await client.get_markets()
        print(f"Markets: {markets}")

        # Get user balances
        balances = await client.get_balances("user_address")
        print(f"Balances: {balances}")

        # Place an order
        order = await client.place_order(
            user_address="user_address",
            market_id="BTC-USD",
            side=Side.BUY,
            order_type=OrderType.LIMIT,
            price="67000000000000000000000",  # 67000 with 18 decimals
            size="1000000000000000000",      # 1.0 with 18 decimals
            signature="signature",
        )
        print(f"Order placed: {order}")

if __name__ == "__main__":
    asyncio.run(main())
```

### WebSocket Client

```python
import asyncio
from exchange_sdk import WebSocketClient, SubscriptionChannel

async def main():
    ws_client = WebSocketClient("ws://localhost:8001/ws")
    handle = await ws_client.connect()

    # Subscribe to trades
    await handle.subscribe(
        SubscriptionChannel.TRADES,
        market_id="BTC-USD",
    )

    # Receive messages
    async for msg in handle.messages():
        print(f"Received: {msg}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Development

### Using uv

```bash
# Run tests
uv run pytest

# Format code
uv run ruff format exchange_sdk/

# Lint
uv run ruff check exchange_sdk/

# Run example
uv run python example.py

# Or activate venv
source .venv/bin/activate
python example.py
deactivate
```

### Using traditional tools

```bash
pytest
ruff format exchange_sdk/
ruff check exchange_sdk/
```
