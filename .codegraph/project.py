# The code graph explorer's project config for shadcn-dioxus (row 486). The tool is MentalGear/code-analyser, run
# through scripts/codegraph.sh; its README documents every name below. What is specific to this repo lives here:
# the comment tokenizer the gates share, the build lock, the TS roots, the component mapping, the themed component
# nodes, the pairing of a themed component with its primitive and the Playwright spec edges.
import collections, json, os, re

# the comment gates' own tokenizer, so the metrics and the gates read comments the same way
TOKENIZER = "scripts/comment-scan.py"
# the SCIP run holds the one build lock, so it waits behind a gate or a build
LOCK = ["scripts/lane-lock.sh", "--label", "codegraph-scip", "--"]
# the files the hooks below read as text: component.json marks a themed component, its .rs files hold the pairing paths
KEY_INPUTS = ["preview/src/components/*/component.json", "preview/src/components/*.rs"]
# the libraries this repo ports or cites, so N1 does not flag the names a comment quotes from them
UPSTREAM = r"Base UI|Radix|radix-ui|Sonner|shadcn|React|react-[a-z-]+|MUI|Floating UI|Headless|Recharts|Vaul|cmdk|[Ee]mbla"
TS_ROOTS = ["primitives/src/ts", "playwright", "primitives/ts-test"]
PAGE = {
    "title": "shadcn-dioxus",
    "crates": ["preview", "primitives", "dioxus-attributes", "cli", "test-harness", "playwright"],
    "pick_crates": ["preview", "primitives"],
    "suffix": {"preview": "pv", "primitives": "pr"},
    "pick": "select",
    "sources": ", spec-name match (tests)",
}
# what the smoke test (the tool's smoke.js) expects of this repo's page, beyond its generic checks
SMOKE = {
    "playwright": "playwright/node_modules/playwright",
    "launch_options": ["playwright/sandbox-chromium.cjs", "sandboxLaunchOptions"],
    "filter": "primitives/src/select",
    "seeds": {"select": ["preview/src/components/select/mod.rs", "preview/src/components/select/component.rs",
                         "primitives/src/select/mod.rs"]},
    # a component without a same-named primitive pairs through its sources
    "pairs": {"preview:area_chart": ["primitives:chart"]},
    # macro and module references resolve to an owner (derive, Props builder, #[component], module)
    "meta_min": {"resolved_by_owner": 5000, "resolved_by_module": 500},
    "meta_max": {"unresolved_workspace_refs": 0},
    "edges_into": ["preview/src/components/button/mod.rs"],
    "doc_lines": ["primitives/src/lib.rs"],
    # an upstream citation (cli/src/add.rs) and Web names (InvalidStateError in lib.rs, dialog.rs)
    "not_stale": ["cli/src/add.rs", "primitives/src/lib.rs", "primitives/src/dialog.rs"],
    "totals_min": {"stale": 1, "js": 1, "jscode": 1, "dup": 1},
    "totals_max": {"stale": 39},
    "items_min": 1001,
    # the lib.rs `pub mod accordion;` line is an item span of one code line
    "items": [["primitives/src/lib.rs", "accordion", {"code": 1, "pub": 1}]],
}

COMPONENTS = "preview/src/components"


def _themed(repo):
    """Themed components: a component.json that depends on dioxus-primitives. {name: its directory}."""
    out = {}
    for d in sorted(os.listdir(os.path.join(repo, COMPONENTS))):
        cj = os.path.join(repo, COMPONENTS, d, "component.json")
        if not os.path.isfile(cj):
            continue
        deps = [x.get("name") for x in json.load(open(cj)).get("cargoDependencies", [])]
        if "dioxus-primitives" in deps:
            out[d] = os.path.dirname(cj)
    return out


_MOD = re.compile(r"^\s*(?:pub\s+)?mod\s+(\w+)\s*;", re.M)
_SPEC = re.compile(r"playwright/([a-z0-9_-]+)\.spec\.ts$")


