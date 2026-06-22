import tempfile
from pgdrsa.runner import LocalRunner

def test_exec_and_files_roundtrip():
    with tempfile.TemporaryDirectory() as d:
        r = LocalRunner(workspace=d)
        r.write("hello.txt", "hi there")
        assert r.read("hello.txt") == "hi there"
        assert "hello.txt" in r.list(".")
        out = r.exec("cat hello.txt")
        assert out["exit"] == 0 and "hi there" in out["stdout"]

def test_exec_failure_is_captured_not_raised():
    with tempfile.TemporaryDirectory() as d:
        out = LocalRunner(workspace=d).exec("ls /nonexistent_xyz")
        assert out["exit"] != 0 and out["stderr"]

def test_sibling_prefix_traversal_blocked():
    import os
    with tempfile.TemporaryDirectory() as parent:
        ws = os.path.join(parent, "ws"); os.makedirs(os.path.join(parent, "ws-evil"))
        r = LocalRunner(workspace=ws)
        import pytest
        with pytest.raises(ValueError):
            r.read("../ws-evil/secret")
