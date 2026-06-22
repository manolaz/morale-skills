import tempfile
from pgdrsa.runner import LocalRunner
from pgdrsa.trace import Trace
from pgdrsa.tools import TOOL_SCHEMA, dispatch

def test_exec_runs_and_traces_success():
    with tempfile.TemporaryDirectory() as d:
        r = LocalRunner(workspace=d); t = Trace(run_id="r1")
        out = dispatch("exec", {"command": "echo hi"}, runner=r, trace=t, canary_tokens=[])
        assert out["exit"] == 0 and "hi" in out["stdout"]
        ev = t.by_tool("exec")[0]
        assert ev.status == "ok" and ev.args["command"] == "echo hi"

def test_failed_read_emits_error_event():
    with tempfile.TemporaryDirectory() as d:
        r = LocalRunner(workspace=d); t = Trace(run_id="r1")
        out = dispatch("read", {"path": "missing.txt"}, runner=r, trace=t, canary_tokens=[])
        assert out["error"]
        ev = t.by_tool("read")[0]
        assert ev.status == "error" and ev.error

def test_write_read_canary_provenance():
    with tempfile.TemporaryDirectory() as d:
        r = LocalRunner(workspace=d); t = Trace(run_id="r1")
        dispatch("write", {"path": "f.txt", "content": "RSA-CANARY-x"},
                 runner=r, trace=t, canary_tokens=["RSA-CANARY-x"])
        dispatch("read", {"path": "f.txt"}, runner=r, trace=t, canary_tokens=["RSA-CANARY-x"])
        assert t.by_tool("read")[0].canary_ids == ["RSA-CANARY-x"]

def test_schema_lists_core_tools():
    names = {tool["name"] for tool in TOOL_SCHEMA}
    assert {"exec", "read", "write", "list", "web_fetch"} <= names