def extend(repo, files, edges):
    """A themed component's `mod.rs` holds no definitions, so the index has no document for it: it is a node with an
    edge to each module it declares (`mod component;`). A Playwright spec has a `test` edge to the component of its
    name (select.spec.ts -> select, native-select.spec.ts -> native_select)."""
    for d in _themed(repo):
        p = "%s/%s/mod.rs" % (COMPONENTS, d)
        if os.path.isfile(os.path.join(repo, p)):
            files.add(p)
            for m in _MOD.finditer(open(os.path.join(repo, p), errors="replace").read()):
                q = "%s/%s/%s.rs" % (COMPONENTS, d, m.group(1))
                if os.path.isfile(os.path.join(repo, q)):
                    files.add(q)
                    edges[(p, q, "rs")] += 1
    names = {f.split("/")[3] for f in files if f.startswith(COMPONENTS + "/") and len(f.split("/")) > 4}
    for f in sorted(files):
        m = _SPEC.match(f)
        if m:
            n = m.group(1).replace("-", "_")
            cand = [k for k in names if n == k or n.startswith(k + "_")]
            if cand:
                k = max(cand, key=len)
                for leaf in ("mod.rs", "component.rs"):
                    t = "%s/%s/%s" % (COMPONENTS, k, leaf)
                    if t in files:
                        edges[(f, t, "test")] += 1
                        break


def component(repo, path):
    """(comp_id, crate, name): a preview component directory, a primitives module, the Playwright specs and support."""
    parts = path.split("/")
    c = parts[0]
    if c == "preview":
        if parts[1:3] == ["src", "components"] and len(parts) > 4:
            return "preview:" + parts[3], "preview", parts[3]
        if len(parts) > 2 and parts[1] == "src":
            s = os.path.splitext(parts[2])[0]
            return "preview:" + s, "preview", "·" + s if s != "components" else "·components"
        return "preview:·" + os.path.splitext(parts[1])[0], "preview", "·" + os.path.splitext(parts[1])[0]
    if c == "primitives":
        if parts[1] == "src" and len(parts) > 2:
            s = os.path.splitext(parts[2])[0]
            if parts[2] == "ts" or parts[2] == "js":
                s = parts[2] + "/" + (os.path.splitext(parts[3])[0] if len(parts) > 4 else "root")
            return "primitives:" + s, "primitives", s
        return "primitives:·" + parts[1], "primitives", "·" + parts[1]
    if c == "playwright":
        kind = "specs" if path.endswith(".spec.ts") else "support"
        return "playwright:" + kind, "playwright", kind
    s = parts[1] if len(parts) > 1 else "·root"
    return c + ":" + os.path.splitext(s)[0], c, os.path.splitext(s)[0]


# Pairing: a themed preview component and the primitive it wraps, read from the `dioxus_primitives::<module>` paths in
# that component's own sources. A module named like the component wins. Otherwise the module referenced most often wins,
# if it is unique and is not a helper (a module with no preview component of its own that three or more components use,
# or any module ten or more components use, such as `direction`).
_USE = re.compile(r"dioxus_primitives::(\w+)")
_BRACE = re.compile(r"dioxus_primitives::\{([^}]*)\}")


def pairs(repo, comps):
    cdirs = _themed(repo)
    refs = {}
    for d, dd in cdirs.items():
        cnt = collections.Counter()
        for root, _, fs in os.walk(dd):
            for f in fs:
                if not f.endswith(".rs"):
                    continue
                t = open(os.path.join(root, f), errors="replace").read()
                cnt.update(m.group(1) for m in _USE.finditer(t))
                for m in _BRACE.finditer(t):
                    cnt.update(re.findall(r"(?:^|,)\s*(\w+)", m.group(1)))
        refs[d] = cnt
    prim = {c["name"]: c["id"] for c in comps.values()
            if c["crate"] == "primitives" and not c["name"].startswith("·") and "/" not in c["name"]}
    referrers = collections.Counter(m for d, c in refs.items() for m in c if m != d)

    def helper(m):
        return referrers[m] >= 10 or (m not in cdirs and referrers[m] >= 3)

    out = []
    for d, cnt in sorted(refs.items()):
        if d in prim:
            out.append(("preview:" + d, prim[d]))
            continue
        c = [(n, m) for m, n in cnt.items() if m in prim and not helper(m)]
        if c:
            top = max(n for n, _ in c)
            best = [m for n, m in c if n == top]
            if len(best) == 1:
                out.append(("preview:" + d, prim[best[0]]))
    return out
