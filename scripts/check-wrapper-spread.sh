#!/usr/bin/env bash
#
# check-wrapper-spread.sh
#
# A themed wrapper that takes a primitive's own props struct forwards it with the struct spread, never field by field
# (row 428): `prim::Comp { attributes: merged, ..props }`. A hand-copied `x: props.x` line is how a prop the
# primitive later gains is silently dropped (row 391 found 25 of them), and the spread makes that impossible by
# construction. In `preview/src/components/*/component.rs`, every `#[component] fn F(props: P)` whose `P` is not a
# struct of the same file (so it is the primitive's) must have `..props` at the top level of its call
# `module::F { .. }` or `module::C { .. }` (`C` + `Props` = `P`).
#
# Where a spread cannot express the call (a builtin-attribute field such as `padding: ".."`, or a custom `"data-*"`
# attribute on the call, which Dioxus refuses next to a spread) the call stays explicit and takes an entry in ALLOW
# below with its reason; an entry whose call now spreads, or whose site is gone, fails as stale (scripts/lib/allow.py).
# A wrapper with its own props struct (extra fields, composite parts) cannot spread and is `check-wrapper-props`'
# business, which still lists every primitive prop it fails to pass. Its self-test runs on every invocation.
#
# Usage: scripts/check-wrapper-spread.sh [FILE ...]   (default: preview/src/components/*/component.rs; explicit
# files are for the self-test and skip the allowlist stale-entry report)
#        scripts/check-wrapper-spread.sh --self-test
#        scripts/check-wrapper-spread.sh --list
# Exit status: 0 when clean, 1 on a violation, 2 on bad usage.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 -I - "$@" <<'PYEOF'
import glob
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402

# (file, wrapper fn, reason): a call that cannot be a spread.
ALLOW = Allowlist("check-wrapper-spread", [
    ("preview/src/components/context_menu/component.rs", "ContextMenuTrigger",
     "Sets builtin style attributes on the call (`padding`, `background`, ...), which the struct spread refuses."),
    ("preview/src/components/drawer/component.rs", "Drawer",
     "Puts a custom `data-slot` attribute on the call, which Dioxus refuses next to a spread."),
    ("preview/src/components/popover/component.rs", "PopoverContent",
     "Puts a custom `data-dx-surface` attribute on the call, which Dioxus refuses next to a spread."),
], fields=("path", "fn"))


def skip_string(s, k):
    j = k + 1
    while j < len(s) and s[j] != '"':
        j += 2 if s[j] == "\\" else 1
    return j + 1


def blank_comments(s):
    """The text with `//` comments and string contents blanked, same length, so offsets and lines survive."""
    out = list(s)
    k, n = 0, len(s)
    while k < n:
        if s.startswith("//", k):
            e = s.find("\n", k)
            e = n if e < 0 else e
            for i in range(k, e):
                out[i] = " "
            k = e
        elif s[k] == '"':
            e = skip_string(s, k)
            for i in range(k + 1, min(e - 1, n)):
                if out[i] != "\n":
                    out[i] = " "
            k = e
        else:
            k += 1
    return "".join(out)


def match_brace(s, i):
    d = 0
    for k in range(i, len(s)):
        if s[k] == "{":
            d += 1
        elif s[k] == "}":
            d -= 1
            if d == 0:
                return k
    return len(s) - 1


def top_level(block):
    """`block` with everything nested inside (), [], {} blanked out."""
    d, out = 0, []
    for c in block:
        if c in "([{":
            d += 1
            out.append(" " if d > 1 else c)
        elif c in ")]}":
            out.append(" " if d > 1 else c)
            d -= 1
        else:
            out.append(c if d <= 1 else (" " if c != "\n" else c))
    return "".join(out)


FN = re.compile(r"#\[component\]\s*pub fn (\w+)(?:<[^>]*>)?\(\s*(?:mut )?props:\s*(\w+)(?:<[^>]*>)?,?\s*\)\s*->\s*Element")
CALL = re.compile(r"(\w+)::(\w+)(?:::<[^>]*>)?\s*\{")


