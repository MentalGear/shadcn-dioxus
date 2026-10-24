#!/usr/bin/env bash
#
# check-installed-paths.sh
#
# Keeps the preview's component and variant sources compiling in a USER's
# tree, not only in this repo. `dx components add <name>` installs a component
# as `src/components/<name>/mod.rs` = `mod component; pub use component::*;`,
# so `component` is a PRIVATE module there and the only public path to a
# component's items is `crate::components::<name>::Item` (or `::*`). Inside
# this repo `examples!` (preview/src/components/mod.rs) keeps `component`
# private for the same reason, so such a path fails to compile here too -- but
# a path like `pub use crate::components::chart::component::*;` or
# `use crate::components::button::component::Button;` compiles fine in a tree
# where `component` is public and fails in an installed one.
#
# Rule: no `components::<name>::component::` path anywhere in
# preview/src/**/*.rs outside comments (the app shell in main.rs may name
# `components::<name>::variants`: that module is repo-only). A component reaches a
# sibling as `crate::components::<name>::*`; a variant reaches its own
# component as `super::super::component::*` (installed_source.rs rewrites that
# for the Copy/View Code tabs).
#
# Second half (install completeness). dioxus-cli 0.7.9 (`src/cli/component.rs`, `add_component` /
# `copy_component_files`) installs a component by copying its WHOLE directory minus the manifest's
# `exclude` list (each entry is canonicalized, so one that does not exist fails the install) to
# `<crate>/src/components/<name>/`, copying every `globalAssets` file into `assets/`, installing every
# `componentDependencies` entry the same way, and appending `pub mod <name>;` to the user's
# `src/components/mod.rs`. There is no `files` field. So a component installs and compiles only if its
# directory itself holds a `mod.rs` that declares `component` (preview's `examples!` declares the module
# inline and never reads that file, so a missing one compiles here and breaks with E0583 in a user's
# tree). For every `preview/src/components/*/component.json` this checks:
#   - the dir is a member of the root `component.json` and the manifest `name` is the dir name (a dir that
#     is deliberately not installable goes in NOT_INSTALLABLE below, with the reason);
#   - `mod.rs` exists and is exactly `mod component; pub use component::*;`, `component.rs` exists;
#   - no manifest repeats a key (serde in dx rejects `duplicate field`; Python's json.load would not notice);
#   - every `exclude` and `globalAssets` entry exists, and no `exclude` covers `mod.rs`/`component.rs`/`style.css`;
#   - every file `component.rs` needs from its own dir survives the copy: `mod x;`, `#[path = ".."]`,
#     `asset!("/src/components/<name>/..")`, `include_str!("..")`;
#   - every `crate::components::<other>::` it names is in its `componentDependencies` closure, so
#     `dx components add <name>` alone brings it in;
#   - every external crate its Rust files use (`dioxus_primitives`, `dioxus_icons`, `time`, `palette`,
#     `dioxus_ssr`, ... anything but std/core/alloc and `dioxus`, which the app owns) is named by a
#     `cargoDependencies` entry of the component itself or of one in its `componentDependencies` closure. Both
#     installers run `cargo add` for the whole closure (dx: `run`, "collect all required rust dependencies";
#     `shadcn-dioxus`: `add.rs`, `to_install`), and nothing else puts the crate in the user's Cargo.toml: an
#     app that was not made from `templates/starter` has no `dioxus-primitives`, and a component with no
#     `cargoDependencies` fails there
#     with E0433 while the starter template, the preview and the smoke script all pre-declare it.
# `scripts/registry-install-smoke.sh` is the definitive (slow, pre-merge) check that this approximates.
#
# Usage: scripts/check-installed-paths.sh
# Exit status: 0 when clean.

set -uo pipefail
cd "$(dirname "$0")/.."

