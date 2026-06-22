from __future__ import annotations
import subprocess
from pathlib import Path
from typing import Any, Protocol

class Runner(Protocol):
    def exec(self, command: str, workdir: str = ".") -> dict[str, Any]: ...
    def read(self, path: str) -> str: ...
    def write(self, path: str, content: str) -> dict[str, Any]: ...
    def list(self, path: str) -> list[str]: ...

class LocalRunner:
    """TRUSTED-TEST-ONLY. Runs host shell. Never use for real/untrusted skills."""
    def __init__(self, workspace: str):
        self.workspace = Path(workspace).resolve()
        self.workspace.mkdir(parents=True, exist_ok=True)

    def _abs(self, path: str) -> Path:
        p = (self.workspace / path).resolve()
        if p != self.workspace and not p.is_relative_to(self.workspace):
            raise ValueError(f"path traversal blocked: {path}")
        return p

    def exec(self, command: str, workdir: str = ".") -> dict[str, Any]:
        try:
            cp = subprocess.run(command, shell=True, cwd=str(self._abs(workdir)),
                                capture_output=True, text=True, timeout=60)
            return {"exit": cp.returncode, "stdout": cp.stdout, "stderr": cp.stderr}
        except subprocess.TimeoutExpired as e:
            return {"exit": 124, "stdout": e.stdout or "", "stderr": "timeout"}

    def read(self, path: str) -> str:
        return self._abs(path).read_text(encoding="utf-8", errors="ignore")

    def write(self, path: str, content: str) -> dict[str, Any]:
        p = self._abs(path); p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")
        return {"path": path, "bytes": len(content)}

    def list(self, path: str) -> list[str]:
        return sorted(p.name for p in self._abs(path).iterdir())
