#!/usr/bin/env bash
#
# check-native-slots.sh  (backlog row 444, phase 0.1; the class: dev-docs/research/sync-js-root-2026-10-26.md, "The vdom is a writer too")
#
# A native page-written slot (`value`, `checked`, `selected`, `open`, `hidden`) is written by the page (the user, `form.reset()`, autofill, form-state restoration, `beforematch`) and by the vdom's edit stream, and Dioxus remembers only its own last write, so a re-render rewrites the page's level or skips a write the page changed. This gate finds every Rust site that writes one of those five slots on a page-writable element, in rsx (`input { value: x }`, a conditional or shorthand attribute inside the block), in `Attribute::new("value", ..)` and in `attributes!(..)`, and holds the sites in a shrink-only baseline until the mount-frozen helper exists.
#
# THE CLASS: lost keystrokes under async results, text that reappears after a form reset, text typed before hydration wiped on the SSG site and an `until-found` veto that shows a panel open while `aria-expanded` is false are one defect: a native slot written from a template that the vdom diffs. No helper exists yet that writes such a slot once at mount and leaves it to the page, so every shipped site is in the baseline and a NEW site fails; sites leave the baseline as the helper lands and each port lowers its file's count.
#
# THE CLOSED TABLE (the root doc names these elements and no other; the slots are the ones it ties to each): `input` value and checked, `textarea` and `select` value, `option` selected (a static `option { value: "a" }` is a label, not a slot: only `selected` is volatile on option), `dialog` and `details` open, and `hidden` on any of them. `hidden` is also counted in its upper-case spelling `HIDDEN` on any element (the repo's `until-found` attribute, written by `beforematch` on any panel), and a write through `Attribute::new("<slot>", ..)` or a bare `attributes!(..)` is counted whatever the element, because the element is not visible there. A `Attribute::new` whose name is not a literal is not seen.
#
# WHAT IS READ: Rust files of `git ls-files` outside docs/, playwright/, scripts/, dev-docs/, vendored trees and `#[cfg(test)]` modules, in the code view of scripts/comment-scan.py (`code_of`: comments removed, string contents blanked), so a slot named in a comment or a string is not a site. Only attributes of the element's own block count: a child element's block is skipped (its own opener is found on its own), a conditional (`if c { value: x }`) is not.
#
# THE RATCHET (scripts/lib/ratchet.py): `scripts/check-native-slots.baseline.tsv` is `path<TAB>value<TAB>checked<TAB>selected<TAB>open<TAB>hidden`, files with a non-zero count only. A count that rises, a new file with a site and a count that fell without being written back all fail; the tree's baseline is also compared with the merge base's, so editing the baseline in the commit is not a way through. A deliberate raise is `--update-baseline --allow-raise "row N reason"`.
#
# Usage:
#   scripts/check-native-slots.sh                          check (the self-test rides along)
#   scripts/check-native-slots.sh --inventory              every site: file:line element attribute
#   scripts/check-native-slots.sh --report                 baseline sites by element and by attribute
#   scripts/check-native-slots.sh --update-baseline [--allow-raise "row N reason"]
#   scripts/check-native-slots.sh --self-test              a planted `input { value: x }`, an `Attribute::new("checked", ..)`, a
#       conditional `open`, a shorthand and an `attributes!` write must be red; comments, strings, child blocks, other attributes and test modules green
# Environment: RATCHET_BASE_REF pins the merge base, RATCHET_NO_BASE=1 skips that half (scripts/lib/ratchet.py).
# Exit status: 0 clean; 1 a count rose, fell unwritten or is stale; 2 bad usage or a broken self-test.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac

exec python3 -I - "$@" <<'PYEOF'
import collections
import importlib.util
import os
import re
import subprocess
import sys
import tempfile

HERE = os.getcwd()
sys.path.insert(0, os.path.join(HERE, "scripts", "lib"))
from ratchet import Ratchet  # noqa: E402

_spec = importlib.util.spec_from_file_location("comment_scan", os.path.join(HERE, "scripts", "comment-scan.py"))
cs = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cs)

GATE = "check-native-slots"
BASELINE = "scripts/check-native-slots.baseline.tsv"
COLUMNS = ("value", "checked", "selected", "open", "hidden")
SLOTS = {"input": ("value", "checked", "hidden"), "textarea": ("value", "hidden"), "select": ("value", "hidden"),
         "option": ("selected", "hidden"), "dialog": ("open", "hidden"), "details": ("open", "hidden")}
