#!/usr/bin/env bash
#
# check-bridge-behaviours.sh
#
# Keeps the generic JS bridge (backlog row 434 step 3: `primitives/src/ts/bridge.ts`, `primitives/src/ts/behaviours/`,
# `primitives/src/bridge/`) closed by construction, by the checks listed below.
#
#  * PARITY BY GENERATION. The Rust `Behaviour` enum (`bridge/behaviours.rs`), the `KINDS` of the generated
#    `ts/behaviours/behaviours.gen.ts`, the modules `ts/behaviours/<kind>.ts` and their committed bundles
#    `js/behaviours/<kind>.js` are the same set of kinds, one to one. The generated file's byte-for-byte golden is
#    `cargo test -p dioxus-primitives behaviours` (it fails on drift and `DX_BLESS=1` rewrites it); this gate checks
#    the structure around it. `bun test` (run here when installed) asserts that the registry's keys equal the
#    generated kinds and runs the shared vectors, `primitives/conformance/behaviours.json`, through the real `decide`.
#  * A BEHAVIOUR IS PURE. A module under `ts/behaviours/` imports only `../lib/register` (and types), registers its
#    own kind, defines `decide(`, and touches no `window`, `document`, `addEventListener`, timer, clock or `performance`:
#    the listener, its phase and passivity, the lease and the epoch belong to the adapter in `bridge.ts`.
#  * SMALL FILES, AS A RATCHET. The byte size of `bridge.ts`, `lib/register.ts`, the ambient types and every behaviour
#    module is held in `scripts/check-bridge-behaviours.budgets.tsv` through the `Ratchet` class (scripts/lib/ratchet.py):
#    a rise, a new file without a row and a stale row fail, a fall must be written back with `--update-baseline`, and a
#    raise passes only against a `js-grant: bytes=<n>` row at the merge base (the owner's decision, never a lane's); two
#    granted rows that raise one file are a chain of steps in its note, one per row (scripts/lib/ratchet.py).
#  * ONE PLACE FOR CANCELLING AND FRAMES. `preventDefault`, `stopPropagation`, `requestAnimationFrame` and the three
#    observers appear in no `primitives/src/ts` file outside `ts/bridge.ts`, `ts/behaviours/` (generated files and
#    `.d.ts` excluded) and a shrink-only residue allowlist (`scripts/check-bridge-behaviours.allow.tsv`,
#    `path<TAB>reason<TAB>row`): the target end state is EMPTY, so each reason names the step 3 lane that removes it
#    (`3.DJ`, `3.B1`, `3.GS`, `3.CA`, `3.MS`, `3.WB`, ...), the row must exist and a stale entry fails.
#  * A CLOSED SURFACE. No computed member access on a DOM-ish receiver (`el[name]`, `event[field]`, `target[prop]`),
#    no `eval`, `new Function` or `JSON.parse` in `ts/`: names reach the DOM only through the closed tables
#    (`EVENTS`, `FIELD_GETTERS`, `METHODS`, `PROPS`), and there is no data language to interpret.
#  * ONE EVAL SINK, CLOSED. Nothing under `primitives/src/bridge/` but `transport.rs` names `document::eval`, and
#    `js_listener.rs` (which runs on the bridge's scope) does not either. The transport runs a closed `Program` enum
#    (`Serve`, `Pump`, `Legacy`) with its arguments as data, and its functions are `pub(in crate::bridge)`: no
#    `pub(crate)` or `pub` function but `is_unsupported`, no `request` or `pump` that takes script text. The one
#    caller-composed script body is `Program::Legacy`, constructed only in `bridge/legacy.rs`, and `legacy::` is used
#    only by `js_listener.rs`: both go when the last bespoke script is ported.
#    `Program::Frozen{module}` is the closed list of frozen modules (`FrozenModule`, which lane 3.FZ of row 434 fills with the
#    first); every `document::eval` argument in the transport is one of three fixed shapes (the bundle, a program's
#    script, a module behind the ready handshake); `legacy::` is named outside `bridge/` only by a row of
#    `scripts/check-bridge-behaviours.callers.tsv` (one row today: `js_listener.rs`).
#  * FORBIDDEN_VERBS. The `Method` variants (each writes an open, focus, scroll or caret slot) are named by no Rust
#    file outside `primitives/src/bridge/`, and `METHODS` is named by no TypeScript file but `ts/bridge.ts`. This item is
#    the list's one owner: a row that needs a verb adds a table row inside `bridge/`, never a list entry.
#  * A CLOSED GLOBAL. `window.__dxBridge` is a frozen object installed with a non-writable, non-configurable property,
#    in `bridge.ts` and in the committed bundle; `lib/register.ts` registers only a generated KIND. (The bridge's own
#    `register` stays until item 0.3 replaces it with the generated `decide` registration.)
#  * LEASES, RUST HALF. A behaviour that can cancel an event gets a lease by construction: every `Declared` built in
#    `bridge/behaviours.rs` takes `lease_ms` from `lease_for`, which gives a non-passive behaviour `CANCEL_LEASE_MS`.
#  * WITNESSES, CONSTRUCTOR. `WitnessViolation` is constructed only by the witness probes (the `probes` directory of the witness harness).
#  * NEW KINDS NEED A ROW. A kind in `behaviours.gen.ts` that the merge base lacks needs an open backlog row at the base
#    whose front matter says `adds: [<kind>]` (scripts/lib/rows.py); citing the umbrella row 434 is refused.
#  * PLATFORM RULES. `primitives/conformance/platform-rules.tsv` is read by scripts/lib/platform_rules.py and every defect
#    it reports is red.
#  * NO ASYNC GESTURE VERBS. `focus`, `setPointerCapture` and `releasePointerCapture` are not rows of `METHODS` and not
#    variants of `Method` (a call lands after the event's dispatch, too late for either), and there is no `commit` verb
#    (not in the bundle's exports, not an `Op`, not a `Transaction` method).
#  * THE BUNDLES. The committed `js/bridge.js` still assigns `window.__dxBridge` with every verb; no committed bundle
#    has an import or export (each is evaluated as a script body).
#
# THE CLASS (row 434, sync-js-root analysis sections 2.2 and 5): a decision that must happen inside an event cannot wait
# for the asynchronous bridge, so it is a NAMED behaviour Rust installs beforehand, never an interpreter or a predicate
# language, and nothing else may cancel an event, run a frame loop or own an observer.
#
# A normal run also executes `bun test primitives/ts-test` and `tsc -p primitives/tsconfig.json`. The compiler is the
# `typescript` package named in `typescript-module.txt` of scripts/lib when that file exists, else the sandbox image's
# /opt/node-tools; a missing one is red when RATCHET_REQUIRE_BASE=1 (a full gate run) and a printed "skipped" otherwise
# (a missing `bun` is skipped either way). The TypeScript compiler-API scans are `check-ts-scan`'s, not repeated here.
# `--self-test` applies about 100 one-line breaks to an in-memory copy of the tree and requires each to turn the gate red;
# `--list` prints the allowlist.
#
# Usage: scripts/check-bridge-behaviours.sh [--self-test|--list|--static|--update-baseline [--allow-raise "row N why"]]
#   (--static skips bun and tsc; --update-baseline rewrites the budgets TSV, shrink-only)
# Exit status: 0 when clean.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

mode=run
raise_reason=""
while [ $# -gt 0 ]; do
  case "$1" in
    --self-test) mode=self-test ;;
    --list) mode=list ;;
    --static) mode=static ;;
    --update-baseline) mode=update ;;
    --allow-raise) shift; raise_reason="${1:-}" ;;
    *) echo "check-bridge-behaviours: unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done

python3 -I - "$mode" "$raise_reason" <<'PYEOF'
import copy
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, "scripts/lib")
import platform_rules  # noqa: E402
from allow import Allowlist  # noqa: E402
from ratchet import Ratchet  # noqa: E402
from rows import Row, front_matter, git_text  # noqa: E402

MODE = sys.argv[1]
RAISE_REASON = sys.argv[2] or None
GATE = "check-bridge-behaviours"

