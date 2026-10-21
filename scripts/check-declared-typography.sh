#!/usr/bin/env bash
#
# check-declared-typography.sh  (backlog row 328, audit 2026-10-21 section 4)
#
# An in-flow component inherits font-family, colour and direction from its host on purpose. Its text SIZE and
# LINE-HEIGHT are part of the design, so the component's root has to declare them: a root that does not looks right
# only because the page around it happens to be 16px / 1.5 (Select's scroll-button chevrons were 14px inside a
# small card footer; row 259 closed that for popups, this gate is the in-flow half).
#
# WHAT IT CHECKS. For every component directory under preview/src/components whose component.rs renders the
# namesake root (`dx-<dir with - for _>`, e.g. `dx-tabs`; the component's outermost class, 64 of 83 components),
# the root's own rules in style.css must declare
#
#   font-size    through a theme token or slot variable (any `var(...)`; `font:` shorthand counts; `inherit`,
#                `unset` and a bare literal such as `0.8rem` do not), and
#   line-height  with any value but `inherit` (the theme has no line-height token yet, so a literal pairing such as
#                `1.25rem` for `--dx-text-sm` is what shadcn's `text-sm` is; a token is the follow-up).
#
# "Own rules" = a rule whose subject compound names the root as its only `dx-` class, ignoring `:hover`-style
# state rules (a hover size is not a base size).
#
# WHY ONLY THE NAMESAKE ROOT, AND NOT EVERY #[component]. The audit's first idea was "the outermost element of
# every #[component]". Measured (dev-docs/backlog/328, hostile-host probe over all 83 component pages): that is 405
# classes and 258 of them would fail, nearly all inert, because a part inherits from ITS component's root and
# should (Table's `dx-table-head` takes the size `dx-table` declares). The root is the question.
#
# POPUP ROOTS ARE OUT OF SCOPE: a `[popover]`/`<dialog>` root gets the theme's root typography from the one
# "Popup root" rule (row 259, `check-popup-root-reset.sh`), host-independent by construction. The namesake roots
# that are themselves the `<dialog>` panel (POPUP_ROOTS in this file: dialog, alert_dialog, sheet, drawer) are
# skipped; the other popup roots (`dx-select-list`, `dx-dropdown-menu-content`, ...) are not namesakes and are
# never checked.
#
# THE ESCAPES.
#   * Deliberate inheritance: a comment in the component's style.css or component.rs,
#         typography-inherit(dx-tabs, font-size): wraps the caller's panels, shadcn's Tabs declares no size either
#     (`(dx-root)` alone covers both properties; `, font-size` or `, line-height` names one). A reason is required.
#   * Debt: scripts/check-declared-typography.baseline.tsv, one `arm<TAB>component<TAB>class<TAB>evidence` per root
#     that existed when the gate was introduced (arm = `font-size` | `line-height`; evidence = `exposed` when the
#     hostile-host probe found a text or icon part of that component following the host, `inert` when it found none
#     because the text sits in parts that declare their own). SHRINK-ONLY and a DEBT REGISTER, NOT AN APPROVAL: the
#     gate fails on a root not in it, and on a stale row (a root that now declares the property, carries a marker,
#     is a popup, or no longer exists). `exposed` rows are the ones to burn down.
#
# SELF-TEST. The same evaluation runs on in-memory mutations (a declared size removed, a size turned into a
# literal, a baseline row left behind after the fix, a marker with no reason, a baseline row deleted) and each
# must turn the gate red.
#
# Usage: scripts/check-declared-typography.sh                check against the baseline
#        scripts/check-declared-typography.sh --inventory    every root with its status
#        scripts/check-declared-typography.sh --update-baseline   rewrite the baseline; refuses to ADD a row
#                                                            (--allow-grow to seed or to grow; say why in the commit)
# Exit status: 0 = clean; 1 = a violation / stale row / self-test not red; 2 = bad usage or parse failure.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mode=check
allow_grow=0
for arg in "$@"; do
    case "$arg" in
        --inventory) mode=inventory ;;
        --update-baseline) mode=update ;;
        --allow-grow) allow_grow=1 ;;
        -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-declared-typography: unknown option: $arg" >&2; exit 2 ;;
    esac
