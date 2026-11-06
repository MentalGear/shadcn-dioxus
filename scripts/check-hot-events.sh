#!/usr/bin/env bash
#
# check-hot-events.sh  (backlog row 444, phase 0.1; the rule: dev-docs/research/sync-js-root-2026-10-26.md, "Hot events stay off the vdom, counted per event name" and section 5.3 item 8)
#
# Dioxus registers ONE root listener per bubbling event name, with no options, and every listener is a blocking round trip on desktop: one `onpointermove`, `onwheel` or `onresize` anywhere makes that event a blocking, non-passive request across the whole app. So a frame-rate event never takes a vdom handler; it goes through the bridge (`listen`/`observe`/`sample`, async and batched). This gate counts the rsx handlers that exist, per event name, and lets the count only fall.
#
# THE HOT TABLE IS READ, NOT COPIED. The frame-rate event names are the rows of `primitives/src/ts/bridge.ts`'s `EVENTS` table that carry `hot: 1` (the flag `listen` already uses to coalesce to one message per animation frame); the gate parses that table and fails (exit 2) when it finds none. One more cost class is added by name: `onmounted`, which is not frame-rate but is a synchronous request from inside the edit application, one per mounted element (a per-row handle is a mounting batch of N round trips); the root doc lists it as a class of its own (section 1.4 item 6, section 5.3 item 8), so it is a column here, `mounted`. `ondrag` is not hot in `EVENTS`, so it is not counted.
#
# THE COMPANION TABLE. On Android a payload event travels in a request header of about 12-16 KB and a failed request silently loses the cancel (`native.ts` `handleVirtualdomEventSync`), so an rsx `prevent_default` on `submit`, `input`, `change`, `paste` or `drop` is not a candidate on mobile (js-interop.md, "platform rules"). Every rsx handler for one of those events whose body calls `prevent_default` (a closure, a block, or a named `let`/`fn` handler in the same file) is counted in the column `pd_<event>`.
#
# WHAT COUNTS. Rust files of `git ls-files` outside docs/, playwright/, scripts/, dev-docs/, vendored trees and `#[cfg(test)]` modules, in the code view of scripts/comment-scan.py (`code_of`: comments removed, string contents blanked), so an event named in a comment or a string is not a site. A site is an `onX: value` or shorthand `onX,` attribute in the block of a lowercase element (a control-flow block inside it still belongs to the element) and a call `events::onX(..)`. A component's props (a block opened by a PascalCase name), struct fields, fn parameters and `let onX = ..` bindings are not sites: the listener exists where an element receives the handler.
#
# THE RATCHET AND THE ALLOWLIST. `scripts/check-hot-events.baseline.tsv` is `path<TAB>count per column` (a `#! columns:` header; files with a non-zero count only; scripts/lib/ratchet.py): a count that rises, a new file with a site, a count that fell without being written back and a stale row all fail, and the tree's baseline is compared with the merge base's, so editing the baseline in the commit is not a way through. The allowlist is per EVENT NAME, not per site (the cost class is the name): `scripts/check-hot-events.allow.tsv` is `event<TAB>reason<TAB>row`, where `event` is a column (`resize`, `mounted`, `pd_submit`, ...), the reason says why the name is still on the vdom, and the row is the open backlog row that drains it. A column with a measured site and no row is red (a new event name is the owner's decision); a row whose column has no site, has no reason, names a missing backlog row or names no column is red too.
#
# Usage:
#   scripts/check-hot-events.sh                          check (the self-test rides along)
#   scripts/check-hot-events.sh --inventory              every site: file:line column
#   scripts/check-hot-events.sh --report                 sites by column
#   scripts/check-hot-events.sh --list                   the allowlist
#   scripts/check-hot-events.sh --update-baseline [--allow-raise "row N reason"]
#   scripts/check-hot-events.sh --self-test              a planted `onwheel:`, a planted `onsubmit` with `prevent_default` and no allowlist row, a stale row and
#       a baseline raise must be red; comments, strings, component props, struct fields, bindings and test modules green
# Environment: RATCHET_BASE_REF pins the merge base, RATCHET_NO_BASE=1 skips that half (scripts/lib/ratchet.py).
# Exit status: 0 clean; 1 a count rose, fell unwritten, is stale or has no allowlist row; 2 bad usage, no hot table or a broken self-test.
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
sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from ratchet import Ratchet  # noqa: E402

