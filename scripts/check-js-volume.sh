#!/usr/bin/env bash
#
# check-js-volume.sh  (backlog row 434; rule and replacements: dev-docs/js-interop.md)
#
# JavaScript volume can only shrink: Rust owns behaviour and browser access goes through the bridge in
# `primitives/src/bridge/` (dev-docs/js-interop.md; ref-ok: lands with row 434 lane 3.0), never through a script string, a new `.js`/`.ts` file or an inline
# script. The gate counts, per tracked file, the places JS can enter (eval sinks, JS held in Rust strings, hand-written
# `.js`/`.ts` bytes, `JsListeners::install` sites, inline `<script>` bytes) and fails when a count rises, against the
# baseline AND against the baseline committed at the merge base with origin/main, so editing the baseline in the same
# commit is not a way through.
#
# THE CLASS: 103 bespoke eval blocks were written because the only written convention for "I need the DOM" was
# `document::eval`, every JS gate checked the SHAPE of a script string (and so blessed the string), and nothing
# stopped block 104. A token ban is beaten by `concat!("prevent", "Default()")`, `use ...::eval as run` or a
# `document::Script { "..." }`; this gate closes the sinks (the places a string becomes code) and measures the volume
# behind them, so a spelling trick still raises a count.
#
# WHAT IS COUNTED, per file (universe: `git ls-files -co --exclude-standard`, tracked and untracked-not-ignored, minus
# docs/, playwright/, scripts/, dev-docs/ and the exclusion rows of `scripts/check-js-volume.allow.tsv`; so primitives/,
# preview/, templates/, labs/, cli/ and any new top-level directory are all in; `.rs`, JS/TS, `.html`, `.svg` and `.md`
# are scanned):
#   sink          in a Rust file's code view (comments removed, string contents blanked, scripts/comment-scan.py
#                 `code_of`): `document::eval` (kind `eval`, the document-rooted path only), an import of `eval` (`eval-use`),
#                 any other path ending in `::eval` such as `dd::eval` taken as a function pointer (`eval-path`), a
#                 `.eval(` / `eval(` call or definition (`eval-call`), the glob `document::*`, `document::Script`/
#                 `Script {`/`script {`/`create_script`/`create_head_element`/`create_link`, `dangerous_inner_html`,
#                 `web_sys`/`js_sys`/`wasm_bindgen`/`gloo_utils`/`gloo_events`/`gloo_file`/`evaluate_script`/
#                 `inline_js`/`srcdoc`, `extern "..."`; in a string: an `include_str!`/`include_bytes!`/`asset!` of a
#                 `.js`/`.ts`/`.html` file, an rsx `"onclick":` attribute key, `Attribute::new("on...")`, a
#                 `javascript:` href and a tag with an `on...=` handler; in an HTML, SVG or Markdown file: an external
#                 `<script src>`, an `on...=` attribute, a `javascript:` href (a Markdown fenced block and code span are
#                 displayed samples and are blanked first)
#   js_lit        lines of Rust string literals that read as JS (scripts/comment-scan.py `literals_of`; a statement or
#                 block shape plus a JS word, no `//` needed), `concat!` pieces joined first, so splitting a script
#                 into fragments does not hide it; a line over 120 characters counts ceil(len/120), so a minified
#                 one-liner is not one line
#   js_file_bytes BYTES of the shipped bundle `primitives/src/js/**` (`.js`/`.mjs`/`.cjs`) plus every hand-written
#                 `.js`/`.mjs`/`.cjs`/`.ts`/`.tsx`/`.jsx` outside the bundle's TypeScript source `primitives/src/ts/`
#                 (whose bytes are the bundle's), its tests `primitives/ts-test/`, `.d.ts` and `.gen.ts`. A file that is
#                 only a displayed code sample leaves the count through an `exclude-file` allow row (a reasoned row
#                 the owner approves; a name or a directory does not make a sample)
#   install       `JsListeners::install` sites
#   inline_script UTF-8 bytes of the inline `<script>` bodies of tracked `.html`/`.svg`/`.md` files
#   comment_eval  comment lines that present `document::eval` as the crate's precedent ("house pattern", "established
#                 escape hatch", "the same eval-channel shape as ..."): an instruction an agent copies from the
#                 neighbouring code
#   expect_disallowed  `#[expect(clippy::disallowed_methods ...)]` attributes: each one is a waiver of the clippy sink
#                 ban (clippy.toml `disallowed-methods`), so the count can only shrink
# The GLOBAL total `js_lit * 120 + js_file_bytes` must not rise either (a `js_lit` unit is at most 120 characters, so
# the total is in bytes and moving text from a Rust string into a bundled file cannot raise it).
#
# NOT COUNTS, also red: `#[allow(clippy::disallowed_..)]` (or `#![allow]`, `cfg_attr`) in any file but
# `primitives/src/bridge/transport.rs`; a stale or malformed row of the allow file; the `disallowed-methods` section of
# `clippy.toml` differing from the `pin` row of the allow file (a hash of the parsed section: a comment or a reformat
# keeps it, an added, removed or edited entry changes it; `--update-baseline` writes the new pin and prints it, the
# diff of the allow file shows the owner the change; a pin that differs from the base's is also printed under RATCHET PIN).
# THE ALLOW FILE (`scripts/check-js-volume.allow.tsv`, header there): the reasoned exceptions, the owner's decision:
# V5, V8, V9, V11 (`preview/src/eager_head.rs`) and `theme_seed` (owner-approved), the exclusion rows
# (`exclude-glob` for vendored or minified third-party files, `exclude-file` for a displayed sample) and the pin.
# An `allow` row excuses that many counts of a column in a file (and needs its anchor text still in the file); a row
# that excuses more than the file has, or whose file or anchor is gone, is stale and fails.
#
# THE RATCHET (scripts/lib/ratchet.py, the one implementation): the baseline `scripts/check-js-volume.baseline.tsv`
# (a `#! columns:` header and `path<TAB>count...`, files with a non-zero count only, no totals line) is the tree's number per file. A count that rises fails; a count that fell and was not
# written back fails (run `--update-baseline`); a new file with a count fails; a row for a file that is gone fails.
# A raise passes only against a GRANT: the row `N` in the baseline row's `# raised: row N <reason>` note must exist, be
# open at the base (origin/main or origin/queue/*) and carry `js-grant: <column>=<n>` with n at least the raise
# (scripts/lib/rows.py); a grant written in the lane's own diff is not at the base, so a lane cannot grant itself:
# ask the owner for a grant row. `--allow-raise "row N reason"` only writes the note (and the `#! total-raised:
# row N total=<n>` line) and refuses unless the grant at the base already covers it. Own and inherited raises print
# separately under `RATCHET RAISES` at each run. There is no inline marker. An exception is a baseline row with a row
# number.
#
# NOT SEEN: JS assembled at run time (`format!` pieces, `[a, b].join("")`) is seen only through the sink count; the
# sinks are closed by clippy `disallowed_methods` (a cargo gate) and the bridge's own gate in row 434; a computed
# property name inside an interpreter is the bridge's design problem, not this gate's.
#
# Usage:
#   scripts/check-js-volume.sh                          check, then run the self-test (the self-test rides along)
#   scripts/check-js-volume.sh --report                 totals now / in the baseline / at the merge base, and every file that differs from the merge base
#   scripts/check-js-volume.sh --inventory              every counted site: file:line kind
#   scripts/check-js-volume.sh --update-baseline [--allow-raise "row N reason"]   also re-seeds the columns the merge base
#       lacks (a column is introduced with the tree's own numbers) and re-pins clippy.toml's disallowed-methods
#   scripts/check-js-volume.sh --self-test              mutations (added eval, aliased `use ... eval as`, `concat!` split, inline
#       `document::Script`, new `.js`/`.ts`, raised count without a note or without a grant at the base, a raise only the merge-base compare sees, total rise, wrong total) must turn red
# Environment: RATCHET_BASE_REF pins the base, RATCHET_NO_BASE=1 skips that half, RATCHET_REQUIRE_BASE=1 makes "no base"
# red (scripts/lib/ratchet.py).
# Exit status: 0 clean; 1 a count rose, fell unwritten, or is stale; 2 bad usage or a broken self-test.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
case "${1:-}" in
    -h|--help) sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
