#!/usr/bin/env python3
"""Generates every component manifest from the component's own sources (backlog row 430).

Run it through `scripts/gen-component-manifests.sh` (python3 -I, from the repo root); the arguments are
`--check` (read-only: exit 1 and a diff when a generated file differs from the tree) or none (write).

What is GENERATED (a manifest cannot disagree with the files it ships, because it is their output):

  preview/src/components/<dir>/mod.rs          `mod component;\\npub use component::*;\\n`, the one shape `dx components add`
                                               installs (`examples!` declares the module inline and never reads the file)
  preview/src/components/<dir>/component.json  everything but `name`, `description`, `authors`:
      exclude               `variants`, `docs.md`, `component.json` (what a user's copy never needs)
      globalAssets          the theme when the directory has a `style.css`, plus `dx-effects.css` when the installed
                            files, or a primitives module they use, name one of its classes (`dx-scroll-fade`, `dx-shimmer`)
      componentDependencies every `crate::components::<other>::` path and every `asset!("/src/components/<other>/..")`
                            of the INSTALLED Rust files (outside comments, strings and `variants/`), as `{name, git}` objects
                            (a bare name resolves in dx's default registry), sorted
      cargoDependencies     `dioxus-primitives` always (the crate every component is written against), plus every
                            other external crate the installed Rust files use, sorted, each as the registry `{name, git}`
                            (+ `features` for the cargo-gated primitives modules the files name) or `{name, version}`
                            with the `Cargo.lock` version (`=x.y.z` for EXACT_ONLY, `x.y.z` otherwise)
  component.json (repo root)                   `members` = every component directory but NOT_INSTALLABLE, sorted

What stays HAND-AUTHORED, read from the existing manifest: `name` (the directory name), `description`, `authors`. A new
component starts as a three-field manifest and this script fills in the rest. The release `rev` of the registry crates
is the one other input: `scripts/stamp-release-rev.sh` writes it (a commit cannot contain its own hash), so an existing
`rev` is carried over, never invented.

Same rules the gates hold (`check-installed-paths.sh`, `check-cargo-dep-versions.sh`, `check-registry-url.sh`,
`check-component-catalog.sh`), which stay as independent cross-checks of this script's output.
"""
import difflib
import glob
import json
import os
import re
import sys
import tomllib

ROOT = "preview/src/components"
EXCLUDE = ["variants", "docs.md", "component.json"]
THEME = "../../../assets/dx-components-theme.css"
EFFECTS = "../../../assets/dx-effects.css"
EFFECTS_CLASSES = re.compile(r"\bdx-(?:scroll-fade|shimmer)\b")
MOD_RS = "mod component;\npub use component::*;\n"
REGISTRY_CRATES = ("dioxus-primitives", "dioxus-attributes")
ALWAYS = "dioxus-primitives"
# dir -> why it has a component.json but is not an installable registry member (mirrors check-installed-paths.sh).
NOT_INSTALLABLE = {"top_layer"}
# crate -> written `=x.y.z` in component.json (mirrors check-cargo-dep-versions.sh EXACT_ONLY).
EXACT_ONLY = {"dioxus-icons", "pulldown-cmark"}
HAND = ("name", "description", "authors")
KEY_ORDER = ("name", "description", "authors", "exclude", "cargoDependencies", "componentDependencies", "globalAssets")
NOT_EXTERNAL = {"crate", "self", "super", "std", "core", "alloc", "dioxus", "r", "b", "br"}
WIDTH = 80

_COMMENTS = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
_STRIP = re.compile(r'/\*.*?\*/|//[^\n]*|r(#*)".*?"\1|b?"(?:\\.|[^"\\])*"|b?\'(?:\\.|[^\'\\])\'', re.S)


def die(msg):
    print(f"gen-component-manifests: {msg}", file=sys.stderr)
    sys.exit(2)


def registry_url():
    m = re.search(r'^pub\(crate\) const REGISTRY_GIT_URL: &str = "(.*)";$', open("preview/src/main.rs").read(), re.M)
    if not m:
        die("no `pub(crate) const REGISTRY_GIT_URL: &str = \"...\";` in preview/src/main.rs")
    return m.group(1)


def strip_comments(src):
    """Comments gone, strings kept (`asset!("..")` paths and class names live in strings)."""
    return _COMMENTS.sub(" ", src)


def strip_all(src):
    """Comments and string literals gone (raw ones too)."""
    return _STRIP.sub(" ", src)


