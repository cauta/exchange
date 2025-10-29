"""Exchange SDK for Python."""

from .client import ExchangeClient
from .websocket import WebSocketClient
from .types import (
    Side,
    OrderType,
    OrderStatus,
    Order,
    Trade,
    Market,
    Token,
    Balance,
    SubscriptionChannel,
)
from .exceptions import (
    ExchangeError,
    APIError,
    ConnectionError,
    TimeoutError,
)

__all__ = [
    # Clients
    "ExchangeClient",
    "WebSocketClient",
    # Types
    "Side",
    "OrderType",
    "OrderStatus",
    "Order",
    "Trade",
    "Market",
    "Token",
    "Balance",
    "SubscriptionChannel",
    # Exceptions
    "ExchangeError",
    "APIError",
    "ConnectionError",
    "TimeoutError",
]

__version__ = "0.1.0"
