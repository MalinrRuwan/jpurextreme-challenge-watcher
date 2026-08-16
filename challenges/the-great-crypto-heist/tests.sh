#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "Compiling..."
rustc -O main.rs -o main

pass=0
fail=0

run_test() {
    local input="$1"
    local expected="$2"
    local name="$3"
    local actual
    actual=$(printf '%s\n' "$input" | ./main)
    if [ "$actual" = "$expected" ]; then
        echo "PASS: $name"
        pass=$((pass + 1))
    else
        echo "FAIL: $name (expected '$expected', got '$actual')"
        fail=$((fail + 1))
    fi
}

run_test "6
2 2 3 3 3 4" "9" "Sample 0"

run_test "5
5 5 5 5 5" "25" "Sample 1"

echo
if [ "$fail" -eq 0 ]; then
    echo "All tests passed ($pass/$pass)"
    exit 0
else
    echo "Tests failed: $fail of $((pass + fail))"
    exit 1
fi
