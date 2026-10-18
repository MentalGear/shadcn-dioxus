#!/usr/bin/env bash
#
# stamp-release-rev.sh
#
# Release-time step: pins every `dioxus-primitives` git dependency a user's `dx components add` will write to ONE
# commit of this repository (dev-docs/backlog.md row 217, dependency audit A4). Without it
# `cargo add dioxus-primitives --git <url>` resolves to the default branch's HEAD at the moment of the user's first
# add, while the component sources come from whichever registry checkout dx cached: after any later primitives
# change the two skew (a component calling an API its primitive no longer has, or the reverse).
# A commit cannot contain its own hash, so this is a step of the release procedure, not a hand edit:
#
#   1. land the release content on the release branch (R = the commit whose primitives users should get);
#   2. scripts/stamp-release-rev.sh R        # writes "rev": "<R full sha>" everywhere listed below
#   3. commit the result (S) and tag S; the install docs say `--git <url> --rev S`.
#      (S differs from R only in the stamped lines, so the primitives at R and S are the same code.)
#   scripts/stamp-release-rev.sh --remove    # back to the development form (no rev anywhere)
#   scripts/stamp-release-rev.sh --check     # report the stamp state; exit 1 when entries disagree
#
# What gets stamped (all idempotent, a re-run with another rev replaces the old one):
#   - the `rev` of every `dioxus-primitives` (and `dioxus-attributes`) entry in preview/src/components/*/component.json;
#   - `rev = "<R>"` on the `dioxus-primitives` line of templates/starter/Cargo.toml.liquid;
#   - `rev = "<R>"` under `[components.registry]` in templates/starter/Dioxus.toml (so a new app's `dx components
#     add` reads the same registry checkout).
# Safety: R must be a commit of this repository (resolved to its full sha), and primitives/, dioxus-attributes/ and
# the root Cargo.toml/Cargo.lock at R must equal the working tree's, otherwise the stamp would name code that is
# not what is being released.
#
# Usage: scripts/stamp-release-rev.sh <rev> | --remove | --check
# Exit status: 0 on success, 1 when --check finds a disagreement, 2 on bad usage / unsafe rev.

set -euo pipefail
cd "$(dirname "$0")/.."

mode="${1:-}"
[ -n "$mode" ] || { sed -n '2,29p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }

rev=""
case "$mode" in
  --remove|--check) ;;
  -*) echo "stamp-release-rev: unknown option $mode" >&2; exit 2 ;;
  *)
    rev="$(git rev-parse --verify --quiet "$mode^{commit}")" || { echo "stamp-release-rev: $mode is not a commit of this repository" >&2; exit 2; }
    if ! git diff --quiet "$rev" -- primitives dioxus-attributes Cargo.toml Cargo.lock; then
      echo "stamp-release-rev: primitives/, dioxus-attributes/, Cargo.toml or Cargo.lock differ between $rev and the working tree;" >&2
      echo "  stamp the commit whose primitives are the ones being released (git diff $rev -- primitives dioxus-attributes)" >&2
      exit 2
    fi ;;
esac

python3 -I - "$mode" "$rev" <<'PY'
import glob, json, re, sys

mode, rev = sys.argv[1], sys.argv[2]
URL_RE = r'https://github\.com/[^"\s]+'
changed = []
seen = {}

def note(path, value): seen.setdefault(value, []).append(path)

for f in sorted(glob.glob("preview/src/components/*/component.json")):
    text = open(f).read()
    json.loads(text)                                   # must parse before and after
    def fix(m):
        block = m.group(0)
        if '"dioxus-primitives"' not in block and '"dioxus-attributes"' not in block:
            return block
        have = re.search(r'"rev":\s*"([^"]*)"', block)
        note(f, have.group(1) if have else None)
        if mode == "--check": return block
        block = re.sub(r',\s*"rev":\s*"[^"]*"', "", block)
        if mode == "--remove": return block
        m2 = re.search(r'^([ \t]*)"git":\s*"[^"]*"', block, re.M)
        sep = (",\n" + m2.group(1)) if m2 else ", "      # keep one key per line when the manifest has that shape
        return re.sub(r'("git":\s*"[^"]*")', lambda g: g.group(1) + sep + '"rev": "%s"' % rev, block, count=1)
    new = re.sub(r'\{[^{}]*"name":\s*"dioxus-(?:primitives|attributes)"[^{}]*\}', fix, text)
    if new != text:
        json.loads(new)
        open(f, "w").write(new); changed.append(f)

cargo = "templates/starter/Cargo.toml.liquid"
text = open(cargo).read()
def fix_cargo(m):
    line = m.group(0)
    have = re.search(r'rev\s*=\s*"([^"]*)"', line)
    note(cargo, have.group(1) if have else None)
    if mode == "--check": return line
    line = re.sub(r',\s*rev\s*=\s*"[^"]*"', "", line)
    if mode == "--remove": return line
    return re.sub(r'(git\s*=\s*"[^"]*")', r'\1, rev = "%s"' % rev, line, count=1)
new = re.sub(r'^dioxus-primitives\s*=\s*\{[^}\n]*\}', fix_cargo, text, flags=re.M)
if new != text: open(cargo, "w").write(new); changed.append(cargo)

dx = "templates/starter/Dioxus.toml"
text = open(dx).read()
m = re.search(r'(^\[components\.registry\]\s*\n(?:[^\[\n]*\n)*?)(\s*git\s*=\s*"[^"]*"[^\n]*\n)((?:[^\[\n]*\n)*)', text, re.M)
if not m: print(f"stamp-release-rev: no [components.registry] git in {dx}", file=sys.stderr); sys.exit(2)
head, gitline, tail = m.group(1), m.group(2), m.group(3)
have = re.search(r'^\s*rev\s*=\s*"([^"]*)"', head + tail, re.M)
note(dx, have.group(1) if have else None)
if mode != "--check":
    tail = re.sub(r'^\s*rev\s*=\s*"[^"]*"[^\n]*\n', "", tail, flags=re.M)
    head = re.sub(r'^\s*rev\s*=\s*"[^"]*"[^\n]*\n', "", head, flags=re.M)
    revline = "" if mode == "--remove" else 'rev = "%s"\n' % rev
    new = text[:m.start()] + head + gitline + revline + tail + text[m.end():]
    if new != text: open(dx, "w").write(new); changed.append(dx)

if mode == "--check":
    if len(seen) > 1:
        print("stamp-release-rev: entries disagree: " + "; ".join(f"{k or 'unstamped'}: {len(v)} file(s)" for k, v in seen.items()), file=sys.stderr)
        sys.exit(1)
    k = next(iter(seen))
    print(f"stamp-release-rev: every entry is {'stamped ' + k if k else 'unstamped (development tree)'} ({sum(map(len, seen.values()))} places).")
else:
    print(f"stamp-release-rev: {mode if mode == '--remove' else 'stamped ' + rev}: {len(changed)} file(s) changed")
PY
