#!/usr/bin/env bash
#
# check-bridge-rules.sh
#
# Keeps the generic JS bridge (backlog row 434 step 3: `primitives/src/ts/bridge.ts`, `primitives/src/ts/rules/`,
# `primitives/src/bridge/`) closed by construction. The Rust `Rule` enum and the interpreter's `switch (rule.kind)` list
# the same kinds one to one with no default arm, each inside one of the marked regions (B1 event gates, B2 drag and
# swipe, B3 measurement publisher, B5 reveal and settle, FIXTURE); every kind has a conformance vector in
# `primitives/conformance/rules.json` and a `bun test` case; `preventDefault`, `stopPropagation`,
# `requestAnimationFrame` and the three observers appear in no `primitives/src/ts` file outside `ts/bridge.ts`,
# `ts/rules/` and a shrink-only residue allowlist; nothing under `primitives/src/bridge/` but `transport.rs` (and
# `js_listener.rs`, which runs through it) names `document::eval`; the committed `js/bridge.js` still exports
# `window.__dxBridge` with every verb.
#
# THE CLASS (row 434, sync-js-root analysis section 2.2): a decision that must happen inside an event or within one
# frame cannot wait for the asynchronous bridge, so it is a rule Rust publishes beforehand and ONE interpreter
# evaluates. That only stays small if the vocabulary is closed (a named variant with fixed parameters, no variables,
# loops or expressions) and if no second place grows its own `preventDefault`, rAF loop or observer. The residue
# allowlist is `scripts/check-bridge-rules.allow.tsv` (`path<TAB>reason<TAB>row`): its target end state is EMPTY, so
# every reason must name the step 3 lane (`3.B1`, `3.GS`, `3.CA`, `3.MS`, `3.WB`, ...) that removes it, the row must
# exist under `dev-docs/backlog/`, and an entry whose file no longer holds a token is stale and fails. A new rule kind
# needs a backlog row, a vector and a `bun test` case.
#
# Besides the static rules a normal run executes `bun test primitives/ts-test` and `tsc -p primitives/tsconfig.json`
# when `bun`/`tsc` are installed (it says when it skipped one). `--self-test` applies about 30 one-line breaks to an
# in-memory copy of the tree and requires each to turn the gate red; `--list` prints the allowlist.
#
# Usage: scripts/check-bridge-rules.sh [--self-test|--list|--static]   (--static skips bun and tsc)
# Exit status: 0 when clean.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

mode=run
for a in "$@"; do
  case "$a" in
    --self-test) mode=self-test ;;
    --list) mode=list ;;
    --static) mode=static ;;
    *) echo "check-bridge-rules: unknown argument $a" >&2; exit 2 ;;
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

RULES_RS = "primitives/src/bridge/rules.rs"
RULES_TS = "primitives/src/ts/rules/index.ts"
TYPES_TS = "primitives/src/ts/rules/types.ts"
VECTORS = "primitives/conformance/rules.json"
RULES_TEST = "primitives/ts-test/rules.test.ts"
BUNDLE = "primitives/src/js/bridge.js"
ALLOW_TSV = "scripts/check-bridge-rules.allow.tsv"
TS_ROOT = "primitives/src/ts"
RS_ROOT = "primitives/src/bridge"
EVAL_OK = {"primitives/src/bridge/transport.rs"}
REGIONS = ["B1", "B2", "B3", "B5", "FIXTURE"]
VERBS = ["version", "channel", "scope", "serve", "pump", "call", "get", "set", "commit", "transaction", "rules"]
FORBIDDEN = ["preventDefault", "stopPropagation", "requestAnimationFrame", "ResizeObserver", "MutationObserver",
             "IntersectionObserver"]
LANE_RE = re.compile(r"\b3\.[A-Z][A-Z0-9]*\b")


def scrub_comments(text):
    """`//` and `/* */` comments blanked, newlines kept (so line numbers survive)."""
    out, i, n = [], 0, len(text)
    while i < n:
        two = text[i:i + 2]
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
        else:
            out.append(text[i])
            i += 1
    return "".join(out)


def kebab(name):
    return re.sub(r"(?<!^)(?=[A-Z])", "-", name).lower()


