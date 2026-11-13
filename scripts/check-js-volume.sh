#!/usr/bin/env bash
#
# check-js-volume.sh  (backlog row 434; rule and replacements: dev-docs/js-interop.md)
#
# JavaScript volume can only shrink: Rust owns behaviour and browser access goes through the bridge in
# `primitives/src/bridge/` (dev-docs/js-interop.md; ref-ok: lands with row 434 lane 3.0), never through a script string, a new `.js`/`.ts` file or an inline
# script. The gate counts, per tracked file, the places JS can enter (eval sinks, JS held in Rust strings, hand-written
# `.js`/`.ts` bytes, `JsListeners::install` sites, inline `<script>` bytes) and fails when a count rises, against the
# baseline AND against the baseline committed at the merge base with origin/main, so editing the baseline in the same
# commit is not a way through. The legacy JS and its sinks are frozen too (THE LEGACY FREEZE below).
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
#                 `inline_js`/`srcdoc`, an `extern` item that
#                 can reach script: an `extern "<abi>" { .. }` or `unsafe extern "<abi>" { .. }` import block (with or
#                 without `#[wasm_bindgen]`), a bare `extern { .. }`, an `extern "<abi>" fn` TYPE (a function pointer can
#                 hold an imported function) and any `extern` shape that is not plainly a definition. A function
#                 DEFINITION with a body, `[pub] [unsafe] extern "<abi>" fn name(..) { .. }`, is not a sink: it is Rust
#                 the host may call (a wasm export, `#[no_mangle]`) and imports nothing; in a string: an `include_str!`/`include_bytes!`/`asset!` of a
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
# number. Two granted rows that raise one file are a CHAIN in its note (`# raised: row 454 <why> (js_file_bytes
# 19956->22234); row 451 <why> (js_file_bytes 22234->22558)`; the total has one `#! total-raised:` line per row,
# each ending `(total A->B)`): the steps run from the base's number to the tree's without a gap and each is charged
# to its own row's grant only. Run `--update-baseline --allow-raise "row N reason"` once per row, in merge order: a
# second run appends its step to a note made in this diff and refuses a step its row does not grant (the file stays
# as it was).
#
# THE LEGACY FREEZE (`scripts/check-js-volume.freeze.tsv`, backlog row 445): the legacy JS stays in the tree until its
# port row deletes it, and until then nothing legacy changes except by deletion or by a granted bug fix. Legacy is
# everything the scan counts outside `primitives/src/bridge/` and the generated bundle `primitives/src/js/`. The gate
# scans the tree and compares what it finds with the freeze data, so a script or sink that exists and is not in the
# data is red (no hand-kept list). Two kinds of entry, each keyed by (file, kind, sha256-16 of its identity text),
# with a count: `script` (a Rust string group that reads as JS, a hand-written `.js`/`.ts` file, an inline `<script>`
# body; the identity is its exact text, so any edit is a new, unfrozen script) and `site` (every `sink:*` site; the
# identity is the whole argument of the call or block, the `use` statement, the string literal or the source line of a
# bare token, whitespace and rustfmt's bracket spacing normalised: never a line number, so unrelated edits keep it,
# while a re-pointed site, a copied script or a new site has an identity the data lacks). A frozen entry that is gone
# from the tree is red until the data is lowered. The tree's data is also compared with the base's (the same base as
# the ratchet): an entry the base lacks (or a higher count) passes only with the note `# fixed: row N <reason>` where
# row N exists and is open AT THE BASE and its front matter says `js-fix: <path>[, <path>]` naming the file (the
# `js-grant` rule: a grant in the lane's own diff is not at the base; a `js-fix` cannot add a sink site, only change
# one, and grants no volume raise, which still needs a `js-grant`). The allow rows do not excuse a freeze entry.
# Lowering: `--update-baseline` rewrites the baseline AND the freeze data from the tree and refuses to add or re-add
# an entry; `--update-freeze` is its freeze half, and `--update-freeze --js-fix "row N <why>"` records a granted fix
# (refused unless the grant at the base names the file). Two lanes that delete different scripts merge cleanly (one
# line per entry); after a merge run `scripts/check-js-volume.sh --update-baseline` on the merged tree.
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
#   scripts/check-js-volume.sh --update-freeze [--js-fix "row N reason"]   the freeze half alone; lowers, or records a granted fix
#   scripts/check-js-volume.sh --self-test              mutations (added eval, aliased `use ... eval as`, `concat!` split, inline
#       `document::Script`, new `.js`/`.ts`, raised count without a note or without a grant at the base, a raise only the merge-base compare sees, total rise, wrong total; for the freeze: an edited
#       script, a re-pointed or new sink site and a copied script, a freeze raised without a grant, a delete-and-lower and a `js-fix` fix) must turn red or green as stated
# Environment: RATCHET_BASE_REF pins the base, RATCHET_NO_BASE=1 skips that half, RATCHET_REQUIRE_BASE=1 makes "no base"
# red (scripts/lib/ratchet.py).
# Exit status: 0 clean; 1 a count rose, fell unwritten, or is stale, or a legacy script or sink changed; 2 bad usage or a broken self-test.
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
from ratchet import Ratchet, selftest_env  # noqa: E402
from rows import CLOSED_STATUS, GRANT_COLUMNS, front_matter, git_text, row_at, row_problems  # noqa: E402

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
FREEZE = "scripts/check-js-volume.freeze.tsv"
FREEZE_EXEMPT = (BUNDLE_DIR, "primitives/src/bridge/")  # the generated bundle and the typed seam are not legacy
FIX_NOTE = re.compile(r"^#\s*fixed:\s*row\s+(\d+)\b.*\S")
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
    ("ffi", re.compile(r"\b(?:web_sys|js_sys|wasm_bindgen|gloo_utils|gloo_events|gloo_file|evaluate_script|inline_js|srcdoc)\b|\bextern(?:\s*\"|(?=\s*(?:\{|fn\b)))")),
]
EXTERN_DEF = re.compile(r"fn\s+(?:r#)?[A-Za-z_]\w*\s*")


