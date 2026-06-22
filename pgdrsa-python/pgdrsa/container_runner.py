from __future__ import annotations
import base64, subprocess
from typing import Any
from .config import CONTAINER_CLI

class ContainerRunner:
    """Runner that executes inside a started podman container. Workspace is /workspace.
    File ops avoid shell interpolation: paths/content are passed via argv or stdin."""
    def __init__(self, container: str, cli: str = CONTAINER_CLI):
        self.container = container
        self.cli = cli

    def _run(self, *cmd: str, input_text: str | None = None, timeout: int = 30):
        return subprocess.run([self.cli, "exec", "-i", self.container, *cmd],
                              input=input_text, capture_output=True, text=True, timeout=timeout)

    def exec(self, command: str, workdir: str = ".") -> dict[str, Any]:
        # command is the skill's own invocation; it runs inside the container only.
        cp = self._run("bash", "-lc", f"cd /workspace && cd {self._q(workdir)} && {command}")
        return {"exit": cp.returncode, "stdout": cp.stdout, "stderr": cp.stderr}

    def read(self, path: str) -> str:
        cp = self._run("cat", self._cpath(path))
        if cp.returncode != 0:
            raise FileNotFoundError(cp.stderr.strip() or path)
        return cp.stdout

    def write(self, path: str, content: str) -> dict[str, Any]:
        b64 = base64.b64encode(content.encode()).decode()
        self._run("bash", "-lc",
                  f"mkdir -p \"$(dirname {self._cpath(path)})\" && "
                  f"echo {b64} | base64 -d > {self._cpath(path)}")
        return {"path": path, "bytes": len(content)}

    def list(self, path: str) -> list[str]:
        cp = self._run("ls", "-1", self._cpath(path))
        return [ln for ln in cp.stdout.splitlines() if ln]

    @staticmethod
    def _q(rel: str) -> str:
        import shlex; return shlex.quote(rel)

    def _cpath(self, rel: str) -> str:
        import shlex
        if rel.startswith("/") or ".." in rel.split("/"):
            raise ValueError(f"unsafe container path: {rel}")
        return shlex.quote(f"/workspace/{rel}")