def drop_cfg_test(text):
    """Remove `#[cfg(test)] mod x { .. }` blocks; they compile only under the user's own `cargo test`."""
    while True:
        m = re.search(r"#\[cfg\(test\)\]\s*(?:pub\s+)?mod\s+\w+\s*\{", text)
        if not m:
            return text
        depth, i = 1, m.end()
        while i < len(text) and depth:
            depth += {"{": 1, "}": -1}.get(text[i], 0)
            i += 1
        text = text[:m.start()] + text[i:]


def sibling_refs(src, prefix):
    """Names after `prefix` (`crate::components::` or `super::super::`), `{a, b::{X}}` groups included (src has no
    comments or strings)."""
    found = set()
    for m in re.finditer(prefix, src):
        rest = src[m.end():]
        if rest.startswith("{"):
            depth, start, parts = 0, 1, []
            for j, ch in enumerate(rest):
                if ch == "{":
                    depth += 1
                elif ch == "}":
                    depth -= 1
                    if depth == 0:
                        parts.append(rest[start:j])
                        break
                elif ch == "," and depth == 1:
                    parts.append(rest[start:j])
                    start = j + 1
            for part in parts:
                mm = re.match(r"\s*([a-z_0-9]+)", part)
                if mm:
                    found.add(mm.group(1))
        else:
            mm = re.match(r"([a-z_0-9]+)", rest)
            if mm:
                found.add(mm.group(1))
    return found


def primitives_modules(src):
    """First path segment after every `dioxus_primitives::` (a `{a, b::{..}}` group names several)."""
    mods = set()
    for m in re.finditer(r"dioxus_primitives::\s*", src):
        i = m.end()
        if src[i:i + 1] == "{":
            depth, j, start, items = 0, i, i + 1, []
            while j < len(src):
                ch = src[j]
                if ch == "{":
                    depth += 1
                elif ch == "}":
                    depth -= 1
                    if depth == 0:
                        items.append(src[start:j])
                        break
                elif ch == "," and depth == 1:
                    items.append(src[start:j])
                    start = j + 1
                j += 1
            for it in items:
                im = re.match(r"\s*(\w+)", it)
                if im:
                    mods.add(im.group(1))
        else:
            im = re.match(r"(\w+)", src[i:])
            if im:
                mods.add(im.group(1))
    return mods


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


class Workspace:
    def __init__(self):
        self.url = registry_url()
        lock = tomllib.load(open("Cargo.lock", "rb"))
        self.locked = {}
        for p in lock["package"]:
            self.locked.setdefault(p["name"], set()).add(p["version"])
        # underscored crate name -> the hyphenated name Cargo.toml writes, from the dependency tables the preview and
        # primitives compile (the same set check-installed-paths.sh reads, minus the manifests this script writes).
        self.crate_names = {}
        self.preview_req = {}
        for f in ("Cargo.toml", "preview/Cargo.toml", "primitives/Cargo.toml"):
            section = None
            for line in open(f):
                h = re.match(r"\s*\[(.*)\]", line)
                if h:
                    section = h.group(1)
                elif section and "dependencies" in section:
                    m = re.match(r"([A-Za-z0-9_-]+)\s*=", line)
                    if m:
                        self.crate_names[m.group(1).replace("-", "_")] = m.group(1)
        pt = tomllib.load(open("preview/Cargo.toml", "rb"))
        for name, spec in pt.get("dependencies", {}).items():
            self.preview_req[name] = spec.get("version") if isinstance(spec, dict) else spec
        lib = read("primitives/src/lib.rs")
        # primitives module -> the cargo feature that gates it
        self.gated = {m: f for f, m in re.findall(r'#\[cfg\(feature = "([\w-]+)"\)\]\s*\n\s*pub mod (\w+);', lib)}
        self._prim_effects = {}
        self.components = {os.path.basename(os.path.dirname(f)) for f in glob.glob(f"{ROOT}/*/component.json")}

    def lock_version(self, crate):
        vs = self.locked.get(crate)
        if not vs:
            die(f"crate {crate} is used by a component but is not in Cargo.lock, so nothing tests it")
        if len(vs) == 1:
            return next(iter(vs))
        req = (self.preview_req.get(crate) or "").lstrip("=^~")
        if req in vs:
            return req
        die(f"Cargo.lock has several versions of {crate} ({sorted(vs)}) and preview/Cargo.toml does not name one")

    def primitive_names_effects(self, module, seen=None):
        """Does primitives module `module` emit a `dx-effects.css` class outside comments, itself or through another
        primitives module it names as `crate::<other>` (`virtual_list` re-exports `scroll_area::ScrollFade`, whose
        `class()` writes `dx-scroll-fade-*`)?"""
        seen = set() if seen is None else seen
        if module in seen:
            return False
        seen.add(module)
        if module not in self._prim_effects:
            paths = [f"primitives/src/{module}.rs"] + glob.glob(f"primitives/src/{module}/**/*.rs", recursive=True)
            code = "".join(strip_comments(read(p)) for p in paths if os.path.isfile(p))
            self._prim_effects[module] = (
                bool(EFFECTS_CLASSES.search(code)),
                sorted(set(re.findall(r"\bcrate::([a-z_0-9]+)", code)) - {module}),
            )
        direct, through = self._prim_effects[module]
        return direct or any(self.primitive_names_effects(m, seen) for m in through)