ELEMENTS = tuple(SLOTS)
EXCLUDE = ("docs/", "playwright/", "scripts/", "dev-docs/", "primitives/src/js/", "target/", "node_modules/")
EXCLUDE_PARTS = ("/node_modules/", "/vendor/", "/vendored/", "/target/")
ADVICE = ("A native slot is written once at mount by the mount-frozen helper and then left to the page (dev-docs/research/"
          "sync-js-root-2026-10-26.md, \"The vdom is a writer too\"); until that helper exists, add no site, lower a count, "
          "or raise on purpose with --update-baseline --allow-raise \"row N reason\".")
ATTR = "value|checked|selected|open|hidden"
CONTROL = r"(?:if|else|for|while|match|loop|let|fn|struct|enum|mod|impl|trait|type|in|use|pub|return)"
OPENER_RX = re.compile(r"(?<![\w.:#])(" + "|".join(ELEMENTS) + r")\s*\{")
ATTR_RX = re.compile(r"(?<![\w.:#])(?:r#)?(" + ATTR + r")(?!\w)\s*(:(?!:)|(?=[,}]))", re.I)
QUOTED_RX = re.compile(r'"( +)"\s*:(?!:)')
NEW_RX = re.compile(r'\bAttribute\s*::\s*new\s*\(\s*"(\w+)"')
MACRO_RX = re.compile(r"\battributes\s*!\s*[(\[{]")
HEAD_RX = re.compile(r"\A\s*(?:else\b|" + CONTROL + r"\b)")
TEST_MOD = re.compile(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{")
TEST_FILE = re.compile(r"(?:^|/)(?:tests?|[\w]*_tests?)\.rs$|/tests/")


def total_of(c):
    return sum(c.values())


def ratchet(root):
    return Ratchet(GATE, BASELINE, COLUMNS, root=root, total=total_of, advice=ADVICE)


def universe(root):
    out = subprocess.run(["git", "-C", root, "ls-files", "-z"], capture_output=True, check=True).stdout.decode()
    return sorted(p for p in out.split("\0")
                  if p.endswith(".rs") and not p.startswith(EXCLUDE) and not any(s in "/" + p for s in EXCLUDE_PARTS)
                  and os.path.isfile(os.path.join(root, p)))


def line_at(text, off):
    return text.count("\n", 0, off) + 1


def match_close(code, open_idx):
    """Index of the bracket that closes the one at `open_idx` (the end of the text when unbalanced)."""
    pairs = {"{": "}", "(": ")", "[": "]"}
    stack = []
    for k in range(open_idx, len(code)):
        ch = code[k]
        if ch in pairs:
            stack.append(pairs[ch])
        elif stack and ch == stack[-1]:
            stack.pop()
            if not stack:
                return k
    return len(code)


def statement_head(code, i):
    """The text from the last `{`, `}` or `;` before `i` up to `i`: what the statement around `i` starts with."""
    k = i
    while k > 0 and code[k - 1] not in "{};":
        k -= 1
    return code[k:i]


def child_blocks(code, a, b):
    """Ranges of the blocks inside code[a:b] (a body) that are child elements, components or expression bodies:
    an identifier-or-path followed by `{`, unless the statement around it starts with a control keyword (`if c {`,
    `else {`, `for x in y {`, `match v {`), whose braces hold more attributes of the same element."""
    out = []
    k = a
    while k < b:
        if code[k] == "{":
            before = code[:k].rstrip()
            if before and (before[-1].isalnum() or before[-1] in "_>") and not HEAD_RX.match(statement_head(code, k)):
                end = match_close(code, k)
                out.append((k, end))
                k = end
        k += 1
    return out


def attr_sites(code, keep, a, b, skip):
    """`(offset, attribute)` of every slot attribute written in code[a:b] outside the `skip` ranges."""
    out = []

    def free(o):
        return not any(s <= o <= e for s, e in skip)

    for m in ATTR_RX.finditer(code, a, b):
        if free(m.start()) and code[:m.start()].rstrip()[-1:] in ("{", ",", "(", "["):
            out.append((m.start(1), m.group(1).lower()))
    for m in QUOTED_RX.finditer(code, a, b):
        name = keep[m.start(1):m.end(1)].strip().lower()
        if name in COLUMNS and free(m.start()):
            out.append((m.start(1), name))
    return out


def test_ranges(code, path):
    if TEST_FILE.search(path):
        return [(0, len(code))]
    out = []
    for m in TEST_MOD.finditer(code):
        out.append((m.start(), match_close(code, m.end() - 1)))
    return out


def measure_rust(text, path=""):
    """`(counts, sites)`; a site is `(line, element, attribute)`."""
    code = cs.code_of(text)
    keep = cs.code_of(text, keep_strings=True)
    tests = test_ranges(code, path)
    found = []
    for m in OPENER_RX.finditer(code):
        if HEAD_RX.match(statement_head(code, m.start(1))):
            continue
        end = match_close(code, m.end() - 1)
        skip = child_blocks(code, m.end(), end)
        for off, attr in attr_sites(code, keep, m.end(), end, skip):
            if attr in SLOTS[m.group(1)]:
                found.append((off, m.group(1), attr))
    # `hidden` in its upper-case spelling on any element: the repo's until-found attribute
    for m in re.finditer(r"(?<![\w.:#])HIDDEN\s*:(?!:)", code):
        found.append((m.start(), "*", "hidden"))
    for m in NEW_RX.finditer(keep):
        name = m.group(1).lower()
        if name in COLUMNS:
            found.append((m.start(), "Attribute::new", name))
    for m in MACRO_RX.finditer(code):
        end = match_close(code, m.end() - 1)
        skip = child_blocks(code, m.end(), end)
        for off, attr in attr_sites(code, keep, m.end(), end, skip):
            found.append((off, "attributes!", attr))
    # drop the same offset reached twice, and anything in a test module
    seen, sites = set(), []
    for off, el, attr in sorted(found):
        if (off, attr) in seen or any(a <= off <= b for a, b in tests):
            continue
        seen.add((off, attr))
        sites.append((line_at(text, off), el, attr))
    c = dict.fromkeys(COLUMNS, 0)
    for _, _, attr in sites:
        c[attr] += 1
    return c, sites


def measure(root):
    out, all_sites = {}, {}
    for path in universe(root):
        with open(os.path.join(root, path), encoding="utf-8", errors="replace") as f:
            c, sites = measure_rust(f.read(), path)
        if any(c.values()):
            out[path] = c
            all_sites[path] = sites
    return out, all_sites


def cmd_check(root):
    measured, _ = measure(root)
    R = ratchet(root)
    rep = R.check(measured)
    text = R.format(rep)
    if rep.errors:
        print(text, file=sys.stderr)
        print(f"{GATE}: FAIL -- {len(rep.errors)} problem(s). {ADVICE}", file=sys.stderr)
        return 1
    if text:
        print(text)
    s = {k: sum(c[k] for c in measured.values()) for k in COLUMNS}
    print(f"{GATE}: OK ({len(measured)} files, {sum(s.values())} baselined sites: " + ", ".join(f"{k} {v}" for k, v in s.items()) + ")")
    return 0


def cmd_inventory(root):
    _, sites = measure(root)
    for path in sorted(sites):
        for line, el, attr in sites[path]:
            print(f"{path}:{line}: {el} {attr}")
    return 0


def cmd_report(root):
    _, sites = measure(root)
    pairs = collections.Counter((el, attr) for p in sites for _, el, attr in sites[p])
    print(f"{GATE} --report: sites by element and attribute")
    for (el, attr), n in sorted(pairs.items()):
        print(f"  {el:16} {attr:10} {n}")
    print(f"  total {sum(pairs.values())} in {len(sites)} files")
    return 0


def cmd_update(root, reason):
    measured, _ = measure(root)
    errors, wrote = ratchet(root).update(measured, reason)
    for e in errors:
        print(e, file=sys.stderr)
    if wrote:
        print(f"{GATE}: wrote {BASELINE} ({len(measured)} files)")
    return 0 if wrote else 1


def self_test():
    fails = []
    n_ok = 0

    def case(label, src, want):
        nonlocal n_ok
        c, sites = measure_rust(src)
        got = sorted((el, attr) for _, el, attr in sites)
        if got != sorted(want):
            fails.append(f"{label}: want {sorted(want)}, got {got}")
        else:
            n_ok += 1

    case("planted input value", "fn v() -> Element { rsx! { input { value: x } } }", [("input", "value")])
    case("input checked and class", 'fn v() -> Element { rsx! { input { class: "a", checked: c, r#type: "checkbox" } } }', [("input", "checked")])
    case("textarea value", "rsx! { textarea { value: x, rows: 3 } }", [("textarea", "value")])
    case("option selected, its value is a label", "rsx! { select { option { selected: s, value: v } } }", [("option", "selected")])
    case("details open", "rsx! { details { open: o, summary { \"a\" } } }", [("details", "open")])
    case("dialog open", "rsx! { dialog { open: true } }", [("dialog", "open")])
    case("conditional attribute", "rsx! { input { if c { value: x } } }", [("input", "value")])
    case("shorthand attribute", "rsx! { input { value, class: \"a\" } }", [("input", "value")])
    case("quoted key", 'rsx! { input { "value": x } }', [("input", "value")])
    case("Attribute::new literal", 'fn a() -> Attribute { Attribute::new("checked", v, None, false) }', [("Attribute::new", "checked")])
    case("Attribute::new HIDDEN", 'fn a() -> Attribute { Attribute::new("HIDDEN", v, None, false) }', [("Attribute::new", "hidden")])
    case("attributes! write", "fn a() { let x = attributes!(input { value: v }); }", [("attributes!", "value")])
    case("HIDDEN on a div", "rsx! { div { HIDDEN: h } }", [("*", "hidden")])
    case("control: a slot named in a comment", "// input { value: x }\nfn f() {}\n", [])
    case("control: a slot named in a string", 'const S: &str = "input { value: x }";\n', [])
    case("control: a child block is the child's", "rsx! { input { class: \"a\" } div { value: x } }", [])
    case("control: a child inside the element", "rsx! { details { div { open: x } } }", [])
    case("control: a slot name as a value", "rsx! { input { aria_selected: selected, \"aria-expanded\": open, class: hidden } }", [])
    case("control: other attributes", 'rsx! { input { id: "a", name: "n", placeholder: p, disabled: d } }', [])
    case("control: a binding named like the element", "fn f(input: u8) { if input { let value: u8 = 1; } }", [])
    case("control: a test module", "#[cfg(test)]\nmod tests { fn t() { rsx! { input { value: x } } } }\n", [])
    case("control: `if input {`", "fn f() { if input { value: 1 } }", [])

    # the ratchet half: a new site in a clean tree fails, an unchanged tree passes
    git_env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid", "GIT_COMMITTER_NAME": "t",
               "GIT_COMMITTER_EMAIL": "t@example.invalid", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_SYSTEM": os.devnull}

    def run_git(root, *args):
        subprocess.run(["git", "-C", root, *args], check=True, capture_output=True, env=git_env)

    def tree(label, want, extra):
        nonlocal n_ok
        with tempfile.TemporaryDirectory(prefix="check-native-slots-st.") as root:
            os.makedirs(os.path.join(root, "primitives/src"))
            with open(os.path.join(root, "primitives/src/a.rs"), "w") as f:
                f.write("fn a() { rsx! { input { value: x } } }\n")
            run_git(root, "init", "-q")
            run_git(root, "add", "-A")
            os.environ["RATCHET_NO_BASE"] = "1"
            try:
                errors, wrote = ratchet(root).update(measure(root)[0], None)
                assert wrote, errors
                if extra:
                    with open(os.path.join(root, "primitives/src/b.rs"), "w") as f:
                        f.write(extra)
                    run_git(root, "add", "-A")
                rep = ratchet(root).check(measure(root)[0])
            finally:
                del os.environ["RATCHET_NO_BASE"]
            got = "red" if rep.errors else "green"
            if got != want:
                fails.append(f"{label}: want {want}, got {got}: {rep.errors}")
            else:
                n_ok += 1

    tree("control: an unchanged tree", "green", None)
    tree("a new file with a planted `input { value: x }`", "red", "fn b() { rsx! { input { value: x } } }\n")
    tree("a new file with a planted `Attribute::new(\"open\", ..)`", "red", 'fn b() { Attribute::new("open", v, None, false); }\n')

    if fails:
        print(f"{GATE}: SELF-TEST FAILED", file=sys.stderr)
        for f in fails:
            print("  " + f, file=sys.stderr)
        return 2
    print(f"{GATE}: self-test OK ({n_ok} scenarios)")
    return 0


def main(argv):
    root = os.getcwd()
    if not argv:
        rc = cmd_check(root)
        return rc if rc else self_test()
    if argv[0] == "--self-test" and len(argv) == 1:
        return self_test()
    if argv[0] == "--inventory" and len(argv) == 1:
        return cmd_inventory(root)
    if argv[0] == "--report" and len(argv) == 1:
        return cmd_report(root)
    if argv[0] == "--update-baseline":
        if len(argv) == 1:
            return cmd_update(root, None)
        if len(argv) == 3 and argv[1] == "--allow-raise":
            return cmd_update(root, argv[2])
    print(__doc__ or "usage: check-native-slots.sh [--inventory|--report|--self-test|--update-baseline [--allow-raise \"row N reason\"]]", file=sys.stderr)
    return 2


sys.exit(main(sys.argv[1:]))
PYEOF
