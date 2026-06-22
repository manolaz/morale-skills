from __future__ import annotations

PREFIX = "RSA-CANARY-"

def make_canary(run_id: str) -> str:
    return f"{PREFIX}{run_id}"

def find_canaries(text: str, tokens: list[str]) -> list[str]:
    return [t for t in tokens if t and t in text]