def installed_files(d):
    """Every file `dx components add` copies: the directory minus EXCLUDE, as (relative path, full path)."""
    out = []
    for dp, dns, fns in os.walk(d):
        rel_dir = os.path.relpath(dp, d)
        dns[:] = sorted(x for x in dns if os.path.normpath(os.path.join(rel_dir, x)) not in EXCLUDE)
        for fn in sorted(fns):
            rel = os.path.normpath(os.path.join(rel_dir, fn))
            if rel not in EXCLUDE:
                out.append((rel, os.path.join(dp, fn)))
    return out


def derive(ws, name, d, old):
    """The computed fields of one component's manifest, from its installed files."""
    rs_text, css_text = "", ""
    for rel, full in installed_files(d):
        if rel.endswith(".rs"):
            rs_text += read(full) + "\n"
        elif rel.endswith(".css"):
            css_text += read(full) + "\n"
    code = strip_comments(rs_text)
    bare = drop_cfg_test(strip_all(rs_text))

    # componentDependencies
    # `super::super::<name>` is a component's own file reaching `components::<name>` relatively (date_picker does);
    # it counts only when <name> is a component directory (`super::super::helper::` is some other item).
    sib = sibling_refs(bare, r"crate::components::")
    sib |= sibling_refs(bare, r"(?<![\w:])super::super::") & ws.components
    sib |= set(re.findall(r'asset!\(\s*"/src/components/([a-z_0-9]+)/', code))
    sib.discard(name)
    for s in sorted(sib - ws.components):
        die(f"{d}: names component `{s}`, which has no directory under {ROOT}")

    # cargoDependencies: external crates the installed Rust files name
    local = set(re.findall(r"\bmod\s+([a-z_0-9]+)", bare))
    used = {r for r in re.findall(r"(?<![\w:])use\s+(?:::)?([a-z_][a-z0-9_]*)", bare) if r not in NOT_EXTERNAL and r not in local}
    body = re.sub(r"(?<![\w:])use\s[^;]*;", " ", bare)
    known = set(ws.crate_names) - {"dioxus"}
    used |= {r for r in re.findall(r"(?<![\w:])([a-z_][a-z0-9_]*)::", body) if r in known and r not in local}
    used |= set(re.findall(r"(?<![\w:])(dioxus_[a-z_0-9]+)::", body))
    crates = set()
    for u in used:
        if u not in ws.crate_names:
            die(f"{d}: the installed Rust files use `{u}`, which no Cargo.toml of the workspace depends on, so it has no version to write")
        crates.add(ws.crate_names[u])
    crates.add(ALWAYS)

    features = sorted({ws.gated[m] for m in primitives_modules(code) if m in ws.gated})
    cargo = []
    for crate in sorted(crates, key=lambda c: (c != ALWAYS, c)):
        if crate in REGISTRY_CRATES:
            entry = {"name": crate, "git": ws.url}
            prev = next((c for c in old.get("cargoDependencies") or [] if isinstance(c, dict) and c.get("name") == crate), {})
            if prev.get("rev"):
                entry["rev"] = prev["rev"]
            if crate == "dioxus-primitives" and features:
                entry["features"] = features
        else:
            v = ws.lock_version(crate)
            entry = {"name": crate, "version": ("=" if crate in EXACT_ONLY else "") + v}
        cargo.append(entry)

    assets = []
    if os.path.isfile(f"{d}/style.css"):
        assets.append(THEME)
    needs_effects = bool(EFFECTS_CLASSES.search(code) or EFFECTS_CLASSES.search(strip_comments_css(css_text)))
    needs_effects = needs_effects or any(ws.primitive_names_effects(m) for m in primitives_modules(code))
    if needs_effects:
        assets.append(EFFECTS)

    out = {k: old[k] for k in HAND}
    out["exclude"] = list(EXCLUDE)
    out["cargoDependencies"] = cargo
    if sib:
        out["componentDependencies"] = [{"name": s, "git": ws.url} for s in sorted(sib)]
    out["globalAssets"] = assets
    return out


def strip_comments_css(text):
    return re.sub(r"/\*.*?\*/", " ", text, flags=re.S)


