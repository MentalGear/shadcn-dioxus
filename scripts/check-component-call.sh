#!/usr/bin/env bash
#
# check-component-call.sh  (backlog row 150)
#
# A component is MOUNTED (`Block {}` in rsx), never CALLED. Calling one as a plain
# function -- `component.call(())` on a `Callback<(), Element>`, `(entry.thumb)()` on a
# `fn() -> Element` field, or `Calendar(props)` on a `#[component] fn` -- runs its hooks
# in the CALLER's scope: `Callback::call` pushes the scope the callback was CREATED in,
# so a `use_signal`/`use_effect`/`use_interval` inside lands on that scope's hook list,
# whose index resets only when that scope re-renders. Every later re-render of the
# caller then allocates a FRESH set beside the old, unreachable one. Example (row 150):
# `MasonryCard` calling `component.call(())` grows a never-cleared interval in
# `BlockPlayer` every ~1.2 s (128 live intervals after ~100 s). It compiles, clippy and
# cargo test stay green, and a hook inside is invisible to grep.
#
# The rule (primitives/src and preview/src, `#[cfg(test)]` modules and modules declared
# `#[cfg(test)] mod x;` excluded; comments and string contents are ignored):
#   1. DIRECT   a call `Name(` / `path::Name(` / `Name::<T>(` of a `#[component] fn Name`
#               (an `fn Name` item and `.Name(` method calls do not count).
#   2. UNIT-CB  `ident.call(())` where `ident` is declared anywhere in the scanned trees
#               as `ident: [Option<]Callback<(), Element>` (the shape of a component
#               passed as data), or is bound by a pattern `W(ident)` where
#               `struct W(pub Callback<(), Element>)` (`ChartIcon(icon)`).
#   3. FN-PTR   `(x.ident)(..)` or a bare `ident(..)` where `ident` is declared as
#               `ident: fn(..) -> Element` (a component fn pointer; mount it with
#               `let Block = entry.component; rsx! { Block {} }` instead).
# NOT flagged on purpose: `Callback<Arg, Element>` render props that take an argument
# (`render_item.call(i)`, `render_row`, `as`, `formatter`): they are keyed row/slot
# renderers, a documented contract, common, and not a component-as-data. `EventHandler`
# and `Callback<_, ()>` `.call(())` are not components (measured below).
#
# Shapes in primitives/src + preview/src, excluding tests:
#   `.call(())` on `Callback<(), Element>`: 2 (chart tooltip/legend `ChartIcon`),
#   On `Callback<(), ()>`/`EventHandler` (event handlers, resets, drop hooks): not flagged.
#   Direct calls of a `#[component]` fn: 2 (`Calendar(self)`, `RangeCalendar(self)`),
#   both in the body of a component that forwards its own props.
#   fn-pointer calls: 1 (`(entry.thumb)()`, hook-free SVG thumbnail).
#   Five real shapes, each a reasoned allowlist entry below.
#
# Escapes: an inline `component-call-ok: <reason>` comment on the line or the two above,
# or a reasoned entry in ALLOW below (`file|callee`). A marker or entry that suppresses
# nothing is STALE and fails. `--self-test` runs mutations that must turn red.
#
# Usage: scripts/check-component-call.sh [--self-test] [root]  (a plain run also runs the self-test)
# Exit 0 clean, 1 violation or stale escape, 2 self-test failure.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

self_test=0
if [[ "${1:-}" == "--self-test" ]]; then self_test=1; shift; fi
root="${1:-$repo_root}"

checker="$(mktemp -t check-component-call.XXXXXX.py)"
trap 'rm -f "$checker"' EXIT

cat >"$checker" <<'PYEOF'
import re
import sys
from pathlib import Path