esac

exec python3 -I - "$@" <<'PYEOF'
import fnmatch
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
ALLOW = "scripts/check-js-volume.allow.tsv"
COLUMNS = ("sink", "js_lit", "js_file_bytes", "install", "inline_script", "comment_eval", "expect_disallowed")
LEGACY_COLUMNS = ("sink", "js_lit", "js_file", "install", "inline_script", "comment_eval")  # a baseline without `#! columns:`
EXCLUDE = ("docs/", "playwright/", "scripts/", "dev-docs/", "target/", "node_modules/")
EXCLUDE_PARTS = ("/node_modules/", "/target/")
JS_EXT = (".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx")
HTML_EXT = (".html", ".htm", ".svg")
MD_EXT = (".md",)
BUNDLE_DIR = "primitives/src/js/"  # the shipped bundle: counted by bytes
TS_SOURCE_DIRS = ("primitives/src/ts/", "primitives/ts-test/")  # the bundle's source and its tests ride the bundle / are not shipped
TS_SKIP_SUFFIX = (".d.ts", ".gen.ts")
TRANSPORT = "primitives/src/bridge/transport.rs"  # the one file that may `#[allow(clippy::disallowed_..)]`
PIN_TARGET = "clippy.toml#disallowed-methods"
ADVICE = ("Browser work goes through the bridge in primitives/src/bridge/ (it lands with row 434 lane 3.0; until then add "
          "no JS, lower a count, or ask the owner): dev-docs/js-interop.md#i-want-to--use-this. A new bridge verb or "
          "rule is the only new JS, with a backlog row; a baseline raise needs a grant row from the owner.")


def total_of(c):
    """Bytes: a `js_lit` unit is a line of at most LONG_LINE characters, so `js_lit * LONG_LINE` is an upper bound of
    the bytes a literal holds and moving text from a Rust string into a bundled file cannot raise the total."""
    return c["js_lit"] * LONG_LINE + c["js_file_bytes"]


class _Ratchet(Ratchet):
    @staticmethod
    def header_columns(text):
        """A baseline without a `#! columns:` line is the pre-header format (the gate's earlier columns), so the merge
        base compare reads a base that predates the header with the columns it really has."""
        cols = Ratchet.header_columns(text)
        if cols is None and any(ln.strip() and not ln.startswith("#") for ln in text.split("\n")):
            return LEGACY_COLUMNS
        return cols


def _base_total_with_new_files(self, brows, T, renames, introduced):
    """The ratchet's base total plus, for each tree file the base has no row for, the contribution of the introduced
    columns (an introduced column has no base number: the tree's own stands in, also for a file the base never listed,
    e.g. the bundle files whose bytes the column starts to count)."""
    total = Ratchet._base_total(self, brows, T, renames, introduced)
    matched = {tp for path in brows for tp in T if tp == path or renames.get(tp) == path}
    for tp, counts in T.items():
        if tp not in matched:
            total += self.total({c: (counts[c] if c in introduced else 0) for c in self.columns})
    return total


