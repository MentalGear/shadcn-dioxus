#!/usr/bin/env bash
#
# check-cargo-locks.sh
#
# Every committed Cargo.lock agrees with its manifest. The repo has detached crates with their own lock
# (`playwright/oracle/attr-synth`, an empty `[workspace]` so it never joins the root workspace, and
# `labs/carousel-lab`) that path-depend on `primitives/`. Nothing builds them in the gates, so when a
# primitives dependency changes (a dropped `dioxus-sdk-time` left both locks stale on main, backlog row 254)
# their locks go stale silently -- and the next lane whose spec run builds attr-synth (hydration-parity
# Rule 4c) gets a rewritten lock in its diff as an unexplained side effect. By construction, then: cargo
# resolves each manifest against its lock with `--locked --offline`; a lock that would have to change fails.
#
# Resolution against the lock happens before any download, so a dependency missing from the local cache
# (`attempting to make an HTTP request, but --offline was specified`) means the lock was consistent and is
# not a failure; any other error is reported and fails. No network. Fix a red: `cargo metadata --offline
# --format-version 1 --manifest-path <dir>/Cargo.toml >/dev/null` (a minimal lock update), then commit it.
#
# Usage: scripts/check-cargo-locks.sh     Exit status: 0 when every lock is current.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
fail=0; n=0
while IFS= read -r lock; do
  dir=$(dirname "$lock"); n=$((n + 1))
  err=$(cargo metadata --locked --offline --format-version 1 --manifest-path "$dir/Cargo.toml" 2>&1 >/dev/null)
  rc=$?
  if [ $rc -eq 0 ] || grep -q 'but --offline was specified' <<<"$err"; then continue; fi
  if grep -q 'cannot update the lock file' <<<"$err"; then
    echo "  - $lock is stale: its manifest no longer resolves to it (cargo would rewrite it)"
  else
    echo "  - $lock: cargo metadata failed: $(tail -1 <<<"$err")"
  fi
  fail=1
done < <(git ls-files '*Cargo.lock')
if [ $fail -ne 0 ]; then
  echo "check-cargo-locks: FAIL -- update each listed lock with 'cargo metadata --offline --format-version 1 --manifest-path <dir>/Cargo.toml >/dev/null' and commit it"
  exit 1
fi
echo "check-cargo-locks: OK -- $n committed Cargo.lock file(s) match their manifests"