BEHAVIOURS_RS = "primitives/src/bridge/behaviours.rs"
GEN_TS = "primitives/src/ts/behaviours/behaviours.gen.ts"
BRIDGE_TS = "primitives/src/ts/bridge.ts"
REGISTER_TS = "primitives/src/ts/lib/register.ts"
VECTORS = "primitives/conformance/behaviours.json"
BEHAVIOUR_TEST = "primitives/ts-test/behaviours.test.ts"
BUNDLE = "primitives/src/js/bridge.js"
ALLOW_TSV = "scripts/check-bridge-behaviours.allow.tsv"
BUDGETS_TSV = "scripts/check-bridge-behaviours.budgets.tsv"
CALLERS_TSV = "scripts/check-bridge-behaviours.callers.tsv"
PLATFORM_TSV = platform_rules.TSV
TYPES_D_TS = "primitives/src/ts/types/bridge.d.ts"
TS_MODULE_TXT = "scripts/lib/typescript-module.txt"
TS_MODULE_IMAGE = "/opt/node-tools/node_modules/typescript"
PROBES = "playwright/witness/probes/"
TS_ROOT = "primitives/src/ts"
JS_ROOT = "primitives/src/js"
RS_ROOT = "primitives/src/bridge"
EVAL_OK = {"primitives/src/bridge/transport.rs"}
VERBS = ["version", "channel", "scope", "serve", "pump", "call", "get", "set", "transaction",
         "behaviours", "register"]
TRANSPORT_RS = "primitives/src/bridge/transport.rs"
LEGACY_RS = "primitives/src/bridge/legacy.rs"
MOD_RS = "primitives/src/bridge/mod.rs"
JS_LISTENER_RS = "primitives/src/js_listener.rs"
PROGRAM_VARIANTS = ["Serve", "Pump", "Legacy", "Frozen"]
# the three document::eval call shapes of the transport, as normalised argument text: the bridge bundle, a program's
# script, and a behaviour module behind the ready handshake. Any other argument is a script text a caller composed.
EVAL_ARGS = ["BRIDGE_JS", "&program.script()", '&format!("{READY}{js}")']
# FORBIDDEN_VERBS (this item is its one owner; item 0.7's slot table partitions it): each `Method` variant writes an
# open, focus, scroll or caret slot, so none is named outside bridge/. (variant, METHODS row)
FORBIDDEN_VERBS = [("ShowPopover", "showPopover"), ("HidePopover", "hidePopover"), ("TogglePopover", "togglePopover"),
                   ("ShowModal", "showModal"), ("Close", "close"), ("Blur", "blur"),
                   ("ScrollIntoView", "scrollIntoView"), ("ScrollTo", "scrollTo"), ("ScrollBy", "scrollBy"),
                   ("SetSelectionRange", "setSelectionRange"), ("Select", "select")]
GESTURE_METHODS = ["focus", "setPointerCapture", "releasePointerCapture"]
FORBIDDEN = ["preventDefault", "stopPropagation", "requestAnimationFrame", "ResizeObserver", "MutationObserver",
             "IntersectionObserver"]
# a behaviour's decide is pure: none of these may appear in a module under ts/behaviours/
IMPURE = ["window", "document", "addEventListener", "removeEventListener", "setTimeout", "setInterval",
          "queueMicrotask", "Date", "performance", "localStorage", "fetch", "navigator", "location"]
# the files whose byte size is a ratchet (plus every behaviour module): budgets TSV, scripts/lib/ratchet.py
BUDGETED = [BRIDGE_TS, REGISTER_TS, TYPES_D_TS]
BUDGET_ADVICE = ("a file grows only by a grant: ask the owner for a `js-grant: bytes=<n>` row at the merge base, or "
                 "shrink the file and run --update-baseline")
LANE_RE = re.compile(r"\b3\.[A-Z][A-Z0-9]*\b")
# computed member access on a receiver that may be a DOM object: names reach the DOM through the closed tables only
COMPUTED = re.compile(r"\b_?(el|elem|element|target|event|ev|source|win|doc|document|window|W|host|node)\s*\[\s*[A-Za-z_\"'`]")
SINKS = re.compile(r"\beval\s*\(|\bnew\s+Function\b|\bJSON\s*\.\s*parse\b")


def scrub(text, strings=False):
    """Comments blanked (and string literals, when asked), newlines kept so line numbers survive."""
    out, i, n = [], 0, len(text)
    while i < n:
        two = text[i:i + 2]
        c = text[i]
        if two == "//":
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
        elif two == "/*":
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(re.sub(r"[^\n]", " ", text[i:j]))
            i = j
        elif strings and c in "\"'`":
            j = i + 1
            while j < n and text[j] != c:
                j += 2 if text[j] == "\\" else 1
            j = min(j + 1, n)
            out.append(c + re.sub(r"[^\n]", " ", text[i + 1:j - 1]) + c)
            i = j
        else:
            out.append(c)
            i += 1
    return "".join(out)


def scrub_rs(text):
    """Rust with `//` comment lines dropped and string literals emptied."""
    code = "\n".join(l for l in text.split("\n") if not l.lstrip().startswith("//"))
    return re.sub(r'"(?:\\.|[^"\\])*"', '""', code, flags=re.S)


def kebab(name):
    return re.sub(r"(?<!^)(?=[A-Z])", "-", name).lower()


def load_tree():
    tree = {"files": {}, "ts": {}, "rs": {}, "js": {}}
    for p in [BEHAVIOURS_RS, GEN_TS, VECTORS, BEHAVIOUR_TEST, BUNDLE, ALLOW_TSV, BUDGETS_TSV, CALLERS_TSV, PLATFORM_TSV]:
        pp = Path(p)
        tree["files"][p] = pp.read_text(encoding="utf-8") if pp.is_file() else None
    for p in sorted(Path(TS_ROOT).rglob("*.ts")):
        tree["ts"][str(p)] = p.read_text(encoding="utf-8")
    for p in sorted(Path(JS_ROOT).rglob("*.js")):
        tree["js"][str(p)] = p.read_text(encoding="utf-8")
    for p in sorted(Path(RS_ROOT).rglob("*.rs")):
        tree["rs"][str(p)] = p.read_text(encoding="utf-8")
    tree["rs"][JS_LISTENER_RS] = Path(JS_LISTENER_RS).read_text(encoding="utf-8")
    # every Rust file of the crate, for the checks of who may name the quarantined legacy path
    tree["rs_all"] = {str(p): p.read_text(encoding="utf-8") for p in sorted(Path("primitives/src").rglob("*.rs"))}
    rows = set()
    for p in list(Path("dev-docs/backlog").glob("[0-9]*-*.md")) + list(Path("dev-docs/backlog/done").glob("[0-9]*-*.md")):
        rows.add(p.name.split("-", 1)[0].lstrip("0") or "0")
    tree["rows"] = rows
    # every Rust file of the repository, for the checks of who may name the `Method` variants
    listing = git_text(".", "ls-files", "-co", "--exclude-standard", "--", "*.rs") or ""
    tree["rs_repo"] = {p: Path(p).read_text(encoding="utf-8") for p in sorted(set(listing.split("\n")) - {""})
                       if Path(p).is_file()}
    # every file that names `WitnessViolation` outside the docs and this script
    hits = git_text(".", "grep", "-lI", "--untracked", "-e", "WitnessViolation", "--", ".", ":!dev-docs", ":!scripts",
                    ":!*.md") or ""
    tree["witness"] = {p: Path(p).read_text(encoding="utf-8") for p in hits.split("\n")
                       if p.endswith((".ts", ".tsx", ".js", ".mjs", ".cjs", ".rs")) and Path(p).is_file()}
    tree["base"] = load_base()
    return tree


def load_base():
    """What the merge base says: its sha, its generated kinds file and every row with an `adds:` line."""
    sha = Ratchet(GATE, BUDGETS_TSV, ("bytes",)).base()[1]
    base = {"sha": sha, "gen": None, "rows": []}
    if not sha:
        return base
    base["gen"] = git_text(".", "show", f"{sha}:{GEN_TS}")
    found = git_text(".", "grep", "-lE", r"^adds:[[:space:]]*\[?[[:space:]]*[^][:space:]]", sha, "--", "dev-docs/backlog") or ""
    for line in found.split("\n"):
        path = line.split(":", 1)[1] if ":" in line else ""
        text = git_text(".", "show", f"{sha}:{path}") if path else None
        if text is None:
            continue
        n = re.match(r"(?:dev-docs/backlog/(?:done/)?)0*(\d+)-", path)
        base["rows"].append(Row(int(n.group(1)) if n else 0, path, "/done/" in path, front_matter(text)))
    return base


def parse_allow(text):
    entries = []
    for ln in (text or "").splitlines():
        if not ln.strip() or ln.lstrip().startswith("#"):
            continue
        parts = ln.split("\t")
        entries.append(tuple(parts) if len(parts) == 3 else (ln, "", ""))
    return entries


