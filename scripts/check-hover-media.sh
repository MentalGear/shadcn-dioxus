#!/usr/bin/env bash
#
# check-hover-media.sh
#
# Every `:hover` rule lives inside `@media (hover: hover) and (pointer: fine)`, unless its selector is on the
# ALLOW list below with a reason. Invisible to cargo and clippy, and with CI frozen (dev-docs/backlog.md row
# 33) nothing else would notice.
#
# THE CLASS (row 205, the owner's touch finding): a touch browser emulates `:hover` on the element it just
# tapped and keeps it there until the next tap elsewhere, so every hover fill, hover border and hover reveal
# SITS on the last-tapped control (a "sticky hover"): the menu row you tapped stays grey, the card you tapped
# stays lifted, the toast stack you tapped stays expanded. Browsers do not fix this (it is how the emulation
# works); the standard construction is to offer hover styling only to a device that can actually hover, via the
# `hover`/`pointer` media features. 114 rules in 39 sheets had it; the gate makes the next one impossible to add
# without the wrapper.
#
# HOW IT READS THE CSS: every `*.css` under `preview/src` and `preview/assets` (node_modules and build output
# skipped), comments stripped (newlines kept, so line numbers survive). The parser walks blocks with their
# at-rule chain. For each style rule, the selector list is split on top-level commas and each ENTRY is classed:
#   positive hover -- a `:hover` that is not inside a `:not(...)`;
#   negated hover  -- `:hover` only inside `:not(...)` (`:not(:hover)` matches EVERYTHING on a touch device, so
#                     it is the rest state, not a hover rule: wrapping it would invert it);
#   neither.
# Four checks, all by construction (instances never need to be named):
#   1. a rule with a positive-hover entry must have an ancestor `@media` that is exactly the hover query
#      (`(hover: hover)` AND `(pointer: fine)`; `any-hover`, an `or`, a `not` or a comma list do not count);
#   2. the converse: a rule INSIDE the hover media may hold only positive-hover entries. A focus, open-state or
#      `:not(:hover)` entry in there would vanish on touch, which is how a split goes wrong. Write the
#      non-hover half as its own rule OUTSIDE the media, keeping source order;
#   3. no parenthesised selector list (`:is`, `:where`, `:has`) may mix a positive-hover argument with a
#      non-hover one (`:is(:hover, :focus-visible)`): split it, check 1 cannot see inside it;
#   4. a negated-hover entry is neither wrapped nor flagged by 1, so it must be on the ALLOW list with a reason.
# Nested CSS (a `&` selector, a `{` inside a style rule) fails the run: the walker does not follow it, so it
# could hide a `:hover`. A fail-closed parse guard compares the raw `:hover` token count with the tokens the
# walker attributed to a selector, and fails on any difference or on zero rules parsed.
#
# CSS INSIDE RUST STRINGS: the same wrapper is required for a `:hover` in any `preview/src` or
# `primitives/src` `.rs` file (demo `style` blocks). It is satisfied by a hover-media line above it whose brace
# is still open; any other `:hover` (a JS `matches(':hover')`) needs an RS_ALLOW entry with a reason.
#
# ALLOW format: (stylesheet path, fnmatch pattern on the whitespace-normalised selector ENTRY, reason). An entry
# that matches nothing fails the run as stale, so the list cannot rot into blanket permission. The PENDING block
# is TEMPORARY: sheets other lanes were editing when this gate landed, so their rules could not be wrapped
# without a merge conflict. Wrap them after those lanes merge and delete the entry; it fails as stale the
# moment the sheet no longer needs it.
#
# Usage: scripts/check-hover-media.sh
# Exit 0: clean. Exit 1: a violation, with file:line detail on stderr.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 - <<'PYEOF'
import fnmatch
import re
import sys
from pathlib import Path

ALLOW = [
    ("preview/assets/main.css", ":not(:hover)",
     "the `@supports (scrollbar-width: auto)` REST state: scrollbars are transparent until the element is hovered. "
     "`:not(:hover)` matches everything on a touch device, which is the intent there (an overlay scrollbar), and "
     "its `:hover` partner is wrapped"),
    ("preview/src/components/toast/style.css", ".dx-toast-container:not(:hover, :focus-within)*",
     "the collapsed-stack entrance animation: it must apply on touch (nothing can hover, so the stack is always "
     "collapsed unless focus is inside), so it stays outside the media; the expanded state is the `:hover` and "
     "`:focus-within` rules, whose hover half is wrapped"),
]

# TEMPORARY (marker: pending-lane-merge). Remove each entry once the sheet is wrapped.
PENDING = [
    (f"preview/src/components/{c}/style.css", "*",
     "pending-lane-merge: lane X edits this sheet concurrently; wrapped after merge")
    for c in ("sheet", "drawer", "calendar", "combobox", "popover", "select", "dropdown_menu")
]
ALLOW += PENDING