def extern_defines(code, m):
    """True when the `extern` match `m` (a `ffi` hit that starts with `extern`) heads a function DEFINITION, `extern
    "<abi>" fn name(..) .. { body }`: Rust code the host may call, which runs no script. Everything else is False, so it
    stays a sink: a block (`extern "C" { .. }`, `unsafe extern "C" { .. }`, bare `extern { .. }`), a function pointer
    TYPE (`extern "C" fn(u32)`, which can hold an imported function), a declaration without a body, and any shape this
    cannot read (unbalanced brackets, no body before the end of the text). The ABI string is blanked in the code view,
    so every ABI is treated alike."""
    i = m.end()
    if code[m.start():i].endswith('"'):
        i = code.find('"', i)
        if i < 0:
            return False
        i += 1
    while i < len(code) and code[i].isspace():
        i += 1
    d = EXTERN_DEF.match(code, i)
    if not d:
        return False
    i = d.end()
    if code.startswith("<", i):
        depth = 0
        while i < len(code):
            if code[i] == "<":
                depth += 1
            elif code[i] == ">" and code[i - 1] != "-":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        else:
            return False
        i += 1
        while i < len(code) and code[i].isspace():
            i += 1
    if not code.startswith("(", i):
        return False
    i = match_close(code, i) + 1
    depth = 0
    while i < len(code):
        c = code[i]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif depth == 0 and c == "{":
            return True
        elif depth == 0 and c == ";":
            return False
        i += 1
    return False
EXPECT_RX = re.compile(r"#\s*!?\s*\[\s*(?:cfg_attr\s*\(\s*dx_repo_lints\s*,\s*)?expect\s*\(([^\]]*)\)\s*\]", re.S)
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


def match_close(code, open_idx):
    """Index of the bracket closing the `(` or `{` at `open_idx` of a code view (strings are blanked, so a bracket in
    a string cannot unbalance it); the end of the text when it never closes."""
    pair = {"(": ")", "{": "}"}
    opener, closer, depth = code[open_idx], pair[code[open_idx]], 0
    for k in range(open_idx, len(code)):
        if code[k] == opener:
            depth += 1
        elif code[k] == closer:
            depth -= 1
            if depth == 0:
                return k
    return len(code) - 1


def site_argument(text, code, m):
    """The text a sink site runs, for the freeze: a call or block is its whole bracketed argument (`eval(<script>)`,
    `Script { .. }`), a `use ... eval;` is the statement, any other token is its source line. Never a line number,
    so an unrelated edit above the site leaves it alone."""
    s, e = m.start(), m.end()
    if m.lastgroup == "eval_use":
        j = code.find(";", e)
        return text[s:(j + 1 if j >= 0 else e)]
    open_i = e - 1 if code[e - 1] in "({" else None
    if open_i is None:
        k = e
        while k < len(code) and code[k].isspace():
            k += 1
        if k < len(code) and code[k] in "({":
            open_i = k
    if open_i is not None:
        return text[s:match_close(code, open_i) + 1]
    ls = text.rfind("\n", 0, s) + 1
    le = text.find("\n", e)
    return text[ls:(len(text) if le < 0 else le)]


def measure_rust(text, path="", idents=None):
    """`(counts, sites)`; `sites` is a list of (line, kind[, n]) for --inventory. `idents`, when a list, receives the
    freeze identity of every legacy script and sink site: `(class, kind, identity text, size)`."""
    c = dict.fromkeys(COLUMNS, 0)
    sites = []
    code = cs.code_of(text)
    for kind, rx in SINK_RX:
        for m in rx.finditer(code):
            if kind == "ffi" and m.group(0).startswith("extern") and extern_defines(code, m):
                continue
            c["sink"] += 1
            name = "sink:" + (m.lastgroup.replace("_", "-") if m.lastgroup else kind)
            sites.append((line_at(text, m.start()), name))
            if idents is not None:
                arg = site_argument(text, code, m)
                idents.append(("site", name[5:], norm(arg), len(arg)))
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
            if idents is not None:
                idents.append(("site", why[5:], norm(content), len(content)))
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
            if idents is not None:
                idents.append(("script", "lit", content, n))
    for line, body in cs.extract_rust(text):
        if COMMENT_EVAL.search(body):
            c["comment_eval"] += 1
            sites.append((line, "comment_eval"))
    return c, sites


# ---------------------------------------------------------------------------------------------------- JS and HTML
def measure_js(text, idents=None):
    """Bytes of the whole file: a minified one-liner or a comment-stuffed file cannot hide in a line count."""
    n = len(text.encode("utf-8", "replace"))
    c = dict.fromkeys(COLUMNS, 0)
    c["js_file_bytes"] = n
    if idents is not None and n:
        idents.append(("script", "file", text, n))
    return c, [(1, f"js_file_bytes:{n}")] if n else []


SCRIPT_RX = re.compile(r"<script\b([^>]*)>(.*?)</script\s*>", re.S | re.I)
HTML_HANDLER = re.compile(r"<[A-Za-z][^>]*\son[a-z]{3,}\s*=|\bhref\s*=\s*[\"']\s*javascript:|\bsrcdoc\s*=", re.I | re.S)


def measure_html(text, idents=None):
    c = dict.fromkeys(COLUMNS, 0)
    sites = []
    for m in SCRIPT_RX.finditer(text):
        if re.search(r"\bsrc\s*=", m.group(1), re.I):
            c["sink"] += 1
            sites.append((line_at(text, m.start()), "sink:external-script"))
            if idents is not None:
                idents.append(("site", "external-script", norm(m.group(1)), len(m.group(1))))
        else:
            n = len(m.group(2).strip().encode())
            c["inline_script"] += n
            sites.append((line_at(text, m.start()), f"inline_script:{n}"))
            if idents is not None:
                idents.append(("script", "inline", m.group(2), n))
    blanked = SCRIPT_RX.sub(lambda mm: " " * len(mm.group(0)), text)
    for m in HTML_HANDLER.finditer(blanked):
        c["sink"] += 1
        sites.append((line_at(text, m.start()), "sink:html-handler"))
        if idents is not None:
            end = blanked.find(">", m.end())
            tag = text[m.start():(m.end() + 200 if end < 0 else end + 1)]
            idents.append(("site", "html-handler", norm(tag), len(tag)))
    return c, sites