_Ratchet._base_total = _base_total_with_new_files


def ratchet(root):
    return _Ratchet(GATE, BASELINE, COLUMNS, root=root, total=total_of, advice=ADVICE)


# ---------------------------------------------------------------------------------------------------- allow file
ALLOW_KINDS = ("exclude-glob", "exclude-file", "allow", "pin")


def load_allow(root):
    """`(rows, problems)`. A row is `kind<TAB>target<TAB>spec<TAB>anchor<TAB>reason` (comment lines start with `#`):
      exclude-glob  target is an fnmatch pattern against `/<path>`; the files it matches leave the universe
      exclude-file  target is one path (a file that is only a displayed code sample) that leaves the universe
      allow         target is a path, spec `column=n[,column=n]`: that many counts of the column are the reasoned
                    exception (the owner's decision), the anchor text must still occur in the file
      pin           target `clippy.toml#disallowed-methods`, spec `sha256=<hex>|absent`
    Every row carries a reason; a row that matches nothing is stale and fails."""
    rows, problems = [], []
    try:
        with open(os.path.join(root, ALLOW), encoding="utf-8") as f:
            text = f.read()
    except OSError:
        return rows, [f"{ALLOW} is missing: the reasoned exceptions of {GATE} live there (deleting it is not a pass)"]
    for n, ln in enumerate(text.split("\n"), 1):
        if not ln.strip() or ln.startswith("#"):
            continue
        parts = ln.split("\t")
        if len(parts) != 5 or parts[0] not in ALLOW_KINDS or not all(p.strip() for p in parts):
            problems.append(f"{ALLOW}:{n}: malformed row (want kind<TAB>target<TAB>spec<TAB>anchor<TAB>reason, kind one of "
                            f"{', '.join(ALLOW_KINDS)}, no empty field; `-` for no spec or anchor)")
            continue
        kind, target, spec, anchor, reason = parts
        if len(reason.split()) < 3:
            problems.append(f"{ALLOW}:{n}: the reason of {target} must say why (at least three words)")
            continue
        counts = {}
        if kind == "allow":
            try:
                counts = {k: int(v) for k, v in (kv.split("=") for kv in spec.split(","))}
            except ValueError:
                counts = {}
            if not counts or any(k not in COLUMNS or v <= 0 for k, v in counts.items()):
                problems.append(f"{ALLOW}:{n}: spec of {target} must be column=n[,column=n] over {', '.join(COLUMNS)}")
                continue
        rows.append({"kind": kind, "target": target, "spec": spec, "anchor": anchor, "reason": reason,
                     "counts": counts, "line": n})
    return rows, problems


def pin_of_text(toml_text):
    """`clippy_pin` of a clippy.toml text (the self-test builds a pin for a fixture)."""
    with tempfile.TemporaryDirectory(prefix="check-js-volume-pin.") as d:
        with open(os.path.join(d, "clippy.toml"), "w", encoding="utf-8") as f:
            f.write(toml_text)
        return clippy_pin(d)


def clippy_pin(root):
    """The pin value of the `disallowed-methods` section of clippy.toml: `absent` or `sha256=<hex>` of its parsed
    value (a comment or a reformat does not change it, an added, removed or edited entry does)."""
    import hashlib
    import json
    import tomllib
    try:
        with open(os.path.join(root, "clippy.toml"), "rb") as f:
            data = tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError):
        return "absent"
    if "disallowed-methods" not in data:
        return "absent"
    blob = json.dumps(data["disallowed-methods"], sort_keys=True, separators=(",", ":")).encode()
    return "sha256=" + hashlib.sha256(blob).hexdigest()


# ---------------------------------------------------------------------------------------------------- universe
def js_counted(path):
    """Is this file's every byte shipped or shippable JS: the bundle under primitives/src/js/ and hand-written JS
    outside the bundle's TypeScript source (primitives/src/ts/, whose bytes are the bundle's), its tests
    (primitives/ts-test/) and declaration or generated `.ts`."""
    if path.startswith(BUNDLE_DIR):
        return path.endswith((".js", ".mjs", ".cjs"))
    if path.startswith(TS_SOURCE_DIRS) or path.endswith(TS_SKIP_SUFFIX):
        return False
    return path.endswith(JS_EXT)


def universe(root, allow=()):
    """`(files, problems)`: `git ls-files -co --exclude-standard` (tracked and untracked-not-ignored, so a new file
    counts before it is added) minus the fixed prefixes and the exclusion rows of the allow file."""
    out = subprocess.run(["git", "-C", root, "ls-files", "-co", "--exclude-standard", "-z"], capture_output=True,
                         check=True).stdout.decode()
    files, problems = [], []
    all_paths = [p for p in sorted(set(out.split("\0"))) if p and os.path.isfile(os.path.join(root, p))]
    for row in allow:
        if row["kind"] == "exclude-glob":
            hit = any(fnmatch.fnmatchcase("/" + p, row["target"]) for p in all_paths)
        elif row["kind"] == "exclude-file":
            hit = row["target"] in all_paths
        else:
            continue
        if not hit:
            problems.append(f"{ALLOW}:{row['line']}: stale {row['kind']} row {row['target']} (it matches no file); "
                            f"delete it")
    for p in all_paths:
        if p.startswith(EXCLUDE) or any(s in "/" + p for s in EXCLUDE_PARTS) or p.endswith(".lock"):
            continue
        if any((r["kind"] == "exclude-glob" and fnmatch.fnmatchcase("/" + p, r["target"]))
               or (r["kind"] == "exclude-file" and r["target"] == p) for r in allow):
            continue
        if p.endswith((".rs",) + HTML_EXT + MD_EXT) or js_counted(p):
            files.append(p)
    return files, problems


