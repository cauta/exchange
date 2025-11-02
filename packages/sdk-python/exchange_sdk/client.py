"""REST API client for the exchange."""

from typing import Optional
from uuid import UUID
import httpx

from .exceptions import APIError, ConnectionError, TimeoutError
from .types import (
    Balance,
    Candle,
    Market,
    Order,
    OrderCancelled,
    OrderPlaced,
    OrdersCancelled,
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

    async def cancel_all_orders(
        self, user_address: str, signature: str, market_id: Optional[str] = None
    ) -> OrdersCancelled:
        """
        Cancel all orders for a user, optionally filtered by market.

        Args:
            user_address: User wallet address
            signature: Cryptographic signature
            market_id: Optional market ID to filter cancellations
        """
        request = {
            "type": "cancel_all_orders",
            "user_address": user_address,
            "signature": signature,
        }
        if market_id:
            request["market_id"] = market_id

        response = await self._post_trade(request)
        return OrdersCancelled(**response)

    # ===== Rounding and Decimal Helpers =====

    @staticmethod
    def round_size_to_lot(size: int, lot_size: int) -> int:
        """
        Round a size to the nearest multiple of lot_size (rounds down).

        Args:
            size: Size in atoms
            lot_size: Lot size in atoms

        Returns:
            Rounded size
        """
        if lot_size == 0:
            return size
        return (size // lot_size) * lot_size

    async def place_order_with_rounding(
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
        Place an order with automatic size rounding to lot_size.

        Args:
            user_address: User wallet address
            market_id: Market ID
            side: Order side (buy/sell)
            order_type: Order type (limit/market)
            price: Price as string (e.g., "67000000000000000000000")
            size: Size as string (e.g., "1000000000000000000")
            signature: Cryptographic signature
        """
        # Get market details to find lot_size
        market = await self.get_market(market_id)

        # Parse size
        size_val = int(size)
        lot_size_val = int(market.lot_size)

        # Round size to lot_size
        rounded_size = self.round_size_to_lot(size_val, lot_size_val)

        # Check if rounded size is 0
        if rounded_size == 0:
            raise APIError(
                400,
                f"Size {size} is too small for lot_size {market.lot_size} (rounded to 0)",
            )

        # Place order with rounded size
        return await self.place_order(
            user_address=user_address,
            market_id=market_id,
            side=side,
            order_type=order_type,
            price=price,
            size=str(rounded_size),
            signature=signature,
        )

    async def place_order_decimal(
        self,
        user_address: str,
        market_id: str,
        side: Side,
        order_type: OrderType,
        price_decimal: str,
        size_decimal: str,
        signature: str,
    ) -> OrderPlaced:
        """
        Place an order with human-readable decimal values (e.g., "0.5" BTC, "110000" USDC).
        Automatically converts to atoms using token decimals from market config.

        Args:
            user_address: User wallet address
            market_id: Market ID
            side: Order side (buy/sell)
            order_type: Order type (limit/market)
            price_decimal: Human-readable price (e.g., "110000.50")
            size_decimal: Human-readable size (e.g., "0.5")
            signature: Cryptographic signature
        """
        from decimal import Decimal

        # Get market and token details
        market = await self.get_market(market_id)
        base_token = await self.get_token(market.base_ticker)
        quote_token = await self.get_token(market.quote_ticker)

        # Convert price from decimal to atoms using quote token decimals
        price_dec = Decimal(price_decimal)
        price_multiplier = Decimal(10) ** quote_token.decimals
        price_atoms = int(price_dec * price_multiplier)

        # Convert size from decimal to atoms using base token decimals
        size_dec = Decimal(size_decimal)
        size_multiplier = Decimal(10) ** base_token.decimals
        size_atoms = int(size_dec * size_multiplier)

        # Round size to lot_size
        lot_size_val = int(market.lot_size)
        rounded_size = self.round_size_to_lot(size_atoms, lot_size_val)

        # Check minimum size
        min_size_val = int(market.min_size)
        if rounded_size < min_size_val:
            raise APIError(
                400,
                f"Size {size_decimal} ({rounded_size} atoms) is below minimum {market.min_size} atoms",
            )

        # Place order with converted values
        return await self.place_order(
            user_address=user_address,
            market_id=market_id,
            side=side,
            order_type=order_type,
            price=str(price_atoms),
            size=str(rounded_size),
            signature=signature,
        )

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

    # ===== Candles Endpoints =====

    async def get_candles(
        self,
        market_id: str,
        interval: str,
        from_timestamp: int,
        to_timestamp: int,
        count_back: Optional[int] = None,
    ) -> list[Candle]:
        """
        Get OHLCV candles for a market.

        Args:
            market_id: Market ID
            interval: Candle interval (e.g., "1m", "5m", "1h", "1d")
            from_timestamp: Start timestamp (Unix seconds)
            to_timestamp: End timestamp (Unix seconds)
            count_back: Optional number of candles to return
        """
        request = {
            "market_id": market_id,
            "interval": interval,
            "from": from_timestamp,
            "to": to_timestamp,
        }
        if count_back is not None:
            request["count_back"] = count_back

        response = await self._post_candles(request)
        return [Candle(**c) for c in response["candles"]]

    # ===== Admin Endpoints (Test/Dev Only) =====

    async def admin_create_token(
        self, ticker: str, decimals: int, name: str
    ) -> Token:
        """
        Create a token (admin only).

        Args:
            ticker: Token ticker symbol
            decimals: Number of decimals
            name: Token name
        """
        request = {
            "type": "create_token",
            "ticker": ticker,
            "decimals": decimals,
            "name": name,
        }
        response = await self._post_admin(request)
        return Token(**response["token"])

    async def admin_create_market(
        self,
        base_ticker: str,
        quote_ticker: str,
        tick_size: str,
        lot_size: str,
        min_size: str,
        maker_fee_bps: int,
        taker_fee_bps: int,
    ) -> Market:
        """
        Create a market (admin only).

        Args:
            base_ticker: Base token ticker
            quote_ticker: Quote token ticker
            tick_size: Minimum price increment
            lot_size: Minimum size increment
            min_size: Minimum order size
            maker_fee_bps: Maker fee in basis points
            taker_fee_bps: Taker fee in basis points
        """
        request = {
            "type": "create_market",
            "base_ticker": base_ticker,
            "quote_ticker": quote_ticker,
            "tick_size": tick_size,
            "lot_size": lot_size,
            "min_size": min_size,
            "maker_fee_bps": maker_fee_bps,
            "taker_fee_bps": taker_fee_bps,
        }
        response = await self._post_admin(request)
        return Market(**response["market"])

    async def admin_faucet(
        self, user_address: str, token_ticker: str, amount: str
    ) -> dict:
        """
        Request testnet tokens via admin endpoint (admin only).

        Args:
            user_address: User wallet address
            token_ticker: Token to request
            amount: Amount as string
        """
        request = {
            "type": "faucet",
            "user_address": user_address,
            "token_ticker": token_ticker,
            "amount": amount,
            "signature": "admin",
        }
        response = await self._post_admin(request)
        return {
            "user_address": response["user_address"],
            "token_ticker": response["token_ticker"],
            "amount": response["amount"],
            "new_balance": response["new_balance"],
        }

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

    async def _post_candles(self, request: dict) -> dict:
        """Make request to /api/candles endpoint."""
        try:
            response = await self._client.post(f"{self.base_url}/api/candles", json=request)
            if response.status_code == 200:
                return response.json()
            error = response.json()
            raise APIError(int(error.get("code", response.status_code)), error.get("error", "Unknown error"))
        except httpx.TimeoutException:
            raise TimeoutError("Request timed out")
        except httpx.ConnectError as e:
            raise ConnectionError(f"Connection failed: {e}")

    async def _post_admin(self, request: dict) -> dict:
        """Make request to /api/admin endpoint."""
        try:
            response = await self._client.post(f"{self.base_url}/api/admin", json=request)
            if response.status_code == 200:
                return response.json()
            error = response.json()
            raise APIError(int(error.get("code", response.status_code)), error.get("error", "Unknown error"))
        except httpx.TimeoutException:
            raise TimeoutError("Request timed out")
        except httpx.ConnectError as e:
            raise ConnectionError(f"Connection failed: {e}")
