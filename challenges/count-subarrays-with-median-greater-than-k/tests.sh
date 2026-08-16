#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

rustc -O main.rs -o main

pass=0
fail=0

run_case() {
  local name="$1" input="$2" expected="$3"
  local actual
  actual=$(printf '%s\n' "$input" | ./main)
  if [[ "$actual" == "$expected" ]]; then
    echo "PASS: $name (got $actual)"
    pass=$((pass + 1))
  else
    echo "FAIL: $name (expected $expected, got $actual)"
    fail=$((fail + 1))
  fi
}

run_case "Example 1" "5
3 2 1 4 5
2" "10"

run_case "Example 2" "4
1 1 1 1
0" "10"

run_case "Sample Input 0" "1
5
3" "1"

echo "---"
echo "$pass passed, $fail failed"
[[ $fail -eq 0 ]]
