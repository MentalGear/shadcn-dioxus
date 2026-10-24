#!/usr/bin/env bash
#
# check-markdown-parser.sh
#
# The parser allowlist of the optional `markdown` component: its `component.json` names exactly `dioxus-primitives`
# and `pulldown-cmark` (no default features), `preview/Cargo.toml` pins `pulldown-cmark` as `=<locked version>` with
# `default-features = false`, the lock's parser has only its four allowed dependencies, `pulldown_cmark` is named only
# inside `preview/src/components/markdown/` (never its `html` renderer) with only the ENABLE_TABLES,
# ENABLE_STRIKETHROUGH and ENABLE_TASKLISTS options, and no `dangerous_inner_html` appears, so model output has no
# path to the DOM as raw HTML.
#
# The parser allowlist for the optional `markdown` component (owner decisions 2026-10-15, backlog row 213):
# what the component may pull in and which parser features it may switch on. The OTHER half -- only an
# allowlisted component may list a parser crate in its `component.json`, and every `component.json` version
# equals `Cargo.lock` -- is `scripts/check-cargo-dep-versions.sh`; this gate covers what that one cannot see.
#   1. THE CRATES. `markdown`'s `component.json` `cargoDependencies` name exactly `dioxus-primitives` and
#      `pulldown-cmark` (no highlighter, no second parser, nothing else a user would compile and trust), and the
#      `pulldown-cmark` entry carries no `features` and no `default_features` (the installer then passes
#      `--no-default-features`: the `html` renderer and the CLI stay out of a user's tree).
#   2. THE PIN. `preview/Cargo.toml`'s normal `pulldown-cmark` dependency is `=<locked version>` with
#      `default-features = false` and no `features` (the build-dependency may keep `html` for docs.md).
#   3. THE LOCK. One `pulldown-cmark` in `Cargo.lock`, depending only on bitflags, memchr, unicase and
#      pulldown-cmark-escape: a new transitive dependency of the parser fails here and needs a decision.
#   4. THE PARSER'S SURFACE. `pulldown_cmark` is named only inside `preview/src/components/markdown/` (docs.md
#      rendering lives in `preview/build.rs`), never its `html` renderer, and the only `Options` flags are
#      ENABLE_TABLES, ENABLE_STRIKETHROUGH and ENABLE_TASKLISTS: heading attributes, metadata blocks, footnotes,
#      math, definition lists, smart punctuation and GFM alerts are injection surface and stay off. No
#      `dangerous_inner_html` in the component or its demos: unsafe markup has no path to the DOM.
#
# Usage: scripts/check-markdown-parser.sh
# Exit status: 0 when clean, 1 on a violation.

set -uo pipefail
cd "$(dirname "$0")/.."

python3 -I - <<'PY'
import glob, json, os, re, sys, tomllib

errors = []
def err(m): errors.append(m)

ALLOWED_OPTIONS = {"ENABLE_TABLES", "ENABLE_STRIKETHROUGH", "ENABLE_TASKLISTS"}
ALLOWED_LOCK_DEPS = {"bitflags", "memchr", "unicase", "pulldown-cmark-escape"}
ALLOWED_COMPONENT_CRATES = {"dioxus-primitives", "pulldown-cmark"}
MD_DIR = "preview/src/components/markdown"

lock = tomllib.load(open("Cargo.lock", "rb"))
pcs = [p for p in lock["package"] if p["name"] == "pulldown-cmark"]
if len(pcs) != 1:
    err(f"Cargo.lock has {len(pcs)} pulldown-cmark versions, want exactly one")
locked = pcs[0]["version"] if pcs else None
if pcs:
    deps = {d.split()[0] for d in pcs[0].get("dependencies", [])}
    extra = deps - ALLOWED_LOCK_DEPS
    if extra:
        err(f"pulldown-cmark {locked} now depends on {sorted(extra)}: not in the parser allowlist {sorted(ALLOWED_LOCK_DEPS)} (ALLOWED_LOCK_DEPS in this script)")

# 1. component.json
mj = os.path.join(MD_DIR, "component.json")
if not os.path.exists(mj):
    err(f"{mj} is missing")
else:
    m = json.load(open(mj))
    names = [c.get("name") if isinstance(c, dict) else c for c in m.get("cargoDependencies") or []]
    if set(names) != ALLOWED_COMPONENT_CRATES or len(names) != len(set(names)):
        err(f"{mj}: cargoDependencies are {names}, want exactly {sorted(ALLOWED_COMPONENT_CRATES)}")
    for c in m.get("cargoDependencies") or []:
        if isinstance(c, dict) and c.get("name") == "pulldown-cmark":
            if c.get("features"):
                err(f"{mj}: pulldown-cmark lists features {c['features']}; the component needs none")
            if c.get("default_features"):
                err(f"{mj}: pulldown-cmark sets default_features; the installer must pass --no-default-features")
            if c.get("version") != f"={locked}":
                err(f"{mj}: pulldown-cmark version {c.get('version')!r}, want '={locked}' (the locked version)")

# 2. preview/Cargo.toml
pv = tomllib.load(open("preview/Cargo.toml", "rb"))
dep = pv.get("dependencies", {}).get("pulldown-cmark")
if dep is None:
    err("preview/Cargo.toml: pulldown-cmark is not a normal dependency, so the docs site never compiles the component")
else:
    if not isinstance(dep, dict):
        dep = {"version": dep}
    if dep.get("version") != f"={locked}":
        err(f"preview/Cargo.toml: pulldown-cmark version {dep.get('version')!r}, want '={locked}'")
    if dep.get("default-features", True) is not False:
        err("preview/Cargo.toml: pulldown-cmark must set default-features = false")
    if dep.get("features"):
        err(f"preview/Cargo.toml: normal pulldown-cmark lists features {dep['features']}; none are needed")

# 4. sources
def strip(src):
    src = re.sub(r"//[^\n]*", "", src)
    return re.sub(r"/\*.*?\*/", "", src, flags=re.S)

for p in glob.glob("preview/src/**/*.rs", recursive=True):
    s = strip(open(p).read())
    inside = p.startswith(MD_DIR + "/")
    if re.search(r"\bpulldown_cmark\b", s) and not inside:
        err(f"{p}: names pulldown_cmark outside {MD_DIR}/ (the parser stays inside the optional component)")
    if inside:
        if re.search(r"pulldown_cmark::html|\bpulldown_cmark_escape\b", s):
            err(f"{p}: uses the pulldown-cmark html renderer; render events into the allowlisted tree instead")
        for flag in re.findall(r"Options::([A-Z_]+)", s):
            if flag not in ALLOWED_OPTIONS:
                err(f"{p}: Options::{flag} is not allowlisted {sorted(ALLOWED_OPTIONS)}: it widens what a model can inject")
        if re.search(r"Options::(all|empty)\(", s):
            err(f"{p}: Options::all()/empty() hides the option allowlist; name each flag")
        if "dangerous_inner_html" in s:
            err(f"{p}: dangerous_inner_html in the markdown component or its demos")

if errors:
    print("check-markdown-parser: the markdown parser allowlist is violated:", file=sys.stderr)
    for e in sorted(set(errors)): print("  " + e, file=sys.stderr)
    sys.exit(1)
print(f"check-markdown-parser: OK -- pulldown-cmark {locked} (exact pin, default features off, deps {sorted(ALLOWED_LOCK_DEPS)}), "
      f"options {sorted(ALLOWED_OPTIONS)}, no html renderer, no dangerous_inner_html.")
PY
