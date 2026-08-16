#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

rustc -O main.rs -o main

pass=0
fail=0

run_test() {
    local name="$1"
    local input="$2"
    local expected="$3"
    local actual
    actual=$(printf '%s' "$input" | ./main)
    if [ "$actual" = "$expected" ]; then
        echo "PASS: $name"
        pass=$((pass + 1))
    else
        echo "FAIL: $name (expected '$expected', got '$actual')"
        fail=$((fail + 1))
    fi
}

run_test "Example 1 (n=5, array=3 2 1 4 5, k=2)" \
"5
3 2 1 4 5
2" \
"10"

run_test "Example 2 (n=4, all ones, k=0)" \
"4
1 1 1 1
0" \
"10"

run_test "Sample 0 (n=1, array=5, k=3)" \
"1
5
3" \
"1"

echo "-----------------------------------"
echo "Total: $((pass + fail)), Passed: $pass, Failed: $fail"
[ "$fail" -eq 0 ]
