#!/usr/bin/env bash
#
# check-state-attributes.sh
#
# Components emit their state attributes (`data-state`, `data-checked`, `data-open`, `data-pressed`, `data-disabled`, ...) only through
# `primitives/src/state_attrs.rs`, which writes Radix's `data-state` and Base UI's presence attributes (empty while the state holds,
# absent otherwise) from one call. A non-test, non-comment line under `primitives/src` (except `chart/`) or in a preview
# `components/*/component.rs` that writes one of the vocabulary names as a literal attribute key fails, and so does any other `data-*`
# attribute given the constant `false` or `"false"`. Valued booleans that stylesheets and specs still select as `="true"` go
# through `state_attrs::valued`, `disabled`, `open_valued` and `presence_true`, whose calls are counted per file against a
# shrink-only baseline. Backlog row 481.
#
# THE CLASS: a boolean written as `"data-x": flag` renders `data-x=false` in the false case (dioxus-ssr writes a `Bool` as
# `name=value`), so a presence selector (`[data-x]`, Tailwind's `data-open:`, every base-nova class string) matches the false case,
# and nothing in a component's source tells a presence attribute from a valued one. A second hand-written site of the same name can
# also disagree with its `data-state` twin.
#
# THE CONSTRUCTION: one module owns the vocabulary. `state_attrs::open|trigger_open|checked|pressed|active|panel_hidden|selected|
# progress|state` return `data-state` and the matching presence attribute together, `presence`/`readonly`/`popup_open`/`panel_open`
# return a presence attribute (present and empty, or absent), and the valued forms live in the same file, so moving a flag to
# presence is one edit there plus its CSS and specs. This gate keeps every other file from writing the names.
#
# Vocabulary: state, checked, unchecked, indeterminate, open, closed, popup-open, panel-open, pressed, disabled, readonly,
# highlighted, selected, invalid, active, dragging, complete, progressing. `dev-docs/state-attributes.md` lists each one (the gate
# checks that). Scope: `primitives/src/**/*.rs` minus `chart/` (a Recharts-conformance surface with its own attribute set), tests,
# and `state_attrs.rs`; `preview/src/components/*/component.rs`. A site that must stay (a JS-owned attribute, a constant) takes an
# entry in ALLOW below with its reason; an entry that no site uses fails as stale.
#
# Baseline: `scripts/check-state-attributes.baseline.tsv` (`path<TAB>call<TAB>count`). Usage:
#   scripts/check-state-attributes.sh                     run the gate
#   scripts/check-state-attributes.sh --update-baseline   lock in a drop (refuses to raise a count without --allow-grow)
#   scripts/check-state-attributes.sh --list              print the allowlist and the baseline totals
#   scripts/check-state-attributes.sh --self-test         run the fixtures only
# Exit status: 0 when clean, 1 on a violation, 2 on bad usage.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 -I - "$@" <<'PYEOF'
import collections
import glob
import os
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402

VOCAB = [
    "state", "checked", "unchecked", "indeterminate", "open", "closed", "popup-open", "panel-open",
    "pressed", "disabled", "readonly", "highlighted", "selected", "invalid", "active", "dragging",
    "complete", "progressing",
]
HELPER = "primitives/src/state_attrs.rs"
BASELINE = "scripts/check-state-attributes.baseline.tsv"
DOC = "dev-docs/state-attributes.md"

# (path, attribute, reason): a literal that must stay outside the helper.
ALLOW = Allowlist("check-state-attributes", [
    ("primitives/src/message_scroller.rs", "data-active",
     "The scroll-to-end button's `data-active` is written by the page script (`message_scroller.js`); the Rust render only seeds its first value."),
], fields=("path", "name"))

# Calls (and constant-false sites) whose count per file is the legacy-valued debt.
CALL_RE = re.compile(
    r"state_attrs::(valued|presence_true)\(\s*(?:state_attrs::)?Flag::(\w+)"
    r"|state_attrs::(disabled|open_valued)\(")


