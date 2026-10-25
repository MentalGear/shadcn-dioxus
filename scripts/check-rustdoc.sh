#!/usr/bin/env bash
#
# check-rustdoc.sh
#
# Rustdoc builds without a warning: `cargo doc --no-deps --document-private-items` on the default arm of the whole
# workspace and on the `web` arm of primitives and preview, so a broken intra-doc link, an unclosed HTML tag, a bare URL
# or a public doc comment that links a private item is red before docs.rs shows it. Strict, with no baseline: any rustdoc warning
# or error fails the gate (dev-docs/backlog.md row 382). Fix a site by rewriting the doc comment: link
# only items that are public, use a plain code span for a private one (or one that exists only on the `web` arm),
# put a literal `<tag>` in a code span, write a URL as `<url>`, disambiguate a name that is both
# a function and a module with `derive()` / `mod@window`. A raw identifier (`r#loop`) cannot be an intra-doc link.
#
# A cargo gate: it compiles the workspace's documentation, so it needs an absolute, isolated CARGO_TARGET_DIR
# (run-gates.sh sets it); scripts/cargo-inputs-hash.sh hashes this script with the cargo inputs.
#
# Usage: CARGO_TARGET_DIR=/abs/dir scripts/check-rustdoc.sh [--inventory]
#   --inventory   print every current warning (file:line, message) and exit 0 instead of checking
# Exit status: 0 when clean, 1 on any warning or error, 2 on bad usage or environment.
set -uo pipefail
cd "$(dirname "$0")/.."

inventory=0
for a in "$@"; do
  case "$a" in
    --inventory) inventory=1 ;;
    -h|--help) sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "check-rustdoc: unknown option $a" >&2; exit 2 ;;
  esac
done
case "${CARGO_TARGET_DIR:-}" in
  /*) ;;
  *) echo "check-rustdoc: CARGO_TARGET_DIR must be set to an absolute, isolated dir" >&2; exit 2 ;;
esac

out_dir="$(mktemp -d)"; trap 'rm -rf "$out_dir"' EXIT
doc() { # label, then the cargo doc arguments
  local label="$1"; shift
  cargo doc --no-deps --document-private-items --message-format short "$@" > "$out_dir/$label.log" 2>&1
  echo $? > "$out_dir/$label.rc"
}
doc default --workspace
doc web -p dioxus-primitives -p preview --features dioxus-primitives/web,preview/web

python3 -I - "$out_dir" "$inventory" <<'PY'
import collections, os, re, sys
out_dir, inventory = sys.argv[1], sys.argv[2] == "1"
line_re = re.compile(r"^(?P<file>[^\s:][^:]*):(?P<line>\d+):(?P<col>\d+): (?P<kind>warning|error)(?:\[[^\]]*\])?: (?P<msg>.*)$")
summary_re = re.compile(r"generated \d+ warnings?|could not document|^warning: build failed")
per_arm = {}
errors = []
sites = collections.defaultdict(list)
for arm in ("default", "web"):
    counts = collections.Counter()
    for raw in open(os.path.join(out_dir, arm + ".log"), encoding="utf-8", errors="replace"):
        raw = raw.rstrip("\n")
        m = line_re.match(raw)
        if m:
            msg = re.sub(r": this item is private$", "", m.group("msg"))
            if m.group("kind") == "error":
                errors.append(f"{m.group('file')}:{m.group('line')}: {msg}")
                continue
            counts[(m.group("file"), msg)] += 1
            sites[(m.group("file"), msg)].append(f"{m.group('file')}:{m.group('line')}")
        elif raw.startswith("error") and not summary_re.search(raw):
            errors.append(raw)
    per_arm[arm] = counts
    rc = open(os.path.join(out_dir, arm + ".rc")).read().strip()
    if rc != "0" and not errors:
        errors.append(f"cargo doc ({arm} arm) exited {rc} without a rustdoc error line; see its output with --inventory")
# a site seen in both arms is one site: the larger count of the two
current = collections.Counter()
for counts in per_arm.values():
    for k, n in counts.items():
        current[k] = max(current[k], n)

if inventory:
    for (f, msg), n in sorted(current.items()):
        print(f"{n}x {', '.join(sorted(set(sites[(f, msg)]))[:3])}  {msg}")
    print(f"check-rustdoc: {sum(current.values())} warnings in {len(current)} distinct sites", file=sys.stderr)
    sys.exit(0)
if errors:
    print("check-rustdoc: rustdoc errors (not warnings):", file=sys.stderr)
    for e in errors[:30]:
        print("  " + e, file=sys.stderr)
    sys.exit(1)
if current:
    print(f"check-rustdoc: {sum(current.values())} rustdoc warnings (the gate is strict; fix the doc comment):", file=sys.stderr)
    for (f, msg), n in sorted(current.items()):
        where = ", ".join(sorted(set(sites[(f, msg)]))[:3])
        print(f"  {where}: {msg}", file=sys.stderr)
    sys.exit(1)
print("check-rustdoc: OK (0 warnings)")
PY