done

MODE="$mode" ALLOW_GROW="$allow_grow" python3 - <<'PYEOF'
import copy
import os
import re
import sys

COMPONENTS = "preview/src/components"
BASELINE = "scripts/check-declared-typography.baseline.tsv"
MODE = os.environ["MODE"]
ALLOW_GROW = os.environ.get("ALLOW_GROW") == "1"
ARMS = ("font-size", "line-height")
STATE = re.compile(r":(hover|focus|focus-visible|focus-within|active|disabled|checked|invalid|placeholder-shown|read-only|visited)\b")
MARKER = re.compile(r"typography-inherit\(\s*(dx-[a-z0-9-]+)\s*(?:,\s*(font-size|line-height)\s*)?\)\s*:\s*(.*)")
# Namesake roots that ARE the popup panel (the `<dialog>` the primitive renders carries the class). Listed, not
# guessed from nearby text: a Popover/ColorPicker/Toggle root merely sits NEXT TO a popup. Each needs its reason.
POPUP_ROOTS = {
    "dialog": "the class is on the `<dialog>` panel (data-slot dialog-content)",
    "alert_dialog": "the class is on the `<dialog>` panel the AlertDialogContent primitive renders",
    "sheet": "the class is on the `<dialog>` panel (data-slot sheet-content)",
    "drawer": "the class is on the `<dialog>` panel the DrawerContent primitive renders",
}


# ----------------------------------------------------------------------------- CSS (same walker as the other gates)
def strip_comments(css):
    return re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), css, flags=re.S)


def parse_rules(css):
    """[(prelude, [declaration text])] for every style rule; quote- and paren-aware."""
    css = strip_comments(css)
    rules, stack, buf = [], [], []
    quote, paren, i, n = None, 0, 0, len(css)
    while i < n:
        ch = css[i]
        if quote:
            buf.append(ch)
            if ch == "\\" and i + 1 < n:
                buf.append(css[i + 1])
                i += 1
            elif ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
            buf.append(ch)
        elif ch == "(":
            paren += 1
            buf.append(ch)
        elif ch == ")":
            paren = max(0, paren - 1)
            buf.append(ch)
        elif paren > 0:
            buf.append(ch)
        elif ch == "{":
            prelude = "".join(buf).strip()
            stack.append({"kind": "at" if prelude.startswith("@") else "rule", "prelude": prelude, "decls": []})
            buf = []
        elif ch == ";":
            decl = "".join(buf).strip()
            if stack and stack[-1]["kind"] == "rule" and decl:
                stack[-1]["decls"].append(decl)
            buf = []
        elif ch == "}":
            decl = "".join(buf).strip()
            if stack:
                top = stack.pop()
                if top["kind"] == "rule":
                    if decl:
                        top["decls"].append(decl)
                    if not any(s["kind"] == "at" and "keyframes" in s["prelude"] for s in stack):
                        rules.append((top["prelude"], top["decls"]))
            buf = []
        else:
            buf.append(ch)
        i += 1
    return rules


def split_top(s, sep):
    out, depth, cur, quote = [], 0, [], None
    for ch in s:
        if quote:
            cur.append(ch)
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
            cur.append(ch)
        else:
            if ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            if ch == sep and depth == 0:
                out.append("".join(cur))
                cur = []
            else:
                cur.append(ch)
    out.append("".join(cur))
    return out


def subject(sel):
    """Rightmost compound of a complex selector (combinators outside parentheses and brackets)."""
    depth, start = 0, 0
    sel = sel.strip()
    for i, ch in enumerate(sel):
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        elif depth == 0 and ch in " >+~":
            start = i + 1
    return sel[start:]


def only_dx_class(compound):
    bare = re.sub(r":(?:not|where|is|has)\((?:[^()]|\([^()]*\))*\)", "", compound)
    found = re.findall(r"\.(dx-[A-Za-z0-9_-]+)", bare)
    return found[0] if len(found) == 1 else None


