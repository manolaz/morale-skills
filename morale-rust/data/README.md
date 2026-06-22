# RSA Skill Corpus

This directory contains the 100-skill evaluation corpus used by Runtime Skill Audit.

- `skills/benign`: 50 benign skills.
- `skills/malicious`: 50 malicious skills.

The malicious split is intended only for controlled security research. Do not install or run these skills outside an isolated sandbox.

## How the corpus flows through the system

The data path is:

1. A skill directory is picked from `data/skills/benign/*` or `data/skills/malicious/*`.
2. `pgdrsa/skills.py` reads `SKILL.md`, extracts front matter, and collects bundled `.sh` and `.py` entry points.
3. `pgdrsa/smoke.py` loads the skill, seeds canary files, starts the receiver, and launches the containerized skill runtime through Podman.
4. `pgdrsa/agent.py` runs the LLM tool loop and dispatches tool calls into the runner.
5. `pgdrsa/trace.py`, `pgdrsa/barrier.py`, and `pgdrsa/judge.py` finalize the trace and decide the verdict.
6. `pgdrsa/run_all.py` repeats that smoke flow over many skills and writes JSONL results.

For a local, no-container join test, `pgdrsa/pipeline.py` wires the same agent loop into a local runner and receiver. That path is what the unit test `pgdrsa/tests/test_pipeline_local_e2e.py` exercises.

## One skill end-to-end

If you want to run exactly one skill from start to finish, this is the full path:

1. Choose a skill directory, for example `data/skills/benign/076-memory-note`.
2. `pgdrsa/skills.py` loads `SKILL.md` and collects the bundled scripts.
3. `pgdrsa/smoke.py` starts Podman, the forwarder, and the receiver.
4. The selected skill is copied into the container and the agent loop begins.
5. `pgdrsa/agent.py` asks the model what tool to use next.
6. The runner executes the tool call inside the sandbox.
7. `pgdrsa/trace.py` records the actions, and `pgdrsa/barrier.py` waits for any in-flight sink activity to settle.
8. `pgdrsa/judge.py` compares the trace and receiver output and returns the verdict.
9. `pgdrsa/run_all.py` or `pgdrsa.smoke` writes the final JSON record.

Use this exact command to run one skill through the full benchmark flow:

```bash
cd /Users/kkkha/Documents/final_submition_ai_safety/morale-hybrid-auditor/morale-rust
PGDRSA_LLM_API_KEY='ollama' \
PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1' \
PGDRSA_LLM_MODEL='llama3.1:8b-instruct' \
PYTHONPATH=../pgdrsa-python \
/usr/bin/python3 -m pgdrsa.run_all --limit 1 --results-file /tmp/pgdrsa_one_skill.jsonl
```

The output file contains one JSON object for that skill, including the verdict, elapsed time, receiver hits, and any error message.

## What is required and what is missing

The core benchmark code path is present:

- skill discovery and parsing
- local pipeline join test
- container smoke benchmark
- full-corpus runner
- random-sample benchmark helper

There is no obvious missing execution code in the current flow. The only documentation gap I found is that `pgdrsa-python/pgdrsa/README.md` mentions `.env.example`, but this repository does not currently contain that file. In practice, you can export the env vars directly in your shell or create your own `.env` file.

## Normal run: audit one skill

Use this when you want to audit a single skill directory.

From `morale-rust/`:

```bash
cd /Users/kkkha/Documents/final_submition_ai_safety/morale-hybrid-auditor/morale-rust
cargo build --release
./target/release/morale data/skills/benign/076-memory-note
```

Common options in `morale-rust/src/main.rs`:

- `--json` prints structured output.
- `--summary` prints a summary only.
- `--lightweight` skips database operations.
- `--enable_ai` enables AI-based analysis for the Rust auditor.

Example in lightweight mode:

```bash
./target/release/morale --lightweight data/skills/malicious/001-network-ip-info
```

## Test run: local code checks

The fastest code checks are the Python tests in `pgdrsa-python`.

From the repo root:

```bash
cd /Users/kkkha/Documents/final_submition_ai_safety/morale-hybrid-auditor
/usr/bin/python3 -m pytest pgdrsa-python/pgdrsa/tests -v
```

The most important end-to-end local test is:

```bash
/usr/bin/python3 -m pytest pgdrsa-python/pgdrsa/tests/test_pipeline_local_e2e.py -v
```

That test confirms the join between the agent loop, runner, receiver, and verdict logic without requiring a container or LLM API spend.

## Benchmark run: one skill smoke test

The benchmark path uses Podman plus an OpenAI-compatible LLM endpoint.

Prerequisites:

- Podman installed and `podman machine` started on macOS.
- A local Llama server with an OpenAI-compatible `/v1/chat/completions` API, for example Ollama.
- `PGDRSA_LLM_BASE_URL` and `PGDRSA_LLM_MODEL` set for your local endpoint.

### Setup local Ollama server

If you use Ollama, install and start it:

```bash
# Install Ollama (macOS)
brew install ollama
ollama serve
```

In another terminal, pull a model:

```bash
ollama pull llama3.1:latest
```

Verify Ollama is running at `http://127.0.0.1:11434/v1/chat/completions`.

### Run benchmark

Steps:

