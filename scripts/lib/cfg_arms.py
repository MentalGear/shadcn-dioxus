"""The Blitz arm of a Rust file, for the gates that exempt it (backlog row 509).

`check-native-slots` and `check-hot-events` forbid an rsx write of a native slot and an rsx hot-event handler because a page
has better routes (an owner write over the bridge, `listen`/`sample`). A renderer with no page (Blitz) has neither, so on it
the rsx write and the rsx handler are the only routes. The primitives' renderer switch is one cfg, `feature = "native"`
(row 511: the browser-engine arm is the default and Blitz is the `native` opt-in; `check-cfg-axis` bans every other spelling
in the primitives), so the exemption is exactly the items under `#[cfg(feature = "native")]` and nothing under
`not(feature = "native")` or no cfg.

    from cfg_arms import native_ranges
    skip = native_ranges(code, keep)        # [(start, end)] offsets into `code`; `keep` is the same text with strings kept
"""
import re

ATTR = re.compile(r'#\[cfg\(\s*feature\s*=\s*"native"\s*\)\]')


def _close(code, open_idx):
    """Index of the bracket that closes the one at `open_idx` (the end of the text when unbalanced)."""
    pairs = {"{": "}", "(": ")", "[": "]"}
    stack = []
    for k in range(open_idx, len(code)):
        ch = code[k]
        if ch in pairs:
            stack.append(pairs[ch])
        elif stack and ch == stack[-1]:
            stack.pop()
            if not stack:
                return k
    return len(code)


def native_ranges(code, keep):
    """Ranges of the items under `#[cfg(feature = "native")]`: from the attribute to the close of the item's first block,
    or to its `;` (a `let`, a `use`, a field)."""
    out = []
    for m in ATTR.finditer(keep):
        k = m.end()
        # further attributes of the same item
        while code[k:].lstrip().startswith("#["):
            k = _close(code, code.index("#[", k) + 1) + 1
        depth = 0
        end = len(code)
        while k < len(code):
            ch = code[k]
            if ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            elif ch == "{" and depth == 0:
                end = _close(code, k)
                break
            elif ch == ";" and depth == 0:
                end = k
                break
            k += 1
        out.append((m.start(), end))
    return out
