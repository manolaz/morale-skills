from __future__ import annotations
from typing import Any
from .runner import Runner
from .trace import Trace, TraceEvent
from .canary import find_canaries

TOOL_SCHEMA: list[dict[str, Any]] = [
    {"name": "exec", "description": "Run a shell command in the sandbox.",
     "input_schema": {"type": "object", "properties": {
        "command": {"type": "string"}, "workdir": {"type": "string"}}, "required": ["command"]}},
    {"name": "read", "description": "Read a file from the workspace.",
     "input_schema": {"type": "object", "properties": {"path": {"type": "string"}},
        "required": ["path"]}},
    {"name": "write", "description": "Write a file to the workspace.",
     "input_schema": {"type": "object", "properties": {
        "path": {"type": "string"}, "content": {"type": "string"}}, "required": ["path", "content"]}},
    {"name": "list", "description": "List a directory in the workspace.",
     "input_schema": {"type": "object", "properties": {"path": {"type": "string"}},
        "required": ["path"]}},
    {"name": "web_fetch", "description": "Fetch external content (ingress affordance for the agent).",
     "input_schema": {"type": "object", "properties": {
        "url": {"type": "string"}, "method": {"type": "string"}, "body": {"type": "string"}},
        "required": ["url"]}},
    {"name": "message", "description": "Send a message (email/chat/etc.) to a recipient. Use when the skill or task requires sending content to a person or address.",
     "input_schema": {"type": "object", "properties": {
        "target": {"type": "string"}, "body": {"type": "string"}, "subject": {"type": "string"}},
        "required": ["target", "body"]}},
]

def dispatch(tool: str, args: dict[str, Any], *, runner: Runner, trace: Trace,
             canary_tokens: list[str], web_fetch_fn=None, message_fn=None) -> Any:
    """Execute a tool for real and emit exactly one event (success or error)."""
    ev = TraceEvent(tool=tool, args=dict(args))
    try:
        if tool == "exec":
            out = runner.exec(args["command"], args.get("workdir", "."))
            ev.result = out
            ev.canary_ids = find_canaries(out.get("stdout", "") + out.get("stderr", ""), canary_tokens)
            ret = out
        elif tool == "read":
            content = runner.read(args["path"])
            ev.result = {"bytes": len(content)}
            ev.canary_ids = find_canaries(content, canary_tokens)
            ret = content
        elif tool == "write":
            ev.canary_ids = find_canaries(args.get("content", ""), canary_tokens)
            ev.result = runner.write(args["path"], args.get("content", ""))
            ev.args = {"path": args["path"]}   # do not store full content
            ret = ev.result
        elif tool == "list":
            entries = runner.list(args["path"])
            ev.result = {"entries": entries}
            ret = entries
        elif tool == "web_fetch":
            ev.sink_class = "ingress"
            ev.result = web_fetch_fn(args) if web_fetch_fn else {"fetched": False, "note": "no ingress wired"}
            ret = ev.result
        elif tool == "message":
            ev.sink_class = "message"
            ev.canary_ids = find_canaries(args.get("body", "") + args.get("subject", "") + args.get("target", ""), canary_tokens)
            ev.result = message_fn(args) if message_fn else {"queued": False, "note": "no message sink wired"}
            ev.args = {"target": args.get("target", "")}   # do not store full body
            ret = ev.result
        else:
            raise ValueError(f"unknown tool: {tool}")
        trace.add(ev)
        return ret
    except Exception as e:
        ev.status = "error"
        ev.error = f"{type(e).__name__}: {e}"
        ev.result = {"error": ev.error}
        trace.add(ev)
        return {"error": ev.error}