FENCE_RX = re.compile(r"^(`{3,}|~{3,}).*?^\1[ \t]*$", re.S | re.M)
CODE_SPAN_RX = re.compile(r"(`+)(?:(?!\1).)+\1", re.S)


def measure_md(text, idents=None):
    """Markdown holds HTML: an inline `<script>`, an `on...=` handler, a `javascript:` link. A fenced block and a code
    span are displayed code samples, not page content, and are blanked first."""
    blank = lambda m: re.sub(r"[^\n]", " ", m.group(0))  # noqa: E731 (keeps the line numbers)
    return measure_html(CODE_SPAN_RX.sub(blank, FENCE_RX.sub(blank, text)), idents)


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
    return measure_all(root)[:3]


def measure_all(root):
    """`measure` plus the freeze identities: `(measured, sites, problems, frozen)`, `frozen` maps a freeze key to its
    `{count, size, preview}` (see `freeze_key`)."""
    allow, problems = load_allow(root)
    files, p2 = universe(root, allow)
    problems += p2
    out, all_sites, texts, frozen = {}, {}, {}, {}
    for path in files:
        with open(os.path.join(root, path), encoding="utf-8", errors="replace") as f:
            text = f.read()
        texts[path] = text
        idents = None if path.startswith(FREEZE_EXEMPT) else []
        if path.endswith(".rs"):
            c, sites = measure_rust(text, path, idents)
            problems += lint_problems(root, path, text)
        elif js_counted(path):
            c, sites = measure_js(text, idents)
        elif path.endswith(MD_EXT):
            c, sites = measure_md(text, idents)
        else:
            c, sites = measure_html(text, idents)
        for cls, kind, ident, size in idents or ():
            ent = frozen.setdefault(freeze_key(path, cls, kind, ident), {"count": 0, "size": size, "preview": preview(ident)})
            ent["count"] += 1
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
    return out, all_sites, problems, frozen


def sums(rows):
    return {k: sum(c[k] for c in rows.values()) for k in COLUMNS}


# ---------------------------------------------------------------------------------------------------- freeze
# THE LEGACY FREEZE. Legacy = everything the scan counts outside the bridge (`primitives/src/bridge/`, the typed seam,
# held by check-bridge-behaviours) and the generated bundle (`primitives/src/js/`, pinned by check-js-bundle). A freeze
# entry is one of
#   script  a unit of legacy JavaScript: `lit` a Rust string group that reads as JS (the `js_lit` unit, `concat!`
#           pieces joined), `file` a hand-written `.js`/`.ts` file, `inline` an inline `<script>` body; identity is the
#           sha256 of the exact text, so ANY edit of it is a different entry
#   site    one sink site (every `sink:*` kind of the scan); identity is the sha256 of the normalised text the site runs
#           (the whole argument of a call or block, the `use` statement, the string literal, or the source line of a
#           bare token), never a line number, so an unrelated edit above it keeps it and re-pointing it changes it
# Counted per (path, class, kind, identity), so two equal sites are `count 2`.
def norm(text):
    """The identity form of a site's text: whitespace runs are one space and the space and trailing comma rustfmt
    moves around brackets and commas are dropped, so a reformat keeps the identity and a different argument does not."""
    t = " ".join(text.split())
    t = re.sub(r",\s*([)\]}])", r"\1", t)
    return re.sub(r"\s*([(){}\[\],])\s*", r"\1", t)


def preview(ident):
    n = norm(ident)
    return n if len(n) <= 60 else n[:57] + "..."


def freeze_key(path, cls, kind, ident):
    import hashlib
    return (path, cls, kind, hashlib.sha256(ident.encode("utf-8", "replace")).hexdigest()[:16])


FREEZE_HEADER = (
    "# check-js-volume freeze (scripts/check-js-volume.sh): the legacy scripts and sink sites, delete-only.\n"
    "# class<TAB>path<TAB>kind<TAB>sha256-16<TAB>count<TAB>size<TAB>preview[<TAB># fixed: row N <reason>]\n"
    "# script = a unit of legacy JS by the hash of its exact text; site = a sink site by the hash of what it runs.\n"
    "# Regenerate by `scripts/check-js-volume.sh --update-baseline` (lowers only); a fix is `--update-freeze --js-fix`.\n")


def parse_freeze(text, label=""):
    """`(entries, problems)`: entries maps a freeze key to {count, size, preview, note}."""
    entries, problems = {}, []
    for n, ln in enumerate(text.split("\n"), 1):
        if not ln.strip() or ln.startswith("#"):
            continue
        parts = ln.split("\t")
        note = parts.pop().strip() if len(parts) == 8 else ""
        if (len(parts) != 7 or parts[0] not in ("script", "site") or not re.fullmatch(r"[0-9a-f]{16}", parts[3])
                or not parts[4].isdigit() or not parts[5].isdigit() or int(parts[4]) < 1
                or (note and not FIX_NOTE.match(note))):
            problems.append(f"{label}{FREEZE}:{n}: malformed row (want class<TAB>path<TAB>kind<TAB>sha256-16<TAB>count<TAB>"
                            f"size<TAB>preview[<TAB># fixed: row N <reason>])")
            continue
        key = (parts[1], parts[0], parts[2], parts[3])
        if key in entries:
            problems.append(f"{label}{FREEZE}:{n}: duplicate row for {parts[1]} {parts[0]} {parts[2]} {parts[3]}")
            continue
        entries[key] = {"count": int(parts[4]), "size": int(parts[5]), "preview": parts[6], "note": note}
    return entries, problems


