#!/usr/bin/env bash
#
# check-js-volume.sh  (backlog row 434; rule and replacements: dev-docs/js-interop.md)
#
# JavaScript volume can only shrink: Rust owns behaviour and browser access goes through the bridge in
# `primitives/src/bridge/` (dev-docs/js-interop.md; ref-ok: lands with row 434 lane 3.0), never through a script string, a new `.js`/`.ts` file or an inline
# script. The gate counts, per tracked file, the places JS can enter (eval sinks, JS held in Rust strings, hand-written
# `.js`/`.ts` lines, `JsListeners::install` sites, inline `<script>` bytes) and fails when a count rises, against the
# baseline AND against the baseline committed at the merge base with origin/main, so editing the baseline in the same
# commit is not a way through.
#
# THE CLASS: 103 bespoke eval blocks were written because the only written convention for "I need the DOM" was
# `document::eval`, every JS gate checked the SHAPE of a script string (and so blessed the string), and nothing
# stopped block 104. A token ban is beaten by `concat!("prevent", "Default()")`, `use ...::eval as run` or a
# `document::Script { "..." }`; this gate closes the sinks (the places a string becomes code) and measures the volume
# behind them, so a spelling trick still raises a count.
#
# WHAT IS COUNTED, per file (universe: `git ls-files` minus docs/, playwright/, scripts/, dev-docs/, vendored trees and
# the generated `primitives/src/js/`; so primitives/, preview/, templates/, labs/, cli/ and any new top-level directory
# are all in):
#   sink          in a Rust file's code view (comments removed, string contents blanked, scripts/comment-scan.py
#                 `code_of`): the identifier `eval` anywhere (`document::eval(`, `.eval(`, `use ...::eval as run`, a
#                 `use` list), the glob `document::*`, `document::Script`/`Script {`/`script {`/`create_script`/
#                 `create_head_element`/`create_link`, `dangerous_inner_html`, `web_sys`/`js_sys`/`wasm_bindgen`/
#                 `gloo_utils`/`evaluate_script`/`inline_js`/`srcdoc`, `extern "..."`; in a string: an `include_str!`/
#                 `include_bytes!`/`asset!` of a `.js`/`.ts`/`.html` file, an rsx `"onclick":` attribute key,
#                 `Attribute::new("on...")`, a `javascript:` href and a tag with an `on...=` handler; in an HTML file:
#                 an external `<script src>`, an `on...=` attribute, a `javascript:` href
#   js_lit        lines of Rust string literals that read as JS (scripts/comment-scan.py `literals_of`; a statement or
#                 block shape plus a JS word, no `//` needed), `concat!` pieces joined first, so splitting a script
#                 into fragments does not hide it; a line over 120 characters counts ceil(len/120), so a minified
#                 one-liner is not one line
#   js_file       non-blank, non-comment lines of hand-written `.js`/`.mjs`/`.cjs`/`.ts`/`.tsx` (the generated
#                 `primitives/src/js/` bundle is outside; its `ts/` source is inside, so moving a script into a new
#                 `.ts` raises this column)
#   install       `JsListeners::install` sites
#   inline_script UTF-8 bytes of the inline `<script>` bodies of tracked `.html` files
#   comment_eval  comment lines that present `document::eval` as the crate's precedent ("house pattern", "established
#                 escape hatch", "the same eval-channel shape as ..."): an instruction an agent copies from the
#                 neighbouring code
# The GLOBAL total `js_lit + js_file` must not rise either: a port that moves 100 lines from a Rust string into a new
# `.ts` lowers one column and raises the other, and the total must still drop.
#
# THE RATCHET (scripts/lib/ratchet.py, the one implementation): the baseline `scripts/check-js-volume.baseline.tsv`
# (`path<TAB>sink<TAB>js_lit<TAB>js_file<TAB>install<TAB>inline_script<TAB>comment_eval`, files with a non-zero count
# only, no totals line) is the tree's number per file. A count that rises fails; a count that fell and was not
# written back fails (run `--update-baseline`); a new file with a count fails; a row for a file that is gone fails.
# A deliberate raise is `--update-baseline --allow-raise "row N reason"`: it writes a visible `# raised: row N reason`
# note on the row (and a `#! total-raised:` line when the total rises), the note counts only when the merge base's
# baseline does not already have it, and every allowed raise prints under a `RATCHET RAISES` heading at each run,
# so the owner sees it at the merge. There is no inline marker. An exception is a baseline row with a row number.
#
# NOT SEEN: JS assembled at run time (`format!` pieces, `[a, b].join("")`) is seen only through the sink count; the
# sinks are closed by clippy `disallowed_methods` (a cargo gate) and the bridge's own gate in row 434; a computed
# property name inside an interpreter is the bridge's design problem, not this gate's.
#
# Usage:
#   scripts/check-js-volume.sh                          check, then run the self-test (the self-test rides along)
#   scripts/check-js-volume.sh --report                 totals now / in the baseline / at the merge base, and every file that differs from the merge base
#   scripts/check-js-volume.sh --inventory              every counted site: file:line kind
#   scripts/check-js-volume.sh --update-baseline [--allow-raise "row N reason"]
#   scripts/check-js-volume.sh --self-test              mutations (added eval, aliased `use ... eval as`, `concat!` split, inline
#       `document::Script`, new `.js`/`.ts`, raised count without a note, a raise only the merge-base compare sees, total rise) must turn red
# Environment: RATCHET_BASE_REF pins the merge base, RATCHET_NO_BASE=1 skips that half (scripts/lib/ratchet.py).
# Exit status: 0 clean; 1 a count rose, fell unwritten, or is stale; 2 bad usage or a broken self-test.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac

exec python3 -I - "$@" <<'PYEOF'
import importlib.util
import math
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

GATE = "check-js-volume"
BASELINE = "scripts/check-js-volume.baseline.tsv"
COLUMNS = ("sink", "js_lit", "js_file", "install", "inline_script", "comment_eval")
EXCLUDE = ("docs/", "playwright/", "scripts/", "dev-docs/", "primitives/src/js/", "target/", "node_modules/")
EXCLUDE_PARTS = ("/node_modules/", "/vendor/", "/vendored/", "/target/")
JS_EXT = (".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx")
HTML_EXT = (".html", ".htm")
ADVICE = ("Browser work goes through the bridge in primitives/src/bridge/ (it lands with row 434 lane 3.0; until then add "
          "no JS, lower a count, or ask the owner): dev-docs/js-interop.md#i-want-to--use-this. A new bridge verb or "
          "rule is the only new JS, with a backlog row; a baseline raise is the owner's decision "
          "(--update-baseline --allow-raise \"row N reason\").")


def total_of(c):
    return c["js_lit"] + c["js_file"]


def ratchet(root):
    return Ratchet(GATE, BASELINE, COLUMNS, root=root, total=total_of, advice=ADVICE)


# ---------------------------------------------------------------------------------------------------- universe
def universe(root):
    out = subprocess.run(["git", "-C", root, "ls-files", "-z"], capture_output=True, check=True).stdout.decode()
    files = []
    for p in out.split("\0"):
        if not p or p.startswith(EXCLUDE) or any(s in "/" + p for s in EXCLUDE_PARTS) or p.endswith((".min.js", ".lock")):
            continue
        if p.endswith(".rs") or p.endswith(JS_EXT) or p.endswith(HTML_EXT):
            if os.path.isfile(os.path.join(root, p)):
                files.append(p)
    return sorted(files)


