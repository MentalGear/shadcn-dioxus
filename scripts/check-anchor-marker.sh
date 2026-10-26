#!/usr/bin/env bash
#
# check-anchor-marker.sh  (backlog row 432)
#
# The anchor engine's stylesheet (`primitives/src/top_layer/styles.rs`) selects on ONE attribute, `data-dx-anchored`,
# which `anchored_content_attributes` (`primitives/src/top_layer/attributes.rs`) writes from an `AnchorKind`: an anchored
# overlay is anchored by calling the function it must already call for its `position-anchor`, and the engine names no
# overlay, so a new one cannot be left unpositioned because it is missing from a list. Before, the engine enumerated ten
# hand-kept `.dx-anchor-<kind>[popover]` marker classes in dozens of selector lines and a missing entry rendered
# un-anchored, a visual failure that only a browser spec found.
#
# This gate keeps the construction closed, statically:
#   1. pairing: a Rust file outside `top_layer/` that calls `use_anchor_position_fallback` calls
#      `anchored_content_attributes` at least as often (one call per anchored leaf), so no anchored panel opts into the
#      JS fallback without the attribute that opts it into the stylesheet;
#   2. no marker class: no `dx-anchor-<name>` class literal in Rust code, stylesheet code (comments are not read) or
#      markup outside a `top_layer/` directory (the `--dx-anchor-width` / `--dx-anchor-offset-*` custom properties are
#      not classes), and no `data-dx-anchored` WRITE in rsx outside `top_layer/` (one writer, like `data-dx-modality`);
#   3. the engine stylesheet enumerates nothing: every rule of `ANCHOR_POSITIONING_CSS_JS_LITERAL` is scoped by
#      `[data-dx-anchored`, names no `.dx-anchor-` class, and the only kind it names is `hover-card` (the one real
#      per-kind difference, a wider gap); every `AnchorKind` value other than that one is therefore reached by the
#      generic rules.
# A legitimate exception to 1 goes on ALLOW below with its reason; an entry that matches nothing fails as stale
# (scripts/lib/allow.py). The engine's stylesheet is read by scripts/lib/css.py, the one CSS reader of the gates.
# Browser half: `playwright/oracle/tier2-html/top-layer.spec.ts` and the per-component placement specs.
#
# Usage: scripts/check-anchor-marker.sh [--self-test]      (no network, no cargo)
# Exit status: 0 when clean (and, with --self-test, every mutation red).

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 -I - "$@" <<'PYEOF'
import glob
import os
import re
import sys

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import rules as css_rules, split_top, strip_comments  # noqa: E402

STYLES = "primitives/src/top_layer/styles.rs"
LITERAL = re.compile(r'const ANCHOR_POSITIONING_CSS_JS_LITERAL: &str = r#"`(.*?)`"#;', re.S)
ENGINE_ATTR = "[data-dx-anchored"
PER_KIND = {"hover-card"}

# (path, reason): a file outside top_layer/ whose use_anchor_position_fallback calls need not pair with
# anchored_content_attributes. Empty today: every JS-fallback consumer is anchored by the engine's stylesheet.
ALLOW = Allowlist("check-anchor-marker", [], fields=("path",))

MARKER_CLASS = re.compile(r"(?<![\w-])\.?dx-anchor-(?!width\b|offset-[xy]\b|positioning-styles\b)[a-z]")
WRITER = re.compile(r'"data-dx-anchored"\s*:')


def outside_top_layer(path):
    return "top_layer" not in path.split("/")[:-1]


def code_lines(text):
    """The lines of a Rust source that are not whole-line comments (`//`, `///`, `//!`)."""
    return [ln for ln in text.splitlines() if not ln.lstrip().startswith("//")]


def check_pairing(path, text, allow=ALLOW):
    code = "\n".join(code_lines(text))
    fb = len(re.findall(r"\buse_anchor_position_fallback\s*\(", code))
    attr = len(re.findall(r"\banchored_content_attributes\s*\(", code))
    if fb and attr < fb and not allow.find(lambda e: e.path == path):
        return [f"{path}: {fb} use_anchor_position_fallback call(s) but {attr} anchored_content_attributes call(s): an "
                f"anchored panel must write the data-dx-anchored attribute the engine selects on"]
    return []


def check_marker_class(path, text):
    out = []
    if path.endswith(".css"):
        text = strip_comments(text)
    elif path.endswith(".rs"):
        text = "\n".join(code_lines(text))
    for m in MARKER_CLASS.finditer(text):
        line = text.count("\n", 0, m.start()) + 1
        out.append(f"{path}:{line}: a dx-anchor-<name> marker class (use AnchorKind and the data-dx-anchored attribute)")
    if path.endswith(".rs"):
        for m in WRITER.finditer(text):
            line = text.count("\n", 0, m.start()) + 1
            out.append(f"{path}:{line}: writes data-dx-anchored outside top_layer/ (anchored_content_attributes is the one writer)")
    return out