def load_tree():
    tree = {"files": {}, "ts": {}, "rs": {}}
    for p in [RULES_RS, RULES_TS, TYPES_TS, VECTORS, RULES_TEST, BUNDLE, ALLOW_TSV]:
        pp = Path(p)
        tree["files"][p] = pp.read_text(encoding="utf-8") if pp.is_file() else None
    for p in sorted(Path(TS_ROOT).rglob("*.ts")):
        tree["ts"][str(p)] = p.read_text(encoding="utf-8")
    for p in sorted(Path(RS_ROOT).rglob("*.rs")):
        tree["rs"][str(p)] = p.read_text(encoding="utf-8")
    tree["rs"]["primitives/src/js_listener.rs"] = Path("primitives/src/js_listener.rs").read_text(encoding="utf-8")
    rows = set()
    for p in list(Path("dev-docs/backlog").glob("[0-9]*-*.md")) + list(Path("dev-docs/backlog/done").glob("[0-9]*-*.md")):
        rows.add(p.name.split("-", 1)[0].lstrip("0") or "0")
    tree["rows"] = rows
    return tree


def parse_allow(text):
    entries = []
    for ln in (text or "").splitlines():
        if not ln.strip() or ln.lstrip().startswith("#"):
            continue
        parts = ln.split("\t")
        entries.append(tuple(parts) if len(parts) == 3 else (ln, "", ""))
    return entries


def block(text, open_re, label, errors):
    """(start line index, lines) of the brace block opened by the first line matching `open_re`."""
    lines = text.split("\n")
    for k, ln in enumerate(lines):
        if re.search(open_re, ln):
            depth, body = 0, []
            for j in range(k, len(lines)):
                code = scrub_comments(lines[j])
                for ch in code:
                    if ch == "{":
                        depth += 1
                    elif ch == "}":
                        depth -= 1
                body.append((j + 1, lines[j], code, depth))
                if depth == 0 and j > k:
                    return body
                if depth == 0 and "{" in code and "}" in code:
                    return body
            errors.append(f"{label}: the block opened by `{ln.strip()}` is not closed")
            return body
    errors.append(f"{label}: no `{open_re}` found")
    return []


def regions_of(lines, label, errors):
    """The region of each line index and the ordered marker list; reports a missing or misordered marker."""
    marks, current, by_line = [], None, {}
    for ln_no, raw, _code, _depth in lines:
        m = re.search(r"//\s*region:(\w+)\s+(begin|end)", raw)
        if m:
            name, kind = m.group(1), m.group(2)
            marks.append((name, kind))
            if kind == "begin":
                if current is not None:
                    errors.append(f"{label}:{ln_no}: region {name} begins inside region {current}")
                current = name
            else:
                if current != name:
                    errors.append(f"{label}:{ln_no}: region {name} ends but {current} is open")
                current = None
        else:
            by_line[ln_no] = current
    want = [(n, k) for n in REGIONS for k in ("begin", "end")]
    if marks != want:
        errors.append(f"{label}: the region markers must be exactly {', '.join(f'{n} {k}' for n, k in want)} in that order "
                      f"(found {', '.join(f'{n} {k}' for n, k in marks) or 'none'})")
    return by_line


def rust_variants(text, errors):
    body = block(text, r"pub\s+enum\s+Rule\s*\{", RULES_RS, errors)
    if not body:
        return {}
    head = text[:text.index("pub enum Rule")]
    attr = head.rstrip().split("\n")[-8:]
    if not any('tag = "kind"' in a and 'rename_all = "kebab-case"' in a for a in attr):
        errors.append(f'{RULES_RS}: `Rule` must carry #[serde(tag = "kind", rename_all = "kebab-case")]: the kind is derived from the variant name')
    by_line = regions_of(body, RULES_RS, errors)
    out = {}
    for k, (ln_no, raw, code, depth) in enumerate(body[1:], start=1):
        # a variant name sits at depth 1 (the line starts inside the enum, not inside a variant's braces)
        if body[k - 1][3] != 1:
            continue
        if re.search(r"#\[serde\([^)]*\brename\s*=", code):
            errors.append(f"{RULES_RS}:{ln_no}: a serde `rename` inside `Rule` would break the one-to-one kind names")
        m = re.match(r"\s*([A-Z][A-Za-z0-9]*)\b", code)
        if m and not code.lstrip().startswith("#"):
            out[kebab(m.group(1))] = by_line.get(ln_no)
    return out


