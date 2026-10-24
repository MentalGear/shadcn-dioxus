#!/usr/bin/env bash
#
# check-rustdoc.sh
#
# Rustdoc builds without a new warning: `cargo doc --no-deps --document-private-items` on the default arm of the
# whole workspace and on the `web` arm of primitives and preview, so a broken intra-doc link, an unclosed HTML tag, a
# bare URL or a public doc comment that links a private item is red before docs.rs shows it. The warnings that exist
# are a shrink-only debt register, `scripts/check-rustdoc.baseline.tsv` (`file<TAB>message`, no line numbers, so an edit
# above a site does not disturb it, a multiset): the gate fails on a warning that is not in it, on any rustdoc error,
# and on a baseline entry that no longer fires (delete the line, or run `--update-baseline`, which refuses to add one
# unless `--allow-grow` is given and the commit says why). Fix a site by rewriting the doc comment: link only items
# that are public, use a plain code span for a private one, escape or fence a literal `<tag>`, write a URL as `<url>`.
#
# A cargo gate: it compiles the workspace's documentation, so it needs an absolute, isolated CARGO_TARGET_DIR
# (run-gates.sh sets it); scripts/cargo-inputs-hash.sh hashes this script and its baseline with the cargo inputs.
#
# Usage: CARGO_TARGET_DIR=/abs/dir scripts/check-rustdoc.sh [--update-baseline [--allow-grow]] [--inventory]
#   --inventory   print every current warning (file:line, message) instead of checking
# Exit status: 0 when clean, 1 on a new warning, an error or a stale entry, 2 on bad usage or environment.
set -uo pipefail
cd "$(dirname "$0")/.."

update=0 grow=0 inventory=0
for a in "$@"; do
  case "$a" in
    --update-baseline) update=1 ;;
    --allow-grow) grow=1 ;;
    --inventory) inventory=1 ;;
    -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "check-rustdoc: unknown option $a" >&2; exit 2 ;;
  esac
done
case "${CARGO_TARGET_DIR:-}" in
  /*) ;;
  *) echo "check-rustdoc: CARGO_TARGET_DIR must be set to an absolute, isolated dir" >&2; exit 2 ;;
esac

baseline=scripts/check-rustdoc.baseline.tsv
out_dir="$(mktemp -d)"; trap 'rm -rf "$out_dir"' EXIT
doc() { # label, then the cargo doc arguments
  local label="$1"; shift
  cargo doc --no-deps --document-private-items --message-format short "$@" > "$out_dir/$label.log" 2>&1
  echo $? > "$out_dir/$label.rc"
}
doc default --workspace
doc web -p dioxus-primitives -p preview --features dioxus-primitives/web,preview/web

python3 -I - "$out_dir" "$baseline" "$update" "$grow" "$inventory" <<'PY'
import collections, os, re, sys
out_dir, baseline, update, grow, inventory = sys.argv[1], sys.argv[2], sys.argv[3] == "1", sys.argv[4] == "1", sys.argv[5] == "1"
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
base = collections.Counter()
if os.path.exists(baseline):
    for line in open(baseline, encoding="utf-8"):
        line = line.rstrip("\n")
        if line and not line.startswith("#"):
            f, _, msg = line.partition("\t")
            base[(f, msg)] += 1

if update:
    grown = {k: n - base[k] for k, n in current.items() if n > base[k]}
    if grown and not grow and os.path.exists(baseline):
        print("check-rustdoc: refusing to ADD baseline entries (shrink-only); fix these or pass --allow-grow and say why in the commit:", file=sys.stderr)
        for (f, msg), n in sorted(grown.items()):
            print(f"  {f}  {msg}", file=sys.stderr)
        sys.exit(1)
    with open(baseline, "w", encoding="utf-8") as fh:
        fh.write("# rustdoc warnings that predate the gate (file<TAB>message); shrink-only, see scripts/check-rustdoc.sh\n")
        for (f, msg), n in sorted(current.items()):
            for _ in range(n):
                fh.write(f"{f}\t{msg}\n")
    print(f"check-rustdoc: baseline written ({sum(current.values())} warnings)")
    sys.exit(0)

new = {k: n - base[k] for k, n in current.items() if n > base[k]}
stale = {k: base[k] - current[k] for k in base if base[k] > current[k]}
if new:
    print("check-rustdoc: rustdoc warnings that are not in scripts/check-rustdoc.baseline.tsv (fix the doc comment):", file=sys.stderr)
    for (f, msg), n in sorted(new.items()):
        where = ", ".join(sorted(set(sites[(f, msg)]))[:3])
        print(f"  {where}: {msg}", file=sys.stderr)
if stale:
    print("check-rustdoc: baseline entries that no longer fire (delete them, or run --update-baseline):", file=sys.stderr)
    for (f, msg), n in sorted(stale.items()):
        print(f"  {f}  {msg}", file=sys.stderr)
if new or stale:
    sys.exit(1)
print(f"check-rustdoc: OK ({sum(current.values())} baselined warnings, none new)")
PY
