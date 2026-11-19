#!/usr/bin/env bash
# Differential test: compiles the same layouts (cases.rs, probe.rs) against bondrewd 0.3
# (compat/old, from the pumpkin-patch registry) and this checkout (compat/new), then
# compares their output line by line.
#
# Usage: compat/run.sh [--offline]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="$here/out"
mkdir -p "$out"

for v in old new; do
    (cd "$here/$v" && cargo build --quiet "$@" --bins)
    "$here/$v/target/debug/compat-$v" > "$out/$v.txt"
    "$here/$v/target/debug/probe" > "$out/probe-$v.txt"
done

if diff -q "$out/probe-old.txt" "$out/probe-new.txt" > /dev/null; then
    echo "probe (block arrays): identical"
else
    echo "probe (block arrays): DIFFERENT (see $out/probe-*.txt)"
fi

for t in $(awk '{print $1}' "$out/old.txt" | uniq); do
    old_lines=$(grep "^$t " "$out/old.txt")
    new_lines=$(grep "^$t " "$out/new.txt")
    differ=$(diff <(echo "$old_lines") <(echo "$new_lines") | grep -c '^<' || true)
    old_rt=$(echo "$old_lines" | grep -c 'rt=false' || true)
    new_rt=$(echo "$new_lines" | grep -c 'rt=false' || true)
    if [ "$differ" -eq 0 ]; then
        status="identical"
    elif [ "$old_rt" -gt 0 ]; then
        status="differs; 0.3 fails its own round-trip ($old_rt values)"
    else
        status="DIFFERS (both round-trip: a real wire change)"
    fi
    [ "$new_rt" -gt 0 ] && status="$status; NEW FAILS ROUND-TRIP ($new_rt values)"
    printf '%-20s %s\n' "$t" "$status"
done
