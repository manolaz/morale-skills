"""TCP forwarder that runs inside a container at 172.17.0.1 to route skill-hardcoded
literal-IP traffic to the Mac-side receiver.

Why this exists: dataset skills hardcode `http://172.17.0.1:9999`. On macOS podman machine,
`curl 172.17.0.1` treats it as a literal IP (so /etc/hosts overrides are bypassed) and the
address isn't otherwise routed to the host. The fix: a podman bridge network whose subnet
contains 172.17.0.1, with this forwarder bound to 0.0.0.0:9999 at --ip 172.17.0.1. It
connects inbound traffic to host.containers.internal:<mac_receiver_port>, which IS routed
to the Mac (verified on podman 5.3.0 / pasta / applehv).

Usage: see pgdrsa/smoke.py. The forwarder is started as a sidecar container; its argv is
`python3 -c <THIS_MODULE_AS_STRING>` so it needs no extra files inside the sandbox image.
"""
from __future__ import annotations
import socket, threading, sys

def pipe(a: socket.socket, b: socket.socket) -> None:
    try:
        while True:
            d = a.recv(4096)
            if not d:
                break
            b.sendall(d)
    except Exception:
        pass
    finally:
        for s in (a, b):
            try: s.shutdown(socket.SHUT_RDWR)
            except Exception: pass

def handle(c: socket.socket, target_host: str, target_port: int) -> None:
    try:
        u = socket.create_connection((target_host, target_port), timeout=5)
        threading.Thread(target=pipe, args=(c, u), daemon=True).start()
        threading.Thread(target=pipe, args=(u, c), daemon=True).start()
    except Exception as e:
        print(f"forwarder connect error: {e}", file=sys.stderr, flush=True)
        try: c.close()
        except Exception: pass

def main(listen_port: int, target_host: str, target_port: int) -> None:
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", listen_port))
    s.listen(50)
    print(f"forwarder listening 0.0.0.0:{listen_port} -> {target_host}:{target_port}", flush=True)
    while True:
        c, _ = s.accept()
        threading.Thread(target=handle, args=(c, target_host, target_port), daemon=True).start()

if __name__ == "__main__":
    # argv: listen_port target_host target_port
    main(int(sys.argv[1]), sys.argv[2], int(sys.argv[3]))
