#!/bin/bash
# Second pass for the two directly comparable surfaces (#563): clean build and
# clean check, revisions alternated sample by sample.
source ~/.cargo/env
export WEBDRIVER_PATH=/usr/bin/true BROWSER_PATH=/usr/bin/true
B=~/osl-bench/b563; OUT=$B/timing-interleaved.log; : > $OUT
now() { python3 -c "import time; print(f\"{time.time():.3f}\")"; }
for surface in "default:" "all-features:--all-features"; do
  name="${surface%%:*}"; flags="${surface#*:}"
  for kind in build check; do
    for i in 1 2 3; do
      for rev in wt-0213 wt-main; do
        cd $B/$rev; cargo clean --quiet
        s=$(now); cargo $kind -p optionstratlib $flags --quiet; rc=$?; e=$(now)
        echo "$(date -u +%FT%TZ) clean-$kind rev=$rev surface=$name sample=$((i+3)) rc=$rc seconds=$(python3 -c "print(f\"{$e-$s:.1f}\")") loadavg=[$(cut -d" " -f1 /proc/loadavg)]" | tee -a $OUT
      done
    done
  done
done
echo "$(date -u +%FT%TZ) DONE" >> $OUT