def strip_comments(src):
    """Rust source with comments blanked (newlines kept); string and char literals left alone."""
    out = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == "/" and src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
        elif c == "/" and src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            out.append(re.sub(r"[^\n]", " ", src[i:j]))
            i = j
        elif c == "r" and re.match(r'r#*"', src[i:i + 8]) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            hashes = len(re.match(r"r(#*)", src[i:]).group(1))
            end = src.find('"' + "#" * hashes, i + 2 + hashes)
            end = n if end < 0 else end + 1 + hashes
            out.append(src[i:end])
            i = end
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            out.append(src[i:j + 1])
            i = j + 1
        elif c == "'":
            m = re.match(r"'(?:\\.[^']*|[^\\'])'", src[i:i + 12])
            if m:
                out.append(m.group(0))
                i += len(m.group(0))
            else:
                out.append(c)
                i += 1
        else:
            out.append(c)
            i += 1
    return "".join(out)


def live(src):
    """Comment-free source up to the first `#[cfg(test)]` (tests close every file)."""
    src = strip_comments(src)
    m = re.search(r"^[ \t]*#\[cfg\(test\)\]", src, re.M)
    return src if m is None else src[:m.start()]


KEY_RE = re.compile(r'"data-([a-z-]+)"\s*:\s*([^\n]*)')
NEW_RE = re.compile(r'Attribute::new\(\s*"data-([a-z-]+)"')
FALSE_RE = re.compile(r'^\s*(false|"false")(?![\w])|else\s*\{\s*"false"\s*\}')


def scan(path, src):
    """Findings for one file: [(kind, line, name, detail)]; kind is 'literal' or 'false'."""
    text = live(src)
    found = []
    for m in list(KEY_RE.finditer(text)) + list(NEW_RE.finditer(text)):
        name = "data-" + m.group(1)
        line = text.count("\n", 0, m.start()) + 1
        if m.group(1) in VOCAB:
            found.append(("literal", line, name, "outside state_attrs.rs"))
        elif m.re is KEY_RE and FALSE_RE.search(m.group(2)):
            found.append(("false", line, name, m.group(2).strip()))
    calls = collections.Counter()
    for m in CALL_RE.finditer(text):
        if m.group(1):
            calls[f"{m.group(1)}:{m.group(2)}"] += 1
        else:
            calls[{"disabled": "valued:Disabled", "open_valued": "valued:Open"}[m.group(3)]] += 1
    return found, calls


def files():
    out = []
    for p in sorted(glob.glob("primitives/src/**/*.rs", recursive=True)):
        base = os.path.basename(p)
        if (p == HELPER or "/chart/" in p or "/tests/" in p or base in ("tests.rs", "ssr_tests.rs")
                or base.endswith("_tests.rs")):
            continue
        out.append(p)
    out += sorted(glob.glob("preview/src/components/*/component.rs"))
    return out


def run_self_test():
    fixtures = [
        ('rsx! { div { "data-state": if open { "open" } else { "closed" } } }', ["literal"]),
        ('attributes!(div { "data-open": open })', ["literal"]),
        ('attributes!(div { "data-checked": c.then_some("") })', ["literal"]),
        ('attributes!(div { ..state_attrs::open(open()) })', []),
        ('attributes!(div { "data-slot": "x", "data-orientation": o })', []),
        ('attributes!(div { "data-thing": "false" })', ["false"]),
        ('attributes!(div { "data-thing": if x { "true" } else { "false" } })', ["false"]),
        ('attributes!(div { "data-thing": false })', ["false"]),
        ('Attribute::new("data-pressed", v, None, false)', ["literal"]),
        ('// "data-state": "open"\nlet a = 1;', []),
        ('/// - `data-state`: open\nfn f() {}', []),
        ('let s = "// \\"data-open\\": x"; /* "data-active": y */', []),
        ('fn f() {}\n#[cfg(test)]\nmod t { fn g() { let _ = r#"data-state"#; "data-state": "x" } }', []),
        ('attributes!(div { "data-thing": other_false })', []),
    ]
    bad = 0
    for src, want in fixtures:
        got = [k for k, *_ in scan("x.rs", src)[0]]
        if got != want:
            print(f"self-test FAILED for {src!r}: want {want}, got {got}")
            bad += 1
    calls = scan("x.rs", "..state_attrs::valued(state_attrs::Flag::Dragging, d), ..state_attrs::disabled(x), "
                 "..state_attrs::presence_true(Flag::Active, a), ..state_attrs::open_valued(o), ..state_attrs::presence(Flag::Selected, s)")[1]
    want_calls = {"valued:Dragging": 1, "valued:Disabled": 1, "presence_true:Active": 1, "valued:Open": 1}
    if dict(calls) != want_calls:
        print(f"self-test FAILED for call counting: want {want_calls}, got {dict(calls)}")
        bad += 1
    return bad


