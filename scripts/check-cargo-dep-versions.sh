#!/usr/bin/env bash
#
# check-cargo-dep-versions.sh
#
# What a user's `dx components add` puts in their Cargo.toml comes only from `component.json`, so over every
# `cargoDependencies` entry: the version is `x.y.z`, `^`, `~` or `=` with a floor equal to the `Cargo.lock` version,
# the crate is a `preview` dependency, `dioxus-icons` and `pulldown-cmark` are pinned `=x.y.z`, only an allowlisted
# component (`markdown`) may list a markdown, HTML or syntax parser, the `dioxus-primitives` `rev`s are all unstamped
# or all one 40-hex value (`scripts/stamp-release-rev.sh <sha>` writes it at release time), and every primitives
# module behind a cargo feature that a component's installed Rust files name has that feature in its closure's
# `features`.
#
# What a user's `dx components add <name>` puts in their Cargo.toml is `cargo add <crate>[@<version>]
# [--features ..]` for every `cargoDependencies` entry of the component and of its `componentDependencies`
# closure (dioxus-cli 0.7.9 `add_rust_dependencies`; `shadcn-dioxus add` the same). Our `Cargo.lock` never reaches
# a user, so the version text in `component.json` is the ONLY thing tying what we tested to what ships. Without this
# gate a declared `time` `0.3.44` can sit beside a lock (and so every test) at 0.3.47. dev-docs/backlog.md row 217,
# dev-docs/research/dependency-audit-2026-10-14.md section A4. Rules over
# preview/src/components/*/component.json:
#   1. VERSIONS MATCH THE LOCK. Every entry with a `version` is `x.y.z`, `^x.y.z`, `~x.y.z` or `=x.y.z` (a floor
#      is always spelled out), and x.y.z is a version of that crate in the workspace `Cargo.lock`: the floor a
#      user's resolver may pick is exactly the version the gates ran on. Bumping a dependency in the lock without
#      the manifests (or the reverse) is red.
#   2. TESTED CRATES ONLY. The crate is a normal dependency of `preview/Cargo.toml` (the docs site compiles every
#      component, so shipped = compiled here). An entry with neither `version` nor `git` (cargo add would take the
#      newest release) or with a `git` other than the registry's own crates is red.
#   3. PINNING POLICY (audit A4 policy 2, owner decision E2): the crates in EXACT_ONLY are written `=x.y.z`
#      (a leaf crate one component owns, or one nobody else's tree is likely to share); the widely shared crates
#      whose types appear in our API (`time`, `palette`) keep a caret with the tested floor.
#   4. PARSERS STAY OPT-IN. Only the components in PARSER_ALLOW may list a markdown/HTML/syntax parser crate, so a
#      parser (a supply-chain surface, owner decision E2: "only `markdown` may list pulldown-cmark") can never
#      become a dependency of an unrelated component such as Message by a copy-paste.
#   5. RELEASE REV. `dioxus-primitives` entries are either all unstamped (the development tree) or all carry the
#      same 40-hex `rev` (`scripts/stamp-release-rev.sh` writes it at release time: a commit cannot contain its own
#      hash, so it is a release step).
#   6. FEATURES FOLLOW USE. A `dioxus-primitives` module behind a cargo feature in primitives/src/lib.rs
#      (`#[cfg(feature = "calendar")] pub mod calendar;`) must have that feature in the `features` of a
#      `dioxus-primitives` entry in the closure of every component whose installed (non-excluded) Rust files name
#      `dioxus_primitives::<module>`, and every listed feature must exist in primitives/Cargo.toml. Otherwise
#      `dx components add` yields E0432 in a user's tree while the repo (all features on) compiles.
#      `scripts/check-primitives-features.sh` is the compile half; `scripts/registry-install-smoke.sh` the
#      definitive pre-merge one.
#
# `scripts/stamp-release-rev.sh <sha>` writes the release rev into every `dioxus-primitives` entry; `--check` verifies
# it and `--remove` returns the tree to the unstamped development state.
#
# Usage: scripts/check-cargo-dep-versions.sh
# Exit status: 0 when clean, 1 on a violation.

