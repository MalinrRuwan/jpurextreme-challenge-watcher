#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "Compiling main.rs..."
rustc -O main.rs -o main

pass=0
fail=0

run_test() {
  local name="$1"
  local input="$2"
  local expected="$3"
  local out
  out=$(printf '%s' "$input" | ./main)
  if [ "$out" == "$expected" ]; then
    echo "PASS: $name (got $out)"
    pass=$((pass + 1))
  else
    echo "FAIL: $name (expected $expected, got $out)"
    fail=$((fail + 1))
  fi
}

run_test "Sample 0" \
"5 1
0 1
1 2
1 3
3 4
-2 4 2 -4 6
" \
"8"

run_test "Example 2" \
"2 0
0 1
-7280 2350
" \
"-4930"

echo "=============================="
echo "Passed: $pass, Failed: $fail"
[ "$fail" -eq 0 ]
