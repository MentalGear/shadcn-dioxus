#!/usr/bin/env bash
#
# check-render-keys.sh  (backlog row 361)
#
# A windowed renderer says who is at an index ONCE: the optional `item_key` prop, made into a DOM key by
# `render_key(item_key, idx)` (primitives/src/virtual/identity.rs). A hand-written positional key --
# `key: "{idx}"`, `"{row.index()}"`, `"{item.key()}"` -- ties the DOM node, and with it focus and any state
# inside the row, to the slot, so after a sort or a reorder focus stays on a slot while the item moves
# (row 361; the same class VirtualList, TableRows and the grids each had). By construction no renderer
# spells a key itself.
#
# Scope: every rsx `key: "..."` in the windowed renderers (primitives/src/virtual_list.rs, virtual_grid.rs,
# table_rows.rs and primitives/src/virtual/*.rs). Each must go through `render_key(`, or be a key that is
# deliberately not an item (a column, a chunk of find placeholders) and say so with a
# `key-ok: <reason>` comment on the line or in the comment lines directly above it, or sit on the reasoned
# ALLOW list below. A `key-ok` marker with no key under it, and a stale ALLOW entry, fail. The helper must
# still exist.
#
# Usage: scripts/check-render-keys.sh [FILE ...]   (explicit files are for self-tests: no ALLOW, no helper check)
# Exit status: 0 when clean.

set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - "$@" <<'PYEOF'
import glob, re, sys

explicit = sys.argv[1:]
HELPER = "primitives/src/virtual/identity.rs"

# (file, substring of the key line, reason).
ALLOW = [
    ("primitives/src/virtual/cv_chunks.rs", 'key: "{chunk.key}"',
     "a content-visibility chunk is a block of positions the browser skips together, not an item; "
     "the rows inside it are keyed by the caller through render_key"),
]

KEY = re.compile(r'\bkey:\s*"([^"]*)"')


def scan(src):
    """(line number, key string) of every rsx key that is neither render_key nor marked; plus orphan markers."""
    lines = src.split("\n")
    bad, orphans = [], []
    marker_pending = []  # marker line numbers awaiting a key line
    for n, line in enumerate(lines, 1):
        stripped = line.strip()
        is_comment = stripped.startswith("//")
        if is_comment:
            if "key-ok:" in stripped:
                if not re.search(r"key-ok:\s*\S", stripped):
                    orphans.append((n, "key-ok marker without a reason"))
                else:
                    marker_pending.append(n)
            continue
        code = line.split("//", 1)[0] if "key-ok:" not in line else line
        m = KEY.search(code)
        if not m:
            if stripped and marker_pending and not stripped.startswith("#["):
                # a marker covers the key on its own line or the next code line only
                orphans.extend((p, "key-ok marker with no key under it") for p in marker_pending)
                marker_pending = []
            continue
        inline = "key-ok:" in line and re.search(r"key-ok:\s*\S", line)
        if marker_pending:
            marker_pending = []
            ok_marker = True
        else:
            ok_marker = False
        if "render_key(" in m.group(1) or inline or ok_marker:
            continue
        bad.append((n, m.group(1)))
    orphans.extend((p, "key-ok marker with no key under it") for p in marker_pending)
    return bad, orphans


# Self-test: the scanner sees positional keys, accepts the helper and the marker, rejects stale markers.
assert scan('rsx! { div { key: "{idx}" } }')[0] == [(1, "{idx}")]
assert scan('div { key: "{render_key(item_key, idx)}" }')[0] == []
assert scan('div { key: "{item.key()}", }')[0] == [(1, "{item.key()}")]
assert scan('// key-ok: a column\ndiv { key: "{column}" }') == ([], [])
assert scan('div { key: "{column}" } // key-ok: a column') == ([], [])
assert scan('/// key: "{idx}" in a doc')[0] == []
assert scan('// key-ok: nothing here\nlet x = 1;')[1] != []
assert scan('// key-ok:\ndiv { key: "{i}" }')[0] != []

if explicit:
    files = explicit
else:
    files = sorted(
        ["primitives/src/virtual_list.rs", "primitives/src/virtual_grid.rs", "primitives/src/table_rows.rs"]
        + glob.glob("primitives/src/virtual/*.rs")
    )
failures, used = [], set()
for path in files:
    src = open(path, encoding="utf-8").read()
    # tests inside a file spell keys freely
    src = src.split("#[cfg(test)]", 1)[0]
    bad, orphans = scan(src)
    for n, key in bad:
        allowed = None if explicit else next(
            (a for a in ALLOW if a[0] == path and a[1] in f'key: "{key}"'), None)
        if allowed:
            used.add(allowed)
            continue
        failures.append(f'{path}:{n}: positional key "{key}": use render_key(item_key, idx), or say why this is not an item with `// key-ok: <reason>`')
    for n, why in orphans:
        failures.append(f"{path}:{n}: {why}")

if not explicit:
    for a in ALLOW:
        if a not in used:
            failures.append(f"stale ALLOW entry: {a[0]} {a[1]}")
    helper = open(HELPER, encoding="utf-8").read()
    if "pub(crate) fn render_key(" not in helper:
        failures.append(f"{HELPER}: render_key helper is gone")

if failures:
    print("check-render-keys: FAIL")
    print("\n".join(failures))
    sys.exit(1)
print(f"check-render-keys: OK ({len(files)} files)")
PYEOF
