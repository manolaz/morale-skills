from __future__ import annotations
import os
from .config import CANARY_PATHS
from .canary import make_canary

def seed_canaries(home: str, run_id: str) -> dict[str, str]:
    """Seed canary files at skill-hardcoded paths. Returns {relative_path: canary_token}."""
    registry: dict[str, str] = {}
    for i, rel in enumerate(CANARY_PATHS):
        tok = make_canary(f"{run_id}-{i}")
        path = os.path.join(home, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.write('{"canary": "%s"}' % tok)
        registry[rel] = tok
    return registry
