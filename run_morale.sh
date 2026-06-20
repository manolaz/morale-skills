#!/bin/bash

# MORALE - AI Skills Auditor
# Sample script to run the auditor

echo "MORALE - AI Skills Auditor"
echo "=========================="

if [ $# -eq 0 ]; then
    echo "Usage: $0 <path_to_ai_skill>"
    echo "Example: $0 /path/to/my/ai-skill"
    exit 1
fi

SKILL_PATH=$1

echo "Auditing AI skill at: $SKILL_PATH"
echo "Running MORALE..."

# Build the project first
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Starting audit..."
./target/release/morale "$SKILL_PATH"

echo ""
echo "Audit completed!"