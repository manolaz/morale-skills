# Morale Hybrid Auditor

This repository contains a two-layer safety evaluation stack for AI agents:

- `morale-rust/`: a Rust-based static auditor that scans skills, scores risk, and produces reports.
- `pgdrsa-python/`: a Python-based dynamic harness that runs skills in a sandbox, records traces, and benchmarks behavior.

The main experiment and analysis documents live at the repository root:

- `EVALUATION_AND_TUNING.md`
- `TUNING_METHODOLOGY.md`
- `TUNING_RESULTS.md`
- `DOCUMENTATION_INDEX.md`

## Prerequisites

- Python 3.10+
- Rust toolchain + Cargo
- Podman
- An OpenAI-compatible LLM endpoint for the Python benchmark, or local Ollama

## Quick Layout

- `morale-rust/data/skills/`: benign and malicious skill corpus
- `morale-rust/src/`: audit logic and risk checkers
- `pgdrsa-python/pgdrsa/`: benchmark engine, sandbox, receiver, judge
- `pgdrsa-python/scripts/`: random benchmark script and sample result files

## Running the Python Benchmark

Change into the Python workspace before running the commands below:

```bash
cd pgdrsa-python
python3 -m venv .venv
. .venv/bin/activate
pip install -e ".[dev]"
```

### 1. Run a Single Skill

```bash
. .venv/bin/activate
export PGDRSA_LLM_API_KEY='ollama'
export PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1'
export PGDRSA_LLM_MODEL='llama3.1:latest'
python -m pgdrsa.smoke --skill ../morale-rust/data/skills/benign/076-memory-note
```

This runs one skill inside the sandbox, collects a trace, and prints the JSON result to the terminal.

### 2. Run the Full Corpus

```bash
. .venv/bin/activate
python -m pgdrsa.run_all --results-file pgdrsa_results.jsonl
```

Useful options:

- `--limit N`: run only the first N skills
- `--retry-errors`: rerun skills that previously failed
- `--only-bad BASELINE_JSONL`: run only skills that failed or were false negatives in the baseline
- `--image IMAGE`: override the sandbox image

### 3. Run a Random Benchmark Sample

```bash
. .venv/bin/activate
python scripts/run_random_benchmark.py --count 10 --seed 123 --results-file scripts/pgdrsa_random_results.jsonl
```

This script randomly selects a subset of skills from the corpus, runs each one, and writes one JSONL record per run. It also computes basic metrics and writes a companion `.metrics.json` file.

### 4. Inspect and Analyze Results

Analysis scripts are available at the repository root:

```bash
python analyze_tuning_results.py
python compare_samples.py
```

The sample benchmark output currently lives in `pgdrsa-python/scripts/pgdrsa_random_results.jsonl`.

## Running the Rust Auditor

```bash
cd morale-rust
cargo test
cargo run -- ../morale-rust/data/skills/benign/076-memory-note
```

The `morale` binary accepts either a skill directory or a `.zip` file, then performs the audit and prints a report based on the CLI flags.

Common flags:

- `--lightweight`: skip database operations
- `--json`: output JSON
- `--summary`: print a summary only
- `--stats`: show risk statistics
- `--enable_ai`: enable AI-powered analysis if an API key is configured

## Local Ollama

If you want to run the Python benchmark fully locally, install and start Ollama, then export the following environment variables:

```bash
brew install ollama
ollama serve
ollama pull llama3.1:latest

export PGDRSA_LLM_API_KEY='ollama'
export PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1'
export PGDRSA_LLM_MODEL='llama3.1:latest'
```

## Results and Reports

If you are looking for the experiment writeup, the recommended reading order is:

1. `DOCUMENTATION_INDEX.md`
2. `PAPER_KNOWLEDGE_BASE.md`
3. `EVALUATION_AND_TUNING.md`
4. `TUNING_RESULTS.md`

## Troubleshooting

- If you see `ModuleNotFoundError: No module named 'openai'`, make sure you installed the Python environment correctly with `pip install -e ".[dev]"` inside `pgdrsa-python/`.
- If the benchmark fails because of Podman or networking, verify that Podman is running and the sandbox image has been built.
- If a skill runs but you do not see the expected trace, try a benign skill first to confirm the end-to-end pipeline.