def render_freeze(entries):
    out = [FREEZE_HEADER]
    for key in sorted(entries, key=lambda k: (k[0], k[1], k[2], k[3])):
        e = entries[key]
        path, cls, kind, h = key
        row = [cls, path, kind, h, str(e["count"]), str(e["size"]), e["preview"]] + ([e["note"]] if e.get("note") else [])
        out.append("\t".join(row) + "\n")
    return "".join(out)


def read_freeze(root):
    try:
        with open(os.path.join(root, FREEZE), encoding="utf-8") as f:
            return parse_freeze(f.read())
    except OSError:
        return None, [f"{FREEZE} is missing: the legacy freeze lives there (deleting it is not a pass; restore it with git)"]


def fix_paths(value):
    """The paths of a `js-fix:` value (`a.rs, b.js (why)`); the column form `js_file=20` names no path."""
    return [p for p in re.split(r"[,\s]+", re.sub(r"\([^)]*\)", " ", value or "")) if p and "=" not in p]


def fix_grant_problem(root, sha, n, path):
    """Why row `n` at `sha` does not grant a fix of `path`, None when it does: the row exists at the base, is open
    and its `js-fix:` front matter lists the path (the same rule as `js-grant`; a grant in the lane's own diff is not
    at the base)."""
    row = row_at(root, sha, n)
    if row is None:
        return f"row {n} does not exist at the base {sha[:10]}"
    if not row.open:
        return f"row {n} is closed at the base ({row.why_closed()})"
    fm = front_matter(git_text(root, "show", f"{sha}:{row.path}") or "")
    if path not in fix_paths(fm.get("js-fix", "")):
        return f"row {n} has no `js-fix: {path}` at the base"
    return None


def _group_counts(entries):
    out = {}
    for (path, cls, kind, _), e in entries.items():
        out[(path, cls, kind)] = out.get((path, cls, kind), 0) + e["count"]
    return out


def freeze_added(new, old):
    """Keys whose count is higher in `new` than in `old` (a changed script or re-pointed site is a new key)."""
    return {k: e for k, e in new.items() if e["count"] > old.get(k, {"count": 0})["count"]}


def freeze_base(R):
    """`(entries or None, sha, note)`: the freeze data committed at the ratchet's base."""
    _, sha, note = R.base()
    if not sha:
        return None, None, "no base: the freeze is compared with the tree only"
    text = R._git("show", f"{sha}:{FREEZE}")
    if text is None:
        return None, sha, f"{FREEZE} does not exist at the base {sha[:10]}: the freeze is being introduced"
    return parse_freeze(text, "base ")[0], sha, ""


WHAT_NOW = "Delete it (then --update-baseline), or ask the owner for a grant row with `js-fix: {path}` at the merge base"


def check_freeze(root, R, frozen):
    """`(errors, fixes, notes)`: the measured freeze identities against the tree's data, then the tree's data against
    the base's. A legacy script or sink site may be deleted; an edit, a re-pointed site, a copy and a new site are red
    unless the tree's entry carries `# fixed: row N` and row N at the base has `js-fix:` naming the file."""
    errors, fixes, notes = [], [], []
    tree, problems = read_freeze(root)
    errors += [f"{GATE}: {p}" for p in problems]
    if tree is None:
        return errors, fixes, notes
    shown = {}
    for key in sorted(set(frozen) | set(tree)):
        m, t = frozen.get(key, {"count": 0}), tree.get(key, {"count": 0})
        path, cls, kind, _ = key
        if m["count"] > t["count"]:
            shown[path] = shown.get(path, 0) + 1
            if shown[path] <= 3:
                what = "a legacy script" if cls == "script" else "a sink site"
                errors.append(f"{GATE}: {path}: {what} ({kind}: `{frozen[key]['preview']}`) is not in the legacy freeze: "
                              f"it was edited, re-pointed, copied or added. " + WHAT_NOW.format(path=path) + ".")
            elif shown[path] == 4:
                errors.append(f"{GATE}: {path}: more legacy changes not listed")
        elif m["count"] < t["count"]:
            errors.append(f"{GATE}: {path}: frozen {cls} {kind} `{t.get('preview', '')}` is gone from the tree: run "
                          f"--update-baseline to lower the freeze")
    base, sha, why = freeze_base(R)
    if why:
        notes.append(why)
    if base is None:
        return errors, fixes, notes
    groups_t, groups_b = _group_counts(tree), _group_counts(base)
    for key, e in sorted(freeze_added(tree, base).items()):
        path, cls, kind, h = key
        mm = FIX_NOTE.match(e["note"])
        if not mm:
            errors.append(f"{GATE}: {path}: the freeze data holds a {cls} {kind} `{e['preview']}` the base does not and "
                          f"it has no `# fixed: row N <reason>` note. " + WHAT_NOW.format(path=path) + ".")
            continue
        n = int(mm.group(1))
        problem = fix_grant_problem(root, sha, n, path)
        if problem:
            errors.append(f"{GATE}: {path}: the fix cited as row {n} is not granted: {problem}. A grant written in this "
                          f"diff does not count: " + WHAT_NOW.format(path=path) + ".")
            continue
        if cls == "site" and groups_t[(path, cls, kind)] > groups_b.get((path, cls, kind), 0):
            errors.append(f"{GATE}: {path}: a fix may change a sink site but not add one (sink {kind} "
                          f"{groups_b.get((path, cls, kind), 0)} -> {groups_t[(path, cls, kind)]})")
            continue
        fixes.append(f"{path}: {cls} {kind} `{e['preview']}`   {e['note']}")
    return errors, fixes, notes