def rust_kinds(text, errors):
    """The kebab-case kinds of the variants of `pub enum Behaviour`, read from its body."""
    m = re.search(r"pub\s+enum\s+Behaviour\s*\{", text)
    if not m:
        errors.append(f"{BEHAVIOURS_RS}: no `pub enum Behaviour`")
        return []
    head = " ".join(text[:m.start()].rstrip().split("\n")[-8:])
    if not ('tag = "kind"' in head and 'rename_all = "kebab-case"' in head):
        errors.append(f'{BEHAVIOURS_RS}: `Behaviour` must carry #[serde(tag = "kind", rename_all = "kebab-case")]: '
                      f"the kind is derived from the variant name")
    depth, body, i = 1, [], m.end()
    start = i
    while i < len(text) and depth:
        ch = text[i]
        depth += ch == "{"
        depth -= ch == "}"
        i += 1
    body = scrub(text[start:i - 1])
    kinds, depth = [], 0
    for line in body.split("\n"):
        if depth == 0:
            if re.search(r"#\[serde\([^)]*\brename\s*=", line):
                errors.append(f"{BEHAVIOURS_RS}: a serde `rename` on a variant breaks the one-to-one kind names")
            mm = re.match(r"\s*([A-Z][A-Za-z0-9]*)\b", line)
            if mm:
                kinds.append(kebab(mm.group(1)))
        depth += line.count("{") - line.count("}")
    return kinds


def gen_kinds(text):
    m = re.search(r"export const KINDS = \[([^\]]*)\]", text or "")
    return re.findall(r'"([^"]+)"', m.group(1)) if m else []


def budget_measure(tree):
    """The byte size of every budgeted file: the named ones and each behaviour module."""
    out = {}
    for path, text in tree["ts"].items():
        p = Path(path)
        module = p.parent.as_posix() == f"{TS_ROOT}/behaviours" and not path.endswith((".gen.ts", ".d.ts"))
        if path in BUDGETED or module:
            out[path] = {"bytes": len(text.encode("utf-8"))}
    return out


def ratchet_for(tree, virtual):
    """The budgets ratchet over the real file, or over a scratch copy of the tree's text (the self-test)."""
    if not virtual:
        return Ratchet(GATE, BUDGETS_TSV, ("bytes",), advice=BUDGET_ADVICE), None
    tmp = tempfile.TemporaryDirectory()
    text = tree["files"].get(BUDGETS_TSV)
    if text is not None:
        target = Path(tmp.name) / BUDGETS_TSV
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")
    return Ratchet(GATE, BUDGETS_TSV, ("bytes",), root=tmp.name, advice=BUDGET_ADVICE), tmp


REPORT = []


def budget_errors(tree, virtual=False):
    keys = ("RATCHET_NO_BASE", "RATCHET_REQUIRE_BASE")
    saved = {k: os.environ.get(k) for k in keys}
    if virtual:  # a scratch copy has no git history to compare with
        os.environ.pop("RATCHET_REQUIRE_BASE", None)
        os.environ["RATCHET_NO_BASE"] = "1"
    tmp = None
    try:
        R, tmp = ratchet_for(tree, virtual)
        rep = R.check(budget_measure(tree))
    finally:
        for k, v in saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
    if tmp is not None:
        tmp.cleanup()
    errors, rep.errors = list(rep.errors), []
    if not virtual:
        REPORT.append(R.format(rep))
    return errors