# ---------------------------------------------------------------------------------------------------- Rust
SINK_RX = [
    # one alternation, so a `document::eval(` is one site: `eval` is the document-rooted path, `eval-use` an import of
    # any `eval`, `eval-path` a path ending in `::eval` that is not document-rooted (`dd::eval` taken as a function
    # pointer, `Document::eval(&d, ..)`), `eval-call` a `.eval(` / `eval(` call or definition
    ("eval", re.compile(r"(?P<eval>\bdocument\s*::\s*eval\b)|(?P<eval_use>\buse\b[^;]*?\beval\b)|"
                        r"(?P<eval_path>\b\w+\s*::\s*eval\b)|(?P<eval_call>\beval\s*\()")),
    ("document-glob", re.compile(r"\bdocument\s*::\s*\*")),
    ("script", re.compile(r"\bdocument\s*::\s*Script\b|\bScript\s*\{|\bscript\s*\{|\bcreate_script\b|\bcreate_head_element\b|\bcreate_link\b")),
    ("dangerous_inner_html", re.compile(r"\bdangerous_inner_html\b")),
    ("ffi", re.compile(r"\b(?:web_sys|js_sys|wasm_bindgen|gloo_utils|gloo_events|gloo_file|evaluate_script|inline_js|srcdoc)\b|\bextern\s+\"")),
]
EXPECT_RX = re.compile(r"#\s*!?\s*\[\s*expect\s*\(([^\]]*)\)\s*\]", re.S)
# `#[allow(clippy::disallowed_methods)]` (and the sibling lints, inner attributes, `cfg_attr`) silences the sink ban
ALLOW_LINT_RX = re.compile(r"\ballow\s*\([^\]]*?clippy\s*::\s*disallowed_", re.S)
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
            sites.append((line_at(text, m.start()), "sink:" + (m.lastgroup.replace("_", "-") if m.lastgroup else kind)))
    for m in EXPECT_RX.finditer(code):
        if "clippy::disallowed_methods" in re.sub(r"\s+", "", m.group(1)):
            c["expect_disallowed"] += 1
            sites.append((line_at(text, m.start()), "expect_disallowed"))
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
    """Bytes of the whole file: a minified one-liner or a comment-stuffed file cannot hide in a line count."""
    n = len(text.encode("utf-8", "replace"))
    c = dict.fromkeys(COLUMNS, 0)
    c["js_file_bytes"] = n
    return c, [(1, f"js_file_bytes:{n}")] if n else []


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


FENCE_RX = re.compile(r"^(`{3,}|~{3,}).*?^\1[ \t]*$", re.S | re.M)
CODE_SPAN_RX = re.compile(r"(`+)(?:(?!\1).)+\1", re.S)


def measure_md(text):
    """Markdown holds HTML: an inline `<script>`, an `on...=` handler, a `javascript:` link. A fenced block and a code
    span are displayed code samples, not page content, and are blanked first."""
    blank = lambda m: re.sub(r"[^\n]", " ", m.group(0))  # noqa: E731 (keeps the line numbers)
    return measure_html(CODE_SPAN_RX.sub(blank, FENCE_RX.sub(blank, text)))


def lint_problems(root, path, text):
    code = cs.code_of(text)
    if path != TRANSPORT and ALLOW_LINT_RX.search(code):
        return [f"{path}:{line_at(text, ALLOW_LINT_RX.search(code).start())}: `#[allow(clippy::disallowed_..)]` outside "
                f"{TRANSPORT}: the sink ban is waived per site with `#[expect(..)]` (counted in the expect_disallowed "
                f"column), never allowed"]
    return []


def apply_allow(allow, out, texts):
    """Subtract the reasoned exceptions of the allow file from the measured counts; returns the problems."""
    problems = []
    for row in allow:
        if row["kind"] != "allow":
            continue
        path = row["target"]
        if path not in texts:
            problems.append(f"{ALLOW}:{row['line']}: stale allow row, {path} is not in the universe; delete it")
            continue
        if row["anchor"] != "-" and row["anchor"] not in texts[path]:
            problems.append(f"{ALLOW}:{row['line']}: stale allow row, the anchor {row['anchor']!r} is gone from {path}")
            continue
        have = out.setdefault(path, dict.fromkeys(COLUMNS, 0))
        for col, n in row["counts"].items():
            if have[col] < n:
                problems.append(f"{ALLOW}:{row['line']}: {path} has {have[col]} {col} but the row allows {n}: the "
                                f"exception shrank or moved, lower the row")
                continue
            have[col] -= n
    return problems