def read_baseline():
    base = {}
    if os.path.exists(BASELINE):
        for line in open(BASELINE):
            if line.startswith("#") or not line.strip():
                continue
            path, call, n = line.rstrip("\n").split("\t")
            base[(path, call)] = int(n)
    return base


def write_baseline(counts):
    with open(BASELINE, "w") as f:
        f.write("# check-state-attributes baseline: calls of the legacy valued forms (valued, disabled, open_valued,\n"
                "# presence_true) per file, shrink-only. Regenerate with scripts/check-state-attributes.sh --update-baseline;\n"
                "# see dev-docs/state-attributes.md.\n")
        for (path, call), n in sorted(counts.items()):
            f.write(f"{path}\t{call}\t{n}\n")


def main(argv):
    if "--self-test" in argv:
        bad = run_self_test()
        print("check-state-attributes: self-test " + ("FAILED" if bad else "OK"))
        return 1 if bad else 0
    if run_self_test():
        print("check-state-attributes: self-test FAILED")
        return 1
    known = {"--update-baseline", "--allow-grow", "--list", "--self-test"}
    if any(a not in known for a in argv):
        print("usage: scripts/check-state-attributes.sh [--update-baseline [--allow-grow]] [--list] [--self-test]", file=sys.stderr)
        return 2

    errors = []
    counts = {}
    for path in files():
        found, calls = scan(path, open(path).read())
        calls = collections.Counter(calls)
        for kind, line, name, detail in found:
            if ALLOW.find(lambda e, p=path, n=name: e.path == p and e.name == n):
                continue
            if kind == "literal":
                errors.append(f"{path}:{line}: `\"{name}\"` is written here; a component gets state attributes only from `state_attrs::*` "
                              f"(primitives/src/state_attrs.rs)")
            else:
                calls[f"false:{name}"] += 1
        for call, n in calls.items():
            counts[(path, call)] = n
    errors += ALLOW.problems()

    # The vocabulary is documented once, and every name is in it.
    if not os.path.exists(DOC):
        errors.append(f"{DOC} is missing")
    else:
        doc = open(DOC).read()
        for name in VOCAB:
            if f"`data-{name}`" not in doc:
                errors.append(f"{DOC} does not list `data-{name}`")
    helper = open(HELPER).read() if os.path.exists(HELPER) else ""
    for name in VOCAB:
        if f"data-{name}" not in helper:
            errors.append(f"{HELPER} never writes `data-{name}`; drop it from VOCAB or add its helper")

    base = read_baseline()
    if "--update-baseline" in argv:
        grown = [(k, base.get(k, 0), v) for k, v in counts.items() if v > base.get(k, 0)]
        if grown and "--allow-grow" not in argv:
            for (path, call), old, new in grown:
                print(f"refusing to raise {path} {call}: {old} -> {new} (new code uses state_attrs::presence; --allow-grow with a reason)")
            return 1
        write_baseline(counts)
        print(f"check-state-attributes: baseline written ({sum(counts.values())} legacy valued calls in {len({p for p, _ in counts})} files)")
        return 0
    if "--list" in argv:
        print(ALLOW.listing())
        print(f"legacy valued calls: {sum(counts.values())} in {len({p for p, _ in counts})} files")
        return 0

    for (path, call), n in sorted(counts.items()):
        if n > base.get((path, call), 0):
            what = ("a `data-*` attribute given the constant `false`/`\"false\"` (a boolean attribute is present or absent, never `false`)"
                    if call.startswith("false:") else "a legacy valued boolean")
            errors.append(f"{path}: {n} `{call}` site(s), baseline {base.get((path, call), 0)}: {what} is selected as "
                          f"`=\"true\"`/`=\"false\"` by existing CSS, JS or specs, so a new one is not added; use `state_attrs::presence`")
    for (path, call), n in sorted(base.items()):
        if counts.get((path, call), 0) < n:
            errors.append(f"{path}: {call} fell from {n} to {counts.get((path, call), 0)}; run scripts/check-state-attributes.sh --update-baseline")

    for e in errors:
        print("FAIL: " + e)
    if not errors:
        print(f"check-state-attributes: OK ({len(files())} files, {sum(counts.values())} legacy valued calls on the baseline)")
    return 1 if errors else 0


sys.exit(main(sys.argv[1:]))
PYEOF