def check(tree, run_tools, virtual=False):
    errors = []
    F = tree["files"]
    for p in [BEHAVIOURS_RS, GEN_TS, VECTORS, BEHAVIOUR_TEST, BUNDLE]:
        if F.get(p) is None:
            errors.append(f"{p} is missing")
    if errors:
        return errors

    # 1. kinds: Rust enum == generated KINDS == modules == bundles == register() calls
    rk = rust_kinds(F[BEHAVIOURS_RS], errors)
    gk = gen_kinds(F[GEN_TS])
    if not F[GEN_TS].startswith("// GENERATED by"):
        errors.append(f"{GEN_TS}: lost its GENERATED header (it is written only by `DX_BLESS=1 cargo test -p dioxus-primitives behaviours`)")
    mods = {Path(p).stem for p in tree["ts"] if Path(p).parent.as_posix() == f"{TS_ROOT}/behaviours"
            and not p.endswith(".gen.ts") and not p.endswith(".d.ts")}
    bundles = {Path(p).stem for p in tree["js"] if Path(p).parent.as_posix() == f"{JS_ROOT}/behaviours"}
    for label, other in (("the generated KINDS", gk), ("ts/behaviours/<kind>.ts modules", sorted(mods)),
                         ("js/behaviours/<kind>.js bundles", sorted(bundles))):
        for k in sorted(set(rk) - set(other)):
            errors.append(f"behaviour `{k}` is a variant of the Rust `Behaviour` but is not in {label}")
        for k in sorted(set(other) - set(rk)):
            errors.append(f"behaviour `{k}` is in {label} but not a variant of the Rust `Behaviour` ({BEHAVIOURS_RS})")
    if len(set(rk)) != len(rk):
        errors.append(f"{BEHAVIOURS_RS}: duplicate kinds {sorted({k for k in rk if rk.count(k) > 1})}")

    # 2. a vector per kind, and the bun test that runs them
    try:
        vectors = json.loads(F[VECTORS])
    except ValueError as e:
        vectors = {}
        errors.append(f"{VECTORS}: not valid JSON ({e})")
    for k in rk:
        if not isinstance(vectors, dict) or not vectors.get(k):
            errors.append(f"behaviour `{k}` has no conformance vector in {VECTORS}")
    if isinstance(vectors, dict):
        for k in sorted(set(vectors) - set(rk)):
            errors.append(f"{VECTORS}: vectors for the unknown behaviour `{k}`")
    if "conformance/behaviours.json" not in F[BEHAVIOUR_TEST] or "KINDS" not in F[BEHAVIOUR_TEST]:
        errors.append(f"{BEHAVIOUR_TEST}: must run the shared vectors and compare the registry with the generated KINDS")

    # 3. each module is pure, registers its own kind, imports only the register helper
    for path, text in sorted(tree["ts"].items()):
        p = Path(path)
        if p.parent.as_posix() != f"{TS_ROOT}/behaviours" or path.endswith((".gen.ts", ".d.ts")):
            continue
        code = scrub(text, strings=True)
        kind = p.stem
        if not re.search(r'\bregister\s*\(\s*"%s"' % re.escape(kind), scrub(text)):
            errors.append(f'{path}: must call `register("{kind}", ...)` (the module name is the kind)')
        if not re.search(r"\bdecide\s*\(", code):
            errors.append(f"{path}: no `decide(`: a behaviour is a pure decide plus the adapter in bridge.ts")
        for m in re.finditer(r"^\s*import\s[^;]*?from\s+([\"'][^\"']+[\"'])", scrub(text), re.M):
            if m.group(1).strip("\"'") != "../lib/register":
                errors.append(f"{path}: imports {m.group(1)}: a behaviour module imports only ../lib/register")
        for token in IMPURE:
            if re.search(r"\b%s\b" % token, code):
                errors.append(f"{path}: names `{token}`: `decide` is pure; the adapter in bridge.ts owns listeners, timers, "
                              f"the clock and the DOM")
    errors += budget_errors(tree, virtual)

    # 4. cancelling, frames and observers only in the bridge and the behaviours (allowlisted residue aside)
    entries = parse_allow(F.get(ALLOW_TSV))
    bad_shape = [e for e in entries if not (e[1].strip() and e[2].strip())]
    for e in bad_shape:
        errors.append(f"{ALLOW_TSV}: `{e[0]}` is not `path<TAB>reason<TAB>row`")
    allow = Allowlist(GATE, [(e[0], e[2], e[1]) for e in entries if e not in bad_shape],
                      fields=("path", "row"), source=ALLOW_TSV)
    for e in allow:
        if not LANE_RE.search(e.reason):
            errors.append(f"{ALLOW_TSV}: `{e.path}` has no step 3 lane in its reason (3.DJ, 3.B1, 3.GS, 3.CA, 3.MS, 3.WB, ...): "
                          f"every residue entry carries an expiry")
        if e.row.strip().lstrip("0") not in tree["rows"]:
            errors.append(f"{ALLOW_TSV}: `{e.path}` names row {e.row}, which is not under dev-docs/backlog/")
    tok = re.compile(r"\b(%s)\b" % "|".join(FORBIDDEN))
    for path, text in sorted(tree["ts"].items()):
        if path.endswith((".gen.ts", ".d.ts")) or path == BRIDGE_TS or path.startswith(f"{TS_ROOT}/behaviours/"):
            continue
        hits = sorted({m.group(1) for m in tok.finditer(scrub(text, strings=True))})
        if not hits:
            continue
        if allow.find(lambda e, path=path: e.path == path) is None:
            errors.append(f"{path}: `{', '.join(hits)}` outside ts/bridge.ts and ts/behaviours/: a decision inside an event, "
                          f"a frame loop or an observer belongs to the bridge or to a named behaviour (row 434); a residue "
                          f"file needs a reasoned line in {ALLOW_TSV}")
    errors += [f"allowlist: {p}" for p in allow.problems()]

    # 5. a closed surface: no computed member access on a DOM receiver, no eval, no data language
    for path, text in sorted(tree["ts"].items()):
        if path.endswith((".gen.ts", ".d.ts")):
            continue
        if path != BRIDGE_TS and not path.startswith(f"{TS_ROOT}/behaviours/") and not path.startswith(f"{TS_ROOT}/lib/"):
            continue
        code = scrub(text, strings=True)
        for m in COMPUTED.finditer(code):
            line = code[:m.start()].count("\n") + 1
            errors.append(f"{path}:{line}: `{m.group(0)}...` is a computed member access on a DOM receiver: a name from "
                          f"outside reaches the DOM only through the closed tables (EVENTS, FIELD_GETTERS, METHODS, PROPS)")
        m = SINKS.search(code)
        if m:
            errors.append(f"{path}: `{m.group(0)}`: the bridge has no eval and no data language to interpret")

    # 6. one transport seam: only transport.rs names document::eval
    ev = re.compile(r"\bdocument\s*::\s*(\{[^}]*\b)?eval\b")
    for path, text in sorted(tree["rs"].items()):
        if path in EVAL_OK:
            continue
        code = "\n".join(l for l in text.split("\n") if not l.lstrip().startswith("//"))
        # a string literal that quotes the name (a test, a doc example) is not a call: only code outside strings counts
        code = re.sub(r'"(?:\\.|[^"\\])*"', '""', code)
        if ev.search(code):
            errors.append(f"{path}: names `document::eval`; the bridge has ONE transport seam, {', '.join(sorted(EVAL_OK))}")

    # 6b. the transport is a closed program list, its functions are private to the bridge, and the one
    # caller-composed script (`Program::Legacy`) is quarantined
    transport = scrub_rs(tree["rs"].get(TRANSPORT_RS, ""))
    m = re.search(r"enum\s+Program\s*<[^>]*>\s*\{(.*?)\n\}", transport, re.S)
    variants = re.findall(r"^\s{4}([A-Z][A-Za-z]*)\b", m.group(1), re.M) if m else []
    if variants != PROGRAM_VARIANTS:
        errors.append(f"{TRANSPORT_RS}: `Program` must be exactly {PROGRAM_VARIANTS}, found {variants}: what the transport runs is a closed list")
    if not re.search(r"pub\(in crate::bridge\)\s+enum\s+Program", transport):
        errors.append(f"{TRANSPORT_RS}: `Program` must be `pub(in crate::bridge)`")
    for m in re.finditer(r"^pub(\(crate\))?\s+(async\s+)?fn\s+(\w+)", transport, re.M):
        if m.group(3) != "is_unsupported":
            errors.append(f"{TRANSPORT_RS}: `{m.group(3)}` is `pub{m.group(1) or ''}`: transport functions are `pub(in crate::bridge)`; only is_unsupported is public")
    for name in ("request", "pump"):
        if re.search(r"\bfn\s+%s\b" % name, transport):
            errors.append(f"{TRANSPORT_RS}: `fn {name}`: the transport takes ops through `fire`/`ask` and a `Program`, never script text")
    legacy_users = set()
    for path, text in sorted(tree["rs_all"].items()):
        code = scrub_rs(text)
        if path != TRANSPORT_RS and path != LEGACY_RS and re.search(r"\bProgram\s*::\s*Legacy\b", code):
            errors.append(f"{path}: constructs `Program::Legacy`: only {LEGACY_RS} may (the quarantine of js_listener's scripts)")
        if not path.startswith(RS_ROOT + "/") and re.search(r"\blegacy\s*::", code):
            legacy_users.add(path)
    # 6c. the callers table: a file outside bridge/ that runs a caller-composed script is a reasoned row, and a row
    # whose file does not name it is stale
    centries = parse_allow(tree["files"].get(CALLERS_TSV))
    cbad = [e for e in centries if not (e[1].strip() and e[2].strip())]
    errors += [f"{CALLERS_TSV}: `{e[0]}` is not `path<TAB>reason<TAB>row`" for e in cbad]
    callers = Allowlist(GATE, [(e[0], e[2], e[1]) for e in centries if e not in cbad], fields=("path", "row"),
                        source=CALLERS_TSV)
    for e in callers:
        if e.row.strip().lstrip("0") not in tree["rows"]:
            errors.append(f"{CALLERS_TSV}: `{e.path}` names row {e.row}, which is not under dev-docs/backlog/")
    for path in sorted(legacy_users):
        if callers.find(lambda e, path=path: e.path == path) is None:
            errors.append(f"{path}: uses `legacy::`, which runs a caller-composed script: a new caller is the owner's "
                          f"decision, a reasoned row in {CALLERS_TSV} (row 434 removes the last one)")
    errors += [f"callers: {p}" for p in callers.problems()]

    # 6d. Program::Frozen names a module of a closed list and carries no text; every eval argument is a known shape
    block = re.search(r"Frozen\s*\{([^}]*)\}", transport)
    if not block or [x.strip() for x in block.group(1).split(",") if x.strip()] != ["module: FrozenModule"]:
        errors.append(f"{TRANSPORT_RS}: `Program::Frozen` must be `Frozen {{ module: FrozenModule }}`: a frozen program is "
                      f"chosen by a module id of a closed enum, never by text")
    if not re.search(r"pub\(in crate::bridge\)\s+enum\s+FrozenModule\s*\{", transport):
        errors.append(f"{TRANSPORT_RS}: no `pub(in crate::bridge) enum FrozenModule`: the closed list of frozen modules")
    if not re.search(r"Program::Frozen\s*\{\s*module\s*\}\s*=>\s*match\s*\*module\s*\{", transport):
        errors.append(f"{TRANSPORT_RS}: `Program::script` must match the module id exhaustively (`match *module {{ .. }}`), "
                      f"so a new module cannot ship without its script")
    raw = "\n".join(l for l in tree["rs"].get(TRANSPORT_RS, "").split("\n") if not l.lstrip().startswith("//"))
    args = []
    for m in re.finditer(r"\bdocument\s*::\s*eval\s*\(", raw):
        depth, i = 1, m.end()
        while i < len(raw) and depth:
            depth += raw[i] == "("
            depth -= raw[i] == ")"
            i += 1
        args.append(" ".join(raw[m.end():i - 1].split()))
    for a in args:
        if a not in EVAL_ARGS:
            errors.append(f"{TRANSPORT_RS}: `document::eval({a})`: the argument must be one of {EVAL_ARGS} (a Program "
                          f"constant, never a caller's text)")
    if sorted(args) != sorted(EVAL_ARGS):
        errors.append(f"{TRANSPORT_RS}: {len(args)} `document::eval` site(s), expected exactly {len(EVAL_ARGS)} "
                      f"({', '.join(EVAL_ARGS)}): a site is added or removed with this list")
    body = re.search(r"fn\s+script\s*\(&self\)[^{]*\{(.*?)\n    \}\n", tree["rs"].get(TRANSPORT_RS, ""), re.S)
    for f in re.finditer(r'format!\(\s*"((?:\\.|[^"\\])*)"', body.group(1) if body else ""):
        extra = set(re.findall(r"\{(\w*)\}", f.group(1))) - {"READY", "body"}
        if extra:
            errors.append(f"{TRANSPORT_RS}: `Program::script` interpolates {sorted(extra)}: a program's script is built "
                          f"from constants only (`Legacy`'s body is the one quarantined exception)")

    # no async gesture verb, no commit
    mod_rs = scrub_rs(tree["rs"].get(MOD_RS, ""))
    method = re.search(r"pub enum Method\s*\{(.*?)\n\}", mod_rs, re.S)
    for bad in ("SetPointerCapture", "ReleasePointerCapture", "Focus", "Commit"):
        if re.search(r"\b%s\b" % bad, mod_rs):
            errors.append(f"{MOD_RS}: names `{bad}`: pointer capture, focus and commit are not bridge verbs")
    if not method:
        errors.append(f"{MOD_RS}: no `pub enum Method`")
    bridge_code = scrub(tree["ts"].get(BRIDGE_TS, ""))
    methods = re.search(r"const METHODS\b.*?\n\};", bridge_code, re.S)
    if not methods:
        errors.append(f"{BRIDGE_TS}: no METHODS table")
    else:
        for bad in GESTURE_METHODS:
            if re.search(r"\n\s{2}\"?%s\"?\s*:" % bad, methods.group(0)):
                errors.append(f"{BRIDGE_TS}: METHODS has a `{bad}` row: a call arrives after the event's dispatch; it is an effect of a named behaviour")
    if re.search(r"\bcommit\s*[(:]", bridge_code):
        errors.append(f"{BRIDGE_TS}: a `commit` verb: a frame wait does not order anything on desktop")

    # 6e. FORBIDDEN_VERBS: every Method variant writes a slot, so none is named outside bridge/ and METHODS is private
    # to bridge.ts. The list must stay true to the enum and the table (a stale entry is an error).
    variant_names = set(re.findall(r"^\s{4}([A-Z][A-Za-z]*)\b", method.group(1), re.M)) if method else set()
    rows_present = set(re.findall(r"\n\s{2}\"?(\w+)\"?\s*:", methods.group(0))) if methods else set()
    for variant, row in FORBIDDEN_VERBS:
        if variant not in variant_names:
            errors.append(f"{MOD_RS}: FORBIDDEN_VERBS lists `Method::{variant}`, which is not a variant: the list of "
                          f"scripts/{GATE}.sh is stale")
        if row not in rows_present:
            errors.append(f"{BRIDGE_TS}: FORBIDDEN_VERBS lists the METHODS row `{row}`, which does not exist: the list is stale")
    names = "|".join(v for v, _ in FORBIDDEN_VERBS)
    used = re.compile(r"\bMethod\s*::\s*(\*|\{[^}]*\b(?:%s|\*)\b[^}]*\}|(?:%s)\b)" % (names, names))
    for path, text in sorted(tree["rs_repo"].items()):
        if path.startswith(RS_ROOT + "/"):
            continue
        hit = used.search(scrub_rs(text))
        if hit:
            errors.append(f"{path}: names `{hit.group(0)}`: the bridge's `Method` variants write a slot (open, focus, scroll, "
                          f"caret) and exist only inside primitives/src/bridge/ as the slot owner's operation "
                          f"(FORBIDDEN_VERBS, dev-docs/js-interop.md)")
    for path, text in sorted(tree["ts"].items()):
        if path != BRIDGE_TS and re.search(r"\bMETHODS\b", scrub(text, strings=True)):
            errors.append(f"{path}: names `METHODS`: the verb table is private to ts/bridge.ts (FORBIDDEN_VERBS)")

    # 7. the committed bundles
    js = F[BUNDLE]
    if not re.search(r"defineProperty\(\s*\w+\s*,\s*\"__dxBridge\"\s*,\s*\{\s*value\s*:\s*Object\.freeze\(.*?\)\s*,\s*writable:\s*(!1|false)\s*,\s*configurable:\s*(!1|false)",
                     js, re.S):
        errors.append(f"{BUNDLE}: no frozen, non-writable, non-configurable `window.__dxBridge`: the exports were stripped "
                      f"or never made, or the global is replaceable")
    for verb in VERBS:
        if not re.search(r"\b%s\b" % verb, js):
            errors.append(f"{BUNDLE}: the verb `{verb}` is missing from the bundle")
    for path, text in sorted(tree["js"].items()):
        if path == BUNDLE or path.startswith(f"{JS_ROOT}/behaviours/"):
            if re.search(r"^\s*(export|import)\b|\bexport\s*\{", text, re.M):
                errors.append(f"{path}: the bundle has an import/export: it is evaluated as a script body, not a module")

    # 8. the closed global: frozen, installed once and never replaceable; the registration helper takes only generated kinds
    bcode = scrub(tree["ts"].get(BRIDGE_TS, ""))
    if not re.search(r"\bObject\.freeze\(\s*bridge\s*\)", bcode) or "__dxBridge" not in bcode:
        errors.append(f"{BRIDGE_TS}: the bridge object is not `Object.freeze(bridge)`d before it is installed")
    if not re.search(r"Object\.defineProperty\(\s*W\s*,\s*\"__dxBridge\"\s*,\s*\{[^}]*\bwritable:\s*false\b[^}]*\bconfigurable:\s*false\b",
                     bcode):
        errors.append(f"{BRIDGE_TS}: window.__dxBridge must be installed with Object.defineProperty(W, \"__dxBridge\", "
                      f"{{ value, writable: false, configurable: false }}): a page script must not replace the bridge")
    if re.search(r"\b(W|window|globalThis|self)\s*\.\s*__dxBridge\s*=[^=]", bcode):
        errors.append(f"{BRIDGE_TS}: assigns window.__dxBridge: the property is defined once, non-writable")
    reg = scrub(tree["ts"].get(REGISTER_TS, ""))
    if not (re.search(r"import\s*\{[^}]*\bKINDS\b[^}]*\}\s*from\s*\"\.\./behaviours/behaviours\.gen\"", reg)
            and re.search(r"if\s*\(\s*!\s*\(\s*KINDS\b[^)]*\)\s*\.\s*includes\(\s*kind\s*\)\s*\)\s*throw\b", reg)):
        errors.append(f"{REGISTER_TS}: must refuse a kind outside the generated KINDS before it calls the bridge")
    for path, text in sorted(tree["ts"].items()):
        if path.endswith(".d.ts") or path in (REGISTER_TS, BRIDGE_TS):
            continue
        if re.search(r"\.\s*register\s*\(", scrub(text, strings=True)):
            errors.append(f"{path}: calls `.register(`: a behaviour module registers through lib/register.ts only, so only a "
                          f"generated kind is accepted")

    # 9. the Rust half of the lease rule: a behaviour that can cancel an event gets a lease by construction
    brs = scrub_rs(F[BEHAVIOURS_RS])
    lease = re.search(r"fn\s+lease_for\s*\(\s*passive\s*:\s*bool[^)]*\)[^{]*\{(.*?)\n\}", brs, re.S)
    const = re.search(r"pub\s+const\s+CANCEL_LEASE_MS\s*:\s*u32\s*=\s*(\d+)\s*;", brs)
    if not lease or not re.search(r"\belse\s*\{\s*Some\(", lease.group(1)):
        errors.append(f"{BEHAVIOURS_RS}: `lease_for(passive, declared)` must give every non-passive behaviour a lease "
                      f"(`else {{ Some(..) }}`): a variant that prevents or stops always has one")
    if not const or int(const.group(1)) <= 0:
        errors.append(f"{BEHAVIOURS_RS}: `CANCEL_LEASE_MS` must be a positive constant")
    elif lease and "CANCEL_LEASE_MS" not in lease.group(1):
        errors.append(f"{BEHAVIOURS_RS}: `lease_for` must default a non-passive behaviour to `CANCEL_LEASE_MS`")
    decl_fn = re.search(r"pub\s+fn\s+declared\s*\(&self\)[^{]*\{(.*?)\n    \}\n", brs, re.S)
    if decl_fn:
        n_decl = len(re.findall(r"\bDeclared\s*\{", decl_fn.group(1)))
        n_lease = len(re.findall(r"\blease_ms\s*:\s*lease_for\s*\(", decl_fn.group(1)))
        if n_decl == 0 or n_decl != n_lease:
            errors.append(f"{BEHAVIOURS_RS}: {n_decl} `Declared` literal(s) in `declared()` but {n_lease} take `lease_ms` "
                          f"from `lease_for`: every variant's lease goes through it")
    else:
        errors.append(f"{BEHAVIOURS_RS}: no `fn declared(&self)`")

    # 10. WitnessViolation is constructed only by the probes
    wv = re.compile(r"\bnew\s+WitnessViolation\b|(?<!class )\bWitnessViolation\s*\(|\bWitnessViolation\s*\.\s*prototype\b")
    for path, text in sorted(tree["witness"].items()):
        if path.startswith(PROBES):
            continue
        if wv.search(scrub(text, strings=True) if not path.endswith(".rs") else scrub_rs(text)):
            errors.append(f"{path}: constructs `WitnessViolation`: only the probes under {PROBES} do, so that a witness "
                          f"passes only with the probe's own typed violation (a thrown timeout or locator error is not one)")

    # 11. a new behaviour kind needs a row at the merge base that names it
    errors += adds_errors(gk, tree["base"])

    # 12. the platform-rule rows
    table = platform_rules.parse(F[PLATFORM_TSV] or "", platform_rules.projects(), PLATFORM_TSV)
    errors += platform_rules.problems(table)

    if run_tools:
        if shutil.which("bun"):
            r = subprocess.run(["bun", "test", "primitives/ts-test"], capture_output=True, text=True)
            if r.returncode != 0:
                errors.append("bun test primitives/ts-test failed:\n" + (r.stdout + r.stderr)[-2500:])
        else:
            print(f"{GATE}: bun not installed: `bun test` skipped", file=sys.stderr)
        tsc = tsc_command()
        gap = tool_gap("typescript", tsc is not None, bool(os.environ.get("RATCHET_REQUIRE_BASE")))
        if gap:
            errors.append(gap)
        elif tsc is None:
            print(f"{GATE}: typescript not installed: the type check was skipped (red under RATCHET_REQUIRE_BASE=1)",
                  file=sys.stderr)
        else:
            r = subprocess.run(tsc + ["-p", "primitives/tsconfig.json"], capture_output=True, text=True)
            if r.returncode != 0:
                errors.append("tsc -p primitives/tsconfig.json failed:\n" + (r.stdout + r.stderr)[-2500:])
    return errors


