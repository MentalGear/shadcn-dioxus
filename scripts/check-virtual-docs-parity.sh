#!/usr/bin/env bash
#
# check-virtual-docs-parity.sh
#
# `VirtualList` and the three grids (`VirtualGrid`, `VirtualWrapGrid`, `VirtualJustifiedGrid`) are one
# engine (`primitives/src/virtual/`, backlog row 309): the same fade, the same overscan, the same pinned
# focus, the same scroll chaining, the same find-in-page limits. Their documentation said so in two
# different wordings that drifted (a fade note on one page and not the other, `buffer` described three
# ways). The parity is made a property of the sources, not of a reviewer's memory:
#
#   1. Docs pages. Every section the two components share sits between
#        <!-- shared:virtual-engine:NAME -->  ...  <!-- /shared:virtual-engine:NAME -->
#      in BOTH `preview/src/components/virtual_list/docs.md` and `.../virtual_grid/docs.md`, and the text
#      between the markers must be byte-identical. Fails on a block that differs, a block present in only
#      one file, a marker that is not closed (or closed twice), and when neither file has a block at all.
#      A new shared feature gets a new block in both files; a feature only one component has stays outside.
#      `preview/src/components/table/docs.md` (the data body `TableRows`, row 343) may carry any of the same blocks
#      (it carries `modes`): each one it has must equal the list's text, and it need not carry the others.
#
#   2. Prop docs. A prop name that `VirtualListProps` shares with any grid props struct
#      (`VirtualGridProps`, `VirtualWrapGridProps`, `VirtualJustifiedGridProps`: today `count`,
#      `render_item`, `buffer`, `fade`, `attributes`) must carry the same `///` text in all of them
#      (whitespace collapsed), because the generated Props reference prints each one as that
#      component's documentation of the prop. A new field added to both is held to the same rule.
#
# Scope: the two docs.md files and primitives/src/virtual_list.rs + primitives/src/virtual_grid.rs. No network.
# Overrides for self-tests: VDP_LIST_DOCS, VDP_GRID_DOCS, VDP_TABLE_DOCS, VDP_LIST_RS, VDP_GRID_RS (file paths).
# Exit status: 0 when in parity, 1 on a difference, 2 when a source cannot be read or parsed.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - <<'PYEOF'
import os
import re
import sys

list_docs = os.environ.get("VDP_LIST_DOCS", "preview/src/components/virtual_list/docs.md")
grid_docs = os.environ.get("VDP_GRID_DOCS", "preview/src/components/virtual_grid/docs.md")
table_docs = os.environ.get("VDP_TABLE_DOCS", "preview/src/components/table/docs.md")
list_rs = os.environ.get("VDP_LIST_RS", "primitives/src/virtual_list.rs")
grid_rs = os.environ.get("VDP_GRID_RS", "primitives/src/virtual_grid.rs")

problems = []

def read(path):
    try:
        return open(path, encoding="utf-8").read()
    except OSError as err:
        print(f"check-virtual-docs-parity: cannot read {path}: {err}", file=sys.stderr)
        sys.exit(2)

OPEN = re.compile(r"<!--\s*shared:virtual-engine:([a-z0-9-]+)\s*-->")
CLOSE = re.compile(r"<!--\s*/shared:virtual-engine:([a-z0-9-]+)\s*-->")

def blocks(path, text):
    """name -> text between its markers; unbalanced or repeated markers are problems."""
    found = {}
    pos = 0
    while True:
        m = OPEN.search(text, pos)
        if not m:
            break
        name = m.group(1)
        c = CLOSE.search(text, m.end())
        nxt = OPEN.search(text, m.end())
        if not c or (nxt and nxt.start() < c.start()):
            problems.append(f"{path}: shared block `{name}` is opened but not closed")
            pos = m.end()
            continue
        if c.group(1) != name:
            problems.append(f"{path}: shared block `{name}` is closed as `{c.group(1)}`")
        if name in found:
            problems.append(f"{path}: shared block `{name}` appears twice")
        found[name] = text[m.end():c.start()].strip("\n")
        pos = c.end()
    for c in CLOSE.finditer(text):
        if c.group(1) not in found:
            problems.append(f"{path}: closing marker for `{c.group(1)}` has no opening marker")
    return found