def size_decls(rules, root):
    """{prop: [value]} declared for `root` by non-state rules whose subject is the root alone."""
    got = {p: [] for p in ARMS}
    for prelude, decls in rules:
        for sel in split_top(prelude, ","):
            subj = subject(sel)
            if only_dx_class(subj) != root or STATE.search(subj):
                continue
            for d in decls:
                k, _, v = d.partition(":")
                k, v = k.strip().lower(), v.strip()
                if k == "font-size":
                    got["font-size"].append(v)
                elif k == "font":
                    # `font: inherit` is not a declaration; a real `font` shorthand carries a size.
                    got["font-size"].append(v)
                elif k == "line-height":
                    got["line-height"].append(v)
    return got


NOT_A_DECL = {"inherit", "initial", "unset", "revert", "revert-layer"}


def arm_ok(arm, values):
    vals = [v for v in values if v.lower() not in NOT_A_DECL and not v.lower().startswith("inherit ")]
    if arm == "font-size":
        return any("var(" in v for v in vals)
    return bool(vals)


# ----------------------------------------------------------------------------- the model
def load():
    comps = {}
    for d in sorted(os.listdir(COMPONENTS)):
        css_p, rs_p = f"{COMPONENTS}/{d}/style.css", f"{COMPONENTS}/{d}/component.rs"
        if not (os.path.isfile(css_p) and os.path.isfile(rs_p)):
            continue
        with open(css_p, encoding="utf-8") as f:
            css = f.read()
        with open(rs_p, encoding="utf-8") as f:
            rs = f.read()
        comps[d] = {"css": css, "rs": rs}
    return comps


def root_info(d, c):
    """(root class or None, is_popup)."""
    root = "dx-" + d.replace("_", "-")
    lines = c["rs"].split("\n")
    hit = [i for i, ln in enumerate(lines) if re.search(r'"[^"]*(?<![\w-])' + re.escape(root) + r'(?![\w-])[^"]*"', ln) and not ln.lstrip().startswith("//")]
    if not hit:
        return None, False
    return root, d in POPUP_ROOTS


def markers(c):
    """{(root, arm|None): reason} from comments in style.css and component.rs."""
    out = {}
    for text in (c["css"], c["rs"]):
        for ln in text.split("\n"):
            m = MARKER.search(ln)
            if m:
                reason = re.sub(r"\*/\s*$", "", m.group(3)).strip()
                out[(m.group(1), m.group(2))] = reason
    return out


def read_baseline():
    rows = {}
    if os.path.isfile(BASELINE):
        for ln in open(BASELINE, encoding="utf-8").read().split("\n"):
            if ln.strip() and not ln.startswith("#"):
                p = ln.split("\t")
                if len(p) != 4 or p[0] not in ARMS or p[3] not in ("exposed", "inert"):
                    print(f"check-declared-typography: malformed baseline row: {ln!r}", file=sys.stderr)
                    sys.exit(2)
                rows[(p[0], p[1], p[2])] = p[3]
    return rows


MIN_REASON = 12


