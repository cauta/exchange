"""REST API client for the exchange."""

from typing import Optional
from uuid import UUID
import httpx

from .exceptions import APIError, ConnectionError, TimeoutError
from .types import (
    Balance,
    Market,
    Order,
    OrderCancelled,
    OrderPlaced,
    OrderType,
    Side,
    Token,
    Trade,
)


class ExchangeClient:
    """REST API client for the exchange."""

    def __init__(self, base_url: str, timeout: float = 30.0):
        """
        Initialize the exchange client.

        Args:
            base_url: Base URL of the exchange API (e.g., "http://localhost:8001")
            timeout: Request timeout in seconds
        """
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self._client = httpx.AsyncClient(timeout=timeout)

    async def close(self):
        """Close the HTTP client."""
        await self._client.aclose()

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.close()

    # ===== Health =====

    async def health(self) -> str:
        """Check API health."""
        response = await self._client.get(f"{self.base_url}/api/health")
        if response.status_code == 200:
            return response.text
        raise APIError(response.status_code, response.text)

    # ===== Info Endpoints =====

    async def get_token(self, ticker: str) -> Token:
        """Get token details."""
        response = await self._post_info({"type": "token_details", "ticker": ticker})
        return Token(**response["token"])

    async def get_market(self, market_id: str) -> Market:
        """Get market details."""
        response = await self._post_info({"type": "market_details", "market_id": market_id})
        return Market(**response["market"])

    async def get_markets(self) -> list[Market]:
        """Get all markets."""
        response = await self._post_info({"type": "all_markets"})
        return [Market(**m) for m in response["markets"]]

    async def get_tokens(self) -> list[Token]:
        """Get all tokens."""
        response = await self._post_info({"type": "all_tokens"})
        return [Token(**t) for t in response["tokens"]]

    # ===== User Endpoints =====

    async def get_orders(
        self, user_address: str, market_id: Optional[str] = None
    ) -> list[Order]:
        """Get user orders."""
        request = {"type": "orders", "user_address": user_address}
        if market_id:
            request["market_id"] = market_id

        response = await self._post_user(request)
        return [Order(**o) for o in response["orders"]]

    async def get_balances(self, user_address: str) -> list[Balance]:
        """Get user balances."""
        response = await self._post_user({"type": "balances", "user_address": user_address})
        return [Balance(**b) for b in response["balances"]]

    async def get_trades(
        self, user_address: str, market_id: Optional[str] = None
    ) -> list[Trade]:
        """Get user trades."""
        request = {"type": "trades", "user_address": user_address}
        if market_id:
            request["market_id"] = market_id

        response = await self._post_user(request)
        return [Trade(**t) for t in response["trades"]]

    # ===== Trade Endpoints =====

    async def place_order(
        self,
        user_address: str,
        market_id: str,
        side: Side,
        order_type: OrderType,
        price: str,
        size: str,
        signature: str,
    ) -> OrderPlaced:
        """
        Place an order.

        Args:
            user_address: User wallet address
            market_id: Market ID
            side: Order side (buy/sell)
            order_type: Order type (limit/market)
            price: Price as string (e.g., "67000000000000000000000")
            size: Size as string (e.g., "1000000000000000000")
            signature: Cryptographic signature
        """
        request = {
            "type": "place_order",
            "user_address": user_address,
            "market_id": market_id,
            "side": side.value,
            "order_type": order_type.value,
            "price": price,
            "size": size,
            "signature": signature,
        }

        response = await self._post_trade(request)
        return OrderPlaced(
            order=Order(**response["order"]), trades=[Trade(**t) for t in response["trades"]]
        )

    async def cancel_order(
        self, user_address: str, order_id: UUID, signature: str
    ) -> OrderCancelled:
        """
        Cancel an order.

        Args:
            user_address: User wallet address
            order_id: Order ID to cancel
            signature: Cryptographic signature
        """
        request = {
            "type": "cancel_order",
            "user_address": user_address,
            "order_id": str(order_id),
            "signature": signature,
        }

        response = await self._post_trade(request)
        return OrderCancelled(**response)

    # ===== Drip/Faucet Endpoint =====

    async def faucet(
        self, user_address: str, token_ticker: str, amount: str, signature: str
    ) -> dict:
        """
        Request testnet tokens from faucet.

        Args:
            user_address: User wallet address
            token_ticker: Token to request
            amount: Amount as string
            signature: Cryptographic signature
        """
        request = {
            "type": "faucet",
            "user_address": user_address,
            "token_ticker": token_ticker,
            "amount": amount,
            "signature": signature,
        }

        return await self._post_drip(request)

    # ===== Internal Helper Methods =====

    async def _post_info(self, request: dict) -> dict:
        """Make request to /api/info endpoint."""
        try:
            response = await self._client.post(f"{self.base_url}/api/info", json=request)
            if response.status_code == 200:
                return response.json()
            error = response.json()
            raise APIError(int(error.get("code", response.status_code)), error.get("error", "Unknown error"))
        except httpx.TimeoutException:
            raise TimeoutError("Request timed out")
        except httpx.ConnectError as e:
            raise ConnectionError(f"Connection failed: {e}")

    async def _post_user(self, request: dict) -> dict:
        """Make request to /api/user endpoint."""
        try:
            response = await self._client.post(f"{self.base_url}/api/user", json=request)
            if response.status_code == 200:
                return response.json()
            error = response.json()
            raise APIError(int(error.get("code", response.status_code)), error.get("error", "Unknown error"))
        except httpx.TimeoutException:
            raise TimeoutError("Request timed out")
        except httpx.ConnectError as e:
            raise ConnectionError(f"Connection failed: {e}")

    async def _post_trade(self, request: dict) -> dict:
        """Make request to /api/trade endpoint."""
        try:
            response = await self._client.post(f"{self.base_url}/api/trade", json=request)
            if response.status_code == 200:
                return response.json()
            error = response.json()
            raise APIError(int(error.get("code", response.status_code)), error.get("error", "Unknown error"))
        except httpx.TimeoutException:
            raise TimeoutError("Request timed out")
        except httpx.ConnectError as e:
            raise ConnectionError(f"Connection failed: {e}")

    async def _post_drip(self, request: dict) -> dict:
        """Make request to /api/drip endpoint."""
        try:
            response = await self._client.post(f"{self.base_url}/api/drip", json=request)
            if response.status_code == 200:
                return response.json()
            error = response.json()
            raise APIError(int(error.get("code", response.status_code)), error.get("error", "Unknown error"))
        except httpx.TimeoutException:
            raise TimeoutError("Request timed out")
        except httpx.ConnectError as e:
            raise ConnectionError(f"Connection failed: {e}")