lb = blocks(list_docs, read(list_docs))
gb = blocks(grid_docs, read(grid_docs))
if not lb and not gb:
    problems.append("neither docs.md has a `shared:virtual-engine` block: the shared sections must be marked")
for name in sorted(set(lb) | set(gb)):
    if name not in lb:
        problems.append(f"shared block `{name}` is in {grid_docs} but not in {list_docs}")
    elif name not in gb:
        problems.append(f"shared block `{name}` is in {list_docs} but not in {grid_docs}")
    elif lb[name] != gb[name]:
        a, b = lb[name].splitlines(), gb[name].splitlines()
        first = next((i for i in range(min(len(a), len(b))) if a[i] != b[i]), min(len(a), len(b)))
        problems.append(
            f"shared block `{name}` differs between the two docs.md files (first difference at line {first + 1} of the block):\n"
            f"    {list_docs}: {a[first] if first < len(a) else '<end of block>'}\n"
            f"    {grid_docs}: {b[first] if first < len(b) else '<end of block>'}"
        )

tb = blocks(table_docs, read(table_docs))
for name in sorted(tb):
    if name not in lb:
        problems.append(f"shared block `{name}` is in {table_docs} but not in {list_docs}")
    elif tb[name] != lb[name]:
        problems.append(f"shared block `{name}` in {table_docs} differs from {list_docs}: copy the list's text")

def props(path, text, struct):
    """field name -> collapsed `///` doc of `pub struct <struct> { ... }`."""
    m = re.search(r"pub struct " + struct + r" \{\n(.*?)\n\}\n", text, re.S)
    if not m:
        print(f"check-virtual-docs-parity: `pub struct {struct}` not found in {path}", file=sys.stderr)
        sys.exit(2)
    out = {}
    doc = []
    for line in m.group(1).split("\n"):
        t = line.strip()
        if t.startswith("///"):
            doc.append(t[3:].strip())
        elif t.startswith("#["):
            continue
        else:
            f = re.match(r"pub (\w+):", t)
            if f:
                out[f.group(1)] = " ".join(" ".join(doc).split())
            doc = []
    return out

list_text, grid_text = read(list_rs), read(grid_rs)
list_props = props(list_rs, list_text, "VirtualListProps")
grid_structs = ["VirtualGridProps", "VirtualWrapGridProps", "VirtualJustifiedGridProps"]
shared_seen = set()
for struct in grid_structs:
    grid_props = props(grid_rs, grid_text, struct)
    for name, doc in grid_props.items():
        if name not in list_props:
            continue
        shared_seen.add(name)
        if not doc:
            problems.append(f"{struct}::{name} has no `///` doc (VirtualListProps::{name} has one)")
        elif doc != list_props[name]:
            problems.append(
                f"the `///` doc of `{name}` differs between VirtualListProps and {struct}:\n"
                f"    VirtualListProps: {list_props[name]}\n"
                f"    {struct}: {doc}"
            )
if not {"fade", "buffer"} <= shared_seen:
    problems.append(f"expected the shared props `fade` and `buffer` to be found in both components (found: {sorted(shared_seen)}): the parser lost them")

if problems:
    for p in problems:
        print(p)
    print(
        "\ncheck-virtual-docs-parity: VirtualList and the grids share one engine, so their shared docs are one text. "
        "Edit the block / prop doc in BOTH places (see this script's header)."
    )
    sys.exit(1)

print(f"check-virtual-docs-parity: OK ({len(lb)} shared docs blocks, {len(shared_seen)} shared props: {', '.join(sorted(shared_seen))})")
PYEOF