def update_freeze(root, R, frozen, reason=None):
    """Write the freeze data from the measurement. Lowering is free; a key the tree's data lacks (an edit, a copy, a
    re-add, a new site) is refused unless `reason` (`row N <why>`) names a row whose `js-fix:` at the base lists the
    file. Returns `(errors, wrote)`."""
    tree, problems = read_freeze(root)
    base, sha, _ = freeze_base(R)
    seed = False
    if tree is None:
        if base is not None:
            return problems, False  # the base has it: restore the file, do not reseed it
        tree, seed = {}, True  # the freeze is being introduced: its first writing records the tree as it is
    elif problems:
        return problems, False
    rm = re.match(r"row\s+(\d+)\b.*\S", reason) if reason is not None else None
    if reason is not None and not rm:
        return [f"{GATE}: --js-fix needs a reason that names a backlog row: \"row N <why>\""], False
    errors, out = [], {}
    added = {} if seed else freeze_added(frozen, tree)
    for key, m in frozen.items():
        path = key[0]
        note = tree.get(key, {}).get("note", "")
        if key in added:
            if rm is None:
                errors.append(f"{GATE}: {path}: {key[1]} {key[2]} `{m['preview']}` is not in the freeze data; the freeze "
                              f"only lowers. " + WHAT_NOW.format(path=path) + ", then --update-freeze --js-fix \"row N <why>\".")
                continue
            if not sha:
                errors.append(f"{GATE}: --js-fix needs a base to read the grant from (origin/main or origin/queue/*)")
                continue
            problem = fix_grant_problem(root, sha, int(rm.group(1)), path)
            if problem:
                errors.append(f"{GATE}: --js-fix \"{reason}\" refused for {path}: {problem}")
                continue
            note = f"# fixed: {reason}"
        elif note and base is not None and not freeze_added({key: m}, base):
            note = ""  # back at or under the base: the note has done its job
        out[key] = {"count": m["count"], "size": m["size"], "preview": m["preview"], "note": note}
    if errors:
        return errors, False
    with open(os.path.join(root, FREEZE), "w", encoding="utf-8") as f:
        f.write(render_freeze(out))
    return [], True


# ---------------------------------------------------------------------------------------------------- commands
def row_grant_problems(root):
    """One error per malformed grant line (`scripts/lib/rows.py`: `row_problems`) of every OPEN row file of the tree, so
    a `js-grant` key that is no column (the legacy `js_file`, a typo) is red where it is written, not at the lane
    that would have spent it. A closed row (under `done/` or a finished status) grants nothing and is not read."""
    errors = []
    folder = os.path.join(root, "dev-docs", "backlog")
    for name in sorted(os.listdir(folder)) if os.path.isdir(folder) else []:
        if not name.endswith(".md"):
            continue
        with open(os.path.join(folder, name), encoding="utf-8") as f:
            fm = front_matter(f.read())
        if CLOSED_STATUS.match(fm.get("status", "")):
            continue
        errors += [f"{GATE}: row {name.split('-')[0].lstrip('0')} (dev-docs/backlog/{name}): {p}" for p in row_problems(fm)]
    return errors


def run_check(root, freeze=True):
    measured, _, problems, frozen = measure_all(root)
    problems = problems + row_grant_problems(root)
    R = ratchet(root)
    rep = R.check(measured)
    rep.errors[:0] = problems
    rep.fixes = []
    if freeze:
        errors, rep.fixes, notes = check_freeze(root, R, frozen)
        rep.errors += errors
        rep.notes += notes
    base_text, sha, _ = R.base()
    if sha and not os.environ.get("RATCHET_NO_BASE"):
        old = R._git("show", f"{sha}:{ALLOW}") or ""
        old_pin = next((ln.split("\t")[2] for ln in old.split("\n") if ln.startswith("pin\t")), None)
        new_pin = clippy_pin(root)
        if old_pin is not None and old_pin != new_pin:
            rep.notes.append(f"RATCHET PIN: the disallowed-methods section of clippy.toml differs from the base ({old_pin} -> "
                             f"{new_pin}): review the clippy.toml diff")
    return R, rep, measured


def format_report(R, rep):
    text = R.format(rep)
    if rep.fixes:
        text = (f"FREEZE FIXES ({GATE}): {len(rep.fixes)} granted change(s) of frozen legacy in this diff, for the owner "
                f"to see:\n" + "\n".join("  " + f for f in rep.fixes) + ("\n" + text if text else ""))
    return text


def cmd_check(root, quiet=False):
    R, rep, measured = run_check(root)
    text = format_report(R, rep)
    if rep.errors:
        print(text, file=sys.stderr)
        print(f"{GATE}: FAIL -- {len(rep.errors)} problem(s). {ADVICE}", file=sys.stderr)
        return 1
    if text and not quiet:
        print(text)
    s = sums(measured)
    tree, _ = read_freeze(root)
    frozen_n = {c: sum(e["count"] for k, e in (tree or {}).items() if k[1] == c) for c in ("script", "site")}
    print(f"{GATE}: OK ({len(measured)} files carry JS: sink {s['sink']}, js_lit {s['js_lit']}, js_file_bytes {s['js_file_bytes']}, "
          f"install {s['install']}, inline_script {s['inline_script']} bytes, comment_eval {s['comment_eval']}, "
          f"expect_disallowed {s['expect_disallowed']}; total js_lit*{LONG_LINE} + js_file_bytes {total_of(s)}; "
          f"frozen: {frozen_n['script']} scripts, {frozen_n['site']} sink sites)")
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
        return cmd_update_freeze(root, None)  # the same command lowers the freeze; it never adds to it
    return 1


