"""WebSocket client for real-time data streams."""

import asyncio
import json
from typing import Any, AsyncIterator, Optional
import websockets
from websockets.client import WebSocketClientProtocol

from .exceptions import WebSocketError
from .types import SubscriptionChannel


class WebSocketClient:
    """WebSocket client for real-time exchange data with auto-reconnect."""

    def __init__(
        self,
        ws_url: str,
        reconnect_delays: Optional[list[float]] = None,
        ping_interval: float = 30.0,
    ):
        """
        Initialize the WebSocket client.

        Args:
            ws_url: WebSocket URL (e.g., "ws://localhost:8001/ws")
            reconnect_delays: List of delays (in seconds) between reconnect attempts.
                             Defaults to [1, 2, 5, 10, 30] (exponential backoff)
            ping_interval: Interval (in seconds) between ping messages. Default 30s.
        """
        self.ws_url = ws_url
        self.reconnect_delays = reconnect_delays or [1, 2, 5, 10, 30]
        self.ping_interval = ping_interval
        self._ws: Optional[WebSocketClientProtocol] = None
        self._subscriptions: list[dict[str, Any]] = []
        self._reconnect_task: Optional[asyncio.Task] = None
        self._ping_task: Optional[asyncio.Task] = None
        self._should_reconnect = True

    async def connect(self) -> "WebSocketHandle":
        """
        Connect to the WebSocket server.

        Returns:
            WebSocketHandle for sending/receiving messages
        """
        try:
            self._ws = await websockets.connect(self.ws_url)
            self._should_reconnect = True

            # Start ping task
            if self._ping_task:
                self._ping_task.cancel()
            self._ping_task = asyncio.create_task(self._ping_loop())

            return WebSocketHandle(self, self._ws)
        except Exception as e:
            raise WebSocketError(f"Failed to connect: {e}")

    async def connect_with_retry(self) -> "WebSocketHandle":
        """
        Connect to the WebSocket server with automatic retry on failure.

        Returns:
            WebSocketHandle for sending/receiving messages
        """
        for i, delay in enumerate(self.reconnect_delays):
            try:
                return await self.connect()
            except Exception as e:
                if i == len(self.reconnect_delays) - 1:
                    raise WebSocketError(f"Failed to connect after {len(self.reconnect_delays)} attempts: {e}")
                await asyncio.sleep(delay)

        raise WebSocketError("Failed to connect")

    async def _reconnect(self):
        """Internal method to handle reconnection."""
        for delay in self.reconnect_delays:
            if not self._should_reconnect:
                break

            try:
                await asyncio.sleep(delay)
                self._ws = await websockets.connect(self.ws_url)

                # Resubscribe to all channels
                for sub in self._subscriptions:
                    await self._send(sub)

                # Restart ping task
                if self._ping_task:
                    self._ping_task.cancel()
                self._ping_task = asyncio.create_task(self._ping_loop())

                return
            except Exception:
                continue

    async def _ping_loop(self):
        """Send periodic ping messages to keep connection alive."""
        try:
            while self._should_reconnect and self._ws:
                await asyncio.sleep(self.ping_interval)
                if self._ws and not self._ws.closed:
                    await self._send({"type": "ping"})
        except asyncio.CancelledError:
            pass
        except Exception:
            pass

    async def _send(self, message: dict[str, Any]):
        """Send a message to the server."""
        if self._ws and not self._ws.closed:
            try:
                await self._ws.send(json.dumps(message))
            except Exception as e:
                raise WebSocketError(f"Failed to send message: {e}")

    def _track_subscription(self, message: dict[str, Any]):
        """Track subscription for reconnection."""
        if message.get("type") == "subscribe":
            # Remove any existing subscription with same channel/market/user
            self._subscriptions = [
                s for s in self._subscriptions
                if not (
                    s.get("channel") == message.get("channel")
                    and s.get("market_id") == message.get("market_id")
                    and s.get("user_address") == message.get("user_address")
                )
            ]
            self._subscriptions.append(message)
        elif message.get("type") == "unsubscribe":
            # Remove subscription
            self._subscriptions = [
                s for s in self._subscriptions
                if not (
                    s.get("channel") == message.get("channel")
                    and s.get("market_id") == message.get("market_id")
                    and s.get("user_address") == message.get("user_address")
                )
            ]

    async def close(self):
        """Close the WebSocket connection."""
        self._should_reconnect = False

        if self._ping_task:
            self._ping_task.cancel()
            try:
                await self._ping_task
            except asyncio.CancelledError:
                pass

        if self._reconnect_task:
            self._reconnect_task.cancel()
            try:
                await self._reconnect_task
            except asyncio.CancelledError:
                pass

        if self._ws:
            await self._ws.close()


class WebSocketHandle:
    """Handle for sending and receiving WebSocket messages with auto-reconnect."""

    def __init__(self, client: WebSocketClient, ws: WebSocketClientProtocol):
        self._client = client
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

        self._client._track_subscription(message)
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

        self._client._track_subscription(message)
        await self._send(message)

    async def ping(self):
        """Send a ping message."""
        await self._send({"type": "ping"})

    async def recv(self) -> Optional[dict[str, Any]]:
        """
        Receive the next message from the server.

        If connection is closed, will attempt to reconnect automatically.

        Returns:
            Parsed JSON message or None if connection closed and reconnection failed
        """
        try:
            message = await self._ws.recv()
            return json.loads(message)
        except websockets.exceptions.ConnectionClosed:
            # Attempt reconnection
            if self._client._should_reconnect:
                try:
                    await self._client._reconnect()
                    self._ws = self._client._ws
                    if self._ws:
                        message = await self._ws.recv()
                        return json.loads(message)
                except Exception:
                    pass
            return None
        except Exception as e:
            raise WebSocketError(f"Failed to receive message: {e}")

    async def messages(self) -> AsyncIterator[dict[str, Any]]:
        """
        Iterate over incoming messages with auto-reconnect.

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
        await self._client.close()
