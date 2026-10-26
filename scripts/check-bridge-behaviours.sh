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
#  * SMALL FILES. Line budgets at the actual size plus a small margin for `bridge.ts`, `lib/register.ts` and the ambient
#    types, and one entry per behaviour module in `BEHAVIOUR_BUDGETS` (a module with no entry fails: the budget is
#    chosen when the behaviour is added, and raising one is a diff in this script that says why).
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
# A normal run also executes `bun test primitives/ts-test` and `tsc -p primitives/tsconfig.json` when `bun` / `tsc`
# are installed; it prints "skipped" for a missing one. `--self-test` applies about 40 one-line breaks to an in-memory
# copy of the tree and requires each to turn the gate red; `--list` prints the allowlist.
#
# Usage: scripts/check-bridge-behaviours.sh [--self-test|--list|--static]   (--static skips bun and tsc)
# Exit status: 0 when clean.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

mode=run
for a in "$@"; do
  case "$a" in
    --self-test) mode=self-test ;;
    --list) mode=list ;;
    --static) mode=static ;;
    *) echo "check-bridge-behaviours: unknown argument $a" >&2; exit 2 ;;
  esac
done

python3 -I - "$mode" <<'PYEOF'
import copy
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402

MODE = sys.argv[1]
GATE = "check-bridge-behaviours"

BEHAVIOURS_RS = "primitives/src/bridge/behaviours.rs"
GEN_TS = "primitives/src/ts/behaviours/behaviours.gen.ts"
BRIDGE_TS = "primitives/src/ts/bridge.ts"
REGISTER_TS = "primitives/src/ts/lib/register.ts"
VECTORS = "primitives/conformance/behaviours.json"
BEHAVIOUR_TEST = "primitives/ts-test/behaviours.test.ts"
BUNDLE = "primitives/src/js/bridge.js"
ALLOW_TSV = "scripts/check-bridge-behaviours.allow.tsv"
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
PROGRAM_VARIANTS = ["Serve", "Pump", "Legacy"]
GESTURE_METHODS = ["focus", "setPointerCapture", "releasePointerCapture"]
FORBIDDEN = ["preventDefault", "stopPropagation", "requestAnimationFrame", "ResizeObserver", "MutationObserver",
             "IntersectionObserver"]
# a behaviour's decide is pure: none of these may appear in a module under ts/behaviours/
IMPURE = ["window", "document", "addEventListener", "removeEventListener", "setTimeout", "setInterval",
          "queueMicrotask", "Date", "performance", "localStorage", "fetch", "navigator", "location"]
# line budgets: actual size plus a small margin, so growth is a decision (a diff here that says why)
BUDGETS = [("primitives/src/ts/bridge.ts", 1080), ("primitives/src/ts/lib/register.ts", 16),
           ("primitives/src/ts/types/bridge.d.ts", 150)]
# one entry per behaviour module, chosen when the behaviour is added: the fixture is the test fixture of step 3.0
BEHAVIOUR_BUDGETS = {"fixture": 26}
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
    for p in [BEHAVIOURS_RS, GEN_TS, VECTORS, BEHAVIOUR_TEST, BUNDLE, ALLOW_TSV]:
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
    tree["budgets"] = dict(BEHAVIOUR_BUDGETS)
    return tree


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


