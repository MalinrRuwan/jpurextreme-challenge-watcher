#!/usr/bin/env bash
set -u

cd "$(dirname "$0")"

echo "== Compiling =="
if ! rustc -O main.rs -o main; then
  echo "COMPILE FAILED"
  exit 1
fi
echo "compiled ok"
echo

FAILED=0

for in in sample*.in; do
  name="${in%.in}"
  out="${name}.out"
  if [ ! -f "$out" ]; then
    echo "$name: SKIP (missing $out)"
    continue
  fi
  ./main < "$in" > "${name}.actual"
  if diff -q "${name}.actual" "$out" > /dev/null 2>&1; then
    echo "$name: PASS"
  else
    echo "$name: FAIL"
    echo "--- expected ---"
    cat "$out"
    echo "--- actual ---"
    cat "${name}.actual"
    FAILED=1
  fi
done

echo
if [ "$FAILED" -eq 0 ]; then
  echo "ALL SAMPLES PASSED"
  exit 0
else
  echo "SOME SAMPLES FAILED"
  exit 1
fi
