"""The one reader of backlog-row front matter for the gates (row 444, phase 0.1).

A backlog row is `dev-docs/backlog/N-<slug>.md` while it is open and `dev-docs/backlog/done/N-<slug>.md` once it is
closed. Its front matter carries the grants the gates read:

    js-grant: js_file=360 (inferred ceiling: ...)     what a RAISE of a `check-js-volume` count may reach, per column
    js-fix:   js_file=20                              the same, for a bug fix inside legacy JS (row 444 item 0.2)
    adds:     [otp_filter]                            behaviour kinds the row may add (item 0.3)

`none`, an empty value and a value without any `column=number` mean "no grant". A grant is read from the tree of a
GIT REVISION (`git show <rev>:<path>`), never from the working tree: a lane that writes a grant into its own diff has
not been granted anything, because the grant is the owner's row at the merge base.

    from rows import row_at, parse_columns
    row = row_at(root, base_sha, 446)          # None when no such row exists at that revision
    row.open, row.grant["js_file"], row.adds

The module uses only the standard library and runs git without a shell.
"""
import re
import subprocess
from typing import Dict, List, Optional

__all__ = ["Row", "front_matter", "parse_columns", "parse_list", "row_at", "git_text", "CLOSED_STATUS"]

_FRONT = re.compile(r"\A---\r?\n(.*?)\r?\n---\r?\n", re.S)
_PAIR = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(\d+)")
# A row whose status says it is finished is closed even when nobody moved it under done/ yet.
CLOSED_STATUS = re.compile(r"^\W*(CLOSED|LANDED|DONE|MERGED|WITHDRAWN|REJECTED|DECLINED|SUPERSEDED)\b", re.I)


def git_text(root: str, *args: str) -> Optional[str]:
    """stdout of `git -C root args`, None when git fails or is missing."""
    try:
        r = subprocess.run(["git", "-C", root, *args], capture_output=True, text=True)
    except OSError:
        return None
    return r.stdout if r.returncode == 0 else None


def front_matter(text: str) -> Dict[str, str]:
    """The `key: value` lines between the leading `---` fences (one line per key, surrounding quotes dropped)."""
    m = _FRONT.match(text)
    if not m:
        return {}
    out: Dict[str, str] = {}
    for ln in m.group(1).split("\n"):
        if ":" not in ln or ln.startswith((" ", "\t", "#", "-")):
            continue
        key, _, val = ln.partition(":")
        val = val.strip()
        if len(val) >= 2 and val[0] == val[-1] and val[0] in "\"'":
            val = val[1:-1]
        out[key.strip()] = val
    return out


def parse_columns(value: str) -> Dict[str, int]:
    """`js_file=360 (inferred ...)` -> {"js_file": 360}; several pairs may be listed (`sink=1, js_lit=20`). A
    parenthesised remark is ignored, `none` is the empty grant."""
    plain = re.sub(r"\([^)]*\)", " ", value or "")
    return {k: int(n) for k, n in _PAIR.findall(plain)}


def parse_list(value: str) -> List[str]:
    """`[a, b]` or `a, b` -> ["a", "b"]; `[]` and `none` are empty."""
    v = (value or "").strip().strip("[]")
    return [p.strip().strip("\"'") for p in v.split(",") if p.strip() and p.strip().lower() != "none"]


class Row:
    """A backlog row as one revision has it."""

    def __init__(self, n: int, path: str, closed: bool, fm: Dict[str, str]):
        self.n = n
        self.path = path
        self.closed = closed
        self.status = fm.get("status", "")
        self.grant = parse_columns(fm.get("js-grant", ""))
        self.fix = parse_columns(fm.get("js-fix", ""))
        self.adds = parse_list(fm.get("adds", ""))

    @property
    def open(self) -> bool:
        return not self.closed and not CLOSED_STATUS.match(self.status)

    def why_closed(self) -> str:
        if self.closed:
            return f"{self.path} is under done/"
        return f"its status says it is finished ({self.status[:40]!r})"


def row_at(root: str, rev: str, n: int) -> Optional[Row]:
    """Row `n` as revision `rev` has it, or None when there is none (or git cannot read the revision). A row
    present both open and under done/ counts as closed."""
    listing = git_text(root, "ls-tree", "-r", "--name-only", rev, "--", "dev-docs/backlog")
    if listing is None:
        return None
    pat = re.compile(rf"^dev-docs/backlog/(done/)?0*{n}-[^/]+\.md$")
    found = [(p, bool(m.group(1))) for p in listing.split("\n") if (m := pat.match(p))]
    if not found:
        return None
    path, closed = sorted(found, key=lambda f: not f[1])[0]
    text = git_text(root, "show", f"{rev}:{path}")
    if text is None:
        return None
    return Row(n, path, closed, front_matter(text))
