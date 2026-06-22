#!/bin/bash
# Setup Ollama for benchmark
# Usage: bash setup_ollama.sh

set -e

echo "=== Ollama Setup Helper ==="
echo ""

# Step 1: Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo "❌ Ollama not found. Installing..."
    brew install ollama
    echo "✓ Ollama installed"
else
    echo "✓ Ollama already installed"
fi

# Step 2: Check if Ollama is running
echo ""
echo "Checking if Ollama is running at http://127.0.0.1:11434..."
if curl -s http://127.0.0.1:11434/api/tags > /dev/null 2>&1; then
    echo "✓ Ollama server is running"
else
    echo "❌ Ollama server is NOT running"
    echo ""
    echo "Start Ollama in a separate terminal with:"
    echo "  ollama serve"
    echo ""
    echo "Then return here and run this script again."
    exit 1
fi

# Step 3: Check if model is available
echo ""
echo "Checking if model llama3.1:latest is available..."
if ollama list 2>/dev/null | grep -q "llama3.1"; then
    echo "✓ Model llama3.1:latest is already available"
else
    echo "⏳ Pulling model llama3.1:latest (this may take a few minutes)..."
    ollama pull llama3.1:latest
    echo "✓ Model pulled successfully"
fi

# Step 4: Verify model can respond
echo ""
echo "Verifying model can respond to requests..."
if curl -s -X POST http://127.0.0.1:11434/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "llama3.1:latest",
    "messages": [{"role": "user", "content": "Hi"}],
    "max_tokens": 10
  }' | jq -e '.choices[0].message.content' > /dev/null 2>&1; then
    echo "✓ Model responds correctly"
else
    echo "❌ Model did not respond. Check Ollama logs."
    exit 1
fi

echo ""
echo "=== ✓ Setup Complete ==="
echo ""
echo "You can now run benchmarks with:"
echo ""
echo "cd morale-rust"
echo "PGDRSA_LLM_API_KEY='ollama' \\"
echo "PGDRSA_LLM_BASE_URL='http://127.0.0.1:11434/v1' \\"
echo "PGDRSA_LLM_MODEL='llama3.1:latest' \\"
echo "PYTHONPATH=../pgdrsa-python \\"
echo "/usr/bin/python3 -m pgdrsa.run_all --limit 1 --results-file /tmp/pgdrsa_benchmark.jsonl"
echo ""
