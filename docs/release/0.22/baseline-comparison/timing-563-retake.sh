#!/bin/bash
# #563 baseline comparison (re-take with per-sample load guard): the M0-01 timing script (doc/BASELINE.md appendix)
# generalised to one package/feature profile per line, run on flumix.
# Usage: timing-563.sh <worktree> <log> <profile-file>
# profile-file lines: name|cargo -p and feature flags|file to touch (relative)[|second touch file]
set -u
WT="$1"; OUT="$2"; PROFILES="$3"; SAMPLES=3
cd "$WT" || exit 1
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }
secs() { python3 -c "print(f'{$2-$1:.1f}')"; }
log() { echo "$(date -u +%FT%TZ) $*" | tee -a "$OUT"; }
# Before every sample: load and the busiest processes, so a sample that
# overlaps foreign load is visible in the log (re-take after the #563 overlap).
guard() { log "guard loadavg=[$(cut -d' ' -f1-3 /proc/loadavg)] top=[$(ps -eo pcpu,comm --sort=-pcpu --no-headers | head -3 | tr -s ' ' | tr '\n' ';')]"; }
: > "$OUT"
log "host=$(hostname) nproc=$(nproc) commit=$(git rev-parse HEAD) rustc=$(rustc --version) cargo=$(cargo --version)"
log "lockfile_sha256=$(sha256sum Cargo.lock | awk '{print $1}') lockfile_packages=$(grep -c '^\[\[package\]\]' Cargo.lock)"
cargo fetch --quiet && log "cargo fetch done"
while IFS='|' read -r name flags touch1 touch2; do
  [ -z "$name" ] && continue
  case "$name" in \#*) continue;; esac
  log "profile=$name flags=[$flags] loadavg=[$(cut -d' ' -f1-3 /proc/loadavg)]"
  for i in $(seq 1 $SAMPLES); do
    cargo clean --quiet; guard
    s=$(now); cargo build $flags --quiet; rc=$?; e=$(now)
    log "clean-build profile=$name sample=$i rc=$rc seconds=$(secs $s $e)"
  done
  top=$(ls target/debug/lib*.rlib 2>/dev/null | head -1)
  own=$(find target/debug/deps -name 'liboptionstratlib*.rlib' -printf '%s\n' | awk '{s+=$1} END{print s+0}')
  log "artifact profile=$name top_rlib=$(basename "$top") top_rlib_bytes=$(stat -c %s "$top") own_rlibs_bytes=$own target_bytes=$(du -sb target | awk '{print $1}')"
  cargo check $flags --quiet
  for i in $(seq 1 $SAMPLES); do
    touch "$touch1"; guard
    s=$(now); cargo check $flags --quiet; rc=$?; e=$(now)
    log "incremental-check profile=$name touched=$touch1 sample=$i rc=$rc seconds=$(secs $s $e)"
  done
  if [ -n "${touch2:-}" ]; then
    for i in $(seq 1 $SAMPLES); do
      touch "$touch2"; guard
      s=$(now); cargo check $flags --quiet; rc=$?; e=$(now)
      log "incremental-check profile=$name touched=$touch2 sample=$i rc=$rc seconds=$(secs $s $e)"
    done
  fi
  for i in $(seq 1 $SAMPLES); do
    cargo clean --quiet; guard
    s=$(now); cargo check $flags --quiet; rc=$?; e=$(now)
    log "clean-check profile=$name sample=$i rc=$rc seconds=$(secs $s $e)"
  done
done < "$PROFILES"
log "DONE"
