#!/bin/bash
# Resolved package name/version pairs per profile (#563); writes the lists too.
set -u
WT="$1"; OUTDIR="$2"; PROFILES="$3"
cd "$WT" || exit 1
mkdir -p "$OUTDIR"
while IFS='|' read -r name flags touch1 touch2; do
  [ -z "$name" ] && continue
  case "$name" in \#*) continue;; esac
  cargo tree -q $flags -e normal --prefix none | sed 's/ (\*)$//' | sort -u > "$OUTDIR/$name.pairs"
  awk '{print $1}' "$OUTDIR/$name.pairs" | sort -u > "$OUTDIR/$name.names"
  echo "$name pairs=$(wc -l < "$OUTDIR/$name.pairs") names=$(wc -l < "$OUTDIR/$name.names")"
done < "$PROFILES"