def evaluate(comps, baseline):
    """-> (status {(arm, dir, root): 'ok'|'marked'|'bad'}, new violations, stale baseline rows, errors, stats)."""
    status, errors, popup_roots, noroot = {}, [], 0, 0
    for d, reason in POPUP_ROOTS.items():
        if d not in comps or root_info(d, comps[d])[0] is None:
            errors.append(f"POPUP_ROOTS lists `{d}`, which has no namesake root in its component.rs (stale entry)")
    for d, c in comps.items():
        root, popup = root_info(d, c)
        if root is None:
            noroot += 1
            continue
        if popup:
            popup_roots += 1
            continue
        got = size_decls(parse_rules(c["css"]), root)
        mk = markers(c)
        for (mroot, marm), reason in mk.items():
            if mroot != root:
                errors.append(f"{d}: typography-inherit names `{mroot}`, which is not this component's root `{root}`")
            elif len(reason) < MIN_REASON:
                errors.append(f"{d}: typography-inherit({mroot}{', ' + marm if marm else ''}) needs a reason (>= {MIN_REASON} characters)")
        for arm in ARMS:
            if arm_ok(arm, got[arm]):
                status[(arm, d, root)] = "ok"
            elif len(mk.get((root, None), "")) >= MIN_REASON or len(mk.get((root, arm), "")) >= MIN_REASON:
                status[(arm, d, root)] = "marked"
            else:
                status[(arm, d, root)] = "bad"
    new = sorted(k for k, v in status.items() if v == "bad" and k not in baseline)
    stale = sorted(k for k in baseline if status.get(k) != "bad")
    return status, new, stale, errors, {"roots": len(status) // len(ARMS), "popup": popup_roots, "noroot": noroot}


def report(new, stale, errors, out):
    ok = True
    if errors:
        ok = False
        print("check-declared-typography: FAILED -- bad typography-inherit marker:", file=out)
        for e in errors:
            print(f"  {e}", file=out)
    if new:
        ok = False
        print("check-declared-typography: FAILED -- a component root inherits its host's typography:", file=out)
        for arm, d, root in new:
            what = "a token or slot `font-size: var(--dx-text-sm)`" if arm == "font-size" else "a `line-height` (e.g. `1.25rem` beside `--dx-text-sm`)"
            print(f"  {d}: `.{root}` declares no {arm}; give it {what}, or mark deliberate inheritance", file=out)
        print("Declare it in the root's rule in style.css. Deliberate inheritance (a slot that wraps the caller's content) takes", file=out)
        print("`/* typography-inherit(<root>[, font-size|line-height]): <reason> */` in the sheet. The baseline only shrinks.", file=out)
    if stale:
        ok = False
        print("check-declared-typography: FAILED -- stale baseline rows (the root now declares it, is marked, is a popup or is gone; delete them with --update-baseline):", file=out)
        for arm, d, root in stale:
            print(f"  {arm}\t{d}\t{root}", file=out)
    return ok


# ----------------------------------------------------------------------------- self-test
def self_test(comps, baseline):
    status, *_ = evaluate(comps, baseline)
    results = []

    def red(label, mutated_comps, mutated_baseline):
        _, new, stale, errors, _ = evaluate(mutated_comps, mutated_baseline)
        results.append((label, bool(new or stale or errors)))

    passing = [k for k, v in status.items() if v == "ok"]

    def plain_rule(key):
        """The body of the rule whose whole selector is `.root` (a mutation can then edit exactly the root's own rule)."""
        return re.search(r"(?:^|\})\s*\." + re.escape(key[2]) + r"\s*\{([^}]*)\}", strip_comments(comps[key[1]]["css"]))

    fs_target = next((k for k in passing if k[0] == "font-size" and (m := plain_rule(k)) and re.search(r"font-size:\s*var\(", m.group(1))), None)
    lh_target = next((k for k in passing if k[0] == "line-height" and (m := plain_rule(k)) and re.search(r"line-height:", m.group(1))), None)
    bad_target = next((k for k, v in status.items() if v == "bad" and k in baseline), None)
    if not (fs_target and lh_target and bad_target):
        print("check-declared-typography: self-test could not find mutation targets", file=sys.stderr)
        sys.exit(1)

    def mutate_root_decl(key, pattern, repl):
        """Rewrite `pattern` in EVERY rule body whose selector names the root (a root may declare it in several rules)."""
        m = copy.deepcopy(comps)
        css = strip_comments(m[key[1]]["css"])
        token = re.compile(r"\." + re.escape(key[2]) + r"(?![\w-])")
        changed = [False]

        def sub(match):
            if not token.search(match.group(1)) or STATE.search(match.group(1)):
                return match.group(0)
            body = re.sub(pattern, repl, match.group(2))
            changed[0] = changed[0] or body != match.group(2)
            return match.group(1) + "{" + body + "}"

        m[key[1]]["css"] = re.sub(r"([^{}]*)\{([^{}]*)\}", sub, css)
        return m if changed[0] else None

    m1 = mutate_root_decl(fs_target, r"font-size:\s*var\([^;]*;", "")
    m2 = mutate_root_decl(fs_target, r"font-size:\s*var\([^;]*;", "font-size: 14px;")
    m3 = mutate_root_decl(lh_target, r"line-height:[^;]*;", "")
    for label, m in (("a declared font-size removed", m1), ("a font-size turned into a literal", m2), ("a declared line-height removed", m3)):
        if m is None:
            print(f"check-declared-typography: self-test could not build the mutation `{label}`", file=sys.stderr)
            sys.exit(1)
        red(label, m, baseline)
    # a baseline row left behind after the root was fixed
    stale_b = dict(baseline)
    stale_b[fs_target] = "exposed"
    red("a baseline row left behind after the fix", comps, stale_b)
    # a baseline row deleted while the root is still bad
    short = {k: v for k, v in baseline.items() if k != bad_target}
    red("a baseline row deleted", comps, short)
    # a marker with no reason
    m6 = copy.deepcopy(comps)
    m6[bad_target[1]]["css"] += f"\n/* typography-inherit({bad_target[2]}): ok */\n"
    red("a marker with no reason", m6, baseline)
    # a marker naming another component's root
    m7 = copy.deepcopy(comps)
    m7[bad_target[1]]["css"] += "\n/* typography-inherit(dx-not-this-root): deliberate, wraps caller content */\n"
    red("a marker naming a foreign root", m7, baseline)
    return results


# ----------------------------------------------------------------------------- main
try:
    comps = load()
    baseline = read_baseline()
    status, new, stale, errors, stats = evaluate(comps, baseline)
except Exception as e:  # a parse failure must not read as "clean"
    print(f"check-declared-typography: internal error: {e!r}", file=sys.stderr)
    sys.exit(2)

if MODE == "inventory":
    for (arm, d, root), v in sorted(status.items(), key=lambda kv: (kv[0][1], kv[0][0])):
        print(f"{d}\t{root}\t{arm}\t{v}\t{baseline.get((arm, d, root), '')}")
    print(f"# {stats['roots']} roots, {stats['popup']} popup roots skipped, {stats['noroot']} components without a namesake root", file=sys.stderr)
    sys.exit(0)

if MODE == "update":
    cur = {k: baseline.get(k) for k, v in status.items() if v == "bad"}
    added = [k for k, v in cur.items() if v is None]
    if (added and baseline) and not ALLOW_GROW:
        print("check-declared-typography: refusing to ADD baseline rows (shrink-only); fix the root or mark it. Rows:", file=sys.stderr)
        for k in added:
            print(f"  {k}", file=sys.stderr)
        sys.exit(1)
    # evidence for a new row comes from the probe: default `exposed` unless given by the caller's TYPO_INERT list
    inert = set(filter(None, os.environ.get("TYPO_INERT", "").split(",")))
    with open(BASELINE, "w", encoding="utf-8") as f:
        f.write("# check-declared-typography.sh baseline (row 328): a component root that still inherits its host's font-size / line-height.\n")
        f.write("# arm<TAB>component<TAB>class<TAB>evidence. Shrink-only debt register, not an approval; see the gate's header.\n")
        f.write("# evidence: `exposed` = a text/icon part of the component followed a hostile host in the 2026-10-21 probe; `inert` = none did.\n")
        for (arm, d, root) in sorted(cur, key=lambda k: (k[0], k[1])):
            ev = cur[(arm, d, root)] or ("inert" if f"{arm}:{d}" in inert else "exposed")
            f.write(f"{arm}\t{d}\t{root}\t{ev}\n")
    print(f"check-declared-typography: baseline rewritten ({len(cur)} rows)")
    sys.exit(0)

if not report(new, stale, errors, sys.stderr):
    sys.exit(1)

results = self_test(comps, baseline)
bad = [label for label, was_red in results if not was_red]
if bad:
    print("check-declared-typography: FAILED -- self-test: these mutations did NOT turn the gate red:", file=sys.stderr)
    for label in bad:
        print(f"  {label}", file=sys.stderr)
    sys.exit(1)

counts = {a: {"ok": 0, "marked": 0, "bad": 0} for a in ARMS}
for (arm, d, root), v in status.items():
    counts[arm][v] += 1
exposed = {a: sum(1 for (arm, d, r), ev in baseline.items() if arm == a and ev == "exposed") for a in ARMS}
parts = "; ".join(
    f"{a} {counts[a]['ok']} declared, {counts[a]['marked']} marked, {counts[a]['bad']} baseline ({exposed[a]} exposed)" for a in ARMS
)
print(f"check-declared-typography: OK -- {stats['roots']} component roots ({stats['popup']} popup roots left to the popup-root rule, {stats['noroot']} components without a namesake root): {parts}; self-test red on {len(results)}/{len(results)} mutations")
PYEOF