def tsc_command():
    """`node <typescript>/bin/tsc`, with the compiler resolved in this one place: the package named by
    typescript-module.txt of scripts/lib (a path from the repository root) when that file exists, else the sandbox image's.
    Falls back to a `tsc` on PATH; None when there is none."""
    mods = []
    txt = Path(TS_MODULE_TXT)
    if txt.is_file():
        mods += [l.strip() for l in txt.read_text(encoding="utf-8").splitlines() if l.strip() and not l.startswith("#")][:1]
    mods.append(TS_MODULE_IMAGE)
    node = shutil.which("node")
    for m in mods:
        entry = Path(m) / "bin" / "tsc"
        if node and entry.is_file():
            return [node, str(entry)]
    found = shutil.which("tsc")
    return [found] if found else None


def tool_gap(name, found, require):
    """The error for a missing type checker: red on a full run (RATCHET_REQUIRE_BASE=1), skipped otherwise."""
    if found or not require:
        return None
    return (f"{GATE}: {name} is not installed and RATCHET_REQUIRE_BASE=1: a full run needs the compiler "
            f"({TS_MODULE_TXT}, else {TS_MODULE_IMAGE}); a skipped type check is not a pass")


def adds_errors(tree_kinds, base):
    """A kind the merge base's generated file lacks needs an open row at the base whose `adds:` names it."""
    if not base.get("sha"):
        if os.environ.get("RATCHET_REQUIRE_BASE"):
            return [f"{GATE}: RATCHET_REQUIRE_BASE is set and there is no merge base: the `adds:` rule cannot be checked"]
        return []
    out = []
    norm = lambda k: k.strip().lower().replace("_", "-")
    old = set(gen_kinds(base.get("gen") or ""))
    for kind in sorted(set(tree_kinds) - old):
        named = [r for r in base["rows"] if norm(kind) in {norm(a) for a in r.adds}]
        if not named:
            out.append(f"behaviour `{kind}` is new against the merge base but no backlog row at the base says "
                       f"`adds: [{kind}]` in its front matter (the umbrella row 434 does not count): ask the owner for the row")
        elif not any(r.open for r in named):
            out.append(f"behaviour `{kind}` is new against the merge base but its row ({named[0].path}) is closed")
    return out