def cmd_update_freeze(root, fix):
    _, _, _, frozen = measure_all(root)
    R = ratchet(root)
    errors, wrote = update_freeze(root, R, frozen, fix)
    for e in errors:
        print(e, file=sys.stderr)
    if wrote:
        scripts = sum(e["count"] for k, e in frozen.items() if k[1] == "script")
        print(f"{GATE}: wrote {FREEZE} ({scripts} scripts, {sum(e['count'] for e in frozen.values()) - scripts} sink sites)")
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
        measured, _, _, frozen = measure_all(root)
        R = ratchet(root)
        R.base_ref = None
        with selftest_env(RATCHET_NO_BASE="1"):
            errors, wrote = R.update(measured, None)
            assert wrote, errors
            errors, wrote = update_freeze(root, R, frozen, None)
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

    def scenario(label, want, files, mutate, base_note=None, env=None, why=None, legacy=False, show=None, freeze=False):
        nonlocal ok_count
        with tempfile.TemporaryDirectory(prefix="check-js-volume-st.") as root:
            sha = make_base(root, files, base_note, legacy)
            with selftest_env(RATCHET_BASE_REF=sha, **(env or {})):
                mutate(root)
                git(root, "add", "-A")
                R, rep, _ = run_check(root, freeze)
            got = "red" if rep.errors else "green"
            detail = format_report(R, rep)
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
    # ---- `extern`: an import block is a sink, a function definition with a body is not
    ext_def = '#[no_mangle]\npub extern "C" fn dx_abi() -> u32 {\n    1\n}\n'
    ext_blk = 'extern "C" {\n    fn host_call(x: u32);\n}\n'
    scenario("an `extern \"C\" fn` export alone is not a sink", "green", base, append("primitives/src/clean.rs", ext_def))
    scenario("an `extern \"C\" { }` import block is a sink", "red", base, append("primitives/src/clean.rs", ext_blk), why="sink 0->1")
    scenario("an `unsafe extern \"C\" { }` block is a sink", "red", base,
             append("primitives/src/clean.rs", 'unsafe extern "C" {\n    pub safe fn host_call(x: u32);\n}\n'), why="sink 0->1")
    scenario("a `#[wasm_bindgen] extern \"C\" { }` block is a sink", "red", base,
             append("primitives/src/clean.rs", '#[wasm_bindgen]\nextern "C" {\n    fn host_call(x: u32);\n}\n'), why="sink 0->2")
    scenario("a file with an export and an import counts only the import", "red", base,
             append("primitives/src/clean.rs", ext_def + ext_blk), why="sink 0->1")
    scenario("an `extern \"C\" fn(u32)` function pointer type is a sink (fail closed)", "red", base,
             append("primitives/src/clean.rs", 'struct Cb {\n    f: extern "C" fn(u32),\n}\n'), why="sink 0->1")
    scenario("a bare `extern { }` block is a sink", "red", base,
             append("primitives/src/clean.rs", 'extern {\n    fn host_call(x: u32);\n}\n'), why="sink 0->1")
    scenario("an `extern \"C\" fn` declaration without a body is a sink (fail closed)", "red", base,
             append("primitives/src/clean.rs", 'extern "C" fn host_call(x: u32);\n'), why="sink 0->1")
    for label, src, want in [
            ("a generic, `where` and array-return export", 'pub unsafe extern "C" fn f<T: Copy>(x: T) -> [u8; 4]\nwhere\n    T: Sized,\n{\n    [0; 4]\n}\n', 0),
            ("an export with a multi-line signature", 'pub(crate) extern "C" fn f(\n    a: u32,\n    b: u32,\n) -> u32 {\n    a + b\n}\n', 0),
            ("an `extern fn` export with no ABI string", 'pub extern fn f() {}\n', 0),
            ("an export and a pointer type", 'extern "C" fn f() {}\nstatic G: Option<extern "C" fn()> = None;\n', 1),
            ("a definition that never reaches its body", 'extern "C" fn f(', 1)]:
        got = len(measure_rust(src)[1])
        if got != want:
            fails.append(f"{label}: expected {want} sink site(s), got {got}")
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

    # ---- the legacy freeze: each scenario runs with the freeze check on (the ratchet scenarios above test the counts alone)
    fz_rs = ('const A: &str = "const el = document.getElementById(\\"a\\"); el.focus(); el.blur();";\n'
             'const B: &str = "const el = document.getElementById(\\"b\\"); el.focus(); el.blur();";\n'
             'fn go() { let _ = dioxus::document::eval(A); }\n')
    fix_row = '---\nid: {n}\ntitle: "t"\nstatus: "{status}"\njs-fix: {fix}\nadds: []\n---\n\n# Row {n}\n'
    fz = {ALLOW: ALLOW_MIN, "primitives/src/a.rs": fz_rs, "primitives/src/clean.rs": CLEAN_RS,
          "primitives/src/old.js": "var a = 1;\nrun(a);\n",
          "preview/page.html": "<html><script>var inline = 1; run(inline);</script></html>\n",
          "dev-docs/backlog/437-fix.md": fix_row.format(n=437, fix="primitives/src/a.rs, primitives/src/old.js", status="APPROVED"),
          "dev-docs/backlog/438-other.md": fix_row.format(n=438, fix="primitives/src/other.rs", status="APPROVED"),
          "dev-docs/backlog/434-grant.md": base["dev-docs/backlog/434-grant.md"],
          "dev-docs/backlog/done/439-closed.md": fix_row.format(n=439, fix="primitives/src/a.rs", status="APPROVED")}

    def edit(path, old, new):
        def mut(root):
            full = os.path.join(root, path)
            with open(full, encoding="utf-8") as f:
                text = f.read()
            assert old in text, (path, old)
            write(root, path, text.replace(old, new))
        return mut

    def seq(*muts):
        return lambda root: [m(root) for m in muts]

    def refreeze(fix=None):
        def mut(root):
            _, _, _, frozen = measure_all(root)
            errors, wrote = update_freeze(root, ratchet(root), frozen, fix)
            assert wrote, errors
        return mut

    def hand_freeze(note=None):
        """What an agent does without the command: the data rewritten from the measurement, the note typed by hand."""
        def mut(root):
            _, _, _, frozen = measure_all(root)
            old, _ = read_freeze(root)
            ents = {k: {**e, "note": (old.get(k) or {}).get("note", "")} for k, e in frozen.items()}
            for k in ents:
                if k not in old and note:
                    ents[k]["note"] = note
            write(root, FREEZE, render_freeze(ents))
        return mut

    def rows_agree(root):
        measured, _, _, _ = measure_all(root)
        R = ratchet(root)
        write(root, BASELINE, R.render({p: (c, "") for p, c in R._clean(measured).items()}, []))

    def refused(label, mutate, fix, want):
        """update_freeze must refuse and leave the data file as it was."""
        nonlocal ok_count
        with tempfile.TemporaryDirectory(prefix="check-js-volume-st.") as root:
            sha = make_base(root, fz)
            with selftest_env(RATCHET_BASE_REF=sha):
                mutate(root)
                git(root, "add", "-A")
                before = open(os.path.join(root, FREEZE), encoding="utf-8").read()
                _, _, _, frozen = measure_all(root)
                errors, wrote = update_freeze(root, ratchet(root), frozen, fix)
                after = open(os.path.join(root, FREEZE), encoding="utf-8").read()
        if wrote or before != after or not any(want in e for e in errors):
            fails.append(f"{label}: update_freeze must refuse ({want!r}) and write nothing, got wrote={wrote} errors={errors}")
        else:
            ok_count += 1

    swap = edit("primitives/src/a.rs", "el.blur();\";\nconst B", "el.click();\";\nconst B")
    scenario("freeze control: an unchanged tree", "green", fz, lambda r: None, freeze=True)
    scenario("freeze control: unrelated edits and a reformatted call keep every identity", "green", fz,
             edit("primitives/src/a.rs", "fn go() { let _ = dioxus::document::eval(A); }",
                  "fn pad() -> u32 { 1 }\n\nfn go() {\n    let _ = dioxus::document::eval(\n        A,\n    );\n}"), freeze=True)
    scenario("(a) editing a frozen script passes the volume ratchet alone", "green", fz, swap)
    scenario("(a) editing a frozen script literal is red", "red", fz, swap, freeze=True, why="a legacy script (lit")
    scenario("(a) editing a frozen .js file is red", "red", fz, edit("primitives/src/old.js", "run(a)", "run(b)"), freeze=True,
             why="a legacy script (file")
    scenario("(a) editing a frozen inline <script> is red", "red", fz, edit("preview/page.html", "run(inline)", "go(inline)"),
             freeze=True, why="a legacy script (inline")
    scenario("(a) copying a frozen script into another file is red", "red", fz,
             lambda r: write(r, "primitives/src/copy.rs", fz_rs.split("fn go")[0].replace("const B", "const C")), freeze=True,
             why="primitives/src/copy.rs")
    scenario("(b) re-pointing a sink site to another script is red", "red", fz,
             edit("primitives/src/a.rs", "document::eval(A)", "document::eval(B)"), freeze=True, why="a sink site (eval")
    scenario("(b) re-pointing a sink site passes the volume ratchet alone", "green", fz,
             edit("primitives/src/a.rs", "document::eval(A)", "document::eval(B)"))
    scenario("(c) a second call of a frozen site is red", "red", fz,
             seq(append("primitives/src/a.rs", "fn two() { let _ = dioxus::document::eval(A); }\n"), rows_agree), freeze=True,
             why="a sink site (eval")
    scenario("(c) a new kind of sink site is red", "red", fz, seq(append("primitives/src/clean.rs", "use web_sys::window;\n"), rows_agree),
             freeze=True, why="a sink site (ffi")
    scenario("an `extern \"C\" fn` export alone leaves the freeze nothing", "green", fz,
             seq(append("primitives/src/clean.rs", ext_def), rows_agree), freeze=True)
    scenario("an `extern \"C\" { }` import block is a frozen sink site", "red", fz,
             seq(append("primitives/src/clean.rs", ext_blk), rows_agree), freeze=True, why="a sink site (ffi")
    scenario("(c) a new legacy file is red", "red", fz, seq(lambda r: write(r, "templates/starter/x.js", "var a = 1;\nrun(a);\n"), rows_agree),
             freeze=True, why="a legacy script (file")
    scenario("(d) the freeze data hand-edited to match an edited script, no note", "red", fz, seq(swap, hand_freeze()), freeze=True,
             why="no `# fixed: row N")
    scenario("(d) the same with a note citing a row that is not at the base", "red", fz, seq(swap, hand_freeze("# fixed: row 999 self")),
             freeze=True, why="row 999 does not exist at the base")
    scenario("(d) the same with a grant written in the lane's own diff", "red", fz,
             seq(swap, lambda r: write(r, "dev-docs/backlog/440-mine.md", fix_row.format(n=440, fix="primitives/src/a.rs", status="APPROVED")),
                 hand_freeze("# fixed: row 440 self")), freeze=True, why="row 440 does not exist at the base")
    scenario("(d) the same with a grant for another path", "red", fz, seq(swap, hand_freeze("# fixed: row 438 other")), freeze=True,
             why="row 438 has no `js-fix: primitives/src/a.rs`")
    scenario("(d) the same with a closed row", "red", fz, seq(swap, hand_freeze("# fixed: row 439 old")), freeze=True, why="is closed")
    scenario("(d) the same with a `js-grant` row, which grants no fix", "red", fz, seq(swap, hand_freeze("# fixed: row 434 grant")),
             freeze=True, why="has no `js-fix:")
    refused("(d) --update-freeze refuses an edited script without --js-fix", swap, None, "the freeze only lowers")
    refused("(d) --update-freeze refuses a fix whose row has no grant for the file", swap, "row 438 other", "has no `js-fix:")
    refused("(d) --update-freeze refuses a fix citing a row that is not at the base", swap, "row 999 other", "does not exist at the base")
    refused("(d) --update-freeze refuses a bare reason without a row", swap, "because", "names a backlog row")
    refused("(d) --update-freeze refuses to re-add a site it lowered", seq(
        lambda r: write(r, "primitives/src/a.rs", CLEAN_RS), rows_agree, refreeze(),
        lambda r: write(r, "primitives/src/a.rs", fz_rs)), None, "the freeze only lowers")
    scenario("(e) deleting a script and its site and lowering the data is green", "green", fz,
             seq(lambda r: write(r, "primitives/src/a.rs", CLEAN_RS), rows_agree, refreeze()), freeze=True)
    scenario("(e) deleting a legacy .js file and lowering the data is green", "green", fz,
             seq(lambda r: os.remove(os.path.join(r, "primitives/src/old.js")), rows_agree, refreeze()), freeze=True)
    scenario("(e) deleting a script without lowering the data is red", "red", fz,
             seq(lambda r: write(r, "primitives/src/a.rs", CLEAN_RS), rows_agree), freeze=True, why="run --update-baseline to lower the freeze")
    scenario("(f) an edit with a valid `js-fix:` grant is green and is listed", "green", fz, seq(swap, refreeze("row 437 bug fix")),
             freeze=True, show="FREEZE FIXES")
    scenario("(f) a fix may re-point a site of its file", "green", fz,
             seq(edit("primitives/src/a.rs", "document::eval(A)", "document::eval(B)"), refreeze("row 437 bug fix")), freeze=True)
    scenario("(f) a fix of a hand-written .js file", "green", fz, seq(edit("primitives/src/old.js", "run(a)", "run(b)"),
             refreeze("row 437 bug fix")), freeze=True)
    scenario("(f) a fix does not reach another file", "red", fz, seq(edit("preview/page.html", "run(inline)", "go(inline)"),
             hand_freeze("# fixed: row 437 reaches too far")), freeze=True, why="row 437 has no `js-fix: preview/page.html`")
    scenario("(f) a fix may not add a sink site", "red", fz,
             seq(swap, append("primitives/src/a.rs", "fn two() { let _ = dioxus::document::eval(B); }\n"), rows_agree,
                 hand_freeze("# fixed: row 437 bug fix")), freeze=True, why="but not add one")
    scenario("the freeze data deleted", "red", fz, lambda r: os.remove(os.path.join(r, FREEZE)), freeze=True, why="is missing")
    scenario("a malformed freeze row", "red", fz, append(FREEZE, "site\tx.rs\teval\tnothex\t1\t1\tp\n"), freeze=True, why="malformed row")
    # ---- a grant whose key is no column cannot exist silently (scripts/lib/rows.py `row_problems`)
    def grant_row(n, grant, adds="[]", fix=None, status="APPROVED"):
        text = GRANT_ROW.format(n=n, grant=grant, status=status).replace("adds: []", f"adds: {adds}")
        return text.replace("adds:", f"js-fix: {fix}\nadds:", 1) if fix else text

    def add_row(n, **kw):
        return lambda root: write(root, f"dev-docs/backlog/{n}-new.md", grant_row(n, **kw))

    cols = ", ".join(GRANT_COLUMNS)
    if not set(COLUMNS) <= set(GRANT_COLUMNS):
        fails.append(f"COLUMNS of the gate are not all in rows.GRANT_COLUMNS: {sorted(set(COLUMNS) - set(GRANT_COLUMNS))}")
    else:
        ok_count += 1
    scenario("a grant key that is no column is red, naming the row, the key and the columns", "red", base,
             add_row(470, grant="js_fil=10"), why=f"row 470 (dev-docs/backlog/470-new.md): `js-grant:` key `js_fil` is not a column; the columns are {cols}")
    scenario("the legacy grant key `js_file` is red and says bytes and `js_file_bytes`", "red", base,
             add_row(471, grant="js_file=150 (inferred ceiling: about 150 lines)"), why="the unit is bytes and the column is `js_file_bytes`")
    for column in COLUMNS:
        scenario(f"a grant of the column `{column}` is green", "green", base, add_row(472, grant=f"{column}=5"))
    scenario("a grant of the bridge budget column `bytes` and a js column together is green", "green", base,
             add_row(473, grant="bytes=56 js_file_bytes=56 (two columns)"))
    scenario("`js-grant: none` and a remark after `none` are green", "green", base, add_row(474, grant="none (no volume)"))
    scenario("a grant of words is red", "red", base, add_row(475, grant="about 150 lines"), why="is not `column=number`")
    scenario("a `js-fix` of column pairs is red", "red", base, add_row(476, grant="none", fix="js_file=20"), why="`js-fix:` is a path list")
    scenario("a `js-fix` path list is green", "green", base, add_row(477, grant="none", fix="primitives/src/a.rs, b.js (why)"))
    scenario("an `adds` that is no bracketed list is red", "red", base, add_row(478, grant="none", adds="otpFilter"), why="`adds:` is 'otpFilter'")
    scenario("a closed row keeps its old grant key unread", "green", base, add_row(479, grant="js_file=0", status="LANDED"))
    scenario("a row under done/ keeps its old grant key unread", "green", base,
             lambda root: write(root, "dev-docs/backlog/done/480-old.md", grant_row(480, grant="js_file=0")))

    with tempfile.TemporaryDirectory(prefix="check-js-volume-st.") as root:
        write(root, ALLOW, ALLOW_MIN)
        write(root, "primitives/src/bridge/x.rs", fz_rs)
        write(root, "primitives/src/js/gen.js", "var a = 1;\nrun(a);\n")
        write(root, "primitives/src/legacy.rs", fz_rs)
        git(root, "init", "-q")
        git(root, "add", "-A")
        legacy = {k[0] for k in measure_all(root)[3]}
    if legacy != {"primitives/src/legacy.rs"}:
        fails.append(f"the bridge and the generated bundle are not legacy, so not frozen; the frozen files are {sorted(legacy)}")
    else:
        ok_count += 1
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
    reason = fix = None
    if "--allow-raise" in args:
        k = args.index("--allow-raise")
        reason = args[k + 1]
        del args[k:k + 2]
    if "--js-fix" in args:
        k = args.index("--js-fix")
        fix = args[k + 1]
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
    if args == ["--update-freeze"]:
        return cmd_update_freeze(root, fix)
    print("usage: check-js-volume.sh [--report|--inventory|--self-test|--update-baseline [--allow-raise \"row N reason\"]|"
          "--update-freeze [--js-fix \"row N reason\"]]", file=sys.stderr)
    return 2


sys.exit(main(sys.argv))
PYEOF