def check(tree, run_tools):
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
        n = text.count("\n") + 1
        budgets = tree["budgets"]
        if kind not in budgets:
            errors.append(f"{path}: no entry in BEHAVIOUR_BUDGETS of scripts/{GATE}.sh: choose its line budget (actual plus a small margin)")
        elif n > budgets[kind]:
            errors.append(f"{path}: {n} lines, over its {budgets[kind]}-line budget (raising it is a decision: say why in this script)")
    for path, budget in BUDGETS:
        if path in tree["ts"] and tree["ts"][path].count("\n") + 1 > budget:
            errors.append(f"{path}: {tree['ts'][path].count(chr(10)) + 1} lines, over its {budget}-line budget "
                          f"(raising it is a decision: say why in this script)")

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
    for path, text in sorted(tree["rs_all"].items()):
        code = scrub_rs(text)
        if path != TRANSPORT_RS and path != LEGACY_RS and re.search(r"\bProgram\s*::\s*Legacy\b", code):
            errors.append(f"{path}: constructs `Program::Legacy`: only {LEGACY_RS} may (the quarantine of js_listener's scripts)")
        if path not in (JS_LISTENER_RS, MOD_RS, LEGACY_RS) and re.search(r"\blegacy\s*::", code):
            errors.append(f"{path}: uses `legacy::`: only js_listener.rs runs a caller-composed script (row 434 removes it)")
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

    # 7. the committed bundles
    js = F[BUNDLE]
    if not re.search(r"\w+\.__dxBridge\s*=", js):
        errors.append(f"{BUNDLE}: no `window.__dxBridge = ...` assignment: the exports were stripped or never made")
    for verb in VERBS:
        if not re.search(r"\b%s\b" % verb, js):
            errors.append(f"{BUNDLE}: the verb `{verb}` is missing from the bundle")
    for path, text in sorted(tree["js"].items()):
        if path == BUNDLE or path.startswith(f"{JS_ROOT}/behaviours/"):
            if re.search(r"^\s*(export|import)\b|\bexport\s*\{", text, re.M):
                errors.append(f"{path}: the bundle has an import/export: it is evaluated as a script body, not a module")

    if run_tools:
        import shutil
        import subprocess
        if shutil.which("bun"):
            r = subprocess.run(["bun", "test", "primitives/ts-test"], capture_output=True, text=True)
            if r.returncode != 0:
                errors.append("bun test primitives/ts-test failed:\n" + (r.stdout + r.stderr)[-2500:])
        else:
            print(f"{GATE}: bun not installed: `bun test` skipped", file=sys.stderr)
        if shutil.which("tsc"):
            r = subprocess.run(["tsc", "-p", "primitives/tsconfig.json"], capture_output=True, text=True)
            if r.returncode != 0:
                errors.append("tsc -p primitives/tsconfig.json failed:\n" + (r.stdout + r.stderr)[-2500:])
        else:
            print(f"{GATE}: tsc not installed: the type check was skipped", file=sys.stderr)
    return errors


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
    base = check(tree, False)
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
        ("a module with no budget entry", lambda t: t["budgets"].pop("fixture")),
        ("a module over its line budget", lambda t: t["ts"].__setitem__(FIX_TS, t["ts"][FIX_TS] + "\n" * 200)),
        ("bridge.ts over its line budget", lambda t: t["ts"].__setitem__(BRIDGE_TS, t["ts"][BRIDGE_TS] + "\n" * 2000)),
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
        ("the bundle without the window export", replace_in(BUNDLE, "__dxBridge=", "__dxBridgeX=")),
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
            RS_SESSION, t["rs_all"].get(RS_SESSION, "") + "\nfn x() { legacy::open(\"1\"); }\n")),
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
    failed = 0
    for name, fn in muts:
        errs = check(mutate(tree, fn), False)
        if not errs:
            print(f"{GATE}: self-test: `{name}` did NOT turn the gate red", file=sys.stderr)
            failed += 1
    greens = [
        ("a token inside a comment and a string", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/stray.ts", "// calls preventDefault() and requestAnimationFrame\n/* ResizeObserver */\nexport const s = 'e.preventDefault()';\n")),
        ("document::eval in a comment and a string", lambda t: t["rs"].__setitem__(
            RS_SESSION, t["rs"][RS_SESSION] + '\n// document::eval\nconst X: &str = "document::eval(";\n')),
        ("a token inside a generated file", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/behaviours/other.gen.ts", "export function f(e: Event) { e.preventDefault(); }\n")),
        ("a table lookup with own()", lambda t: t["ts"].__setitem__(
            BRIDGE_TS, t["ts"][BRIDGE_TS] + "\nconst x = (table: Record<string, number>, key: string) => table[key];\n")),
    ]
    for name, fn in greens:
        errs = check(mutate(tree, fn), False)
        if errs:
            print(f"{GATE}: self-test: `{name}` must stay green but reported: {errs[0]}", file=sys.stderr)
            failed += 1
    if failed:
        return 1
    print(f"{GATE}: self-test OK ({len(muts)} mutations red, {len(greens)} non-mutations green)")
    return 0


tree = load_tree()
if MODE == "list":
    ents = parse_allow(tree["files"].get(ALLOW_TSV))
    print(Allowlist(GATE, [(e[0], e[2], e[1]) for e in ents], fields=("path", "row")).listing())
    sys.exit(0)
if MODE == "self-test":
    sys.exit(self_test(tree))
errors = check(tree, MODE == "run")
if errors:
    for e in errors:
        print(e, file=sys.stderr)
    print(f"\n{GATE}: {len(errors)} problem(s). See the header of scripts/{GATE}.sh.", file=sys.stderr)
    sys.exit(1)
print(f"{GATE}: OK -- the Rust `Behaviour`, the generated KINDS, the modules and bundles agree, behaviours are pure, "
      "no stray preventDefault/rAF/observer in ts/, a closed surface, one transport seam, the bundles export the bridge"
      + ("" if MODE == "static" else ", bun test and tsc green") + ".")
PYEOF