set -uo pipefail
cd "$(dirname "$0")/.."

python3 -I - <<'PY'
import glob, json, os, re, sys, tomllib

# crate -> reason; written `=x.y.z` in component.json.
EXACT_ONLY = {
    "dioxus-icons": "leaf crate used by 27 components, pre-1.0 and one maintainer: an exact pin keeps every install on the version we render with",
    "pulldown-cmark": "markdown parser (owner decision E2: exact pin, because model output reaches this parser and 0.13.4 carries a panic fix)",
}
# Markdown / HTML / syntax parser crates: a component may list one only if PARSER_ALLOW[crate] names it.
PARSER_ALLOW = {
    "pulldown-cmark": ["markdown"],   # the markdown component (dev-docs/backlog.md row 213)
    "pulldown-cmark-escape": ["markdown"],
}
PARSERS = set(PARSER_ALLOW) | {
    "markdown", "comrak", "mdstream", "remend", "dioxus-markdown", "dioxus-nox-markdown", "markdown-dx",
    "dioxus-mdx", "syntect", "tree-sitter", "arborium", "dioxus-code", "html5ever", "scraper", "ammonia",
}
REGISTRY_CRATES = {"dioxus-primitives", "dioxus-attributes"}

errors = []
def err(msg): errors.append(msg)

lock = tomllib.load(open("Cargo.lock", "rb"))
locked = {}
for p in lock["package"]:
    locked.setdefault(p["name"], set()).add(p["version"])

def dep_names(path):
    t = tomllib.load(open(path, "rb"))
    return set(t.get("dependencies", {}))
preview_deps = dep_names("preview/Cargo.toml")
prim_toml = tomllib.load(open("primitives/Cargo.toml", "rb"))
prim_features = set(prim_toml.get("features", {}))

# feature-gated modules of primitives
lib = open("primitives/src/lib.rs").read()
gated = {m: f for f, m in re.findall(r'#\[cfg\(feature = "([\w-]+)"\)\]\s*\n\s*pub mod (\w+);', lib)}

comps = {}
for f in sorted(glob.glob("preview/src/components/*/component.json")):
    d = os.path.dirname(f)
    try:
        comps[os.path.basename(d)] = (d, json.load(open(f)))
    except ValueError as e:
        print(f"check-cargo-dep-versions: {f} is not valid JSON: {e}", file=sys.stderr); sys.exit(1)

def closure(name, seen=None):
    seen = set() if seen is None else seen
    if name in seen or name not in comps: return seen
    seen.add(name)
    for dep in comps[name][1].get("componentDependencies") or []:
        closure(dep["name"] if isinstance(dep, dict) else dep, seen)
    return seen

req_re = re.compile(r'^(=|\^|~)?(\d+\.\d+\.\d+)$')
revs = {}
for name, (d, m) in comps.items():
    for c in m.get("cargoDependencies") or []:
        if not isinstance(c, dict):
            err(f"{d}/component.json: cargoDependencies entry {c!r} is not an object"); continue
        crate = c.get("name")
        where = f"{d}/component.json: {crate}"
        if crate in REGISTRY_CRATES:
            if "rev" in c: revs.setdefault(c["rev"], []).append(f"{name}:{crate}")
            else: revs.setdefault(None, []).append(f"{name}:{crate}")
            for ft in c.get("features") or []:
                if crate == "dioxus-primitives" and ft not in prim_features:
                    err(f"{where} lists feature {ft!r}, which primitives/Cargo.toml does not define ({sorted(prim_features)})")
            continue
        if crate in PARSERS and name not in PARSER_ALLOW.get(crate, []):
            err(f"{where}: parser crate; only {PARSER_ALLOW.get(crate) or 'no component'} may list it (PARSER_ALLOW in scripts/check-cargo-dep-versions.sh)")
        if "git" in c or "path" in c:
            err(f"{where}: a git/path source other than the registry's own crates is not allowed"); continue
        ver = c.get("version")
        if not ver:
            err(f"{where}: no `version` (cargo add would take the newest release; write the version the lock tests)"); continue
        mm = req_re.match(ver)
        if not mm:
            err(f"{where}: version {ver!r} is not `x.y.z`, `^x.y.z`, `~x.y.z` or `=x.y.z`"); continue
        op, base = mm.groups()
        if crate not in locked:
            err(f"{where}: {crate} is not in Cargo.lock, so nothing tests it")
        elif base not in locked[crate]:
            err(f"{where}: version {ver!r} but Cargo.lock has {sorted(locked[crate])}: the floor must equal the tested version (shipped == tested)")
        if crate not in preview_deps:
            err(f"{where}: not a dependency of preview/Cargo.toml, so the docs site never compiles it")
        if crate in EXACT_ONLY and op != "=":
            err(f"{where}: write \"={base}\" ({EXACT_ONLY[crate]})")