# ---------------------------------------------------------------------------------------------------- Rust
SINK_RX = [
    ("eval", re.compile(r"\bdocument\s*::\s*eval\b|\buse\b[^;]*?\beval\b|\beval\s*\(")),
    ("document-glob", re.compile(r"\bdocument\s*::\s*\*")),
    ("script", re.compile(r"\bdocument\s*::\s*Script\b|\bScript\s*\{|\bscript\s*\{|\bcreate_script\b|\bcreate_head_element\b|\bcreate_link\b")),
    ("dangerous_inner_html", re.compile(r"\bdangerous_inner_html\b")),
    ("ffi", re.compile(r"\b(?:web_sys|js_sys|wasm_bindgen|gloo_utils|evaluate_script|inline_js|srcdoc)\b|\bextern\s+\"")),
]
INSTALL_RX = re.compile(r"\bJsListeners\s*::\s*install\b")
CONCAT_RX = re.compile(r"\bconcat\s*!\s*\(")
INCLUDE_TAIL = re.compile(r"(?:include_str|include_bytes|asset)\s*!\s*\(\s*$")
CODE_EXT = re.compile(r"\.(?:m?js|cjs|tsx?|jsx|html?)$", re.I)
ATTR_KEY = re.compile(r"^on[a-z]{3,}$")
ATTR_NEW_TAIL = re.compile(r"Attribute\s*::\s*new\s*\(\s*$")
HANDLER_TAG = re.compile(r"<[A-Za-z][^>]*\son[a-z]{3,}\s*=", re.S)
# What makes a string read as JS: a statement or block shape plus a word that only code has.
JS_SHAPE = re.compile(r"[;{}]")
JS_WORD = re.compile(
    r"=>|\b(?:const|let|var)\s+[A-Za-z_$][\w$]*\s*[=;,]|\b(?:function|typeof|addEventListener|removeEventListener|querySelector(?:All)?|getElementById|"
    r"getBoundingClientRect|getComputedStyle|setTimeout|setInterval|requestAnimationFrame|classList|dataset|"
    r"preventDefault|dioxus\.(?:send|recv))\b|\b(?:document|window|globalThis|navigator)\s*\.|\bnew\s+[A-Z]\w+\s*\(|"
    r"\breturn\b[^;]*;|\b(?:if|for|while)\s*\([^)]*\)\s*[{a-z]")
LONG_LINE = 120
COMMENT_EVAL = re.compile(
    r"(?i)house (?:pattern|style)|established escape hatch|"
    r"(?:the )?same eval-channel|every other eval|(?:other|the) eval-channel hook|eval-listener-then-stream|"
    r"the \*shape\* -- a small `document::eval`|escape\s+hatch for DOM access")