def check_engine(text):
    m = LITERAL.search(text)
    if not m:
        return [f"{STYLES}: ANCHOR_POSITIONING_CSS_JS_LITERAL not found (the gate reads the engine stylesheet from it)"]
    rs = css_rules(m.group(1))
    if not rs:
        return [f"{STYLES}: the engine stylesheet parsed to no rules"]
    out = []
    for r in rs:
        sels = [s.strip() for s in split_top(r.prelude)]
        for sel in sels:
            if ENGINE_ATTR not in sel:
                out.append(f"{STYLES}:{r.line}: engine selector `{sel}` is not scoped by [data-dx-anchored]")
            if ".dx-anchor-" in sel:
                out.append(f"{STYLES}:{r.line}: engine selector `{sel}` names a marker class")
            for kind in re.findall(r'data-dx-anchored\s*=\s*"?([\w-]+)"?', sel):
                if kind not in PER_KIND:
                    out.append(f"{STYLES}:{r.line}: engine selector `{sel}` names the kind `{kind}`: only per-kind "
                               f"differences ({', '.join(sorted(PER_KIND))}) may; everything else is the generic rule")
    return out


def scan():
    errors = []
    rs_files = sorted(glob.glob("primitives/src/**/*.rs", recursive=True) + glob.glob("preview/src/**/*.rs", recursive=True)
                      + glob.glob("templates/**/*.rs", recursive=True))
    other = sorted(glob.glob("preview/src/**/*.css", recursive=True) + glob.glob("preview/assets/**/*.css", recursive=True)
                   + ["preview/index.html"])
    for p in rs_files:
        if not outside_top_layer(p):
            continue
        text = open(p, encoding="utf-8").read()
        errors += check_pairing(p, text)
        errors += check_marker_class(p, text)
    for p in other:
        if os.path.exists(p) and outside_top_layer(p):
            errors += check_marker_class(p, open(p, encoding="utf-8").read())
    errors += check_engine(open(STYLES, encoding="utf-8").read())
    return errors


def self_test():
    good_rs = "fn a() { use_anchor_position_fallback(a); let x = anchored_content_attributes(k, &id, attrs); }"
    engine = open(STYLES, encoding="utf-8").read()
    cases = [
        ("pairing: fallback without the attribute", check_pairing("p/x.rs", "fn a() { use_anchor_position_fallback(a); }", Allowlist("t", [], fields=("path",))), True),
        ("pairing: a paired file", check_pairing("p/x.rs", good_rs), False),
        ("pairing: a comment is not a call", check_pairing("p/x.rs", "// use_anchor_position_fallback(a)\nfn a() {}"), False),
        ("pairing: an allowlisted file", check_pairing("p/x.rs", "fn a() { use_anchor_position_fallback(a); }", Allowlist("t", [("p/x.rs", "reason")], fields=("path",))), False),
        ("marker class in rust code", check_marker_class("p/x.rs", 'let c = "dx-anchor-newthing";'), True),
        ("marker class in a rust comment", check_marker_class("p/x.rs", '// the old "dx-anchor-tooltip" class'), False),
        ("marker class in css code", check_marker_class("p/x.css", ".dx-anchor-foo[popover] { top: 0 }"), True),
        ("marker class in a css comment", check_marker_class("p/x.css", "/* .dx-anchor-foo was */ .a { top: 0 }"), False),
        ("anchor-width custom property", check_marker_class("p/x.css", ".a { min-width: var(--dx-anchor-width, 100%) }"), False),
        ("second writer in rsx", check_marker_class("p/x.rs", 'attributes!(div { "data-dx-anchored": "x" })'), True),
        ("engine as committed", check_engine(engine), False),
        ("engine rule that names a class", check_engine(engine.replace("[data-dx-anchored=\"hover-card\"][popover] {", ".dx-anchor-hover-card[popover] {", 1)), True),
        ("engine rule not scoped by the attribute", check_engine(engine.replace(":where([data-dx-anchored]:is([popover], :modal)) {", ":where([popover]) {", 1)), True),
        ("engine rule that names a kind", check_engine(engine.replace("[data-dx-anchored][popover][data-side=\"top\"][data-align=\"start\"]:dir(rtl)", "[data-dx-anchored=\"tooltip\"][popover][data-side=\"top\"][data-align=\"start\"]:dir(rtl)", 1)), True),
        ("engine literal missing", check_engine("fn nothing() {}"), True),
    ]
    bad = 0
    for name, errs, want_red in cases:
        ok = bool(errs) == want_red
        if not ok:
            bad += 1
            print(f"check-anchor-marker: self-test FAIL -- {name}: expected {'red' if want_red else 'green'}, got {errs[:2]}", file=sys.stderr)
    if bad:
        sys.exit(1)
    print(f"check-anchor-marker: self-test OK -- {len(cases)} cases")


if "--self-test" in sys.argv:
    self_test()
    sys.exit(0)
if "--list" in sys.argv:
    print(ALLOW.listing())
    sys.exit(0)

self_test()
errors = scan()
errors += ALLOW.problems()
if errors:
    print("check-anchor-marker: FAIL", file=sys.stderr)
    for e in errors:
        print("  " + e, file=sys.stderr)
    sys.exit(1)
print("check-anchor-marker: OK -- one attribute (data-dx-anchored), no marker class, engine stylesheet names no kind but hover-card")
PYEOF
