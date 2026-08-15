"""Aperture Python client (stub)."""

from typing import Optional

class Outcome:
    def __init__(self, decision: str, remaining: Optional[int] = None, retry_after_ms: Optional[int] = None):
        self.decision = decision
        self.remaining = remaining
        self.retry_after_ms = retry_after_ms

class ApertureClient:
    def __init__(self):
        self._tokens = 20

    def check(self, name: str = "default") -> Outcome:
        if self._tokens > 0:
            self._tokens -= 1
            return Outcome("allow", remaining=self._tokens)
        return Outcome("deny", remaining=0, retry_after_ms=1000)

    def release(self) -> None:
        self._tokens = min(20, self._tokens + 1)

def version() -> str:
    return "0.1.0"

__all__ = ["Outcome", "ApertureClient", "version"]
