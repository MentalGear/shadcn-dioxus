# shellcheck shell=bash
# Sourced by the gates that read a component's slot variables (check-slot-docs.sh, check-css-vars-defined.sh).
# `build_slot_scan <dir>` compiles the one scanner (`scripts/slot-scan/main.rs`, which includes
# `preview/slot_scan.rs`, the parser `preview/build.rs` renders the docs page from) into `<dir>/slot-scan`
# with a bare `rustc` (std only, about a second; no cargo lock) and returns non-zero, printing the compiler's
# log, when it does not build. Backlog row 433.
build_slot_scan() {
  local dir="$1"
  rustc --edition 2021 -O -o "$dir/slot-scan" scripts/slot-scan/main.rs 2>"$dir/rustc.log" \
    || { cat "$dir/rustc.log" >&2; echo "slot scanner failed to compile" >&2; return 1; }
}
