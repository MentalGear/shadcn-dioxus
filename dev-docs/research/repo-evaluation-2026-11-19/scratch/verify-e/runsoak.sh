#!/bin/bash
# usage: runsoak.sh <base> <comp> <cycles> <tag> [diff]
V=/tmp/claude-0/-home-user-shadcn-dioxus/2f9af3b0-3995-5f94-a96b-6c0dfde7973a/scratchpad/verify-e
cd $V
L=$V/soak-$4-$2.log
echo "node $V/soak.js $1 $2 $3 100 ${5:-0}" > $L
node $V/soak.js $1 $2 $3 100 ${5:-0} >> $L 2>&1