# file|callee -> reason. Stale entries (no matching hit) fail.
ALLOW = {
    "primitives/src/calendar.rs|Calendar":
        "`DefaultCalendarProps::default_calendar` is the `fn(T) -> Element` the date picker "
        "mounts as `Calendar {}`; the body forwards its own props, so hooks run in the "
        "mounted component's own scope",
    "primitives/src/calendar.rs|RangeCalendar":
        "same as Calendar: the mounted fn pointer forwards its own props",
    "primitives/src/chart/components/tooltip.rs|icon":
        "`ChartIcon` is the documented render-prop for a series icon (an SVG, hook-free by "
        "contract, 'Unused for now' in chart/config.rs); nothing in the tree sets it",
    "primitives/src/chart/components/legend.rs|icon":
        "same `ChartIcon` render prop as the tooltip's",
    "preview/src/site/demos.rs|thumb":
        "`email_client_thumb` is a hook-free `rsx!` of one icon in a `DEMO_ENTRIES` const; "
        "a `const` table cannot hold a mounted element",
}

MARK = re.compile(r"component-call-ok:\s*\S")


def strip_rust(text):
    out = []
    i, n = 0, len(text)
    while i < n:
        two = text[i:i + 2]
        if two == "//":
            while i < n and text[i] != "\n":
                out.append(" "); i += 1
            continue
        if two == "/*":
            out.append("  "); i += 2
            while i < n and text[i:i + 2] != "*/":
                out.append("\n" if text[i] == "\n" else " "); i += 1
            if i < n:
                out.append("  "); i += 2
            continue
        m = re.match(r'[bB]?r(#*)"', text[i:])
        if m:
            closer = '"' + m.group(1)
            start = i + m.end()
            end = text.find(closer, start)
            end = n if end == -1 else end + len(closer)
            for ch in text[i:end]:
                out.append("\n" if ch == "\n" else " ")
            i = end
            continue
        if text[i] == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2; continue
                if text[j] == '"':
                    j += 1; break
                j += 1
            for ch in text[i:j]:
                out.append("\n" if ch == "\n" else " ")
            i = j
            continue
        if text[i] == "'":
            mm = re.match(r"'(\\.|[^'\\])'", text[i:])
            if mm:
                out.append(" " * len(mm.group(0))); i += len(mm.group(0)); continue
        out.append(text[i]); i += 1
    return "".join(out)


