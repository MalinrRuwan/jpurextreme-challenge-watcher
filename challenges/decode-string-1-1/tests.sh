#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

rustc -O main.rs -o main

pass=0
fail=0

run_test() {
    local input="$1"
    local expected="$2"
    local actual
    actual=$(printf '%s\n' "$input" | ./main)
    if [ "$actual" = "$expected" ]; then
        echo "PASS: '$input' -> '$actual'"
        pass=$((pass + 1))
    else
        echo "FAIL: '$input' -> '$actual' (expected '$expected')"
        fail=$((fail + 1))
    fi
}

run_test "3[a]2[bc]" "aaabcbc"
run_test "3[a2[c]]" "accaccacc"
run_test "2[abc]3[cd]ef" "abcabccdcdcdef"

echo
echo "Passed: $pass, Failed: $fail"
[ "$fail" -eq 0 ]
