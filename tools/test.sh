#!/usr/bin/env bash
# -------------------------------------------------------
# Integration test runner for automata-mini-compiler
# Builds the release binary, runs each test case, and
# compares MIPS output via the MARS simulator.
# -------------------------------------------------------

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BINARY="$REPO_ROOT/target/release/compiler"
TEST_IN="$REPO_ROOT/test/in"
TEST_OUT="$REPO_ROOT/test/out"
MARS_JAR="$REPO_ROOT/tools/Mars.jar"

# Build first
echo "==> Building compiler..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

pass=0
fail=0

for src_file in "$TEST_IN"/*.txt; do
    name="$(basename "$src_file")"
    expected="$TEST_OUT/$name"

    # Run compiler (with optimization flag)
    "$BINARY" "$src_file" opt 2>/dev/null

    # Simulate with MARS
    actual="$(java -jar "$MARS_JAR" nc mips_out.asm 2>/dev/null || true)"
    expected_content="$(cat "$expected" 2>/dev/null || echo '')"

    if [ "$actual" = "$expected_content" ]; then
        echo "  [PASS] $name"
        ((pass++))
    else
        echo "  [FAIL] $name"
        ((fail++))
    fi
done

echo ""
echo "Results: $pass passed, $fail failed."
