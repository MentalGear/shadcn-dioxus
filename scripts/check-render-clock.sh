#!/usr/bin/env bash
#
# check-render-clock.sh
#
# A component that reads the clock while rendering, or in a prop default, bakes the day the page
# was prerendered into the static HTML, and hydration keeps it: the calendar's "today", the month
# it opens on, a date picker's default segment values and every demo built on them show the build
# day (backlog row 380). So the clock is read in one place, `dioxus_primitives::calendar::use_today`,
# which renders the server's date during hydration and reads the visitor's clock after mount.
#
# This gate fails on any clock read in the Rust sources of primitives/src and preview/src (where
# every component, demo and page lives) except on a line that ends with the marker
# `// render-clock: allowed`. Comment lines (`//`, `///`, `//!`) are skipped, so a doc example may
# still name the API. A new date source goes through `use_today`; mark a line only when it is
# outside every render path (the helper `use_today` itself calls).
#
# Usage: scripts/check-render-clock.sh
# Exit status: 0 when clean, 1 on a violation.
set -uo pipefail
cd "$(dirname "$0")/.."

# Files whose clock read is inside a `use_effect` (row 377: the date's space is reserved at first paint).
# Move them to `use_today` and drop them here.
skip_files='^preview/src/components/date_picker/variants/(natural_language|unavailable_dates)/mod\.rs:'

pattern='\b(UtcDateTime|OffsetDateTime|PrimitiveDateTime|SystemTime)::now(_utc|_local)?\b|\bjs_sys::Date::(now|new_0)\b|\b(Utc|Local)::now\b|\bnow_local_date\b'
marker='// render-clock: allowed[[:space:]]*$'

hits=$(grep -rnE --include='*.rs' "$pattern" primitives/src preview/src \
  | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' \
  | grep -vE "$skip_files" \
  | grep -vE "$marker")
if [ -n "$hits" ]; then
  echo "FAIL: clock read outside use_today (read the date with dioxus_primitives::calendar::use_today," >&2
  echo "      or end the line with '// render-clock: allowed' if no render path reaches it; backlog row 380):" >&2
  echo "$hits" >&2
  exit 1
fi
allowed=$(grep -rnE --include='*.rs' "$pattern" primitives/src preview/src | grep -cE "$marker")
echo "OK: no clock read outside use_today ($allowed marked line(s))"
