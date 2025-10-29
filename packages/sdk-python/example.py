"""Example usage of the Exchange SDK."""

import asyncio
from exchange_sdk import ExchangeClient


async def main():
    # Create client
    client = ExchangeClient("http://localhost:8001")

    try:
        # Check health
        health = await client.health()
        print(f"✅ Server health: {health}")

        # Get markets
        markets = await client.get_markets()
        print(f"📊 Markets: {len(markets)} available")

        # Get tokens
        tokens = await client.get_tokens()
        print(f"🪙 Tokens: {len(tokens)} available")

    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
