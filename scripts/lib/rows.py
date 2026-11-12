"""The one reader of backlog-row front matter for the gates (row 444, phase 0.1).

A backlog row is `dev-docs/backlog/N-<slug>.md` while it is open and `dev-docs/backlog/done/N-<slug>.md` once it is
closed. Its front matter carries the grants the gates read:

    js-grant: js_file_bytes=1200 (inferred ceiling: ...)   what a RAISE of a ratchet count may reach, per column
    js-fix:   <path>, <path> (why)                         the legacy files a bug fix may change (`check-js-volume`)
    adds:     [otp_filter]                                 behaviour kinds the row may add (item 0.3)

`js-grant` is `column=number` pairs over the ratchet columns (`GRANT_COLUMNS`, the raising gate's own columns
besides); `none` and an empty value mean "no grant". `js-fix` is a path list and never carries `column=number`;
`adds` is a bracketed list. A value of another shape, or a column no gate has, is a PROBLEM of the row
(`Row.problems`, `row_problems`): the gates refuse it by name instead of reading it as "no grant". A grant is read from the tree of a
GIT REVISION (`git show <rev>:<path>`), never from the working tree: a lane that writes a grant into its own diff has
not been granted anything, because the grant is the owner's row at the merge base.

    from rows import row_at, parse_columns
    row = row_at(root, base_sha, 446)          # None when no such row exists at that revision
    row.open, row.grant["js_file_bytes"], row.adds, row.problems

The module uses only the standard library and runs git without a shell.
"""
import re
import subprocess
from typing import Dict, List, Optional, Sequence

__all__ = ["Row", "front_matter", "parse_columns", "parse_list", "row_problems", "row_at", "git_text", "CLOSED_STATUS",
           "GRANT_COLUMNS"]

_FRONT = re.compile(r"\A---\r?\n(.*?)\r?\n---\r?\n", re.S)
_PAIR = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(\d+)")
# The columns a `js-grant` may name: those of `check-js-volume` (its COLUMNS, held equal by its self-test), the
# `bytes` of the bridge budgets and those of `check-native-slots`. A ratchet also accepts its own columns.
GRANT_COLUMNS = ("sink", "js_lit", "js_file_bytes", "install", "inline_script", "comment_eval", "expect_disallowed",
                 "bytes", "value", "checked", "selected", "open", "hidden")
LEGACY_SPELLING = {"js_file": "js_file_bytes"}
_PARENS = re.compile(r"\([^)]*\)")
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
    v = _PARENS.sub(" ", value or "").strip().strip("[]")
    return [p.strip().strip("\"'") for p in v.split(",") if p.strip() and p.strip().lower() != "none"]


def row_problems(fm: Dict[str, str], valid: Sequence[str] = GRANT_COLUMNS) -> List[str]:
    """What is malformed in the grant-bearing front matter `fm` (empty when sound): a `js-grant` key that is not one of
    `valid` (the legacy `js_file` gets its own message: the unit is bytes), a `js-grant` that is neither `none` nor
    `column=number` pairs, a `js-fix` carrying `column=number` (it is a path list) and an `adds` that is not a
    bracketed list. The messages lack the row's name; `Row.problems` and the gates add it."""
    out: List[str] = []
    grant = _PARENS.sub(" ", fm.get("js-grant", "")).replace(",", " ").split()
    for tok in grant:
        m = _PAIR.fullmatch(tok)
        if not m:
            if not (len(grant) == 1 and tok.lower() == "none"):
                out.append(f"`js-grant:` has {tok!r}, which is not `column=number` (or `none`); the columns are {', '.join(valid)}")
            continue
        key = m.group(1)
        if key in LEGACY_SPELLING:
            out.append(f"`js-grant:` key `{key}` is the legacy spelling: the unit is bytes and the column is "
                       f"`{LEGACY_SPELLING[key]}` (`{key}=<lines>` is dead); the columns are {', '.join(valid)}")
        elif key not in valid:
            out.append(f"`js-grant:` key `{key}` is not a column; the columns are {', '.join(valid)}")
    if "=" in _PARENS.sub(" ", fm.get("js-fix", "")):
        out.append("`js-fix:` is a path list (`a.rs, b.js (why)`), not `column=number`; a volume raise is a `js-grant:`")
    adds = _PARENS.sub(" ", fm.get("adds", "")).strip()
    if adds and adds.lower() != "none" and not re.fullmatch(r"\[[^\[\]]*\]", adds):
        out.append(f"`adds:` is {adds!r}, want a bracketed list such as `[otpFilter]` or `[]`")
    return out


class Row:
    """A backlog row as one revision has it."""

    def __init__(self, n: int, path: str, closed: bool, fm: Dict[str, str]):
        self.n = n
        self.path = path
        self.closed = closed
        self.fm = fm
        self.status = fm.get("status", "")
        self.grant = parse_columns(fm.get("js-grant", ""))
        self.fix = parse_columns(fm.get("js-fix", ""))
        self.adds = parse_list(fm.get("adds", ""))
        self.problems = [f"row {n} ({path}): {p}" for p in row_problems(fm)]

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