1. Start Podman on macOS if it is not already running.
2. Start your local Llama server (for example Ollama).
3. Export the LLM environment variables in the shell you will use for the run.
4. Run the smoke command from `morale-rust/`.
5. Read the JSONL output or console summary for verdicts and errors.

Run one smoke benchmark from `morale-rust/`:

```bash
cd /Users/kkkha/Documents/final_submition_ai_safety/morale-hybrid-auditor/morale-rust
PGDRSA_LLM_API_KEY='ollama' \
PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1' \
PGDRSA_LLM_MODEL='llama3.1:latest' \
PYTHONPATH=../pgdrsa-python \
/usr/bin/python3 -m pgdrsa.run_all --limit 1 --results-file /tmp/pgdrsa_benchmark_preview.jsonl
```

What this does:

- enumerates skills from `data/skills`
- runs `pgdrsa.smoke.run_smoke()` for each selected skill
- writes one JSONL record per skill
- prints verdict, receiver hits, and elapsed time

If you want to benchmark a different local model, keep the same command and only change `PGDRSA_LLM_MODEL` plus the matching local endpoint.

## Benchmark run: random 10-skill sample

I added a helper script for your requested random sample flow:

`pgdrsa-python/scripts/run_random_benchmark.py`

Run it from the repo root:

```bash
cd /Users/kkkha/Documents/final_submition_ai_safety/morale-hybrid-auditor
PGDRSA_LLM_API_KEY='ollama' \
PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1' \
PGDRSA_LLM_MODEL='llama3.1:latest' \
/usr/bin/python3 pgdrsa-python/scripts/run_random_benchmark.py --count 10 --results-file /tmp/pgdrsa_random_10.jsonl
```

Optional flags:

- `--seed N` makes the random sample reproducible.
- `--image IMG` overrides the Podman image tag.
- `--results-file FILE` changes the JSONL output path.

Steps:

1. Make sure Podman is installed and the machine is started.
2. Start your local Llama server and set the endpoint for the model you want to test.
3. Run the random benchmark helper from the repo root.
4. Inspect the JSONL file written to `--results-file`.

## Full-corpus benchmark

To run all 100 skills, omit `--limit` and let `pgdrsa.run_all` walk the entire corpus:

Steps:

1. Ensure the benchmark image exists or let Podman use the configured tag.
2. Export the LLM environment variables for the local Llama model you want to benchmark.
3. Run `pgdrsa.run_all` from `morale-rust/` without `--limit`.
4. Wait for all 100 skills to finish and review the JSONL result file.

```bash
cd /Users/kkkha/Documents/final_submition_ai_safety/morale-hybrid-auditor/morale-rust
PGDRSA_LLM_API_KEY='ollama' \
PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1' \
PGDRSA_LLM_MODEL='llama3.1:latest' \
PYTHONPATH=../pgdrsa-python \
/usr/bin/python3 -m pgdrsa.run_all --results-file /tmp/pgdrsa_full_run.jsonl
```

This is the correct command when you want to benchmark the entire corpus, not just one skill or one random sample.

## Notes on the runtime layout

- The Rust auditor in `morale-rust/` is the normal single-skill audit entry point.
- The Python harness in `pgdrsa-python/` is the benchmark and smoke-test path.
- `pgdrsa/pipeline.py` is the local no-container join test.
- `pgdrsa/smoke.py` is the real benchmark runner used for corpus skills.
- `pgdrsa/run_all.py` is the batch driver.

## Troubleshooting

**Error: "model 'llama3.1:latest' not found"**

This means the Ollama server is running, but the model has not been pulled.

Solution: Pull the model in a separate terminal:

```bash
ollama pull llama3.1:latest
```

Then run the benchmark command again.

**Error: "Connection refused" or timeout connecting to 127.0.0.1:11434**

This means Ollama is not running.

Solution:

1. Start Ollama in a terminal: `ollama serve`
2. Verify it is listening: `curl http://127.0.0.1:11434/api/tags`
3. Pull the model: `ollama pull llama3.1:8b-instruct`
4. Return to the benchmark terminal and run the command again.

**Error: benchmark takes ~21-22 seconds per skill then fails with verdict=ERR**

This indicates a consistent timeout. Common causes:

- Ollama is not running or the endpoint is wrong (check `PGDRSA_LLM_BASE_URL`).
- The model `PGDRSA_LLM_MODEL` (default `llama3.1:latest`) is not available (run `ollama pull llama3.1:latest`).
- The network between your machine and Ollama is blocked (check firewall).
- The Ollama machine is out of memory (try a smaller model or increase memory).

Solution: Debug step by step:

1. Verify Ollama is running: `curl http://127.0.0.1:11434/api/tags`
2. Verify the model is available: `ollama list | grep llama3.1`
3. Try a simple manual LLM call:

```bash
curl -X POST http://127.0.0.1:11434/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "llama3.1:latest",
    "messages": [{"role": "user", "content": "Hi"}],
    "max_tokens": 10
  }' | jq .
```

If the curl succeeds, Ollama is working. If it fails, fix Ollama before running the benchmark.

4. If Ollama is working, check that the LLM environment variables match in your shell.

## Safety note

The malicious corpus is only for isolated analysis. Do not execute these skills on a real workstation, with real credentials, or outside the containerized benchmark harness.
