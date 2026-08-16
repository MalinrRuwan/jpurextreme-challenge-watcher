#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "Compiling with rustc -O main.rs -o main ..."
rustc -O main.rs -o main
echo "Compilation OK."
echo

pass=0
fail=0

run_case() {
    local name="$1" input="$2" expected="$3"
    local out
    out=$(printf '%s' "$input" | ./main)
    if [ "$out" = "$expected" ]; then
        echo "PASS: $name (got $out)"
        pass=$((pass + 1))
    else
        echo "FAIL: $name (expected $expected, got $out)"
        fail=$((fail + 1))
    fi
}

run_case "Sample 0" "1
5
3" "1"

run_case "Sample 1 (Example 1)" "5
3 2 1 4 5
2" "10"

run_case "Sample 2 (Example 2)" "4
1 1 1 1
0" "10"

echo
echo "--------------------------------"
echo "Passed: $pass, Failed: $fail"

if [ "$fail" -eq 0 ]; then
    exit 0
else
    exit 1
fi
