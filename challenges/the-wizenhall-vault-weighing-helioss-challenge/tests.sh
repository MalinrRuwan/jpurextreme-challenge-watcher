#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

rustc -O main.rs -o main

pass=0
fail=0

run_case() {
    local input="$1" expected="$2" name="$3"
    local actual
    actual=$(printf '%s\n' "$input" | ./main)
    if [ "$actual" = "$expected" ]; then
        echo "PASS: $name (input=$input expected=$expected got=$actual)"
        pass=$((pass + 1))
    else
        echo "FAIL: $name (input=$input expected=$expected got=$actual)"
        fail=$((fail + 1))
    fi
}

run_case "4" "3" "Sample 0"
run_case "5" "3" "Sample 1"

echo "------------------------------"
echo "Passed: $pass, Failed: $fail"
if [ "$fail" -eq 0 ]; then
    echo "ALL TESTS PASSED"
    exit 0
else
    echo "SOME TESTS FAILED"
    exit 1
fi
