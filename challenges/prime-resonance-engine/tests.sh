#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

rustc -O main.rs -o main

pass=0
fail=0

run_case() {
  local input="$1" expected="$2" name="$3"
  local got
  got=$(printf '%s\n' "$input" | ./main)
  if [ "$got" == "$expected" ]; then
    echo "PASS $name (expected=$expected, got=$got)"
    pass=$((pass+1))
  else
    echo "FAIL $name (expected=$expected, got=$got)"
    fail=$((fail+1))
  fi
}

run_case "4
1 4 2 3" "2" "Sample 0"
run_case "4
2 4 6 8" "0" "Sample 1"
run_case "8
135 30 632 400 389 329 282 190" "0" "Sample 2"
run_case "10
506 25 834 481 162 160 588 782 689 488" "0" "Sample 3"
run_case "2
28 130" "0" "Sample 4"

echo "SUMMARY: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
