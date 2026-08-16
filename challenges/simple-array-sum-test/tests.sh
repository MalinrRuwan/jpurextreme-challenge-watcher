#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

rustc -O main.rs -o main

run_sample() {
    local input="$1"
    local expected="$2"
    local label="$3"
    local actual
    actual=$(printf '%s\n' "$input" | ./main)
    if [ "$actual" = "$expected" ]; then
        echo "PASS: $label"
    else
        echo "FAIL: $label (expected '$expected', got '$actual')"
        return 1
    fi
}

pass=0
fail=0

if run_sample "6
1 2 3 4 10 11" "31" "Sample Input 0"; then
    pass=$((pass + 1))
else
    fail=$((fail + 1))
fi

echo "Passed: $pass, Failed: $fail"
[ "$fail" -eq 0 ]
