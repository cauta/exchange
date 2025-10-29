"""Exceptions for the Exchange SDK."""


class ExchangeError(Exception):
    """Base exception for all SDK errors."""

    pass


class APIError(ExchangeError):
    """Error returned by the API."""

    def __init__(self, status_code: int, message: str):
        self.status_code = status_code
        self.message = message
        super().__init__(f"API Error ({status_code}): {message}")


class ConnectionError(ExchangeError):
    """Connection error."""

    pass


class TimeoutError(ExchangeError):
    """Request timeout."""

    pass


class WebSocketError(ExchangeError):
    """WebSocket error."""

    pass
