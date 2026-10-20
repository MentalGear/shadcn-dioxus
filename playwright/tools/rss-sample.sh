#!/usr/bin/env bash
# Samples the resident memory of the Playwright run every 10 s (backlog row 152a): total RSS of every chrome process,
# of the node worker processes, and MemAvailable. One line per sample; run it beside a suite run, kill it after.
#   playwright/tools/rss-sample.sh /abs/out.tsv &
out="${1:?out file}"
echo -e "epoch\tchrome_mb\tchrome_procs\tnode_mb\tmem_available_mb" > "$out"
while sleep 10; do
  c=$(ps -eo rss,comm | awk '$2 ~ /chrom/ {s+=$1; n++} END {print int(s/1024) "\t" n+0}')
  n=$(ps -eo rss,comm | awk '$2 == "node" {s+=$1} END {print int(s/1024)}')
  a=$(awk '/MemAvailable/ {print int($2/1024)}' /proc/meminfo)
  echo -e "$(date +%s)\t$c\t$n\t$a" >> "$out"
done