def mutate(tree, fn):
    t = copy.deepcopy(tree)
    fn(t)
    return t


def replace_in(path, old, new, section="files"):
    def f(t):
        text = t[section][path]
        assert old in text, f"self-test anchor missing in {path}: {old!r}"
        t[section][path] = text.replace(old, new, 1)
    return f


def self_test(tree):
    # a deterministic merge base: the tree's own generated file and no rows, whatever clone this runs in
    tree["base"] = {"sha": "self-test", "gen": tree["files"][GEN_TS], "rows": []}
    base = check(tree, False, virtual=True)
    if base:
        print(f"{GATE}: the unmutated tree is not green, the self-test cannot start:", file=sys.stderr)
        for e in base:
            print("  " + e, file=sys.stderr)
        return 1
    FIX_TS = "primitives/src/ts/behaviours/fixture.ts"
    FIX_JS = "primitives/src/js/behaviours/fixture.js"
    RS_SESSION = "primitives/src/bridge/session.rs"
    muts = [
        ("a Rust variant with no module", replace_in(BEHAVIOURS_RS, "pub enum Behaviour {", "pub enum Behaviour {\n    Zzz { x: u8 },")),
        ("a generated kind with no variant", replace_in(GEN_TS, '["fixture"]', '["fixture", "ghost"]')),
        ("a module with no variant", lambda t: (t["ts"].__setitem__("primitives/src/ts/behaviours/ghost.ts",
            'import { register } from "../lib/register";\nregister("ghost", { fields: [], decide() { return {}; } });\n'))),
        ("a bundle with no variant", lambda t: t["js"].__setitem__("primitives/src/js/behaviours/ghost.js", "1;")),
        ("a variant with no bundle", lambda t: t["js"].pop(FIX_JS)),
        ("the generated header lost", replace_in(GEN_TS, "// GENERATED by", "// written by")),
        ("the kebab tag removed", replace_in(BEHAVIOURS_RS, 'rename_all = "kebab-case"', 'rename_all = "snake_case"')),
        ("a serde rename on a variant", replace_in(BEHAVIOURS_RS, "    Fixture {", '    #[serde(rename = "other")]\n    Fixture {')),
        ("a kind with no conformance vector", lambda t: t["files"].__setitem__(VECTORS, "{}")),
        ("vectors for an unknown kind", lambda t: t["files"].__setitem__(
            VECTORS, json.dumps({**json.loads(t["files"][VECTORS]), "ghost": [{}]}))),
        ("the bun test not reading the vectors", replace_in(BEHAVIOUR_TEST, "conformance/behaviours.json", "x.json")),
        ("a module registering another kind", replace_in(FIX_TS, 'register("fixture"', 'register("other"', "ts")),
        ("a module with no decide", replace_in(FIX_TS, "decide(", "other(", "ts")),
        ("a module importing the bridge", replace_in(FIX_TS, 'from "../lib/register"', 'from "../bridge"', "ts")),
        ("a module reading the document", replace_in(FIX_TS, "return {};", "return document.title ? {} : {};", "ts")),
        ("a module reading the clock", replace_in(FIX_TS, "return {};", "return Date.now() ? {} : {};", "ts")),
        ("a module with no budget row", replace_in(BUDGETS_TSV, "primitives/src/ts/behaviours/fixture.ts\t", "primitives/src/ts/behaviours/zzz.ts\t")),
        ("a module over its byte budget", lambda t: t["ts"].__setitem__(FIX_TS, t["ts"][FIX_TS] + "\n" * 200)),
        ("bridge.ts over its byte budget", lambda t: t["ts"].__setitem__(BRIDGE_TS, t["ts"][BRIDGE_TS] + "\n" * 2000)),
        ("bridge.ts shrunk and the budget not written back", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS].replace("export {};", "", 1))),
        ("a budget row for a file that is gone", lambda t: t["files"].__setitem__(
            BUDGETS_TSV, t["files"][BUDGETS_TSV] + "primitives/src/ts/behaviours/ghost.ts\t10\n")),
        ("the budgets file missing", lambda t: t["files"].__setitem__(BUDGETS_TSV, None)),
        ("an unknown allowlist row", replace_in(ALLOW_TSV, "\t434", "\t99999")),
        ("an allowlist reason with no lane", replace_in(ALLOW_TSV, "3.DJ (dead-JS deletion)", "later (dead-JS deletion)")),
        ("an allowlist entry for a clean file", lambda t: t["files"].__setitem__(
            ALLOW_TSV, t["files"][ALLOW_TSV] + "primitives/src/ts/lib/register.ts\tx 3.B1\t434\n")),
        ("an allowlist entry that is stale", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/focus-trap.ts", t["ts"]["primitives/src/ts/focus-trap.ts"].replace("preventDefault", "cancel"))),
        ("an allowlist line without a reason", lambda t: t["files"].__setitem__(
            ALLOW_TSV, t["files"][ALLOW_TSV] + "primitives/src/ts/x.ts\t\t434\n")),
        ("a computed member access in bridge.ts", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS] + "\nfunction f(el: any, name: string) { return el[name](); }\n")),
        ("a computed event read in a behaviour", replace_in(FIX_TS, "return {};", "return _event[behaviour.attr] ? {} : {};", "ts")),
        ("an eval in bridge.ts", lambda t: t["ts"].__setitem__(BRIDGE_TS, t["ts"][BRIDGE_TS] + "\neval('1');\n")),
        ("new Function in bridge.ts", lambda t: t["ts"].__setitem__(BRIDGE_TS, t["ts"][BRIDGE_TS] + "\nnew Function('1');\n")),
        ("JSON.parse in bridge.ts", lambda t: t["ts"].__setitem__(BRIDGE_TS, t["ts"][BRIDGE_TS] + "\nJSON.parse('1');\n")),
        ("the bundle without the window export", replace_in(BUNDLE, 'writable:!1', 'writable:!0')),
        ("the bundle with a configurable global", replace_in(BUNDLE, 'configurable:!1', 'configurable:!0')),
        ("the bundle with an unfrozen bridge", replace_in(BUNDLE, 'value:Object.freeze(', 'value:void(')),
        ("the bundle without a verb", lambda t: t["files"].__setitem__(BUNDLE, re.sub(r"\btransaction\b", "txn", t["files"][BUNDLE]))),
        ("the bundle without register", lambda t: t["files"].__setitem__(BUNDLE, re.sub(r"\bregister\b", "reg", t["files"][BUNDLE]))),
        ("the bundle as a module", lambda t: t["js"].__setitem__(BUNDLE, t["js"][BUNDLE] + "\nexport{W};")),
        ("a behaviour bundle as a module", lambda t: t["js"].__setitem__(FIX_JS, t["js"][FIX_JS] + "\nexport{x};")),
        ("document::eval outside the transport", lambda t: t["rs"].__setitem__(
            RS_SESSION, t["rs"][RS_SESSION] + "\nfn x() { let _ = document::eval(\"1\"); }\n")),
        ("document::eval in js_listener.rs", lambda t: t["rs"].__setitem__(
            "primitives/src/js_listener.rs", t["rs"]["primitives/src/js_listener.rs"] + "\nfn x() { document::eval(\"1\"); }\n")),
    ]
    TRANSPORT = "primitives/src/bridge/transport.rs"
    muts += [
        ("a fourth transport program", lambda t: t["rs"].__setitem__(
            TRANSPORT, t["rs"][TRANSPORT].replace("    Legacy(&'a str),", "    Legacy(&'a str),\n    Raw,", 1))),
        ("Program no longer private to the bridge", lambda t: t["rs"].__setitem__(
            TRANSPORT, t["rs"][TRANSPORT].replace("pub(in crate::bridge) enum Program", "pub(crate) enum Program", 1))),
        ("a pub(crate) transport function", lambda t: t["rs"].__setitem__(
            TRANSPORT, t["rs"][TRANSPORT].replace("pub(in crate::bridge) fn fire", "pub(crate) fn fire", 1))),
        ("a request(&str) in the transport", lambda t: t["rs"].__setitem__(
            TRANSPORT, t["rs"][TRANSPORT] + "\nfn request(script: &str) {}\n")),
        ("Program::Legacy built outside legacy.rs", lambda t: t["rs_all"].__setitem__(
            RS_SESSION, t["rs_all"].get(RS_SESSION, "") + "\nfn x() { let _ = Program::Legacy(\"1\"); }\n")),
        ("legacy:: used outside js_listener.rs", lambda t: t["rs_all"].__setitem__(
            "primitives/src/stray.rs", "fn x() { legacy::open(\"1\"); }\n")),
        ("a Focus method in Rust", lambda t: t["rs"].__setitem__(
            MOD_RS, t["rs"][MOD_RS].replace("    Blur,", "    Blur,\n    Focus,", 1))),
        ("a Commit op in Rust", lambda t: t["rs"].__setitem__(
            MOD_RS, t["rs"][MOD_RS].replace("    Blur,", "    Blur,\n    Commit,", 1))),
        ("a focus row in METHODS", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS].replace("  blur: {", "  focus: { recv: \"e\", run: (el) => el.focus() },\n  blur: {", 1))),
        ("a setPointerCapture row in METHODS", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS].replace("  blur: {", "  setPointerCapture: { recv: \"e\", run: (el, a) => el.setPointerCapture(a[0]) },\n  blur: {", 1))),
        ("a commit verb in bridge.ts", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS] + "\nconst commit = () => 1;\nconst x = { commit() {} };\n")),
    ]
    for token in FORBIDDEN:
        muts.append((f"`{token}` in a stray ts file", lambda t, token=token: t["ts"].__setitem__(
            "primitives/src/ts/stray.ts", f"export function f(e: Event) {{ e.{token}(); }}\n")))
    def rs_edit(path, old, new, section="rs"):
        def f(t):
            assert old in t[section][path], f"self-test anchor missing in {path}: {old!r}"
            t[section][path] = t[section][path].replace(old, new, 1)
        return f

    def base_edit(fn):
        def f(t):
            t["base"] = copy.deepcopy(t["base"])
            fn(t["base"])
        return f

    BEHAV_RS = BEHAVIOURS_RS
    fixture_row = Row(999, "dev-docs/backlog/999-x.md", False, {"adds": "[fixture]"})
    muts += [
        # Program::Frozen and the eval arguments
        ("Program::Frozen lost", rs_edit(TRANSPORT, "    Frozen {", "    Frosen {"), "closed list"),
        ("Program::Frozen carrying text", rs_edit(TRANSPORT, "module: FrozenModule,", "module: &'a str,"), "Program::Frozen"),
        ("FrozenModule lost", rs_edit(TRANSPORT, "enum FrozenModule {}", "enum FrozenModules {}"), "FrozenModule"),
        ("FrozenModule public", rs_edit(TRANSPORT, "pub(in crate::bridge) enum FrozenModule", "pub enum FrozenModule"), "FrozenModule"),
        ("Program::script not matching the module", rs_edit(TRANSPORT, "match *module {}", "String::new()"), "exhaustively"),
        ("a document::eval of a caller's text", lambda t: t["rs"].__setitem__(
            TRANSPORT, t["rs"][TRANSPORT] + "\nfn run(s: &str) { let _ = document::eval(s); }\n"), "must be one of"),
        ("a fourth document::eval site", lambda t: t["rs"].__setitem__(
            TRANSPORT, t["rs"][TRANSPORT] + "\nfn run() { let _ = document::eval(BRIDGE_JS); }\n"), "site(s), expected exactly"),
        ("a program script interpolating a caller's text", rs_edit(
            TRANSPORT, 'Program::Pump => format!("{READY}await __dxB.pump(__dxB.channel(dioxus));"),',
            'Program::Pump => format!("{READY}{extra}await __dxB.pump(__dxB.channel(dioxus));"),'), "constants only"),
        # the callers table
        ("legacy:: from a file with no callers row", lambda t: t["rs_all"].__setitem__(
            "primitives/src/stray.rs", "fn x() { legacy::open(\"1\"); }\n"), "callers.tsv"),
        ("the js_listener callers row removed", lambda t: t["files"].__setitem__(
            CALLERS_TSV, "\n".join(l for l in t["files"][CALLERS_TSV].split("\n") if "js_listener" not in l)), "legacy::"),
        ("a callers row for a file that is clean", lambda t: t["files"].__setitem__(
            CALLERS_TSV, t["files"][CALLERS_TSV] + "primitives/src/lib.rs\tx\t434\n"), "stale"),
        ("a callers row naming no row", replace_in(CALLERS_TSV, "\t434", "\t99999"), "not under dev-docs/backlog"),
        ("a callers row without a reason", lambda t: t["files"].__setitem__(
            CALLERS_TSV, t["files"][CALLERS_TSV] + "primitives/src/lib.rs\t\t434\n"), "path<TAB>reason<TAB>row"),
        # FORBIDDEN_VERBS
        ("Method::HidePopover named in a component", lambda t: t["rs_repo"].__setitem__(
            "preview/src/x.rs", "fn f() { let _ = bridge::Method::HidePopover; }\n"), "FORBIDDEN_VERBS"),
        ("Method::Close(..) named in a hook", lambda t: t["rs_repo"].__setitem__(
            "primitives/src/hook.rs", "fn f() { let _ = Method::Close(None); }\n"), "FORBIDDEN_VERBS"),
        ("a Method glob import in a hook", lambda t: t["rs_repo"].__setitem__(
            "primitives/src/hook.rs", "use crate::bridge::Method::*;\n"), "FORBIDDEN_VERBS"),
        ("a Method variant list import in a hook", lambda t: t["rs_repo"].__setitem__(
            "primitives/src/hook.rs", "use crate::bridge::Method::{Blur, Select};\n"), "FORBIDDEN_VERBS"),
        ("METHODS named outside bridge.ts", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/stray.ts", "import { METHODS } from './bridge';\n"), "METHODS"),
        ("a FORBIDDEN_VERBS entry that is not a variant", rs_edit(MOD_RS, "    HidePopover,", "    HidePopoverX,"), "stale"),
        ("a FORBIDDEN_VERBS row that is not in METHODS", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS].replace("  hidePopover: {", "  hidePopoverX: {", 1)), "stale"),
        # the closed global and the registration helper
        ("bridge.ts not freezing the bridge", replace_in(BRIDGE_TS, "Object.freeze(bridge)", "bridge", "ts"), "freeze"),
        ("bridge.ts with a writable global", replace_in(BRIDGE_TS, "writable: false", "writable: true", "ts"), "defineProperty"),
        ("bridge.ts with a configurable global", replace_in(BRIDGE_TS, "configurable: false", "configurable: true", "ts"), "defineProperty"),
        ("bridge.ts assigning the global", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS] + "\nW.__dxBridge = {};\n"), "assigns window.__dxBridge"),
        ("register.ts not checking the kinds", replace_in(REGISTER_TS, "KINDS as readonly string[]", "[] as readonly string[]", "ts"), "generated KINDS"),
        ("register.ts without the throw", replace_in(REGISTER_TS, 'throw new Error("dx behaviour " + kind + " is not a generated kind")', "void 0", "ts"), "generated KINDS"),
        ("a module calling bridge.register", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/stray.ts", "window.__dxBridge!.register('x', {} as any);\n"), ".register("),
        # the Rust half of the lease rule
        ("a non-passive behaviour with no lease", rs_edit(
            BEHAV_RS, "Some(declared.unwrap_or(CANCEL_LEASE_MS))", "declared", "files"), "lease_for"),
        ("a lease that is not the default constant", rs_edit(
            BEHAV_RS, "Some(declared.unwrap_or(CANCEL_LEASE_MS))", "Some(declared.unwrap_or(0))", "files"), "CANCEL_LEASE_MS"),
        ("a Declared that skips lease_for", rs_edit(
            BEHAV_RS, "lease_ms: lease_for(!*cancel, *lease_ms),", "lease_ms: *lease_ms,", "files"), "lease_for"),
        # WitnessViolation
        ("WitnessViolation constructed in a spec", lambda t: t["witness"].__setitem__(
            "playwright/some.spec.ts", "throw new WitnessViolation('x');\n"), "WitnessViolation"),
        ("WitnessViolation called without new", lambda t: t["witness"].__setitem__(
            "playwright/witness/lib.ts", "const v = WitnessViolation('x');\n"), "WitnessViolation"),
        ("WitnessViolation built by prototype", lambda t: t["witness"].__setitem__(
            "playwright/witness/lib.ts", "Object.create(WitnessViolation.prototype);\n"), "WitnessViolation"),
        # new kinds need a row
        ("a new kind with no row at the base", base_edit(lambda b: b.update(gen=b["gen"].replace('["fixture"]', '[]'))), "adds:"),
        ("a new kind whose row is closed", base_edit(lambda b: (b.update(gen=b["gen"].replace('["fixture"]', '[]')),
            b["rows"].append(Row(999, "dev-docs/backlog/done/999-x.md", True, {"adds": "[fixture]"})))), "closed"),
        ("a new kind whose row is the umbrella", base_edit(lambda b: (b.update(gen=b["gen"].replace('["fixture"]', '[]')),
            b["rows"].append(Row(434, "dev-docs/backlog/434-x.md", False, {"adds": "[]"})))), "adds:"),
        # the platform rules
        ("a platform rule with an unknown category", lambda t: t["files"].__setitem__(
            PLATFORM_TSV, t["files"][PLATFORM_TSV] + "prevent\tsite\tzzz\tcite\tnone\tline\t444\n"), "platform-rules.tsv"),
    ]
    # the compiler must be there on a full run
    if tool_gap("typescript", False, True) is None or tool_gap("typescript", True, True) is not None \
            or tool_gap("typescript", False, False) is not None:
        print(f"{GATE}: self-test: a missing compiler is not red under RATCHET_REQUIRE_BASE=1 (or is red without it)", file=sys.stderr)
        return 1
    failed = 0
    for name, fn, *rest in muts:
        needle = rest[0] if rest else None
        errs = check(mutate(tree, fn), False, virtual=True)
        if not errs:
            print(f"{GATE}: self-test: `{name}` did NOT turn the gate red", file=sys.stderr)
            failed += 1
        elif needle and not any(needle in e for e in errs):
            print(f"{GATE}: self-test: `{name}` turned the gate red, but not by the rule under test ({needle!r}): {errs[0][:160]}",
                  file=sys.stderr)
            failed += 1
    greens = [
        ("a token inside a comment and a string", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/stray.ts", "// calls preventDefault() and requestAnimationFrame\n/* ResizeObserver */\nexport const s = 'e.preventDefault()';\n")),
        ("document::eval in a comment and a string", lambda t: t["rs"].__setitem__(
            RS_SESSION, t["rs"][RS_SESSION] + '\n// document::eval\nconst X: &str = "document::eval(";\n')),
        ("a token inside a generated file", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/behaviours/other.gen.ts", "export function f(e: Event) { e.preventDefault(); }\n")),
        ("Method variants inside bridge/", lambda t: t["rs_repo"].__setitem__(
            "primitives/src/bridge/x.rs", "fn f() { let _ = Method::HidePopover; }\n")),
        ("WitnessViolation constructed by a probe", lambda t: t["witness"].__setitem__(
            "playwright/witness/probes/p.ts", "throw new WitnessViolation('x');\nclass WitnessViolation extends Error {}\n")),
        ("a new kind with an open row at the base", base_edit(lambda b: (b.update(gen=b["gen"].replace('["fixture"]', '[]')),
            b["rows"].append(fixture_row)))),
        ("a table lookup with own()", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS] + "\nconst x = (table: Record<string, number>, key: string) => table[key];\n")),
    ]
    for name, fn in greens:
        t = mutate(tree, fn)
        # a green that edits a budgeted file is blessed first: only the rule under test is on trial
        t["files"][BUDGETS_TSV] = Ratchet(GATE, BUDGETS_TSV, ("bytes",)).render(
            {p: (c, "") for p, c in budget_measure(t).items()})
        errs = check(t, False, virtual=True)
        if errs:
            print(f"{GATE}: self-test: `{name}` must stay green but reported: {errs[0]}", file=sys.stderr)
            failed += 1
    if failed:
        return 1
    print(f"{GATE}: self-test OK ({len(muts)} mutations red, {len(greens)} non-mutations green)")
    return 0


tree = load_tree()
if MODE == "update":
    R = Ratchet(GATE, BUDGETS_TSV, ("bytes",), advice=BUDGET_ADVICE)
    errs, wrote = R.update(budget_measure(tree), RAISE_REASON)
    for e in errs:
        print(e, file=sys.stderr)
    print(f"{GATE}: {'wrote ' + BUDGETS_TSV if wrote else 'baseline unchanged'}")
    sys.exit(1 if errs else 0)
if MODE == "list":
    ents = parse_allow(tree["files"].get(ALLOW_TSV))
    print(Allowlist(GATE, [(e[0], e[2], e[1]) for e in ents], fields=("path", "row")).listing())
    sys.exit(0)
if MODE == "self-test":
    sys.exit(self_test(tree))
errors = check(tree, MODE == "run")
for line in REPORT:
    if line:
        print(line)
if errors:
    for e in errors:
        print(e, file=sys.stderr)
    print(f"\n{GATE}: {len(errors)} problem(s). See the header of scripts/{GATE}.sh.", file=sys.stderr)
    sys.exit(1)
print(f"{GATE}: OK -- the Rust `Behaviour`, the generated KINDS, the modules and bundles agree, behaviours are pure, "
      "no stray preventDefault/rAF/observer in ts/, a closed surface, one transport seam, the bundles export the bridge"
      + ("" if MODE == "static" else ", bun test and tsc green") + ".")
PYEOF