_spec = importlib.util.spec_from_file_location("comment_scan", os.path.join(HERE, "scripts", "comment-scan.py"))
cs = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cs)

GATE = "check-hot-events"
BASELINE = "scripts/check-hot-events.baseline.tsv"
ALLOW_TSV = "scripts/check-hot-events.allow.tsv"
BRIDGE = "primitives/src/ts/bridge.ts"
EXTRA = ("mounted",)
PAYLOAD = ("submit", "input", "change", "paste", "drop")
EXCLUDE = ("docs/", "playwright/", "scripts/", "dev-docs/", "primitives/src/js/", "target/", "node_modules/")
EXCLUDE_PARTS = ("/node_modules/", "/vendor/", "/vendored/", "/target/")
ADVICE = ("A frame-rate event goes through the bridge (listen/observe/sample), not an rsx handler, and a payload event is "
          "cancelled declaratively or page-side (dev-docs/js-interop.md, dev-docs/research/sync-js-root-2026-10-26.md, "
          "\"Hot events stay off the vdom\"); add no site, lower a count, or raise on purpose with --update-baseline "
          "--allow-raise \"row N reason\".")
CONTROL_RX = re.compile(r"\b(?:if|else|for|while|match|loop)\b")
TEST_MOD = re.compile(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{")
TEST_FILE = re.compile(r"(?:^|/)(?:tests?|[\w]*_tests?)\.rs$|/tests/")
STRUCT_HEAD = re.compile(r"\b(?:struct|enum|union)\s+\w+[^{;]*$")
ENTRY_RX = re.compile(r'^\s*"([a-z]+)"\s*:\s*\{([^}]*)\}', re.M)
EVENTS_RX = re.compile(r"const\s+EVENTS\b[^=]*=\s*\{(.*?)^\};", re.S | re.M)


def hot_events(root):
    """The event names of bridge.ts's EVENTS table that carry `hot: 1`."""
    try:
        with open(os.path.join(root, BRIDGE), encoding="utf-8") as f:
            src = f.read()
    except OSError as e:
        print(f"{GATE}: cannot read {BRIDGE}: {e}", file=sys.stderr)
        sys.exit(2)
    table = EVENTS_RX.search(src)
    if not table:
        print(f"{GATE}: no `const EVENTS = {{ .. }};` table in {BRIDGE}", file=sys.stderr)
        sys.exit(2)
    hot = [m.group(1) for m in ENTRY_RX.finditer(table.group(1)) if re.search(r"\bhot\s*:\s*1\b", m.group(2))]
    if not hot:
        print(f"{GATE}: the EVENTS table in {BRIDGE} has no `hot: 1` row; the gate would count nothing", file=sys.stderr)
        sys.exit(2)
    return hot


def columns_of(hot):
    return tuple(hot) + EXTRA + tuple("pd_" + p for p in PAYLOAD)


def universe(root):
    out = subprocess.run(["git", "-C", root, "ls-files", "-z"], capture_output=True, check=True).stdout.decode()
    return sorted(p for p in out.split("\0")
                  if p.endswith(".rs") and not p.startswith(EXCLUDE) and not any(s in "/" + p for s in EXCLUDE_PARTS)
                  and os.path.isfile(os.path.join(root, p)))


def line_at(text, off):
    return text.count("\n", 0, off) + 1


def match_close(code, open_idx):
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
    k = i
    while k > 0 and code[k - 1] not in "{};":
        k -= 1
    return code[k:i]


def enclosing(code, off):
    """Index of the nearest unmatched opening bracket before `off`, or -1."""
    closers = {"}": "{", ")": "(", "]": "["}
    stack = []
    k = off - 1
    while k >= 0:
        ch = code[k]
        if ch in closers:
            stack.append(ch)
        elif ch in "{([":
            if stack and closers[stack[-1]] == ch:
                stack.pop()
            elif not stack:
                return k
        k -= 1
    return -1


def owner(code, off):
    """What the attribute at `off` belongs to: `element`, `component`, or `other` (struct field, parameters, a call)."""
    k = enclosing(code, off)
    for _ in range(64):
        if k < 0 or code[k] != "{":
            return "other"
        before = code[:k].rstrip()
        head = statement_head(code, k)
        if STRUCT_HEAD.search(head.rstrip() + " "):
            return "other"
        if before.endswith("=>") or before.endswith("|"):
            return "other"
        if CONTROL_RX.search(head):
            k = enclosing(code, k)
            continue
        m = re.search(r"([A-Za-z_][\w:]*)\s*$", before)
        if not m:
            return "other"
        last = m.group(1).split("::")[-1]
        if last[:1].isupper():
            return "component"
        return "element" if last[:1].islower() or last[:1] == "_" else "other"
    return "other"


def test_ranges(code, path):
    if TEST_FILE.search(path):
        return [(0, len(code))]
    return [(m.start(), match_close(code, m.end() - 1)) for m in TEST_MOD.finditer(code)]


def closure_end(code, i):
    """End offset (exclusive) of the expression that starts at `i` (a closure, block, path or call)."""
    n = len(code)
    depth = 0
    k = i
    while k < n:
        ch = code[k]
        if ch in "{([":
            depth += 1
        elif ch in "})]":
            if depth == 0:
                return k
            depth -= 1
        elif ch == "," and depth == 0:
            return k
        k += 1
    return n


def handler_text(code, off_after_colon):
    """The text of the handler expression that starts at `off_after_colon`."""
    m = re.compile(r"\s*").match(code, off_after_colon)
    i = m.end()
    if code.startswith("{", i):
        return code[i:match_close(code, i) + 1]
    # a closure: optional async/move, then |params|, then the body
    k = i
    cm = re.compile(r"(?:async\s+)?(?:move\s+)?\|").match(code, k)
    if cm:
        p = code.find("|", cm.end())
        if p < 0:
            return code[i:closure_end(code, i)]
        body = re.compile(r"\s*(?:->\s*[\w:<>&' ,\[\]()]+?\s*)?").match(code, p + 1).end()
        if code.startswith("{", body):
            return code[i:match_close(code, body) + 1]
        return code[i:closure_end(code, body)]
    return code[i:closure_end(code, i)]


def resolve_named(code, text):
    """A handler written as a bare name resolves to the `let`/`fn` of that name in the same file."""
    m = re.fullmatch(r"\s*([A-Za-z_]\w*)\s*", text)
    if not m:
        return text
    name = m.group(1)
    d = re.search(r"\b(?:let(?:\s+mut)?|fn)\s+" + re.escape(name) + r"\b", code)
    if not d:
        return text
    if code[d.end():].lstrip().startswith("(") or code[d.start():d.start() + 2] == "fn":
        end = code.find("{", d.end())
        return code[end:match_close(code, end) + 1] if end >= 0 else text
    eq = code.find("=", d.end())
    return handler_text(code, eq + 1) if eq >= 0 else text


def measure_rust(text, path, columns):
    """`(counts, sites)`; a site is `(line, column)`."""
    code = cs.code_of(text)
    tests = test_ranges(code, path)
    names = [c for c in columns if not c.startswith("pd_")]
    ev = {("on" + c): c for c in names}
    found = []
    attr = re.compile(r"(?<![\w.:#])(" + "|".join(sorted(set(ev) | {"on" + p for p in PAYLOAD}, key=len, reverse=True))
                      + r")(?!\w)\s*(:(?!:)|(?=[,}]))")
    for m in attr.finditer(code):
        prev = code[:m.start()].rstrip()[-1:]
        if prev not in ("{", ",", "(", "[") or owner(code, m.start()) != "element":
            continue
        name = m.group(1)
        colon = m.group(2) == ":"
        if name in ev:
            found.append((m.start(), ev[name]))
        if name[2:] in PAYLOAD and colon:
            body = resolve_named(code, handler_text(code, m.end()))
            if re.search(r"\bprevent_default\b", body):
                found.append((m.start(), "pd_" + name[2:]))
    for m in re.finditer(r"\bevents\s*::\s*(on\w+)\s*\(", code):
        if m.group(1) in ev:
            found.append((m.start(1), ev[m.group(1)]))
    seen, sites = set(), []
    for off, col in sorted(found):
        if (off, col) in seen or any(a <= off <= b for a, b in tests):
            continue
        seen.add((off, col))
        sites.append((line_at(text, off), col))
    c = dict.fromkeys(columns, 0)
    for _, col in sites:
        c[col] += 1
    return c, sites


def measure(root, columns):
    out, all_sites = {}, {}
    for path in universe(root):
        with open(os.path.join(root, path), encoding="utf-8", errors="replace") as f:
            c, sites = measure_rust(f.read(), path, columns)
        if any(c.values()):
            out[path] = c
            all_sites[path] = sites
    return out, all_sites


def ratchet(root, columns):
    return Ratchet(GATE, BASELINE, columns, root=root, total=lambda c: sum(c.values()), advice=ADVICE)


def read_allow(root):
    """`(entries, problems)`: (event, row, reason) tuples from the allowlist TSV."""
    entries, problems = [], []
    path = os.path.join(root, ALLOW_TSV)
    if not os.path.isfile(path):
        return entries, [f"{ALLOW_TSV} is missing"]
    with open(path, encoding="utf-8") as f:
        for n, ln in enumerate(f.read().split("\n"), 1):
            if not ln.strip() or ln.startswith("#"):
                continue
            parts = ln.split("\t")
            if len(parts) != 3 or not parts[2].strip().isdigit():
                problems.append(f"{ALLOW_TSV}:{n}: want event<TAB>reason<TAB>row (row a number)")
                continue
            entries.append((parts[0].strip(), parts[2].strip(), parts[1].strip()))
    return entries, problems


def allow_problems(root, columns, measured):
    entries, problems = read_allow(root)
    A = Allowlist(GATE, entries, fields=("event", "row"), source=ALLOW_TSV)
    totals = {c: sum(r[c] for r in measured.values()) for c in columns}
    for e in A:
        if e.event not in columns:
            problems.append(f"{ALLOW_TSV}: `{e.event}` is not a column of this gate ({', '.join(columns)})")
            continue
        if not any(os.path.isfile(os.path.join(root, "dev-docs/backlog", f)) for f in
                   (os.listdir(os.path.join(root, "dev-docs/backlog")) if os.path.isdir(os.path.join(root, "dev-docs/backlog")) else [])
                   if f.startswith(e.row + "-")):
            problems.append(f"{ALLOW_TSV}: `{e.event}` names backlog row {e.row}, which does not exist")
        if totals.get(e.event, 0):
            A.mark(e)
    problems += A.problems()
    listed = {e.event for e in A}
    for c in columns:
        if totals[c] and c not in listed:
            problems.append(f"{c}: {totals[c]} rsx site(s) and no row in {ALLOW_TSV}; a new event name on the vdom is the owner's decision "
                            f"(add `{c}<TAB>reason<TAB>row` only with the owner's approval)")
    return problems


def run_check(root, columns):
    measured, _ = measure(root, columns)
    R = ratchet(root, columns)
    rep = R.check(measured)
    errs = list(rep.errors) + allow_problems(root, columns, measured)
    return measured, rep, errs, R


def cmd_check(root, columns):
    measured, rep, errs, R = run_check(root, columns)
    text = R.format(rep)
    if errs:
        print(text, file=sys.stderr)
        for e in errs:
            if e not in text:
                print(e, file=sys.stderr)
        print(f"{GATE}: FAIL -- {len(errs)} problem(s). {ADVICE}", file=sys.stderr)
        return 1
    if text:
        print(text)
    s = {k: sum(c[k] for c in measured.values()) for k in columns}
    print(f"{GATE}: OK ({len(measured)} files, {sum(s.values())} baselined sites: " + ", ".join(f"{k} {v}" for k, v in s.items() if v) + ")")
    return 0


def cmd_inventory(root, columns):
    _, sites = measure(root, columns)
    for path in sorted(sites):
        for line, col in sites[path]:
            print(f"{path}:{line}: {col}")
    return 0


def cmd_report(root, columns):
    _, sites = measure(root, columns)
    n = collections.Counter(col for p in sites for _, col in sites[p])
    print(f"{GATE} --report: sites by column")
    for c in columns:
        print(f"  {c:16} {n.get(c, 0)}")
    print(f"  total {sum(n.values())} in {len(sites)} files")
    return 0


def cmd_list(root):
    entries, problems = read_allow(root)
    print(Allowlist(GATE, entries, fields=("event", "row"), source=ALLOW_TSV).listing())
    for p in problems:
        print(p, file=sys.stderr)
    return 1 if problems else 0


def cmd_update(root, columns, reason):
    measured, _ = measure(root, columns)
    errors, wrote = ratchet(root, columns).update(measured, reason)
    for e in errors:
        print(e, file=sys.stderr)
    if wrote:
        print(f"{GATE}: wrote {BASELINE} ({len(measured)} files)")
    return 0 if wrote else 1


def self_test():
    fails = []
    n_ok = 0
    cols = columns_of(["pointermove", "wheel", "scroll", "resize"])

    def case(label, src, want, path="a.rs"):
        nonlocal n_ok
        _, sites = measure_rust(src, path, cols)
        got = sorted(c for _, c in sites)
        if got != sorted(want):
            fails.append(f"{label}: want {sorted(want)}, got {got}")
        else:
            n_ok += 1

    case("planted onwheel", "fn v() -> Element { rsx! { div { onwheel: move |e| f(e) } } }", ["wheel"])
    case("onresize and onmounted", "rsx! { div { onresize: r, onmounted: m } }", ["resize", "mounted"])
    case("shorthand", "rsx! { div { class: \"a\", onscroll, } }", ["scroll"])
    case("conditional attribute", "rsx! { div { if c { onpointermove: h } } }", ["pointermove"])
    case("events:: call", "fn a(h: H) -> Attribute { dioxus::html::events::onwheel(h) }", ["wheel"])
    case("planted payload prevent_default", "rsx! { form { onsubmit: move |e| e.prevent_default() } }", ["pd_submit"])
    case("block closure prevent_default", "rsx! { input { onpaste: move |e: Event<ClipboardData>| { e.prevent_default(); } } }", ["pd_paste"])
    case("named handler prevent_default", "let h = move |e: FormEvent| { e.prevent_default(); };\nrsx! { input { oninput: h } }", ["pd_input"])
    case("fn handler prevent_default", "fn h(e: DragEvent) { e.prevent_default(); }\nrsx! { div { ondrop: h } }", ["pd_drop"])
    case("control: payload handler without prevent_default", "rsx! { input { oninput: move |e| s.set(e.value()) } }", [])
    case("control: a comment", "// div { onwheel: h }\nfn f() {}\n", [])
    case("control: a string", 'const S: &str = "div { onwheel: h }";\n', [])
    case("control: a component prop", "rsx! { Select { onscroll: h, onmounted: m } }", [])
    case("control: a struct field", "struct P { onwheel: Option<EventHandler<WheelEvent>>, onresize: u8 }", [])
    case("control: a fn parameter", "fn f(onresize: u8, onscroll: u8) {}", [])
    case("control: a binding", "fn f() { let onresize = move |i| i; }", [])
    case("control: a non-hot handler", "rsx! { div { onclick: h, onkeydown: k, onscrollend: s } }", [])
    case("control: a test module", "#[cfg(test)]\nmod tests { fn t() { rsx! { div { onwheel: h } } } }\n", [])

    git_env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid", "GIT_COMMITTER_NAME": "t",
               "GIT_COMMITTER_EMAIL": "t@example.invalid", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_SYSTEM": os.devnull}

    def run_git(root, *args):
        subprocess.run(["git", "-C", root, *args], check=True, capture_output=True, env=git_env)

    BRIDGE_SRC = ('const EVENTS: Record<string, { recv: Recv; hot?: 1 }> = {\n  "click": { recv: "wde" },\n'
                  '  "wheel": { recv: "wde", hot: 1 },\n  "resize": { recv: "w", hot: 1 },\n};\n')

    def tree(label, want, extra=None, allow=("resize",), baseline_edit=None, drop_hot=False):
        nonlocal n_ok
        with tempfile.TemporaryDirectory(prefix="check-hot-events-st.") as root:
            for d in ("primitives/src", "primitives/src/ts", "scripts", "dev-docs/backlog"):
                os.makedirs(os.path.join(root, d), exist_ok=True)
            with open(os.path.join(root, "dev-docs/backlog/460-x.md"), "w") as f:
                f.write("x\n")
            with open(os.path.join(root, BRIDGE), "w") as f:
                f.write(BRIDGE_SRC)
            with open(os.path.join(root, "primitives/src/a.rs"), "w") as f:
                f.write("fn a() { rsx! { div { onresize: r } } }\n")
            tcols = columns_of(hot_events(root))
            run_git(root, "init", "-q")
            run_git(root, "add", "-A")
            os.environ["RATCHET_NO_BASE"] = "1"
            try:
                errors, wrote = ratchet(root, tcols).update(measure(root, tcols)[0], None)
                assert wrote, errors
                with open(os.path.join(root, ALLOW_TSV), "w") as f:
                    f.write("".join(f"{e}\ttest reason\t460\n" for e in allow))
                if extra:
                    with open(os.path.join(root, "primitives/src/b.rs"), "w") as f:
                        f.write(extra)
                    run_git(root, "add", "-A")
                if baseline_edit:
                    bp = os.path.join(root, BASELINE)
                    with open(bp) as f:
                        t = f.read()
                    with open(bp, "w") as f:
                        f.write(baseline_edit(t))
                _, rep, errs, _ = run_check(root, tcols)
            finally:
                del os.environ["RATCHET_NO_BASE"]
            got = "red" if errs else "green"
            if got != want:
                fails.append(f"{label}: want {want}, got {got}: {errs}")
            else:
                n_ok += 1

    tree("control: an unchanged tree", "green")
    tree("a new file with a planted `onwheel:`", "red", "fn b() { rsx! { div { onwheel: h } } }\n", allow=("resize",))
    tree("a planted `onwheel:` even with an allowlist row, the baseline does not cover it", "red",
         "fn b() { rsx! { div { onwheel: h } } }\n", allow=("resize", "wheel"))
    tree("a planted payload prevent_default without its companion row", "red",
         "fn b() { rsx! { form { onsubmit: move |e| e.prevent_default() } } }\n", allow=("resize",))
    tree("a stale allowlist row", "red", allow=("resize", "wheel"))
    tree("an allowlist row for an unknown column", "red", allow=("resize", "nonsense"))
    tree("a baseline that falls without being written back", "red", None, allow=("resize",),
         baseline_edit=lambda t: t.replace("\t1", "\t2"))

    if fails:
        print(f"{GATE}: SELF-TEST FAILED", file=sys.stderr)
        for f in fails:
            print("  " + f, file=sys.stderr)
        return 2
    print(f"{GATE}: self-test OK ({n_ok} scenarios)")
    return 0


def main(argv):
    root = os.getcwd()
    if argv and argv[0] == "--self-test" and len(argv) == 1:
        return self_test()
    columns = columns_of(hot_events(root))
    if not argv:
        rc = cmd_check(root, columns)
        return rc if rc else self_test()
    if argv[0] == "--inventory" and len(argv) == 1:
        return cmd_inventory(root, columns)
    if argv[0] == "--report" and len(argv) == 1:
        return cmd_report(root, columns)
    if argv[0] == "--list" and len(argv) == 1:
        return cmd_list(root)
    if argv[0] == "--update-baseline":
        if len(argv) == 1:
            return cmd_update(root, columns, None)
        if len(argv) == 3 and argv[1] == "--allow-raise":
            return cmd_update(root, columns, argv[2])
    print("usage: check-hot-events.sh [--inventory|--report|--list|--self-test|--update-baseline [--allow-raise \"row N reason\"]]", file=sys.stderr)
    return 2


sys.exit(main(sys.argv[1:]))
PYEOF
