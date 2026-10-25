"""The one allowlist engine of the gate scripts (backlog row 429).

A gate that lets a few sites through keeps them in a table of tuples whose LAST item is the reason, and asks
this class whether a site is on it. By construction then: an entry has a reason (non-blank), no two entries are
the same, every entry that no site used is reported as stale (so the list cannot rot into blanket permission),
and `--list` prints the table the same way for every gate.

    import sys
    sys.path.insert(0, "scripts/lib")
    from allow import Allowlist

    ALLOW = Allowlist("check-hover-media", [
        ("preview/assets/main.css", ":not(:hover)", "the REST state of an overlay scrollbar"),
    ], fields=("path", "pattern"))

    entry = ALLOW.find(lambda e: e.path == path and fnmatch.fnmatchcase(sel, e.pattern))   # marks it used
    ...
    errors += ALLOW.problems()          # blank reason, duplicate, stale
    if "--list" in sys.argv: print(ALLOW.listing())

`find` returns the first matching entry and marks it used; `find_all` marks and returns every match (a site that
two entries cover keeps both alive). A gate that skips part of its scan (an explicit file list, a self-test run)
passes `stale=False` to `problems`, since an entry's site may then be legitimately out of view.
"""
from collections import namedtuple
from typing import Callable, Iterable, List, Optional, Sequence

__all__ = ["Allowlist"]


class Allowlist:
    def __init__(self, gate: str, entries: Iterable[Sequence], fields: Sequence[str] = ("key",), source: Optional[str] = None):
        self.gate = gate
        self.source = source or f"scripts/{gate}.sh"
        self.fields = tuple(fields)
        self.Entry = namedtuple("Entry", self.fields + ("reason",))
        self.entries: List = []
        self._used: List[bool] = []
        self._build_errors: List[str] = []
        for raw in entries:
            raw = tuple(raw)
            if len(raw) != len(self.fields) + 1:
                raise ValueError(f"{gate}: allowlist entry {raw!r} must be ({', '.join(self.fields)}, reason)")
            self.entries.append(self.Entry(*raw))
            self._used.append(False)

    def __len__(self):
        return len(self.entries)

    def __iter__(self):
        return iter(self.entries)

    def find(self, pred: Callable) -> Optional[tuple]:
        """The first entry `pred(entry)` accepts, marked used, or None."""
        for k, e in enumerate(self.entries):
            if pred(e):
                self._used[k] = True
                return e
        return None

    def find_all(self, pred: Callable) -> List[tuple]:
        """Every entry `pred(entry)` accepts, all marked used."""
        out = []
        for k, e in enumerate(self.entries):
            if pred(e):
                self._used[k] = True
                out.append(e)
        return out

    def mark(self, entry) -> None:
        """Mark `entry` used (for a gate that matched it by its own rule)."""
        self._used[self.entries.index(entry)] = True

    def used(self, entry) -> bool:
        return self._used[self.entries.index(entry)]

    def unused(self) -> List[tuple]:
        return [e for e, u in zip(self.entries, self._used) if not u]

    def _label(self, e) -> str:
        return "(" + ", ".join(str(v) for v in e[:-1]) + ")"

    def problems(self, *, stale: bool = True) -> List[str]:
        """One line per defect: an entry with a blank reason, a duplicate entry, and (unless `stale=False`) an entry
        no site used."""
        out: List[str] = []
        seen = set()
        for e in self.entries:
            if not str(e.reason).strip():
                out.append(f"allowlist entry {self._label(e)} has no reason")
            key = tuple(map(repr, e[:-1]))
            if key in seen:
                out.append(f"duplicate allowlist entry {self._label(e)}")
            seen.add(key)
        if stale:
            for e in self.unused():
                out.append(f"stale allowlist entry {self._label(e)}: it matches nothing any more -- remove it from {self.source}")
        return out

    def listing(self) -> str:
        """The `--list` text: one `fields...: reason` block per entry."""
        lines = [f"{self.gate}: {len(self.entries)} allowlist entr{'y' if len(self.entries) == 1 else 'ies'}"]
        for e in self.entries:
            lines.append("  " + " | ".join(str(v) for v in e[:-1]))
            lines.append("      " + str(e.reason))
        return "\n".join(lines)
