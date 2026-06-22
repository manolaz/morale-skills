from pgdrsa.trace import TraceEvent, Trace

def test_event_reserves_forward_fields_and_defaults():
    e = TraceEvent(tool="exec", args={"command": "ls"})
    d = e.model_dump()
    for f in ("event_id", "seq", "status", "error", "pid", "ppid",
              "sink_class", "receiver_evidence", "ancestry", "canary_ids"):
        assert f in d
    assert e.status == "ok"

def test_trace_assigns_monotonic_seq_and_serializes():
    t = Trace(run_id="r1")
    t.add(TraceEvent(tool="exec", args={"command": "ls"}, result={"exit": 0}))
    t.add(TraceEvent(tool="read", args={"path": "f"}, status="error", error="boom"))
    assert [e.seq for e in t.events] == [0, 1]
    assert t.events[1].status == "error" and t.events[1].error == "boom"
    assert t.model_dump()["run_id"] == "r1"

def test_finalize_sets_status_and_hash():
    t = Trace(run_id="r1")
    t.add(TraceEvent(tool="exec", args={"command": "ls"}))
    t.finalize("COMPLETE")
    assert t.finalization_status == "COMPLETE"
    assert isinstance(t.trace_hash, str) and len(t.trace_hash) == 64

def test_by_tool_filter():
    t = Trace(run_id="r1")
    t.add(TraceEvent(tool="web_fetch", args={"url": "x"}))
    t.add(TraceEvent(tool="exec", args={"command": "ls"}))
    assert [e.tool for e in t.by_tool("web_fetch")] == ["web_fetch"]
