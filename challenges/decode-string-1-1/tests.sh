#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "Compiling with rustc -O main.rs -o main ..."
rustc -O main.rs -o main

pass=0
fail=0

run_test() {
  local name="$1" input="$2" expected="$3"
  local got
  got=$(printf '%s' "$input" | ./main)
  if [[ "$got" == "$expected" ]]; then
    echo "PASS: $name"
    pass=$((pass + 1))
  else
    echo "FAIL: $name"
    echo "  input:    $input"
    echo "  expected: $expected"
    echo "  got:      $got"
    fail=$((fail + 1))
  fi
}

run_test "Example 1 (3[a]2[bc])" "3[a]2[bc]" "aaabcbc"
run_test "Example 2 (3[a2[c]])" "3[a2[c]]" "accaccacc"
run_test "Example 3 (2[abc]3[cd]ef)" "2[abc]3[cd]ef" "abcabccdcdcdef"
run_test "Sample Input 0 (3[a2[c]])" "3[a2[c]]" "accaccacc"

if [[ $fail -gt 0 ]]; then
  echo "$fail test(s) failed."
  exit 1
fi
echo "All $pass test(s) passed."
exit 0
