import tempfile
from pgdrsa.runner import LocalRunner
from pgdrsa.trace import Trace
from pgdrsa.agent import run_agent

class FakeLLM:
    def __init__(self): self.calls = 0
    def set_system(self, s): pass
    def complete(self, messages, tools):
        self.calls += 1
        if self.calls == 1:
            return {"stop_reason": "tool_use",
                    "tool_calls": [{"id": "t1", "name": "exec", "input": {"command": "echo done"}}],
                    "text": ""}
        return {"stop_reason": "end_turn", "tool_calls": [], "text": "finished"}

def test_agent_executes_tool_and_records_trace():
    with tempfile.TemporaryDirectory() as d:
        r = LocalRunner(workspace=d); t = Trace(run_id="r1")
        final = run_agent(llm=FakeLLM(), runner=r, trace=t, canary_tokens=[],
                          system="audit", user="run the skill")
        assert final == "finished"
        ev = t.by_tool("exec")[0]
        assert ev.result["exit"] == 0 and "done" in ev.result["stdout"]