def measure(root):
    """`(measured, sites, problems)`: counts per file (files with a non-zero count only), the sites behind them and the
    rule violations (stale or malformed allow rows, a lint allow, a pin that differs) that are not counts."""
    allow, problems = load_allow(root)
    files, p2 = universe(root, allow)
    problems += p2
    out, all_sites, texts = {}, {}, {}
    for path in files:
        with open(os.path.join(root, path), encoding="utf-8", errors="replace") as f:
            text = f.read()
        texts[path] = text
        if path.endswith(".rs"):
            c, sites = measure_rust(text, path)
            problems += lint_problems(root, path, text)
        elif js_counted(path):
            c, sites = measure_js(text)
        elif path.endswith(MD_EXT):
            c, sites = measure_md(text)
        else:
            c, sites = measure_html(text)
        if any(c.values()):
            out[path] = c
            all_sites[path] = sorted(sites)
    problems += apply_allow(allow, out, texts)
    out = {p: c for p, c in out.items() if any(c.values())}
    if not any(r["kind"] == "pin" and r["target"] == PIN_TARGET for r in allow):
        problems.append(f"{ALLOW}: no `pin` row for {PIN_TARGET}: the sink ban list must be pinned")
    for row in allow:
        if row["kind"] == "pin" and row["target"] == PIN_TARGET and row["spec"] != clippy_pin(root):
            problems.append(f"{ALLOW}:{row['line']}: the disallowed-methods section of clippy.toml is {clippy_pin(root)}, "
                            f"the pin says {row['spec']}: the sink ban list changed; review the clippy.toml diff, then "
                            f"--update-baseline writes the new pin")
    return out, all_sites, problems


def sums(rows):
    return {k: sum(c[k] for c in rows.values()) for k in COLUMNS}


# ---------------------------------------------------------------------------------------------------- commands
def run_check(root):
    measured, _, problems = measure(root)
    R = ratchet(root)
    rep = R.check(measured)
    rep.errors[:0] = problems
    base_text, sha, _ = R.base()
    if sha and not os.environ.get("RATCHET_NO_BASE"):
        old = R._git("show", f"{sha}:{ALLOW}") or ""
        old_pin = next((ln.split("\t")[2] for ln in old.split("\n") if ln.startswith("pin\t")), None)
        new_pin = clippy_pin(root)
        if old_pin is not None and old_pin != new_pin:
            rep.notes.append(f"RATCHET PIN: the disallowed-methods section of clippy.toml differs from the base ({old_pin} -> "
                             f"{new_pin}): review the clippy.toml diff")
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
    print(f"{GATE}: OK ({len(measured)} files carry JS: sink {s['sink']}, js_lit {s['js_lit']}, js_file_bytes {s['js_file_bytes']}, "
          f"install {s['install']}, inline_script {s['inline_script']} bytes, comment_eval {s['comment_eval']}, "
          f"expect_disallowed {s['expect_disallowed']}; total js_lit*{LONG_LINE} + js_file_bytes {total_of(s)})")
    return 0


def cmd_inventory(root):
    _, sites, _ = measure(root)
    for path in sorted(sites):
        for line, kind in sites[path]:
            print(f"{path}:{line}: {kind}")
    return 0


def cmd_report(root):
    measured, _, _ = measure(root)
    R = ratchet(root)
    now = sums(measured)
    text = R._tree_text()
    pad = lambda c: {k: c.get(k, 0) for k in COLUMNS}  # noqa: E731 (a column the file predates reads 0)
    tree_rows = R.parse(text, columns=R.header_columns(text) or COLUMNS)[0] if text else {}
    tree_rows = {p: (pad(c), n) for p, (c, n) in tree_rows.items()}
    tree = sums({p: c for p, (c, _) in tree_rows.items()}) if tree_rows else None
    base_text, sha, note = R.base()
    base_rows = ({p: pad(c) for p, (c, _) in R.parse(base_text, columns=R.header_columns(base_text))[0].items()}
                 if base_text else None)
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


def seed_introduced(root, R, measured):
    """Columns the merge base's header lacks are compared with the tree only (scripts/lib/ratchet.py), so they are
    re-seeded from the measurement here: this is how a column is introduced, and how the count of an introduced column
    follows a merge of code that carries it. Every other column stays shrink-only."""
    base_text, _, _ = R.base()
    text = R._tree_text()
    if not base_text or text is None:
        return
    introduced = [c for c in COLUMNS if c not in (R.header_columns(base_text) or COLUMNS)]
    if not introduced:
        return
    rows, dirs, problems = R.parse(text, columns=R.header_columns(text) or LEGACY_COLUMNS)
    if problems:
        return
    M = R._clean(measured)
    new = {}
    for path in set(rows) | set(M):
        have = rows[path][0] if path in rows else {}
        row = {c: (M.get(path, {}).get(c, 0) if c in introduced else have.get(c, 0)) for c in COLUMNS}
        new[path] = (row, rows[path][1] if path in rows else "")
    with open(os.path.join(root, BASELINE), "w", encoding="utf-8") as f:
        f.write(R.render(new, dirs))


def write_pin(root):
    """Rewrite the clippy pin row of the allow file to the section's current value; returns (old, new)."""
    path = os.path.join(root, ALLOW)
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    new, old = clippy_pin(root), None
    for k, ln in enumerate(lines):
        parts = ln.split("\t")
        if len(parts) == 5 and parts[0] == "pin" and parts[1] == PIN_TARGET:
            old, parts[2] = parts[2], new
            lines[k] = "\t".join(parts)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    return old, new


def cmd_update(root, reason):
    measured, _, problems = measure(root)
    # the pin differs only because clippy.toml changed: --update-baseline is the one step that re-pins it
    fatal = [p for p in problems if "the pin says" not in p]
    if fatal:
        for p in fatal:
            print(p, file=sys.stderr)
        return 1
    R = ratchet(root)
    seed_introduced(root, R, measured)
    errors, wrote = R.update(measured, reason)
    for e in errors:
        print(e, file=sys.stderr)
    if wrote:
        old, new = write_pin(root)
        if old is not None and old != new:
            print(f"{GATE}: PIN CHANGED {PIN_TARGET}: {old} -> {new} (review the clippy.toml diff)")
        print(f"{GATE}: wrote {BASELINE} ({len(R._clean(measured))} files)")
    return 0 if wrote else 1