def blank_test_modules(s):
    """Blank every `#[cfg(test)] mod x { ... }` body (length preserved)."""
    out = list(s)
    for m in re.finditer(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{", s):
        i = m.end() - 1
        depth = 0
        j = i
        while j < len(s):
            if s[j] == "{": depth += 1
            elif s[j] == "}":
                depth -= 1
                if depth == 0: break
            j += 1
        for k in range(m.start(), min(j + 1, len(s))):
            if out[k] != "\n": out[k] = " "
    return "".join(out)


def test_only_files(files, root):
    """Files of modules declared `#[cfg(test)] mod x;`."""
    skip = set()
    for p, raw in files.items():
        for m in re.finditer(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", strip_rust(raw)):
            d = p.parent
            for cand in (d / f"{m.group(1)}.rs", d / m.group(1) / "mod.rs"):
                skip.add(cand)
    return skip


def main():
    root = Path(sys.argv[1])
    if len(sys.argv) > 2 and sys.argv[2] == "fixture":
        ALLOW.clear()  # self-test trees carry none of the real allowlisted shapes
    files = {}
    for sub in ("primitives/src", "preview/src"):
        base = root / sub
        if base.is_dir():
            for p in sorted(base.rglob("*.rs")):
                files[p] = p.read_text(encoding="utf-8")
    skip = test_only_files(files, root)
    code = {}
    for p, raw in files.items():
        if p in skip:
            continue
        code[p] = blank_test_modules(strip_rust(raw))

    comps, unit_cb, fn_ptr, wrappers = set(), set(), set(), set()
    for s in code.values():
        for m in re.finditer(r"#\[component\][^{;]*?\bfn\s+(\w+)", s, re.S):
            comps.add(m.group(1))
        for m in re.finditer(r"(\w+)\s*:\s*(?:Option\s*<\s*)?Callback\s*<\s*\(\s*\)\s*,\s*Element\s*>", s):
            unit_cb.add(m.group(1))
        for m in re.finditer(r"(\w+)\s*:\s*(?:Option\s*<\s*)?fn\s*\([^)]*\)\s*->\s*Element", s):
            fn_ptr.add(m.group(1))
        for m in re.finditer(r"struct\s+(\w+)\s*\(\s*(?:pub\s+)?Callback\s*<\s*\(\s*\)\s*,\s*Element\s*>\s*\)", s):
            wrappers.add(m.group(1))
    # enum variants and tuple structs share a component's name (`Item(String)`,
    # `CodeBlock(..)`): their constructor call is not a component call.
    shadow = {}
    for p, s in code.items():
        shadow[p] = set()
        for m in re.finditer(r"\benum\s+\w+[^{;]*\{", s):
            depth, j = 0, m.end() - 1
            body_start = j
            while j < len(s):
                if s[j] == "{": depth += 1
                elif s[j] == "}":
                    depth -= 1
                    if depth == 0: break
                j += 1
            body = s[body_start:j + 1]
            d = 0
            for k, ch in enumerate(body):
                if ch in "{([": d += 1
                elif ch in "})]": d -= 1
                elif d == 1 and (ch.isupper()) and not body[k - 1].isalnum() and body[k - 1] != "_":
                    mm = re.match(r"[A-Z]\w*", body[k:])
                    shadow[p].add(mm.group(0))
        for m in re.finditer(r"\bstruct\s+([A-Z]\w*)\s*[(<]", s):
            shadow[p].add(m.group(1))
    for s in code.values():
        for w in wrappers:
            for m in re.finditer(r"\b" + w + r"\s*\(\s*([a-z_]\w*)\s*\)", s):
                unit_cb.add(m.group(1))

    hits = []  # (path, line, key, text)
    for p, s in code.items():
        rel = str(p.relative_to(root))
        def add(pos, callee, what):
            hits.append((rel, s.count("\n", 0, pos) + 1, callee, what))
        for m in re.finditer(r"(?<![\w.])((?:\w+::)*)([A-Z]\w*)\s*(?:::<[^>]*>)?\(", s):
            c = m.group(2)
            if c not in comps:
                continue
            # `Enum::Variant(` / `Self::X(` is a constructor; a module path is lowercase
            if any(seg[:1].isupper() for seg in m.group(1).split("::") if seg):
                continue
            if not m.group(1) and c in shadow[p]:
                continue
            if re.search(r"\bfn\s+$", s[max(0, m.start() - 40):m.start()]):
                continue
            add(m.start(), c, f"component `{c}` called as a function")
        for c in unit_cb:
            for m in re.finditer(r"\b" + c + r"\s*\.\s*call\s*\(\s*\(\s*\)\s*\)", s):
                add(m.start(), c, f"`{c}.call(())` on a `Callback<(), Element>`")
        for c in fn_ptr:
            for m in re.finditer(r"\(\s*[\w.]*\b" + c + r"\s*\)\s*\(", s):
                add(m.start(), c, f"`({c})(..)` calls a `fn(..) -> Element` pointer")
            for m in re.finditer(r"(?<![\w.:])" + c + r"\s*\(", s):
                if re.search(r"\b(?:fn|struct|enum)\s+$", s[max(0, m.start() - 20):m.start()]):
                    continue
                add(m.start(), c, f"`{c}(..)` calls a `fn(..) -> Element` pointer")

    bad, used_allow, used_marks = [], set(), set()
    for rel, line, callee, what in sorted(set(hits)):
        raw_lines = files[root / rel].split("\n")
        window = range(max(0, line - 3), line)
        marked = [i for i in window if MARK.search(raw_lines[i])]
        key = f"{rel}|{callee}"
        if marked:
            used_marks.update((rel, i) for i in marked)
            continue
        if key in ALLOW:
            used_allow.add(key)
            continue
        bad.append(f"{rel}:{line}: {what}")
    stale = [f"ALLOW entry `{k}` suppresses nothing (stale)" for k in ALLOW if k not in used_allow]
    # a marker is stale when no hit sits within its window
    for p, raw in files.items():
        rel = str(p.relative_to(root))
        for i, l in enumerate(raw.split("\n")):
            if "//" in l and MARK.search(l) and (rel, i) not in used_marks and p not in skip:
                if not any(h[0] == rel and i + 1 <= h[1] <= i + 3 for h in hits):
                    stale.append(f"{rel}:{i + 1}: `component-call-ok` marker suppresses nothing (stale)")
    if bad or stale:
        print("check-component-call: a component is invoked as a function instead of mounted in rsx;", file=sys.stderr)
        print("its hooks would run in the CALLER's scope (backlog row 150). Mount it: "
              "`let Block = x.component; rsx! { Block {} }`.", file=sys.stderr)
        for b in bad + stale:
            print("  " + b, file=sys.stderr)
        sys.exit(1)
    print(f"check-component-call: OK ({len(comps)} components, {len(unit_cb)} unit element callbacks, "
          f"{len(fn_ptr)} fn-pointer fields; {len(used_allow)} allowlisted, {len(used_marks)} marked)")


main()
PYEOF

if [[ $self_test -eq 0 ]]; then
  # the gate command is the bare script path (run-gates.sh's list guard), so the mutation
  # self-test rides along with every real run; `--self-test` alone skips the tree scan
  python3 "$checker" "$root" || exit $?
  exec "$0" --self-test >/dev/null || { echo "check-component-call: self-test failed" >&2; exit 2; }
fi

# --- self-test: each mutation must turn red, each control must stay green ---------------------
tmp="$(mktemp -d -t check-component-call-st.XXXXXX)"
trap 'rm -rf "$tmp" "$checker"' EXIT
fail=0
mk() { # name, file body
  rm -rf "$tmp/t"; mkdir -p "$tmp/t/primitives/src" "$tmp/t/preview/src"
  printf '%s\n' "$1" >"$tmp/t/primitives/src/a.rs"
}
base='#[component]
fn Block() -> Element { rsx! { div {} } }
pub struct W(pub Callback<(), Element>);
pub struct P { pub render: Callback<(), Element>, pub thumb: fn() -> Element, pub on: Callback<(), ()> }'
expect() { # want (red|green), label, body
  mk "$3"
  if python3 "$checker" "$tmp/t" fixture >/dev/null 2>&1; then got=green; else got=red; fi
  if [[ "$got" != "$1" ]]; then echo "self-test FAIL: $2 expected $1, got $got" >&2; fail=1; else echo "self-test ok: $2 -> $1"; fi
}
expect green "clean control" "$base
fn ok(p: P) -> Element { rsx! { Block {} {p.on.call(())} } }"
expect red   "component called as a function" "$base
fn bad() -> Element { Block() }"
expect red   "path-qualified component call" "$base
fn bad() -> Element { crate::Block() }"
expect red   "unit Element callback .call(())" "$base
fn bad(p: P) -> Element { rsx! { {p.render.call(())} } }"
expect red   "fn-pointer field call" "$base
fn bad(p: P) -> Element { rsx! { {(p.thumb)()} } }"
expect red   "tuple-struct wrapper binding" "$base
fn bad(w: Option<W>) -> Element { if let Some(W(icon)) = w { return icon.call(()); } rsx!{} }"
expect green "unit () callback is not a component" "$base
fn ok(p: P) { p.on.call(()); }"
expect green "cfg(test) module excluded" "$base
#[cfg(test)]
mod tests { fn t() -> Element { Block() } }"
expect green "inline marker suppresses" "$base
fn m() -> Element {
    // component-call-ok: self-test
    Block()
}"
expect red   "stale marker fails" "$base
// component-call-ok: nothing here
fn fine() {}"
expect green "comment and string mentions ignored" "$base
fn s() { let _ = \"Block()\"; /* Block() */ }"
# stale allowlist entry: a tree without the allowlisted shapes must fail on every ALLOW key
mk "$base"
out="$(python3 "$checker" "$tmp/t" 2>&1 || true)"
if grep -q "stale" <<<"$out"; then echo "self-test ok: stale ALLOW entries fail -> red"; else echo "self-test FAIL: stale ALLOW entry did not fail" >&2; fail=1; fi
[[ $fail -eq 0 ]] || exit 2
echo "check-component-call --self-test: OK"