def scan(path, text):
    """[(fn name, line)] of the wrappers in `text` that render their primitive without `..props`."""
    clean = blank_comments(text)
    local = set(re.findall(r"\bstruct (\w+)", clean))
    bad = []
    for m in FN.finditer(clean):
        fname, ptype = m.groups()
        if ptype in local:
            continue
        bs = clean.index("{", m.end())
        be = match_brace(clean, bs)
        body = clean[bs:be + 1]
        for c in CALL.finditer(body):
            cname = c.group(2)
            if cname != fname and cname + "Props" != ptype:
                continue
            o = bs + c.end() - 1
            inner = top_level(clean[o:match_brace(clean, o) + 1])
            if not re.search(r"(^|[\s,{])\.\.props\b", inner):
                bad.append((fname, clean.count("\n", 0, o) + 1))
    return bad


def run(files, report_stale):
    errors = []
    for f in files:
        for fname, line in scan(f, open(f, encoding="utf-8").read()):
            if ALLOW.find(lambda e: e.path == f and e.fn == fname):
                continue
            errors.append(f"{f}:{line}: `{fname}` takes the primitive's props struct but does not spread it "
                          f"(write `..props`; a call a spread cannot express goes on ALLOW with its reason)")
    if report_stale:
        errors += ALLOW.problems()
    return errors


def self_test(quiet=False):
    good = '#[component]\npub fn A(props: AProps) -> Element {\n  rsx! { p::A { attributes: merged, ..props } }\n}\n'
    ident = '#[component]\npub fn A(props: AProps) -> Element {\n  rsx! { p::A { id: props.id, attributes: merged, {props.children} } }\n}\n'
    local = 'struct AProps { x: u8 }\n#[component]\npub fn A(props: AProps) -> Element {\n  rsx! { p::A { x: props.x } }\n}\n'
    other = '#[component]\npub fn A(props: AProps) -> Element {\n  rsx! { p::B { id: props.id } }\n}\n'
    commented = '#[component]\npub fn A(props: AProps) -> Element {\n  rsx! { p::A { id: props.id, // ..props\n  } }\n}\n'
    nested = '#[component]\npub fn A(props: AProps) -> Element {\n  rsx! { p::A { id: props.id, x { ..props } } }\n}\n'
    turbofish = '#[component]\npub fn A<T>(props: AProps<T>) -> Element {\n  rsx! { p::A::<T> { id: props.id } }\n}\n'
    named = '#[component]\npub fn A(props: SharedProps) -> Element {\n  rsx! { p::A { attributes: m, ..props, {c} } }\n}\n'
    cases = [("spread", good, 0), ("identity fields", ident, 1), ("local props struct", local, 0),
             ("other primitive", other, 0), ("spread only in a comment", commented, 1),
             ("spread of a nested block", nested, 1), ("turbofish call", turbofish, 1), ("spread with a body", named, 0)]
    bad = [f"{label}: expected {want}, got {len(scan('x', text))}" for label, text, want in cases
           if len(scan("x", text)) != want]
    if bad:
        print("check-wrapper-spread: self-test FAILED:\n  " + "\n  ".join(bad), file=sys.stderr)
        sys.exit(1)
    if not quiet:
        print("check-wrapper-spread: self-test OK")


args = sys.argv[1:]
if args == ["--self-test"]:
    self_test()
    sys.exit(0)
if args == ["--list"]:
    print(ALLOW.listing())
    sys.exit(0)
if any(a.startswith("--") for a in args):
    print("usage: check-wrapper-spread.sh [FILE ...] | --self-test | --list", file=sys.stderr)
    sys.exit(2)
files = args or sorted(glob.glob("preview/src/components/*/component.rs"))
if not files:
    print("check-wrapper-spread: no component files found (run from a checkout)", file=sys.stderr)
    sys.exit(2)
self_test(quiet=True)
errors = run(files, report_stale=not args)
if errors:
    print("\n".join(f"check-wrapper-spread: {e}" for e in errors), file=sys.stderr)
    sys.exit(1)
print(f"check-wrapper-spread: OK ({len(files)} files, {len(ALLOW)} explicit calls allowlisted)")
PYEOF
