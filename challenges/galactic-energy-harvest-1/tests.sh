#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

rustc -O main.rs -o main

pass=0
fail=0

run_sample() {
  local name="$1" input="$2" expected="$3"
  local actual
  actual=$(printf '%s\n' "$input" | ./main)
  if [[ "$actual" == "$expected" ]]; then
    echo "PASS: $name"
    pass=$((pass + 1))
  else
    echo "FAIL: $name (expected '$expected', got '$actual')"
    fail=$((fail + 1))
  fi
}

run_sample "Sample 0" $'7 3 9\n3 2 4 5 1 1 2' "0 2"
run_sample "Sample 1" $'5 2 100\n10 20 30 40 50' "-1"
run_sample "Sample 2" $'6 3 15\n1 2 3 4 5 6' "3 5"
run_sample "Sample 3" $'4 2 5\n1 5 2 3' "1 1"

echo
echo "Summary: $pass passed, $fail failed"
[[ $fail -eq 0 ]]