stamped = [r for r in revs if r is not None]
if stamped:
    if len(revs) > 1 or not re.fullmatch(r"[0-9a-f]{40}", stamped[0]):
        err(f"dioxus-primitives `rev`s are not one 40-hex value on every entry: {({k: len(v) for k, v in revs.items()})} (scripts/stamp-release-rev.sh)")

# 6. features follow use
def comp_rust(name):
    d, m = comps[name]
    ex = [os.path.normpath(os.path.join(d, e)) for e in m.get("exclude") or []]
    out = []
    for root, _, files in os.walk(d):
        for fn in files:
            p = os.path.join(root, fn)
            if fn.endswith(".rs") and not any(os.path.normpath(p).startswith(e + os.sep) or os.path.normpath(p) == e for e in ex):
                out.append(p)
    return out

def primitives_modules(src):
    src = re.sub(r"//[^\n]*", "", src)
    mods = set()
    for m in re.finditer(r"dioxus_primitives::\s*", src):
        i = m.end()
        if src[i:i+1] == "{":
            depth, j, start = 0, i, i + 1
            items = []
            while j < len(src):
                ch = src[j]
                if ch == "{": depth += 1
                elif ch == "}":
                    depth -= 1
                    if depth == 0: items.append(src[start:j]); break
                elif ch == "," and depth == 1:
                    items.append(src[start:j]); start = j + 1
                j += 1
            for it in items:
                im = re.match(r"\s*(\w+)", it)
                if im: mods.add(im.group(1))
        else:
            im = re.match(r"(\w+)", src[i:])
            if im: mods.add(im.group(1))
    return mods

for name in comps:
    have = set()
    for dn in closure(name):
        for c in comps[dn][1].get("cargoDependencies") or []:
            if isinstance(c, dict) and c.get("name") == "dioxus-primitives":
                have |= set(c.get("features") or [])
    for p in comp_rust(name):
        for mod in primitives_modules(open(p).read()):
            ft = gated.get(mod)
            if ft and ft not in have:
                err(f"{p}: uses dioxus_primitives::{mod}, which needs the primitives feature {ft!r}; add \"features\": [\"{ft}\"] to the dioxus-primitives entry of {name}/component.json (or of a component in its closure)")

if errors:
    print("check-cargo-dep-versions: component.json out of step with Cargo.lock / policy:", file=sys.stderr)
    for e in sorted(set(errors)): print("  " + e, file=sys.stderr)
    sys.exit(1)
n = sum(len([c for c in m.get("cargoDependencies") or [] if isinstance(c, dict) and c.get("version")]) for _, m in comps.values())
print(f"check-cargo-dep-versions: OK -- {n} versioned cargoDependencies equal the Cargo.lock versions; gated primitives modules: {gated}; "
      f"{'release rev stamped' if stamped else 'primitives unstamped (development tree)'}.")
PY