hits=$(grep -rnE 'components::[a-z_0-9]+::component::' preview/src --include='*.rs' \
  | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' \
  | grep -v "^preview/src/installed_source.rs:" || true)
if [ -n "$hits" ]; then
  echo "check-installed-paths: path(s) through a component's private \`component\` module (does not compile after \`dx components add\`):" >&2
  echo "$hits" >&2
  echo "Use crate::components::<name>::* (or ::Item) instead." >&2
  exit 1
fi
echo "check-installed-paths: OK -- no components::<name>::component:: paths in preview/src."

python3 -I - <<'PY' || exit 1
import glob, json, os, re, sys

# dir -> why it has a component.json but is not an installable registry member.
NOT_INSTALLABLE = {
    "top_layer": "oracle fixture for playwright/oracle/tier2-html/top-layer.spec.ts, not a component",
}
root = "preview/src/components"
members = {m.rsplit("/", 1)[-1] for m in json.load(open("component.json"))["members"]}
dirs = {os.path.basename(os.path.dirname(f)): f for f in glob.glob(f"{root}/*/component.json")}
problems = []

for name in sorted(members - dirs.keys()):
    problems.append(f"root component.json lists member '{name}' but {root}/{name}/component.json does not exist")
for name in sorted(dirs.keys() - members):
    if name not in NOT_INSTALLABLE:
        problems.append(f"{root}/{name}/component.json exists but '{name}' is not a member of the root component.json (dx cannot find it); add it, or list it in NOT_INSTALLABLE")
for name in sorted(NOT_INSTALLABLE.keys() & members):
    problems.append(f"'{name}' is in NOT_INSTALLABLE but is also a root component.json member")

def no_dup_keys(pairs):
    # json.load keeps the LAST of a repeated key silently; dx's serde rejects the file ("duplicate field").
    keys = [k for k, _ in pairs]
    for k in sorted({k for k in keys if keys.count(k) > 1}):
        problems.append(f"{current[0]}: key '{k}' appears more than once (dx components add fails with `duplicate field`)")
    return dict(pairs)

current = [""]
manifests = {}
for n, f in dirs.items():
    current[0] = f
    manifests[n] = json.load(open(f), object_pairs_hook=no_dup_keys)

def deps(name, seen=None):
    seen = set() if seen is None else seen
    for d in manifests.get(name, {}).get("componentDependencies") or []:
        dn = d if isinstance(d, str) else d.get("name")
        if dn not in seen:
            seen.add(dn)
            deps(dn, seen)
    return seen

for name in sorted(dirs.keys() & members):
    d = f"{root}/{name}"
    m = manifests[name]
    if m.get("name") != name:
        problems.append(f"{d}/component.json: name is {m.get('name')!r}, want the directory name {name!r}")
    for dn in sorted(deps(name)):
        if dn not in members:
            problems.append(f"{d}/component.json: componentDependencies names '{dn}', which is not a root member")
    excluded = [os.path.normpath(e) for e in (m.get("exclude") or [])]
    for e in excluded:
        if not os.path.exists(os.path.join(d, e)):
            problems.append(f"{d}/component.json: exclude entry '{e}' does not exist (dx canonicalizes it and fails the whole install)")
    for a in m.get("globalAssets") or []:
        if not os.path.exists(os.path.join(d, a)):
            problems.append(f"{d}/component.json: globalAssets entry '{a}' does not exist")

    def survives(rel):
        rel = os.path.normpath(rel)
        return not any(rel == e or rel.startswith(e + os.sep) for e in excluded)

    for need in ("mod.rs", "component.rs", "style.css"):
        if not os.path.isfile(f"{d}/{need}"):
            problems.append(f"{d}/{need} is missing (dx components add {name} would not install it)" + (" -> E0583 `file not found for module`" if need == "mod.rs" else ""))
        elif not survives(need):
            problems.append(f"{d}/component.json: exclude covers {need}, which an install needs")
    if os.path.isfile(f"{d}/mod.rs"):
        body = re.sub(r"\s+", " ", re.sub(r"//[^\n]*", "", open(f"{d}/mod.rs").read())).strip()
        if body != "mod component; pub use component::*;":
            problems.append(f"{d}/mod.rs must be exactly `mod component; pub use component::*;` (it declares what component.rs exports), got: {body!r}")
    if not os.path.isfile(f"{d}/component.rs"):
        continue
    src = "\n".join(l for l in open(f"{d}/component.rs").read().splitlines() if not l.lstrip().startswith("//"))
    for mm in re.finditer(r'(?:#\[path\s*=\s*"([^"]+)"\]\s*)?^\s*(?:pub(?:\([a-z]+\))?\s+)?mod\s+([a-z_0-9]+)\s*;', src, re.M):
        rel = mm.group(1) or mm.group(2) + ".rs"
        if not os.path.isfile(f"{d}/{rel}"):
            problems.append(f"{d}/component.rs declares `mod {mm.group(2)};` but {d}/{rel} does not exist")
        elif not survives(rel):
            problems.append(f"{d}/component.rs declares `mod {mm.group(2)};` but exclude drops {rel}")
    for a in set(re.findall(r'asset!\(\s*"([^"]+)"', src)):
        if a.startswith("/src/components/"):
            rel = a[len("/src/components/"):]
            if not os.path.isfile(f"{root}/{rel}"):
                problems.append(f"{d}/component.rs: asset!(\"{a}\") does not exist")
            elif rel.split("/", 1)[0] not in deps(name) | {name}:
                problems.append(f"{d}/component.rs: asset!(\"{a}\") points into a component that is not in its componentDependencies closure")
            elif rel.split("/", 1)[0] == name and not survives(rel.split("/", 1)[1]):
                problems.append(f"{d}/component.rs: asset!(\"{a}\") is dropped by exclude")
    for a in re.findall(r'include_(?:str|bytes)!\(\s*"([^"]+)"', src):
        if not os.path.isfile(os.path.join(d, a)):
            problems.append(f"{d}/component.rs: include_str!(\"{a}\") does not exist relative to the component directory")
        elif not survives(a):
            problems.append(f"{d}/component.rs: include_str!(\"{a}\") is dropped by exclude")
    if re.search(r"\bvariants\b", re.sub(r'"[^"]*"', "", src)):
        problems.append(f"{d}/component.rs names `variants`, which exclude drops from an install")
    allowed = deps(name) | {name}
    for other in sorted(set(re.findall(r"crate::components::([a-z_0-9]+)::", src)) - allowed):
        problems.append(f"{d}/component.rs uses crate::components::{other}::, but '{other}' is not in its componentDependencies closure (dx components add {name} would not bring it)")

def known_crates():
    """Every external crate name the workspace or any manifest mentions (underscored), plus dioxus_* siblings."""
    k = set()
    for f in ("Cargo.toml", "preview/Cargo.toml", "primitives/Cargo.toml"):
        section = None
        for line in open(f):
            h = re.match(r"\s*\[(.*)\]", line)
            if h:
                section = h.group(1)
            elif section and "dependencies" in section:
                m2 = re.match(r"([A-Za-z0-9_-]+)\s*=", line)
                if m2:
                    k.add(m2.group(1).replace("-", "_"))
    for m in manifests.values():
        for c in m.get("cargoDependencies") or []:
            k.add(c["name"].replace("-", "_"))
    return k - {"dioxus"}

KNOWN = known_crates()
NOT_EXTERNAL = {"crate", "self", "super", "std", "core", "alloc", "dioxus", "r", "b", "br"}
_STRIP = re.compile(r'/\*.*?\*/|//[^\n]*|r(#*)".*?"\1|b?"(?:\\.|[^"\\])*"|b?\'(?:\\.|[^\'\\])\'', re.S)

def drop_cfg_test(text):
    """Remove `#[cfg(test)] mod x { .. }` blocks (comments and strings are already gone, so braces balance).
    They compile only under the user's `cargo test`, where dev-dependencies such as dioxus-ssr are theirs to add."""
    while True:
        m = re.search(r"#\[cfg\(test\)\]\s*(?:pub\s+)?mod\s+\w+\s*\{", text)
        if not m:
            return text
        depth, i = 1, m.end()
        while i < len(text) and depth:
            depth += {"{": 1, "}": -1}.get(text[i], 0)
            i += 1
        text = text[:m.start()] + text[i:]

def crates_used(path_dir, survives):
    """External crate roots named by `use` statements and by `crate_name::` paths in the installed .rs files."""
    text = ""
    for dp, _, fs in os.walk(path_dir):
        for fn in fs:
            full = os.path.join(dp, fn)
            rel = os.path.relpath(full, path_dir)
            if fn.endswith(".rs") and survives(rel):
                text += _STRIP.sub(" ", open(full).read()) + "\n"
    text = drop_cfg_test(text)
    local = set(re.findall(r"\bmod\s+([a-z_0-9]+)", text))
    used = {r for r in re.findall(r"(?<![\w:])use\s+(?:::)?([a-z_][a-z0-9_]*)", text) if r not in NOT_EXTERNAL and r not in local}
    # Paths outside `use` statements only: inside `use dioxus_primitives::{dioxus_attributes::attributes, ..}` the
    # inner `dioxus_attributes` is a re-export of the root crate, not a crate of its own in the user's tree.
    body = re.sub(r"(?<![\w:])use\s[^;]*;", " ", text)
    used |= {r for r in re.findall(r"(?<![\w:])([a-z_][a-z0-9_]*)::", body) if r in KNOWN and r not in local}
    used |= set(re.findall(r"(?<![\w:])(dioxus_[a-z_0-9]+)::", body))
    return used

for name in sorted(dirs.keys() & members):
    d = f"{root}/{name}"
    excluded = [os.path.normpath(e) for e in (manifests[name].get("exclude") or [])]
    surv = lambda rel, ex=excluded: not any(os.path.normpath(rel) == e or os.path.normpath(rel).startswith(e + os.sep) for e in ex)
    declared = set()
    for n in deps(name) | {name}:
        for c in manifests.get(n, {}).get("cargoDependencies") or []:
            declared.add(c["name"].replace("-", "_"))
    for crate in sorted(crates_used(d, surv) - declared):
        problems.append(f"{d}: its Rust files use the crate `{crate}`, but neither its component.json nor a componentDependencies closure member lists it in cargoDependencies (an app without `{crate.replace('_', '-')}` fails with E0433 after `dx components add {name}`)")

if problems:
    print("check-installed-paths: components that would not install and compile with `dx components add`:", file=sys.stderr)
    for p in problems:
        print("  " + p, file=sys.stderr)
    sys.exit(1)
print(f"check-installed-paths: OK -- {len(members)} components each carry mod.rs + component.rs + style.css, with exclude/assets/dependencies (component and cargo) consistent.")
PY
