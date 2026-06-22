from __future__ import annotations
import hashlib, json
from typing import Any, Optional
from pydantic import BaseModel, Field

class TraceEvent(BaseModel):
    tool: str
    args: dict[str, Any] = Field(default_factory=dict)
    result: dict[str, Any] = Field(default_factory=dict)
    status: str = "ok"                      # "ok" | "error"
    error: Optional[str] = None
    canary_ids: list[str] = Field(default_factory=list)
    # --- reserved for Phase B (populated later) ---
    event_id: Optional[str] = None
    seq: Optional[int] = None
    pid: Optional[int] = None
    ppid: Optional[int] = None
    sink_class: Optional[str] = None
    receiver_evidence: Optional[dict[str, Any]] = None
    ancestry: list[int] = Field(default_factory=list)

class Trace(BaseModel):
    run_id: str
    events: list[TraceEvent] = Field(default_factory=list)
    finalization_status: Optional[str] = None   # COMPLETE | CENSORED | INVALID
    trace_hash: Optional[str] = None

    def add(self, event: TraceEvent) -> None:
        event.seq = len(self.events)
        self.events.append(event)

    def by_tool(self, name: str) -> list[TraceEvent]:
        return [e for e in self.events if e.tool == name]

    def finalize(self, status: str) -> None:
        self.finalization_status = status
        payload = json.dumps([e.model_dump() for e in self.events], sort_keys=True, default=str)
        self.trace_hash = hashlib.sha256(payload.encode()).hexdigest()