# ---------------------------------------------------------------------------------------------------- self-test
GIT_ENV = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid", "GIT_COMMITTER_NAME": "t",
           "GIT_COMMITTER_EMAIL": "t@example.invalid", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_SYSTEM": os.devnull}

CLEAN_RS = 'fn plain() -> u32 { 1 }\n'
GRANT_ROW = '---\nid: {n}\ntitle: "t"\nstatus: "{status}"\njs-grant: {grant}\nadds: []\n---\n\n# Row {n}\n'
ALLOW_MIN = ("# test allow file\npin\tclippy.toml#disallowed-methods\tabsent\t-\tthe sink ban list is held by hash in the test\n")
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

    def make_base(root, files, note=None, legacy=False):
        git(root, "init", "-q")
        for p, t in files.items():
            write(root, p, t)
        git(root, "add", "-A")
        measured, _, _ = measure(root)
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
        if legacy:  # the baseline format that predates the `#! columns:` header and the bytes column
            with open(os.path.join(root, BASELINE), encoding="utf-8") as f:
                rows = [ln.split("\t") for ln in f.read().split("\n") if ln and not ln.startswith("#")]
            write(root, BASELINE, "# old\n" + "".join("\t".join(r[:7]) + "\n" for r in rows))
        git(root, "add", "-A")
        git(root, "commit", "-q", "-m", "base")
        return git(root, "rev-parse", "HEAD")

    def scenario(label, want, files, mutate, base_note=None, env=None, why=None, legacy=False, show=None):
        nonlocal ok_count
        with tempfile.TemporaryDirectory(prefix="check-js-volume-st.") as root:
            sha = make_base(root, files, base_note, legacy)
            saved = {k: os.environ.get(k) for k in ("RATCHET_BASE_REF", "RATCHET_NO_BASE")}
            os.environ["RATCHET_BASE_REF"] = sha
            os.environ.pop("RATCHET_NO_BASE", None)
            for k, v in (env or {}).items():
                os.environ[k] = v
            try:
                mutate(root)
                git(root, "add", "-A")
                R, rep, _ = run_check(root)
            finally:
                for k, v in saved.items():
                    if v is None:
                        os.environ.pop(k, None)
                    else:
                        os.environ[k] = v
            got = "red" if rep.errors else "green"
            detail = R.format(rep)
        if show and got == want and show not in detail:
            fails.append(f"{label}: expected the output to say {show!r}, got:\n{detail}")
        elif got == want and why and why not in detail:
            fails.append(f"{label}: red, but not for the expected reason ({why!r}):\n{detail}")
        elif got != want:
            fails.append(f"{label}: expected {want}, got {got}\n{detail}")
        else:
            ok_count += 1
        return detail

    base = {ALLOW: ALLOW_MIN, "primitives/src/a.rs": EVAL_RS, "primitives/src/clean.rs": CLEAN_RS,
            "dev-docs/backlog/434-grant.md": GRANT_ROW.format(n=434, grant="sink=10, js_lit=100, js_file_bytes=100000", status="APPROVED"),
            "dev-docs/backlog/435-small.md": GRANT_ROW.format(n=435, grant="js_file_bytes=1", status="APPROVED"),
            "dev-docs/backlog/done/436-closed.md": GRANT_ROW.format(n=436, grant="sink=10", status="APPROVED")}

    def append(path, text):
        return lambda root: write(root, path, open(os.path.join(root, path), encoding="utf-8").read() + text)

    def baseline_edit(fn):
        def mut(root):
            p = os.path.join(root, BASELINE)
            with open(p, encoding="utf-8") as f:
                write(root, BASELINE, fn(f.read()))
        return mut

    def write_rows(root, note=None, note_path="primitives/src/a.rs", directive=None):
        # the baseline is rewritten to match the code, as an agent would; the note and the directive are the lane's claim
        measured, _, _ = measure(root)
        R = ratchet(root)
        rows = {p: (c, note if note and p == note_path else "") for p, c in R._clean(measured).items()}
        write(root, BASELINE, R.render(rows, [directive] if directive else []))

    def bump_baseline(note):
        return lambda root: write_rows(root, note)

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
    scenario("a new file in the generated bundle directory counts by its bytes", "red", base,
             lambda r: write(r, "primitives/src/js/gen.js", "const a = 1;\nfoo(a);\n"), why='js_file_bytes 0->')
    scenario("an inline <script> in a new html file", "red", base,
             lambda r: write(r, "preview/page.html", "<html><script>var a = 1;</script></html>\n"))
    scenario("a comment that presents eval as the house pattern", "red", base,
             append("primitives/src/clean.rs", "// this crate's `document::eval` house pattern\n"))
    scenario("a count that fell and was not written back", "red", base,
             lambda r: write(r, "primitives/src/a.rs", "fn gone() {}\n"), why='lock the drop in')
    scenario("a baseline row whose file is gone", "red", base,
             lambda r: os.remove(os.path.join(r, "primitives/src/a.rs")), why='stale baseline row')
    scenario("a baseline that was deleted", "red", base, lambda r: os.remove(os.path.join(r, BASELINE)), why='is missing but the base')
    # raised counts: the code and the baseline agree, only the merge-base compare can see it
    raised_code = append("primitives/src/a.rs", 'fn two() { let _ = eval("x"); }\n')

    def raise_with(note):
        def mut(root):
            raised_code(root)
            bump_baseline(note)(root)
        return mut

    scenario("a raised count with no note (tree and baseline agree)", "red", base, raise_with(None), why='rises above the base')
    detail = scenario("a raised count with a `# raised: row N` note and a grant at the base", "green", base,
                      raise_with("# raised: row 434 test raise"))
    if "RATCHET RAISES" not in detail or "test raise" not in detail:
        fails.append("a raise with a note must print under the RATCHET RAISES heading, got:\n" + detail)
    scenario("a note that cites a row with no grant at the base", "red", base, raise_with("# raised: row 999 self-service"),
             why='row 999 does not exist at the base')
    scenario("a note that cites a closed row", "red", base, raise_with("# raised: row 436 test"), why='is under done/')
    scenario("a note that cites a grant too small for the raise", "red", base, raise_with("# raised: row 435 test"),
             why='row 435 grants sink=0')
    scenario("a malformed note (no row number)", "red", base, raise_with("# raised: because"), why='malformed')
    scenario("a note the merge base already has cannot be spent again", "red", base, raise_with("# raised: row 434 old"),
             base_note="# raised: row 434 old", why='already has')
    scenario("the same raise passes when only the tree is compared (the merge-base half is what catches it)", "green",
             base, raise_with(None), env={"RATCHET_NO_BASE": "1"})
    scenario("the same raise fails under RATCHET_REQUIRE_BASE with no base", "red", base, raise_with(None),
             env={"RATCHET_NO_BASE": "1", "RATCHET_REQUIRE_BASE": "1"}, why='RATCHET_REQUIRE_BASE')
    # the total
    moved_files = dict(base)

    def move_into_js(directive):
        def mut(root):
            write(root, "primitives/src/a.rs", CLEAN_RS)  # js_lit of a.rs drops to zero (3 units of 120 bytes)
            write(root, "templates/starter/new.js", "\n".join(f"const value{k} = compute({k});" for k in range(60)) + "\n")
            git(root, "add", "-A")
            line = directive
            if line is not None:
                measured, _, _ = measure(root)
                line = line.replace("TOTAL", str(total_of(sums(measured))))
            write_rows(root, "# raised: row 434 moved", "templates/starter/new.js", line)
        return mut

    scenario("moving text from a Rust string into a new .js raises the total", "red", moved_files, move_into_js(None), why='total rose')
    detail = scenario("the same move with a `#! total-raised:` line", "green", moved_files,
                      move_into_js("#! total-raised: row 434 total=TOTAL moved"))
    if "TOTAL" not in detail:
        fails.append("a total raise must be listed under RATCHET RAISES, got:\n" + detail)
    scenario("the same move with a wrong total number", "red", moved_files, move_into_js("#! total-raised: row 434 total=1 moved"),
             why='carries total=1')

    # ---- what is counted
    scenario("a minified one-liner .js counts by bytes, not by lines", "red", base,
             lambda r: write(r, "templates/starter/min.js", "a" * 4000 + "\n"), why='js_file_bytes 0->4001')
    scenario("control: TypeScript source under primitives/src/ts/ rides the bundle", "green", base,
             lambda r: write(r, "primitives/src/ts/x.ts", "export const a = 1;\n"))
    scenario("control: ts-test, .d.ts and .gen.ts are not shipped", "green", base, lambda r: (
        write(r, "primitives/ts-test/x.test.ts", "export const a = 1;\n"),
        write(r, "templates/starter/types.d.ts", "declare const a: number;\n"),
        write(r, "templates/starter/x.gen.ts", "export const a = 1;\n")))
    scenario("an untracked file is in the universe", "red", base, lambda r: write(r, "preview/src/new.js", "const a = 1;\n"))
    scenario("a markdown file with an inline <script>", "red", base,
             lambda r: write(r, "templates/starter/README.md", "# t\n\n<script>run()</script>\n"), why='inline_script')
    scenario("control: a markdown fenced block and code span are displayed samples", "green", base,
             lambda r: write(r, "templates/starter/README.md",
                             "# t\n\n```html\n<script>run()</script>\n<a onclick=\"x()\">y</a>\n```\n\nuse `<script>x</script>`\n"))
    scenario("an svg with a <script>", "red", base,
             lambda r: write(r, "preview/assets/x.svg", "<svg xmlns='http://www.w3.org/2000/svg'><script>run()</script></svg>\n"))
    scenario("an svg with an onload handler", "red", base,
             lambda r: write(r, "preview/assets/x.svg", "<svg onload=\"run()\" xmlns='http://www.w3.org/2000/svg'></svg>\n"),
             why='sink 0->1')
    # ---- the sinks
    scenario("a function pointer through a module alias `dd::eval`", "red", base,
             append("primitives/src/clean.rs", 'use dioxus::document as dd;\nfn t() { let f = dd::eval; let _ = f; }\n'),
             why='new file with a non-zero count')
    scenario("gloo_events named in code", "red", base, append("primitives/src/clean.rs", 'use gloo_events::EventListener;\n'))
    scenario("gloo_file named in code", "red", base, append("primitives/src/clean.rs", 'use gloo_file::Blob;\n'))
    sites = measure_rust('fn t() { let g = dioxus::document::eval; let f = dd::eval; }\n')[1]
    if [k for _, k in sites] != ["sink:eval", "sink:eval-path"]:
        fails.append(f"sink:eval must be the document-rooted path and sink:eval-path the other path, got {sites}")
    # ---- the expect column and the lint allow
    expect_rs = '#[expect(clippy::disallowed_methods, reason = "bridge")]\nfn e() {}\n'
    scenario("an added #[expect(clippy::disallowed_methods)] raises the column", "red", base,
             lambda r: (append("primitives/src/clean.rs", expect_rs)(r), bump_baseline(None)(r)), why='rises above the base')
    scenario("an #[expect(clippy::disallowed_methods)] with the baseline untouched", "red", base,
             append("primitives/src/clean.rs", expect_rs), why='new file with a non-zero count')
    scenario("control: an unrelated #[expect(clippy::too_many_arguments)] is not counted", "green", base,
             append("primitives/src/clean.rs", '#[expect(clippy::too_many_arguments)]\nfn e() {}\n'))
    scenario("a column the base predates is introduced, not raised (old-format base)", "green", base,
             lambda r: (append("primitives/src/clean.rs", expect_rs)(r), cmd_update(r, None)), legacy=True,
             show='expect_disallowed introduced')
    scenario("#[allow(clippy::disallowed_methods)] outside transport.rs", "red", base,
             append("primitives/src/clean.rs", '#[allow(clippy::disallowed_methods)]\nfn e() {}\n'), why='outside primitives/src/bridge/transport.rs')
    scenario("#![allow(clippy::disallowed_methods)] crate-wide", "red", base,
             append("primitives/src/clean.rs", '#![allow(clippy::disallowed_methods)]\n'), why='outside')
    scenario("control: the same allow in transport.rs", "green", base,
             lambda r: write(r, TRANSPORT, '#[allow(clippy::disallowed_methods)]\nfn e() {}\n'))
    # ---- the clippy pin
    clippy_toml = 'disallowed-methods = [{ path = "dioxus::document::eval", reason = "bridge only" }]\n'

    def repin(root):
        write(root, "clippy.toml", clippy_toml)
        write_pin(root)

    scenario("a disallowed-methods section added to clippy.toml with the pin unchanged", "red", base,
             lambda r: write(r, "clippy.toml", clippy_toml), why='the pin says')
    scenario("the same change with the pin rewritten", "green", base, repin)
    scenario("a comment-only edit of clippy.toml leaves the pin", "green", base,
             lambda r: write(r, "clippy.toml", "# note\nallow-unwrap-in-tests = true\n"))
    scenario("an entry removed from the pinned section", "red", {**base, "clippy.toml": clippy_toml,
             ALLOW: ALLOW_MIN.replace("absent", pin_of_text(clippy_toml))},
             lambda r: write(r, "clippy.toml", "disallowed-methods = []\n"), why='the pin says')
    scenario("an allow file without a pin row", "red", base,
             lambda r: write(r, ALLOW, "# nothing\n"), why='no `pin` row')
    scenario("a deleted allow file", "red", base, lambda r: os.remove(os.path.join(r, ALLOW)), why='is missing')
    # ---- the allow rows
    a_row = "allow\tprimitives/src/a.rs\tsink=2,js_lit=2\tel.focus()\tthe fixture's eval is the owner's reasoned exception"

    def with_rows(*rows, rewrite=True):
        def mut(root):
            write(root, ALLOW, ALLOW_MIN + "\n".join(rows) + "\n")
            if rewrite:
                write_rows(root)
        return mut

    scenario("an allow row excuses its counts (the baseline falls with it)", "green", base, with_rows(a_row))
    scenario("an allow row with the baseline not written back", "red", base, with_rows(a_row, rewrite=False), why='lock the drop in')
    scenario("an allow row whose anchor is gone", "red", base, with_rows(a_row.replace("el.focus()", "gone()")), why='anchor')
    scenario("an allow row for more than the file has", "red", base, with_rows(a_row.replace("sink=2", "sink=9")), why='the exception shrank')
    scenario("an allow row for a file that is gone", "red", base, with_rows(a_row.replace("a.rs", "b.rs")), why='stale allow row')
    scenario("an allow row with no reason", "red", base, with_rows(a_row.rsplit("\t", 1)[0] + "\tbecause"), why='must say why')
    scenario("an allow row cannot excuse more than its count: a second eval is ratcheted", "red", base,
             lambda r: (with_rows(a_row)(r), append("primitives/src/a.rs", 'fn two() { let _ = eval("x"); }\n')(r)))
    vendor = "templates/starter/vendor/lib.js"
    scenario("a vendored file is counted (no exclusion without a row)", "red", base, lambda r: write(r, vendor, "var a = 1;\n"))
    glob_row = "exclude-glob\t*/vendor/*\t-\t-\tthird-party code that the owner reviewed"
    scenario("an exclude-glob row takes a vendored file out", "green", base,
             lambda r: (write(r, vendor, "var a = 1;\n"), with_rows(glob_row, rewrite=False)(r)))
    scenario("an exclude-glob row that matches nothing is stale", "red", base, with_rows(glob_row, rewrite=False), why='stale exclude-glob')
    file_row = "exclude-file\ttemplates/starter/demo.js\t-\t-\tonly shown on a docs page, never loaded"
    scenario("an exclude-file row takes a displayed sample out", "green", base,
             lambda r: (write(r, "templates/starter/demo.js", "var a = 1;\n"), with_rows(file_row, rewrite=False)(r)))
    scenario("an exclude-file row whose file is gone is stale", "red", base, with_rows(file_row, rewrite=False), why='stale exclude-file')
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
