#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "Compiling main.rs..."
rustc -O main.rs -o main

pass=0
fail=0

run_test() {
    local name="$1"
    local expected="$2"
    local input="$3"
    local actual
    actual=$(printf '%s\n' "$input" | ./main)
    if [ "$actual" = "$expected" ]; then
        echo "PASS: $name (expected '$expected')"
        pass=$((pass + 1))
    else
        echo "FAIL: $name (expected '$expected', got '$actual')"
        fail=$((fail + 1))
    fi
}

run_test "Sample 0" "7" "4 5
R0010
01010
01010
0000B"

run_test "Sample 1" "4" "5 5
00000
00B00
00000
00000
0R000"

run_test "Sample 2" "5" "10 7
0000B00
0010100
0100100
11R1000
0000100
0001000
1001000
0010101
0001001
0000000"

run_test "Extra: TRAPPED" "TRAPPED" "3 3
1B1
111
R10"

echo ""
echo "Results: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
