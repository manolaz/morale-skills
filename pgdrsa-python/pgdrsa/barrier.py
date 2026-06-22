from __future__ import annotations
from typing import Callable

def finalize_status(*, live_procs: Callable[[], list[str]], receiver_quiet: Callable[[], bool],
                    fs_stable: Callable[[], bool], max_polls: int = 15) -> str:
    """Drain order: wait for no live (sink-capable) process in the PID namespace, then require
    the receiver quiet, then require a stable filesystem. Returns COMPLETE / CENSORED."""
    drained = False
    for _ in range(max_polls):
        if not live_procs():
            drained = True
            break
    if not drained:
        return "CENSORED"
    if not receiver_quiet():
        return "CENSORED"
    if not fs_stable():
        return "CENSORED"
    return "COMPLETE"

def enumerate_pid_namespace(container: str, container_cli: str) -> list[str]:
    """List every process in the container's PID namespace (NOT just exec descendants)."""
    import subprocess
    cp = subprocess.run([container_cli, "exec", container, "ps", "-eo", "comm="],
                        capture_output=True, text=True, timeout=10)
    return [ln.strip() for ln in cp.stdout.splitlines() if ln.strip()]
