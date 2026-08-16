#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

rustc -O main.rs -o main

run_case() {
  local name="$1" input="$2" expected="$3"
  local got
  got=$(printf '%s\n' "$input" | ./main)
  if [ "$got" = "$expected" ]; then
    echo "$name: PASS"
  else
    echo "$name: FAIL (expected '$expected', got '$got')"
    exit 1
  fi
}

run_case "Sample 0" "7
0 1
0 2
1 4
1 5
2 3
2 6
0 0 1 0 1 1 0" "6"

run_case "Sample 1" "7
0 1
0 2
1 4
1 5
2 3
2 6
0 1 0 0 1 0 0" "2"

run_case "Sample 2" "7
0 1
0 2
1 4
1 5
2 3
2 6
0 0 0 0 0 0 0" "0"

echo "All samples passed."