def j(v):
    return json.dumps(v, ensure_ascii=False)


def inline_or_expanded(key, items):
    """`"key": ["a", "b"]` when it fits in 80 columns, else one string per line (prettier's rule)."""
    line = f'  "{key}": [{", ".join(j(i) for i in items)}]'
    if len(line) <= WIDTH:
        return line
    return f'  "{key}": [\n' + ",\n".join("    " + j(i) for i in items) + "\n  ]"


def render(m):
    parts = []
    for k in KEY_ORDER:
        if k not in m:
            continue
        v = m[k]
        if k in ("name", "description"):
            parts.append(f'  "{k}": {j(v)}')
        elif k in ("authors", "exclude", "globalAssets"):
            parts.append(inline_or_expanded(k, v))
        elif k == "cargoDependencies":
            objs = []
            for c in v:
                fields = []
                for ck in ("name", "git", "version", "rev", "features"):
                    if ck in c:
                        fields.append(f'      "{ck}": {j(c[ck])}')
                objs.append("    {\n" + ",\n".join(fields) + "\n    }")
            parts.append('  "cargoDependencies": [\n' + ",\n".join(objs) + "\n  ]")
        elif k == "componentDependencies":
            parts.append('  "componentDependencies": [\n' + ",\n".join(
                f'    {{ "name": {j(c["name"])}, "git": {j(c["git"])} }}' for c in v) + "\n  ]")
    return "{\n" + ",\n".join(parts) + "\n}\n"


def render_root(old, dirs):
    members = [f"preview/src/components/{n}" for n in sorted(dirs) if n not in NOT_INSTALLABLE]
    return ('{\n  "name": %s,\n  "description": %s,\n  "members": [\n' % (j(old["name"]), j(old["description"]))
            + ",\n".join("    " + j(m) for m in members) + "\n  ]\n}\n")


def load_hand(path, name):
    try:
        old = json.loads(read(path))
    except ValueError as e:
        die(f"{path} is not valid JSON: {e}")
    unknown = set(old) - set(KEY_ORDER)
    if unknown:
        die(f"{path}: unknown key(s) {sorted(unknown)} (the generator writes only {list(KEY_ORDER)})")
    for k in HAND:
        if k not in old:
            die(f"{path}: hand-authored field `{k}` is missing (a new component starts as {{name, description, authors}})")
    if old["name"] != name:
        die(f"{path}: name is {old['name']!r}, want the directory name {name!r}")
    return old


def main(argv):
    check = argv == ["--check"]
    if argv and not check:
        die("usage: scripts/gen-component-manifests.sh [--check]")
    ws = Workspace()
    dirs = {}
    for f in sorted(glob.glob(f"{ROOT}/*/component.json")):
        dirs[os.path.basename(os.path.dirname(f))] = os.path.dirname(f)
    for p in sorted(glob.glob(f"{ROOT}/*/component.rs")):
        n = os.path.basename(os.path.dirname(p))
        if n not in dirs:
            die(f"{os.path.dirname(p)} has a component.rs but no component.json: start one with {{name, description, authors}}")

    want = {}
    for name, d in dirs.items():
        old = load_hand(f"{d}/component.json", name)
        want[f"{d}/component.json"] = render(derive(ws, name, d, old))
        want[f"{d}/mod.rs"] = MOD_RS
    try:
        root_old = json.loads(read("component.json"))
    except ValueError as e:
        die(f"component.json is not valid JSON: {e}")
    want["component.json"] = render_root(root_old, dirs)

    stale = []
    for path, text in want.items():
        have = read(path) if os.path.isfile(path) else None
        if have != text:
            stale.append((path, have, text))
    if check:
        if stale:
            print("gen-component-manifests: generated file(s) differ from the tree (run scripts/gen-component-manifests.sh and commit the result):", file=sys.stderr)
            for path, have, text in stale:
                diff = list(difflib.unified_diff((have or "").splitlines(), text.splitlines(), f"{path} (tree)", f"{path} (generated)", lineterm="", n=1))
                print("\n".join(diff[:40]) + ("\n  ..." if len(diff) > 40 else ""), file=sys.stderr)
            print(f"gen-component-manifests: {len(stale)} of {len(want)} file(s) stale", file=sys.stderr)
            return 1
        print(f"gen-component-manifests: OK -- {len(want)} generated files ({len(dirs)} component.json + mod.rs, the root members) are byte-identical to the sources' output.")
        return 0
    for path, _, text in stale:
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
    print(f"gen-component-manifests: wrote {len(stale)} of {len(want)} file(s)")
    return 0


sys.exit(main(sys.argv[1:]))