TEST_MOD = re.compile(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{")
TEST_FILE = re.compile(r"(?:^|/)(?:tests?|[\w]*_tests?)\.rs$|/tests/")


def test_ranges(code, path):
    """Offsets covered by `#[cfg(test)] mod x { .. }` (or the whole file of a test module): strings there are
    assertion fixtures, not JS the page runs."""
    if TEST_FILE.search(path):
        return [(0, len(code))]
    out = []
    for m in TEST_MOD.finditer(code):
        depth, k = 0, m.end() - 1
        while k < len(code):
            if code[k] == "{":
                depth += 1
            elif code[k] == "}":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        out.append((m.start(), k))
    return out


def reads_as_js(content):
    return len(content) > 8 and JS_SHAPE.search(content) is not None and JS_WORD.search(content) is not None


def js_units(content):
    return sum(math.ceil(len(ln.strip()) / LONG_LINE) for ln in content.split("\n") if ln.strip())


def line_at(text, off):
    return text.count("\n", 0, off) + 1


def match_paren(code, open_idx):
    depth = 0
    for k in range(open_idx, len(code)):
        ch = code[k]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return k
    return len(code)


def measure_rust(text, path=""):
    """`(counts, sites)`; `sites` is a list of (line, kind[, n]) for --inventory."""
    c = dict.fromkeys(COLUMNS, 0)
    sites = []
    code = cs.code_of(text)
    for kind, rx in SINK_RX:
        for m in rx.finditer(code):
            c["sink"] += 1
            sites.append((line_at(text, m.start()), "sink:" + kind))
    for m in INSTALL_RX.finditer(code):
        c["install"] += 1
        sites.append((line_at(text, m.start()), "install"))
    lits = cs.literals_of(text)
    for lit in lits:
        content, tail = lit.content, code[max(0, lit.start - 40):lit.start]
        why = None
        if INCLUDE_TAIL.search(tail) and CODE_EXT.search(content.strip()):
            why = "sink:include-js"
        elif ATTR_KEY.match(content) and re.match(r"\s*:(?!:)", code[lit.end:lit.end + 8]):
            why = "sink:on-attribute"
        elif ATTR_NEW_TAIL.search(tail) and content.startswith("on"):
            why = "sink:on-attribute"
        elif content.lstrip().lower().startswith("javascript:"):
            why = "sink:javascript-url"
        elif HANDLER_TAG.search(content):
            why = "sink:handler-markup"
        if why:
            c["sink"] += 1
            sites.append((lit.line, why))
    # JS held in strings: a concat! group is read joined, every other literal on its own
    consumed = set()
    groups = []
    for m in CONCAT_RX.finditer(code):
        if m.start() in consumed:
            continue
        close = match_paren(code, m.end() - 1)
        inner = [k for k, lit in enumerate(lits) if m.end() <= lit.start and lit.end <= close]
        for k in inner:
            consumed.add(lits[k].start)
        consumed.update(range(m.end(), close))  # a nested concat! is part of this group
        if inner:
            groups.append((lits[inner[0]].line, "".join(lits[k].content for k in inner), lits[inner[0]].start))
    for lit in lits:
        if lit.start not in consumed:
            groups.append((lit.line, lit.content, lit.start))
    tests = test_ranges(code, path)
    for line, content, off in groups:
        if reads_as_js(content) and not any(a <= off <= b for a, b in tests):
            n = js_units(content)
            c["js_lit"] += n
            sites.append((line, f"js_lit:{n}"))
    for line, body in cs.extract_rust(text):
        if COMMENT_EVAL.search(body):
            c["comment_eval"] += 1
            sites.append((line, "comment_eval"))
    return c, sites


# ---------------------------------------------------------------------------------------------------- JS and HTML
def measure_js(text):
    n = 0
    for ln in text.split("\n"):
        s = ln.strip()
        if s and not s.startswith(("//", "/*", "*")):
            n += 1
    c = dict.fromkeys(COLUMNS, 0)
    c["js_file"] = n
    return c, [(1, f"js_file:{n}")] if n else []


SCRIPT_RX = re.compile(r"<script\b([^>]*)>(.*?)</script\s*>", re.S | re.I)
HTML_HANDLER = re.compile(r"<[A-Za-z][^>]*\son[a-z]{3,}\s*=|\bhref\s*=\s*[\"']\s*javascript:|\bsrcdoc\s*=", re.I | re.S)


def measure_html(text):
    c = dict.fromkeys(COLUMNS, 0)
    sites = []
    for m in SCRIPT_RX.finditer(text):
        if re.search(r"\bsrc\s*=", m.group(1), re.I):
            c["sink"] += 1
            sites.append((line_at(text, m.start()), "sink:external-script"))
        else:
            n = len(m.group(2).strip().encode())
            c["inline_script"] += n
            sites.append((line_at(text, m.start()), f"inline_script:{n}"))
    for m in HTML_HANDLER.finditer(SCRIPT_RX.sub(lambda mm: " " * len(mm.group(0)), text)):
        c["sink"] += 1
        sites.append((line_at(text, m.start()), "sink:html-handler"))
    return c, sites


def measure(root):
    """`(measured, sites)`: counts per file (files with a non-zero count only) and the sites behind them."""
    out, all_sites = {}, {}
    for path in universe(root):
        with open(os.path.join(root, path), encoding="utf-8", errors="replace") as f:
            text = f.read()
        if path.endswith(".rs"):
            c, sites = measure_rust(text, path)
        elif path.endswith(JS_EXT):
            c, sites = measure_js(text)
        else:
            c, sites = measure_html(text)
        if any(c.values()):
            out[path] = c
            all_sites[path] = sorted(sites)
    return out, all_sites


def sums(rows):
    return {k: sum(c[k] for c in rows.values()) for k in COLUMNS}


# ---------------------------------------------------------------------------------------------------- commands
def run_check(root):
    measured, _ = measure(root)
    R = ratchet(root)
    rep = R.check(measured)
    return R, rep, measured


def cmd_check(root, quiet=False):
    R, rep, measured = run_check(root)
    text = R.format(rep)
    if rep.errors:
        print(text, file=sys.stderr)
        print(f"{GATE}: FAIL -- {len(rep.errors)} problem(s). {ADVICE}", file=sys.stderr)
        return 1
    if text and not quiet:
        print(text)
    s = sums(measured)
    print(f"{GATE}: OK ({len(measured)} files carry JS: sink {s['sink']}, js_lit {s['js_lit']}, js_file {s['js_file']}, "
          f"install {s['install']}, inline_script {s['inline_script']} bytes, comment_eval {s['comment_eval']}; "
          f"total js_lit + js_file {s['js_lit'] + s['js_file']})")
    return 0


def cmd_inventory(root):
    _, sites = measure(root)
    for path in sorted(sites):
        for line, kind in sites[path]:
            print(f"{path}:{line}: {kind}")
    return 0


def cmd_report(root):
    measured, _ = measure(root)
    R = ratchet(root)
    now = sums(measured)
    text = R._tree_text()
    tree_rows = R.parse(text)[0] if text else {}
    tree = sums({p: c for p, (c, _) in tree_rows.items()}) if tree_rows else None
    base_text, sha, note = R.base()
    base_rows = {p: c for p, (c, _) in R.parse(base_text)[0].items()} if base_text else None
    base = sums(base_rows) if base_rows is not None else None
    print(f"{GATE} --report ({note})")
    print(f"  {'':14}" + "".join(f"{k:>14}" for k in COLUMNS))
    print(f"  {'now':14}" + "".join(f"{now[k]:>14}" for k in COLUMNS))
    if tree:
        print(f"  {'tree baseline':14}" + "".join(f"{tree[k]:>14}" for k in COLUMNS))
    if base:
        print(f"  {'merge base':14}" + "".join(f"{base[k]:>14}" for k in COLUMNS))
    ref = base_rows if base_rows is not None else {p: c for p, (c, _) in tree_rows.items()}
    zero = dict.fromkeys(COLUMNS, 0)
    print("  files that differ from " + ("the merge base" if base_rows is not None else "the tree baseline") + " (before -> after):")
    shown = 0
    for p in sorted(set(measured) | set(ref)):
        a, b = ref.get(p, zero), measured.get(p, zero)
        if a != b:
            shown += 1
            print(f"    {p}: " + ", ".join(f"{k} {a[k]}->{b[k]}" for k in COLUMNS if a[k] != b[k]))
    if not shown:
        print("    (none)")
    return 0


def cmd_update(root, reason):
    measured, _ = measure(root)
    R = ratchet(root)
    errors, wrote = R.update(measured, reason)
    for e in errors:
        print(e, file=sys.stderr)
    if wrote:
        print(f"{GATE}: wrote {BASELINE} ({len(R._clean(measured))} files)")
    return 0 if wrote else 1


# ---------------------------------------------------------------------------------------------------- self-test
GIT_ENV = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid", "GIT_COMMITTER_NAME": "t",
           "GIT_COMMITTER_EMAIL": "t@example.invalid", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_SYSTEM": os.devnull}

CLEAN_RS = 'fn plain() -> u32 { 1 }\n'
EVAL_RS = ('use dioxus::document::eval;\n'
           'fn go() {\n    let _ = eval(r#"\n        const el = document.getElementById("a");\n        el.focus();\n    "#);\n}\n')


def git(root, *args):
    r = subprocess.run(["git", "-C", root, *args], capture_output=True, text=True, env={**os.environ, **GIT_ENV})
    if r.returncode:
        raise RuntimeError(f"git {' '.join(args)}: {r.stderr}")
    return r.stdout.strip()


def self_test():
    fails = []
    ok_count = 0

    def write(root, path, text):
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as f:
            f.write(text)

    def make_base(root, files, note=None):
        git(root, "init", "-q")
        for p, t in files.items():
            write(root, p, t)
        git(root, "add", "-A")
        measured, _ = measure(root)
        R = ratchet(root)
        R.base_ref = None
        os.environ["RATCHET_NO_BASE"] = "1"
        try:
            errors, wrote = R.update(measured, None)
        finally:
            del os.environ["RATCHET_NO_BASE"]
        assert wrote, errors
        if note:
            with open(os.path.join(root, BASELINE), encoding="utf-8") as f:
                lines = f.read().split("\n")
            lines = [ln + "\t" + note if ln.startswith("primitives/src/a.rs\t") else ln for ln in lines]
            write(root, BASELINE, "\n".join(lines))
        git(root, "add", "-A")
        git(root, "commit", "-q", "-m", "base")
        return git(root, "rev-parse", "HEAD")

    def scenario(label, want, files, mutate, base_note=None, env=None, why=None):
        nonlocal ok_count
        with tempfile.TemporaryDirectory(prefix="check-js-volume-st.") as root:
            sha = make_base(root, files, base_note)
            mutate(root)
            git(root, "add", "-A")
            saved = {k: os.environ.get(k) for k in ("RATCHET_BASE_REF", "RATCHET_NO_BASE")}
            os.environ["RATCHET_BASE_REF"] = sha
            os.environ.pop("RATCHET_NO_BASE", None)
            for k, v in (env or {}).items():
                os.environ[k] = v
            try:
                R, rep, _ = run_check(root)
            finally:
                for k, v in saved.items():
                    if v is None:
                        os.environ.pop(k, None)
                    else:
                        os.environ[k] = v
            got = "red" if rep.errors else "green"
            detail = R.format(rep)
        if got == want and why and why not in detail:
            fails.append(f"{label}: red, but not for the expected reason ({why!r}):\n{detail}")
        elif got != want:
            fails.append(f"{label}: expected {want}, got {got}\n{detail}")
        else:
            ok_count += 1
        return detail

    base = {"primitives/src/a.rs": EVAL_RS, "primitives/src/clean.rs": CLEAN_RS}

    def append(path, text):
        return lambda root: write(root, path, open(os.path.join(root, path), encoding="utf-8").read() + text)

    def baseline_edit(fn):
        def mut(root):
            p = os.path.join(root, BASELINE)
            with open(p, encoding="utf-8") as f:
                write(root, BASELINE, fn(f.read()))
        return mut

    def bump_baseline(note):
        # the code is already raised (by `append`), now the tree's baseline is rewritten to match, as an agent would
        def mut(root):
            measured, _ = measure(root)
            R = ratchet(root)
            os.environ["RATCHET_NO_BASE"] = "1"
            try:
                R.update(measured, "row 434 test")
            finally:
                del os.environ["RATCHET_NO_BASE"]
            with open(os.path.join(root, BASELINE), encoding="utf-8") as f:
                lines = f.read().split("\n")
            for k, ln in enumerate(lines):
                if ln.startswith("primitives/src/a.rs\t"):
                    parts = [x for x in ln.split("\t") if not x.startswith("#")]
                    lines[k] = "\t".join(parts + ([note] if note else []))
            write(root, BASELINE, "\n".join(lines))
        return mut

    scenario("control: an unchanged tree", "green", base, lambda r: None)
    scenario("control: eval named only in a comment and a string", "green", base,
             append("primitives/src/clean.rs", '// document::eval is gone from here\nfn s() -> &\'static str { "use eval here" }\n'))
    scenario("an added eval call", "red", base, append("primitives/src/a.rs", 'fn two() { let _ = eval("x"); }\n'), why='count rose above the baseline')
    scenario("an aliased `use dioxus::document::eval as run`", "red", base,
             append("primitives/src/clean.rs", 'use dioxus::document::eval as run;\nfn t() { let _ = run("1"); }\n'), why='new file with a non-zero count')
    scenario("a Document trait call `document().eval(..)`", "red", base,
             append("primitives/src/clean.rs", 'fn t() { let _ = document().eval("1".to_string()); }\n'))
    scenario("a `document::*` glob import", "red", base, append("primitives/src/clean.rs", 'use dioxus::document::*;\n'))
    split = ('const S: &str = concat!("el.add", "EventListener(\'x\', () =>", " { f(); });");\n')
    scenario("a script split into concat! pieces", "red", base, append("primitives/src/clean.rs", split), why='new file with a non-zero count')
    scenario("control: one short non-JS literal", "green", base, append("primitives/src/clean.rs", 'const S: &str = concat!("a", "b");\n'))
    scenario("an inline document::Script with a string body", "red", base,
             append("primitives/src/clean.rs", 'fn v() -> Element { rsx! { document::Script { "console.log(1)" } } }\n'))
    scenario("a script { dangerous_inner_html } element", "red", base,
             append("primitives/src/clean.rs", 'fn v() -> Element { rsx! { script { dangerous_inner_html: X } } }\n'))
    scenario("an rsx string `\"onclick\":` attribute", "red", base,
             append("primitives/src/clean.rs", 'fn v() -> Element { rsx! { a { "onclick": "go()" } } }\n'))
    scenario("web_sys named in code", "red", base, append("primitives/src/clean.rs", 'use web_sys::window;\n'))
    scenario("a JsListeners::install site", "red", base,
             append("primitives/src/clean.rs", 'fn l() { let _ = JsListeners::install(1); }\n'))
    scenario("an include_str! of a .js file", "red", base,
             append("primitives/src/clean.rs", 'const J: &str = include_str!("x.js");\n'))
    scenario("a new .js file", "red", base, lambda r: write(r, "primitives/src/new.js", "const a = 1;\nfoo(a);\n"), why='new file with a non-zero count')
    scenario("a new .ts file", "red", base, lambda r: write(r, "templates/starter/x.ts", "export const a = 1;\n"))
    scenario("control: a generated bundle is outside", "green", base,
             lambda r: write(r, "primitives/src/js/gen.js", "const a = 1;\nfoo(a);\n"))
    scenario("an inline <script> in a new html file", "red", base,
             lambda r: write(r, "preview/page.html", "<html><script>var a = 1;</script></html>\n"))
    scenario("a comment that presents eval as the house pattern", "red", base,
             append("primitives/src/clean.rs", "// this crate's `document::eval` house pattern\n"))
    scenario("a count that fell and was not written back", "red", base,
             lambda r: write(r, "primitives/src/a.rs", "fn gone() {}\n"), why='lock the drop in')
    scenario("a baseline row whose file is gone", "red", base,
             lambda r: os.remove(os.path.join(r, "primitives/src/a.rs")), why='stale baseline row')
    scenario("a baseline that was deleted", "red", base, lambda r: os.remove(os.path.join(r, BASELINE)), why='is missing but the merge base')
    # raised counts: the code and the baseline agree, only the merge-base compare can see it
    raised_code = append("primitives/src/a.rs", 'fn two() { let _ = eval("x"); }\n')

    def raise_with(note):
        def mut(root):
            raised_code(root)
            bump_baseline(note)(root)
        return mut

    scenario("a raised count with no note (tree and baseline agree)", "red", base, raise_with(None), why='rises above the merge base')
    detail = scenario("a raised count with a `# raised: row N` note", "green", base,
                      raise_with("# raised: row 434 test raise"))
    if "RATCHET RAISES" not in detail or "test raise" not in detail:
        fails.append("a raise with a note must print under the RATCHET RAISES heading, got:\n" + detail)
    scenario("a malformed note (no row number)", "red", base, raise_with("# raised: because"), why='malformed')
    scenario("a note the merge base already has cannot be spent again", "red", base, raise_with("# raised: row 434 old"),
             base_note="# raised: row 434 old", why='already has')
    scenario("the same raise passes when only the tree is compared (the merge-base half is what catches it)", "green",
             base, raise_with(None), env={"RATCHET_NO_BASE": "1"})
    # the total
    moved_files = dict(base)

    def move_into_ts(note):
        def mut(root):
            write(root, "primitives/src/a.rs", CLEAN_RS)  # js_lit of a.rs drops to zero (3 lines)
            write(root, "primitives/src/ts/new.ts", "\n".join(f"const v{k} = {k};" for k in range(30)) + "\n")
            git(root, "add", "-A")
            measured, _ = measure(root)
            R = ratchet(root)
            os.environ["RATCHET_NO_BASE"] = "1"
            try:
                R.update(measured, "row 434 moved")
            finally:
                del os.environ["RATCHET_NO_BASE"]
            if note is not None:
                p = os.path.join(root, BASELINE)
                with open(p, encoding="utf-8") as f:
                    txt = f.read()
                write(root, BASELINE, txt.replace("#! total-raised:", "#! total-raised-removed:") if note == "drop" else txt)
        return mut

    scenario("moving lines into a new .ts raises the total", "red", moved_files, move_into_ts("drop"), why='total rose')
    detail = scenario("the same move with a `#! total-raised:` line", "green", moved_files, move_into_ts(None))
    if "TOTAL" not in detail:
        fails.append("a total raise must be listed under RATCHET RAISES, got:\n" + detail)
    for f in fails:
        print("self-test FAIL: " + f, file=sys.stderr)
    if fails:
        return 2
    print(f"{GATE} --self-test: OK ({ok_count} scenarios)")
    return 0


def main(argv):
    root = HERE
    args = list(argv[1:])
    if "--root" in args:
        k = args.index("--root")
        root = os.path.abspath(args[k + 1])
        del args[k:k + 2]
    reason = None
    if "--allow-raise" in args:
        k = args.index("--allow-raise")
        reason = args[k + 1]
        del args[k:k + 2]
    if not args:
        rc = cmd_check(root)
        st = self_test()
        return rc if rc else st
    if args == ["--self-test"]:
        return self_test()
    if args == ["--inventory"]:
        return cmd_inventory(root)
    if args == ["--report"]:
        return cmd_report(root)
    if args == ["--update-baseline"]:
        return cmd_update(root, reason)
    print("usage: check-js-volume.sh [--report|--inventory|--self-test|--update-baseline [--allow-raise \"row N reason\"]]", file=sys.stderr)
    return 2


sys.exit(main(sys.argv))
PYEOF
