"""WebSocket client for real-time data streams."""

import asyncio
import json
from typing import Any, AsyncIterator, Optional
import websockets
from websockets.client import WebSocketClientProtocol

from .exceptions import WebSocketError
from .types import SubscriptionChannel


class WebSocketClient:
    """WebSocket client for real-time exchange data."""

    def __init__(self, ws_url: str):
        """
        Initialize the WebSocket client.

        Args:
            ws_url: WebSocket URL (e.g., "ws://localhost:8001/ws")
        """
        self.ws_url = ws_url
        self._ws: Optional[WebSocketClientProtocol] = None

    async def connect(self) -> "WebSocketHandle":
        """
        Connect to the WebSocket server.

        Returns:
            WebSocketHandle for sending/receiving messages
        """
        try:
            self._ws = await websockets.connect(self.ws_url)
            return WebSocketHandle(self._ws)
        except Exception as e:
            raise WebSocketError(f"Failed to connect: {e}")

    async def close(self):
        """Close the WebSocket connection."""
        if self._ws:
            await self._ws.close()


class WebSocketHandle:
    """Handle for sending and receiving WebSocket messages."""

    def __init__(self, ws: WebSocketClientProtocol):
        self._ws = ws

    async def subscribe(
        self,
        channel: SubscriptionChannel,
        market_id: Optional[str] = None,
        user_address: Optional[str] = None,
    ):
        """
        Subscribe to a channel.

        Args:
            channel: Subscription channel
            market_id: Optional market ID (for trades/orderbook)
            user_address: Optional user address (for user updates)
        """
        message = {
            "type": "subscribe",
            "channel": channel.value,
        }
        if market_id:
            message["market_id"] = market_id
        if user_address:
            message["user_address"] = user_address

        await self._send(message)

    async def unsubscribe(
        self,
        channel: SubscriptionChannel,
        market_id: Optional[str] = None,
        user_address: Optional[str] = None,
    ):
        """
        Unsubscribe from a channel.

        Args:
            channel: Subscription channel
            market_id: Optional market ID
            user_address: Optional user address
        """
        message = {
            "type": "unsubscribe",
            "channel": channel.value,
        }
        if market_id:
            message["market_id"] = market_id
        if user_address:
            message["user_address"] = user_address

        await self._send(message)

    async def ping(self):
        """Send a ping message."""
        await self._send({"type": "ping"})

    async def recv(self) -> Optional[dict[str, Any]]:
        """
        Receive the next message from the server.

        Returns:
            Parsed JSON message or None if connection closed
        """
        try:
            message = await self._ws.recv()
            return json.loads(message)
        except websockets.exceptions.ConnectionClosed:
            return None
        except Exception as e:
            raise WebSocketError(f"Failed to receive message: {e}")

    async def messages(self) -> AsyncIterator[dict[str, Any]]:
        """
        Iterate over incoming messages.

        Yields:
            Parsed JSON messages
        """
        while True:
            msg = await self.recv()
            if msg is None:
                break
            yield msg

    async def _send(self, message: dict[str, Any]):
        """Send a message to the server."""
        try:
            await self._ws.send(json.dumps(message))
        except Exception as e:
            raise WebSocketError(f"Failed to send message: {e}")

    async def close(self):
        """Close the WebSocket connection."""
        await self._ws.close()
