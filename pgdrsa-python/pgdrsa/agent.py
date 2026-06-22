from __future__ import annotations
import json
from typing import Any, Protocol
from .config import MAX_AGENT_TURNS, LLM_BASE_URL, LLM_API_KEY, LLM_MODEL
from .tools import TOOL_SCHEMA, dispatch
from .trace import Trace
from .runner import Runner

class LLM(Protocol):
    def complete(self, messages: list[dict[str, Any]], tools: list[dict[str, Any]]) -> dict[str, Any]: ...

def run_agent(*, llm: LLM, runner: Runner, trace: Trace, canary_tokens: list[str],
              system: str, user: str, web_fetch_fn=None, message_fn=None) -> str:
    if hasattr(llm, "set_system"):
        llm.set_system(system)
    messages: list[dict[str, Any]] = [{"role": "user", "content": user}]
    final_text = ""
    for _ in range(MAX_AGENT_TURNS):
        resp = llm.complete(messages, TOOL_SCHEMA)
        final_text = resp.get("text", final_text)
        if resp.get("stop_reason") != "tool_use":
            break
        results = []
        for call in resp.get("tool_calls", []):
            out = dispatch(call["name"], call["input"], runner=runner, trace=trace,
                           canary_tokens=canary_tokens, web_fetch_fn=web_fetch_fn,
                           message_fn=message_fn)
            results.append({"id": call["id"], "content": str(out)})
        messages.append({"role": "assistant", "tool_calls": resp["tool_calls"],
                         "text": resp.get("text", "")})
        messages.append({"role": "tool", "results": results})
    return final_text


class OpenAILLM:
    """Adapter wrapping any OpenAI-compatible /v1/chat/completions endpoint into the
    internal LLM.complete protocol. Configured via .env (PGDRSA_LLM_*)."""
    def __init__(self, model: str = LLM_MODEL, base_url: str = LLM_BASE_URL,
                 api_key: str = LLM_API_KEY, client=None):
        from openai import OpenAI
        self.model = model
        self.base_url = base_url
        # Hard limits: 60s per call, no retries. The SDK default (600s × 2 retries = 30 min)
        # let a single hung request stall the whole 100-skill run.
        self.client = client or OpenAI(base_url=base_url, api_key=api_key,
                                       timeout=60.0, max_retries=0)
        self._system = ""

    def set_system(self, system: str) -> None:
        self._system = system

    def complete(self, messages, tools):
        resp = self.client.chat.completions.create(
            model=self.model,
            messages=self._to_api(messages),
            tools=self._tools_to_api(tools),
            tool_choice="auto",
        )
        msg = resp.choices[0].message
        tool_calls = []
        for c in (msg.tool_calls or []):
            try:
                args = json.loads(c.function.arguments) if c.function.arguments else {}
            except json.JSONDecodeError:
                args = {"_raw_arguments": c.function.arguments}
            tool_calls.append({"id": c.id, "name": c.function.name, "input": args})
        text = msg.content or ""
        stop_reason = "tool_use" if tool_calls else "end_turn"
        return {"stop_reason": stop_reason, "tool_calls": tool_calls, "text": text}

    @staticmethod
    def _tools_to_api(tools):
        return [{"type": "function",
                 "function": {"name": t["name"], "description": t.get("description", ""),
                              "parameters": t.get("input_schema", {"type": "object", "properties": {}})}}
                for t in tools]

    def _to_api(self, messages):
        api = []
        if messages and messages[0]["role"] == "user":
            api.append({"role": "system", "content": self._system or ""})
        for m in messages:
            if m["role"] == "user":
                api.append({"role": "user", "content": m["content"]})
            elif m["role"] == "assistant":
                if m.get("tool_calls"):
                    api.append({"role": "assistant", "content": m.get("text") or None,
                                "tool_calls": [
                                    {"id": c["id"], "type": "function",
                                     "function": {"name": c["name"],
                                                  "arguments": json.dumps(c["input"])}}
                                    for c in m["tool_calls"]]})
                else:
                    api.append({"role": "assistant", "content": m.get("text", "")})
            elif m["role"] == "tool":
                for r in m["results"]:
                    api.append({"role": "tool", "tool_call_id": r["id"], "content": r["content"]})
        return api