# (rust path, substring on the line, reason) for a `:hover` in a .rs file that is not CSS.
RS_ALLOW = [
    ("primitives/src/combobox/components/option.rs", "matches(':hover')",
     "JavaScript `Element.matches(':hover')` (is the pointer over this option right now), not a stylesheet rule"),
]

def strip_comments(src: str) -> str:
    return re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), src, flags=re.S)

def norm(s: str) -> str:
    return re.sub(r"\s+", " ", s.strip())

def is_hover_media(params: str) -> bool:
    p = norm(params.lower())
    if "," in p or " or " in p or "not " in p or "any-hover" in p:
        return False
    return bool(re.search(r"\(\s*hover\s*:\s*hover\s*\)", p) and re.search(r"\(\s*pointer\s*:\s*fine\s*\)", p))

def split_top(sel):
    out, depth, cur, q = [], 0, [], None
    for c in sel:
        if q:
            cur.append(c)
            if c == q:
                q = None
        elif c in "\"'":
            q = c
            cur.append(c)
        elif c in "([":
            depth += 1
            cur.append(c)
        elif c in ")]":
            depth -= 1
            cur.append(c)
        elif c == "," and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(c)
    out.append("".join(cur))
    return out

def hover_kind(entry):
    pos = neg = False
    for m in re.finditer(r":hover(?![\w-])", entry):
        stack = []
        for j, c in enumerate(entry[: m.start()]):
            if c == "(":
                k = j
                while k > 0 and (entry[k - 1].isalnum() or entry[k - 1] in "-_:"):
                    k -= 1
                stack.append(entry[k:j])
            elif c == ")" and stack:
                stack.pop()
        if any(s.endswith(":not") for s in stack):
            neg = True
        else:
            pos = True
    return "pos" if pos else ("neg" if neg else None)

def list_mix(entry):
    for m in re.finditer(r":(is|where|has|matches)\(", entry):
        j, depth = m.end(), 1
        while depth and j < len(entry):
            depth += {"(": 1, ")": -1}.get(entry[j], 0)
            j += 1
        args = split_top(entry[m.end() : j - 1])
        if len(args) > 1:
            kinds = [hover_kind(a) == "pos" for a in args]
            if any(kinds) and not all(kinds):
                return True
    return False

def skip_str_find(src, i, chars):
    """index of the first char in `chars` at/after i that is outside quotes and parentheses, or -1."""
    q, par = None, 0
    while i < len(src):
        c = src[i]
        if q:
            if c == q:
                q = None
        elif c in "\"'":
            q = c
        elif c == "(":
            par += 1
        elif c == ")":
            par -= 1
        elif c in chars and par <= 0:
            return i
        i += 1
    return -1

def match_brace(src, i):
    depth, q = 0, None
    while i < len(src):
        c = src[i]
        if q:
            if c == q:
                q = None
        elif c in "\"'":
            q = c
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError("unbalanced braces")

errors = []
used = [0] * len(ALLOW)
counters = {"rules": 0, "wrapped": 0, "allowed": 0, "tokens": 0, "attributed": 0}

def allow_idx(path, entry):
    n = norm(entry)
    for k, (p, pat, _) in enumerate(ALLOW):
        if p == path and fnmatch.fnmatchcase(n, pat):
            return k
    return None

def walk(path, src, start, end, hover):
    i = start
    while i < end:
        while i < end and src[i].isspace():
            i += 1
        if i >= end:
            break
        stop = skip_str_find(src, i, "{;}")
        if stop < 0 or stop >= end:
            break
        if src[stop] != "{":
            i = stop + 1
            continue
        close = match_brace(src, stop)
        prelude, line = src[i:stop], src.count("\n", 0, i) + 1
        line += len(prelude) - len(prelude.lstrip("\n")) if False else 0
        if prelude.lstrip().startswith("@"):
            at = prelude.strip()
            counters["attributed"] += len(re.findall(r":hover(?![\w-])", at))
            name = re.match(r"@[\w-]+", at).group(0)
            if name in ("@media", "@supports", "@container", "@layer"):
                inner = hover or (name == "@media" and is_hover_media(at[len("@media"):]))
                walk(path, src, stop + 1, close, inner)
            i = close + 1
            continue
        body = src[stop + 1 : close]
        counters["rules"] += 1
        if "{" in body:
            errors.append(f"{path}:{line}: a `{{` inside the style rule `{norm(prelude)[:60]}` (nested CSS): this "
                          f"script does not follow it, so it could hide a `:hover`; write the rules flat")
        if "&" in prelude:
            errors.append(f"{path}:{line}: nesting selector `&` in `{norm(prelude)[:60]}`: write the rules flat")
        entries = split_top(prelude)
        for e in entries:
            counters["attributed"] += len(re.findall(r":hover(?![\w-])", e))
        kinds = [hover_kind(e) for e in entries]
        for e, k in zip(entries, kinds):
            en = norm(e)
            if hover:
                if k != "pos":
                    errors.append(f"{path}:{line}: `{en[:90]}` sits inside the hover media but is not a `:hover` "
                                  f"selector, so it would not apply on touch; move it to its own rule outside the media")
                else:
                    counters["wrapped"] += 1
            if k == "pos":
                if list_mix(e):
                    errors.append(f"{path}:{line}: `{en[:90]}` mixes `:hover` with another state inside one "
                                  f"`:is()`/`:where()`/`:has()` list; split it into a hover rule (in the media) and "
                                  f"a rule for the other states")
                if not hover:
                    idx = allow_idx(path, e)
                    if idx is None:
                        errors.append(f"{path}:{line}: `{en[:90]}` has a `:hover` outside "
                                      f"`@media (hover: hover) and (pointer: fine)` (sticky hover on touch): wrap it, "
                                      f"keeping any focus/open-state selector in its own rule outside the media")
                    else:
                        used[idx] += 1
                        counters["allowed"] += 1
            elif k == "neg" and not hover:
                idx = allow_idx(path, e)
                if idx is None:
                    errors.append(f"{path}:{line}: `{en[:90]}` is a NEGATED `:hover` (the rest state, it matches "
                                  f"everything on touch): it must not be wrapped, so list it in ALLOW with a reason "
                                  f"in scripts/check-hover-media.sh")
                else:
                    used[idx] += 1
        i = close + 1

