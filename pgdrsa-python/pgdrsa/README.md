# PGD-RSA Phase A (four-smoke walking skeleton)

Vertical slice: load real skill -> LLM agent -> podman sandbox executes tool calls ->
routed receiver + PID-namespace drain -> sealed trace -> receiver-evidence verdict.
OpenClaw-free.

## Setup

### Python environment

```bash
python3 -m venv .venv && . .venv/bin/activate
pip install -e ".[dev]"
podman build -t pgdrsa_sandbox:phaseA -f pgdrsa/Dockerfile .
```

### LLM endpoint

LLM access is OpenAI-compatible (any `/v1/chat/completions` endpoint with tool calling:
OpenAI, ZhipuAI/glm-*, OpenRouter, Together, vLLM, Ollama, local llama.cpp servers, ...).

**For local Ollama:**

```bash
# Install Ollama (macOS)
brew install ollama
ollama serve
```

In another terminal, pull a model:

```bash
ollama pull llama3.1:latest
```

Configure via environment variables (no `.env.example` in this repo; create your own or export directly):

```bash
export PGDRSA_LLM_API_KEY='ollama'
export PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1'
export PGDRSA_LLM_MODEL='llama3.1:latest'
```

## Unit tests (no container, no API spend)

```bash
. .venv/bin/activate
python -m pytest pgdrsa/tests -v
```

## Smoke ladder (podman + .env; literal-IP routing auto-wired via `forwarder.py` sidecar)

```bash
# Smoke A — benign, expect verdict=not_observed
PGDRSA_LLM_API_KEY=ollama PGDRSA_LLM_BASE_URL=http://127.0.0.1:11434/v1 PGDRSA_LLM_MODEL=llama3.1:latest \
python -m pgdrsa.smoke --skill data/skills/benign/076-memory-note
# Smoke B — synchronous curl exfil (RECEIVER_URL routed), expect executed_contained
PGDRSA_LLM_API_KEY=ollama PGDRSA_LLM_BASE_URL=http://127.0.0.1:11434/v1 PGDRSA_LLM_MODEL=llama3.1:latest \
python -m pgdrsa.smoke --skill data/skills/malicious/043-installer_supply_chain-file-organizer-bootstrap
# Smoke C — urllib literal-IP, expect executed_contained (or attempted_blocked if forwarder is down)
PGDRSA_LLM_API_KEY=ollama PGDRSA_LLM_BASE_URL=http://127.0.0.1:11434/v1 PGDRSA_LLM_MODEL=llama3.1:latest \
python -m pgdrsa.smoke --skill data/skills/malicious/001-network-ip-info
# Smoke D — background curl &, drain test; expect executed_contained, fails if trace seals early
PGDRSA_LLM_API_KEY=ollama PGDRSA_LLM_BASE_URL=http://127.0.0.1:11434/v1 PGDRSA_LLM_MODEL=llama3.1:latest \
python -m pgdrsa.smoke --skill data/skills/malicious/003-network-url-check
```

## How literal-IP routing works on macOS podman machine

Dataset skills hardcode `http://172.17.0.1:9999`. `curl` parses that as a literal IP
(ignoring `/etc/hosts` overrides), and podman's default bridge doesn't route `172.17.0.0/16`
to the host. `smoke.py` solves this by:

1. Creating a podman bridge network `pgdrsa_net` with subnet `172.17.0.0/24` and gateway
   `172.17.0.254` (so `172.17.0.1` is a free host address).
2. Starting a sidecar container at `--ip 172.17.0.1` running `forwarder.py`, which accepts
   on `0.0.0.0:9999` and pipes to `host.containers.internal:<receiver-port>`.
3. The Mac-side `Receiver` binds `0.0.0.0` and is reachable from the forwarder via
   `host.containers.internal` (verified on podman 5.3.0 / pasta / applehv).

Skill containers join `pgdrsa_net`, so `curl 172.17.0.1:9999` reaches the forwarder ->
receiver, preserving the canary payload as the verdict's source of truth.