def ts_cases(text, errors):
    body = block(text, r"switch\s*\(\s*rule\.kind\s*\)\s*\{", RULES_TS, errors)
    if not body:
        return {}
    by_line = regions_of(body, RULES_TS, errors)
    out = {}
    for k, (ln_no, raw, code, depth) in enumerate(body[1:], start=1):
        before = body[k - 1][3]
        if re.search(r"\bdefault\s*:", code):
            errors.append(f"{RULES_TS}:{ln_no}: the rule switch has a default arm: an unknown kind must be a compile error (the `never` check after the switch), not a silent fallback")
        m = re.match(r'\s*case\s+"([a-z0-9-]+)"\s*:', raw) if before == 1 else None
        if m:
            out[m.group(1)] = by_line.get(ln_no)
    return out


def check(tree, run_tools):
    errors = []
    F = tree["files"]
    for p in [RULES_RS, RULES_TS, TYPES_TS, VECTORS, RULES_TEST, BUNDLE]:
        if F.get(p) is None:
            errors.append(f"{p} is missing")
    if errors:
        return errors

    # 1. Rust enum and interpreter switch, one to one, every kind inside a region
    rs = rust_variants(F[RULES_RS], errors)
    ts = ts_cases(F[RULES_TS], errors)
    for kind in sorted(set(rs) - set(ts)):
        errors.append(f"rule kind `{kind}` is a variant of the Rust `Rule` but has no case in {RULES_TS}")
    for kind in sorted(set(ts) - set(rs)):
        errors.append(f"rule kind `{kind}` is a case in {RULES_TS} but not a variant of the Rust `Rule` ({RULES_RS})")
    for kind, region in sorted(rs.items()):
        if region is None:
            errors.append(f"{RULES_RS}: variant `{kind}` is outside every region marker (B1/B2/B3/B5/FIXTURE)")
    for kind, region in sorted(ts.items()):
        if region is None:
            errors.append(f"{RULES_TS}: case `{kind}` is outside every region marker (B1/B2/B3/B5/FIXTURE)")
        elif kind in rs and rs[kind] is not None and rs[kind] != region:
            errors.append(f"rule kind `{kind}` is in region {rs[kind]} in Rust but {region} in the interpreter")
    types_code = scrub_comments(F[TYPES_TS])
    for kind in sorted(ts):
        if not re.search(r'kind\s*:\s*"%s"' % re.escape(kind), types_code):
            errors.append(f'{TYPES_TS}: no `kind: "{kind}"` member in the `Rule` type')

    # 2. a vector and a bun test per kind
    try:
        vectors = json.loads(F[VECTORS])["vectors"]
    except (ValueError, KeyError) as e:
        vectors = []
        errors.append(f"{VECTORS}: not valid vector JSON ({e})")
    vkinds = {v.get("rule", {}).get("kind") for v in vectors}
    for kind in sorted(rs):
        if kind not in vkinds:
            errors.append(f"rule kind `{kind}` has no conformance vector in {VECTORS}")
        if not re.search(r'rule kind "%s"' % re.escape(kind), F[RULES_TEST]):
            errors.append(f'rule kind `{kind}` has no `describe(\'rule kind "{kind}"\')` in {RULES_TEST}')
    for kind in sorted(k for k in vkinds if k and k not in rs):
        errors.append(f"{VECTORS}: a vector names the unknown rule kind `{kind}`")

    # 3. forbidden tokens outside the bridge, the interpreter and the allowlist
    entries = parse_allow(F.get(ALLOW_TSV))
    bad_shape = [e for e in entries if not (e[1].strip() and e[2].strip())]
    for e in bad_shape:
        errors.append(f"{ALLOW_TSV}: `{e[0]}` is not `path<TAB>reason<TAB>row`")
    # Allowlist wants (path, row, reason); the file is path, reason, row
    allow = Allowlist("check-bridge-rules", [(e[0], e[2], e[1]) for e in entries if e not in bad_shape],
                      fields=("path", "row"), source=ALLOW_TSV)
    for e in allow:
        if not LANE_RE.search(e.reason):
            errors.append(f"{ALLOW_TSV}: `{e.path}` has no step 3 lane in its reason (3.B1, 3.GS, 3.CA, 3.MS, 3.WB, ...): "
                          f"every residue entry carries an expiry")
        if e.row.strip().lstrip("0") not in tree["rows"]:
            errors.append(f"{ALLOW_TSV}: `{e.path}` names row {e.row}, which is not under dev-docs/backlog/")
    tok = re.compile(r"\b(%s)\b" % "|".join(FORBIDDEN))
    for path, text in sorted(tree["ts"].items()):
        if path == "primitives/src/ts/bridge.ts" or path.startswith("primitives/src/ts/rules/"):
            continue
        hits = sorted({m.group(1) for m in tok.finditer(scrub_comments(text))})
        if not hits:
            continue
        if allow.find(lambda e, path=path: e.path == path) is None:
            errors.append(f"{path}: `{', '.join(hits)}` outside ts/bridge.ts and ts/rules/: a decision inside an event, "
                          f"a frame loop or an observer belongs to the bridge or to a rule (row 434); a residue file "
                          f"needs a reasoned line in {ALLOW_TSV}")
    errors += [f"allowlist: {p}" for p in allow.problems()]

    # 4. the transport seam: only transport.rs names document::eval
    ev = re.compile(r"\bdocument\s*::\s*(\{[^}]*\b)?eval\b")
    for path, text in sorted(tree["rs"].items()):
        if path in EVAL_OK:
            continue
        code = "\n".join(l for l in text.split("\n") if not l.lstrip().startswith("//"))
        # a string literal that quotes the name (a test, a doc example) is not a call: only code outside strings counts
        code = re.sub(r'"(?:\\.|[^"\\])*"', '""', code)
        if ev.search(code):
            errors.append(f"{path}: names `document::eval`; the bridge has ONE transport seam, {', '.join(sorted(EVAL_OK))}")

    # 5. the committed bundle still exports the bridge
    js = F[BUNDLE]
    if not re.search(r"\w+\.__dxBridge\s*=", js):
        errors.append(f"{BUNDLE}: no `window.__dxBridge = ...` assignment: the exports were stripped or never made")
    for verb in VERBS:
        if not re.search(r"\b%s\b" % verb, js):
            errors.append(f"{BUNDLE}: the verb `{verb}` is missing from the bundle")
    if re.search(r"^\s*(export|import)\b|\bexport\s*\{", js, re.M):
        errors.append(f"{BUNDLE}: the bundle has an import/export: it is evaluated as a script body, not a module")

    if run_tools:
        import shutil
        import subprocess
        if shutil.which("bun"):
            r = subprocess.run(["bun", "test", "primitives/ts-test"], capture_output=True, text=True)
            if r.returncode != 0:
                errors.append("bun test primitives/ts-test failed:\n" + (r.stdout + r.stderr)[-2500:])
        else:
            print("check-bridge-rules: bun not installed: `bun test` skipped", file=sys.stderr)
        if shutil.which("tsc"):
            r = subprocess.run(["tsc", "-p", "primitives/tsconfig.json"], capture_output=True, text=True)
            if r.returncode != 0:
                errors.append("tsc -p primitives/tsconfig.json failed:\n" + (r.stdout + r.stderr)[-2500:])
        else:
            print("check-bridge-rules: tsc not installed: the type check was skipped", file=sys.stderr)
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
        print("check-bridge-rules: the unmutated tree is not green, the self-test cannot start:", file=sys.stderr)
        for e in base:
            print("  " + e, file=sys.stderr)
        return 1
    muts = [
        ("a Rust variant with no case", replace_in(RULES_RS, "    // region:B1 end", "    Zzz,\n    // region:B1 end")),
        ("a TS case with no variant", replace_in(RULES_TS, "    // region:B1 end", '    case "zzz":\n      return () => {};\n    // region:B1 end')),
        ("a default arm", replace_in(RULES_TS, '    case "fixture":', '    default:\n      break;\n    case "fixture":')),
        ("a Rust region marker removed", replace_in(RULES_RS, "    // region:B3 begin (measurement publisher)\n", "")),
        ("a TS region marker removed", replace_in(RULES_TS, "    // region:B5 end\n", "")),
        ("a variant outside every region", replace_in(RULES_RS, "    // region:FIXTURE end", "    // region:FIXTURE end\n    Outside,")),
        ("a serde rename", replace_in(RULES_RS, "    Fixture {", '    #[serde(rename = "other")]\n    Fixture {')),
        ("the kebab tag removed", replace_in(RULES_RS, 'rename_all = "kebab-case")]\npub enum Rule', ')]\npub enum Rule')),
        ("a kind with no conformance vector", lambda t: t["files"].__setitem__(VECTORS, json.dumps({"vectors": []}))),
        ("a kind with no bun test", replace_in(RULES_TEST, 'rule kind "fixture"', "rule kind 'x'")),
        ("a kind without a type member", replace_in(TYPES_TS, 'kind: "fixture";', 'kind: "other";')),
        ("an unknown allowlist row", replace_in(ALLOW_TSV, "\t434", "\t99999")),
        ("an allowlist reason with no lane", replace_in(ALLOW_TSV, "3.B1 replaces", "later replaces")),
        ("an allowlist entry for a clean file", lambda t: t["files"].__setitem__(
            ALLOW_TSV, t["files"][ALLOW_TSV] + "primitives/src/ts/bridge.ts\tx 3.B1\t434\n")),
        ("an allowlist entry that is stale", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/focus-trap.ts", t["ts"]["primitives/src/ts/focus-trap.ts"].replace("preventDefault", "cancel"))),
        ("an allowlist line without a reason", lambda t: t["files"].__setitem__(
            ALLOW_TSV, t["files"][ALLOW_TSV] + "primitives/src/ts/x.ts\t\t434\n")),
        ("the bundle without the window export", replace_in(BUNDLE, "__dxBridge=", "__dxBridgeX=")),
        ("the bundle without a verb", lambda t: t["files"].__setitem__(BUNDLE, re.sub(r"\btransaction\b", "txn", t["files"][BUNDLE]))),
        ("the bundle as a module", lambda t: t["files"].__setitem__(BUNDLE, t["files"][BUNDLE] + "\nexport{W};")),
        ("document::eval outside the transport", lambda t: t["rs"].__setitem__(
            "primitives/src/bridge/session.rs", t["rs"]["primitives/src/bridge/session.rs"] + "\nfn x() { let _ = document::eval(\"1\"); }\n")),
        ("document::eval in js_listener.rs", lambda t: t["rs"].__setitem__(
            "primitives/src/js_listener.rs", t["rs"]["primitives/src/js_listener.rs"] + "\nfn x() { document::eval(\"1\"); }\n")),
    ]
    for token in FORBIDDEN:
        muts.append((f"`{token}` in a stray ts file", lambda t, token=token: t["ts"].__setitem__(
            "primitives/src/ts/stray.ts", f"export function f(e: Event) {{ e.{token}(); }}\n")))
    failed = 0
    for name, fn in muts:
        errs = check(mutate(tree, fn), False)
        if not errs:
            print(f"check-bridge-rules: self-test: `{name}` did NOT turn the gate red", file=sys.stderr)
            failed += 1
    greens = [
        ("a token inside a comment", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/stray.ts", "// calls preventDefault() and requestAnimationFrame\n/* ResizeObserver */\nexport {};\n")),
        ("document::eval in a comment and a string", lambda t: t["rs"].__setitem__(
            "primitives/src/bridge/session.rs", t["rs"]["primitives/src/bridge/session.rs"] + '\n// document::eval\nconst X: &str = "document::eval(";\n')),
        ("a token inside ts/rules/", lambda t: t["ts"].__setitem__(
            "primitives/src/ts/rules/other.ts", "export function f(e: Event) { e.preventDefault(); }\n")),
    ]
    for name, fn in greens:
        errs = check(mutate(tree, fn), False)
        if errs:
            print(f"check-bridge-rules: self-test: `{name}` must stay green but reported: {errs[0]}", file=sys.stderr)
            failed += 1
    if failed:
        return 1
    print(f"check-bridge-rules: self-test OK ({len(muts)} mutations red, {len(greens)} non-mutations green)")
    return 0


tree = load_tree()
if MODE == "list":
    ents = parse_allow(tree["files"].get(ALLOW_TSV))
    print(Allowlist("check-bridge-rules", [(e[0], e[2], e[1]) for e in ents], fields=("path", "row")).listing())
    sys.exit(0)
if MODE == "self-test":
    sys.exit(self_test(tree))
errors = check(tree, MODE == "run")
if errors:
    for e in errors:
        print(e, file=sys.stderr)
    print(f"\ncheck-bridge-rules: {len(errors)} problem(s). See the header of scripts/check-bridge-rules.sh.", file=sys.stderr)
    sys.exit(1)
print("check-bridge-rules: OK -- the Rust `Rule` and the interpreter switch agree, no default arm, no stray "
      "preventDefault/rAF/observer in ts/, one transport seam, the bundle exports the bridge"
      + ("" if MODE == "static" else ", bun test and tsc green") + ".")
PYEOF