stylesheets = []
for root in ("preview/src", "preview/assets"):
    for p in sorted(Path(root).rglob("*.css")):
        if "node_modules" in p.parts or "target" in p.parts:
            continue
        stylesheets.append(p)

for css in stylesheets:
    src = strip_comments(css.read_text())
    counters["tokens"] += len(re.findall(r":hover(?![\w-])", src))
    try:
        walk(css.as_posix(), src, 0, len(src), False)
    except ValueError as e:
        errors.append(f"{css}: {e}")

# CSS inside Rust strings
rs_used = [0] * len(RS_ALLOW)
n_rs = 0
for root in ("preview/src", "primitives/src"):
    for p in sorted(Path(root).rglob("*.rs")):
        if "target" in p.parts:
            continue
        lines = p.read_text().split("\n")
        for ln, text in enumerate(lines):
            code = re.sub(r"//.*$", "", text) if text.lstrip().startswith("//") else text
            if text.lstrip().startswith("//") or not re.search(r":hover(?![\w-])", code):
                continue
            n_rs += 1
            k = next((k for k, (rp, sub, _) in enumerate(RS_ALLOW) if rp == p.as_posix() and sub in text), None)
            if k is not None:
                rs_used[k] += 1
                continue
            window = "\n".join(lines[max(0, ln - 80) : ln + 1]).replace("{{", "{").replace("}}", "}")
            ok = False
            for m in re.finditer(r"@media[^{]*\{", window):
                if is_hover_media(m.group(0)[6:-1]):
                    tail = window[m.end() - 1 :]
                    if tail.count("{") > tail.count("}"):
                        ok = True
            if not ok:
                errors.append(f"{p}:{ln + 1}: `:hover` in a Rust string that is not inside "
                              f"`@media (hover: hover) and (pointer: fine)` (sticky hover on touch)")

for (path, pat, reason), k in zip(ALLOW, used):
    if not reason.strip():
        errors.append(f"allowlist entry ({path}, {pat}) has no reason")
    if not k:
        errors.append(f"stale allowlist entry ({path}, {pat}): no `:hover` selector needs it any more -- remove it "
                      f"from scripts/check-hover-media.sh")
for (path, sub, reason), k in zip(RS_ALLOW, rs_used):
    if not reason.strip():
        errors.append(f"RS_ALLOW entry ({path}, {sub}) has no reason")
    if not k:
        errors.append(f"stale RS_ALLOW entry ({path}, {sub}): no such line any more -- remove it")

if not stylesheets or not counters["rules"]:
    errors.append("no stylesheet or no rule was parsed -- the scan in this script is broken")
if counters["tokens"] != counters["attributed"]:
    errors.append(f"parse guard: {counters['tokens']} `:hover` tokens in the stylesheets but the walker attributed "
                  f"{counters['attributed']} to a selector -- a selector was skipped; fix the parse, do not skip them")

if errors:
    for e in errors:
        print(f"check-hover-media: {e}", file=sys.stderr)
    print(f"check-hover-media: FAILED -- {len(errors)} problem(s)", file=sys.stderr)
    sys.exit(1)
pending = sum(1 for (p, pat, r), k in zip(ALLOW, used) if r.startswith("pending-lane-merge") for _ in range(k))
print(
    f"check-hover-media: OK -- {counters['tokens']} `:hover` tokens in {len(stylesheets)} stylesheets, "
    f"{counters['wrapped']} selectors inside the hover media, {counters['allowed']} allowlisted "
    f"({pending} pending-lane-merge), {n_rs} in Rust files ({len(RS_ALLOW)} allowed)"
)
PYEOF
