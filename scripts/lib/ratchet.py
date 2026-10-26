"""The one shrink-only ratchet of the gate scripts (backlog row 434, lane R; the construction of the board's finding 4).

A ratchet gate keeps a per-file table of counts (JS sinks, history comments, ...) in a committed baseline TSV and
fails when a count rises. Comparing the measurement with the SAME tree's baseline is not enough: an agent that raises
a count edits the baseline in the same commit and the gate is green. So this class compares the tree's baseline with
the baseline committed at the MERGE BASE with `origin/main` (read through `git show <base>:<path>`), and a raise
needs a visible, new annotation that is printed under a `RATCHET RAISES` heading at every run.

    import sys
    sys.path.insert(0, "scripts/lib")
    from ratchet import Ratchet

    R = Ratchet("check-js-volume", "scripts/check-js-volume.baseline.tsv", ("sink", "js_lit"),
                total=lambda c: c["js_lit"], advice="use the bridge: dev-docs/js-interop.md")
    report = R.check({"a.rs": {"sink": 2, "js_lit": 10}})         # measured counts per file (zeros may be omitted)
    print(R.format(report)); sys.exit(1 if report.errors else 0)
    # --update-baseline:  errors, wrote = R.update(measured, raise_reason=None)

The baseline file (`path<TAB>count<TAB>count...[<TAB># raised: row N <reason>]`, sorted, rows for non-zero files only,
no totals line, so lanes that lower different files merge without conflicts) is checked in four steps:
  1. measured against the tree's baseline: a rise (or a new file with a non-zero count) fails; a fall that is not
     written back fails with a hint to run `--update-baseline`; a row whose file is gone or clean is stale and fails;
  2. the tree's baseline against the merge base's: a rise is allowed ONLY when the tree's row carries a
     `# raised: row N <reason>` note that the base's row does not carry (a note already merged cannot be spent twice);
     every allowed raise is listed under `RATCHET RAISES`;
  3. the optional total (sum of `total(row)` over all rows) against the base's: a rise needs a new
     `#! total-raised: row N <reason>` line in the baseline, also listed;
  4. with no base available (no `origin/main`, a shallow clone, the file is not in the base yet because the gate is
     being introduced) the compare with the tree alone is the check and the report says so, like `check-js-bundle`'s
     skipped half. A baseline that exists at the base and is missing in the tree fails (deleting it is not a pass).

`--update-baseline` never raises on its own (shrink-only); a deliberate raise is `--allow-raise "row N reason"`, which
writes the note itself, with the per-column delta, so the raise is visible in the diff and at the merge. A note is
dropped again once its row is back at or under the base.

Environment: `RATCHET_BASE_REF=<commit>` pins the base (the self-test, or a CI that knows it); `RATCHET_NO_BASE=1`
skips the base compare. The module uses only the standard library and runs git without a shell.
"""
import os
import re
import subprocess
from typing import Callable, Dict, List, Optional, Sequence, Tuple

__all__ = ["Ratchet", "Report", "RAISE_NOTE", "TOTAL_NOTE"]

RAISE_NOTE = re.compile(r"^#\s*raised:\s*row\s+\d+\b.*\S")
TOTAL_NOTE = re.compile(r"^#!\s*total-raised:\s*row\s+\d+\b.*\S")
_DIRECTIVE = "#! total-raised:"

Counts = Dict[str, int]


class Report:
    """What `Ratchet.check` found: `errors` fail the gate, `raises` are the allowed raises (always printed), `notes`
    are informational lines (the base used, or why the base compare was skipped)."""

    def __init__(self):
        self.errors: List[str] = []
        self.raises: List[str] = []
        self.notes: List[str] = []


class Ratchet:
    def __init__(self, gate: str, baseline: str, columns: Sequence[str], root: str = ".",
                 total: Optional[Callable[[Counts], int]] = None, advice: str = "", base_ref: Optional[str] = None):
        self.gate = gate
        self.baseline = baseline
        self.columns = tuple(columns)
        self.root = root
        self.total = total
        self.advice = advice
        self.base_ref = base_ref or os.environ.get("RATCHET_BASE_REF") or None

    # ------------------------------------------------------------------------------------------ parsing
    def parse(self, text: str, label: str = "") -> Tuple[Dict[str, Tuple[Counts, str]], List[str], List[str]]:
        """`(rows, directives, problems)` of a baseline text: rows maps path -> (counts, note), directives are the
        `#! total-raised:` lines, problems are malformed lines."""
        rows: Dict[str, Tuple[Counts, str]] = {}
        directives: List[str] = []
        problems: List[str] = []
        for n, ln in enumerate(text.split("\n"), 1):
            if not ln.strip():
                continue
            if ln.startswith("#!"):
                directives.append(ln.strip())
                continue
            if ln.startswith("#"):
                continue
            parts = ln.split("\t")
            note = ""
            if len(parts) > 1 and parts[-1].startswith("#"):
                note = parts.pop().strip()
            if len(parts) != len(self.columns) + 1 or not all(p.isdigit() for p in parts[1:]):
                problems.append(f"{label}{self.baseline}:{n}: malformed row (want path and {len(self.columns)} counts: "
                                f"{', '.join(self.columns)})")
                continue
            if parts[0] in rows:
                problems.append(f"{label}{self.baseline}:{n}: duplicate row for {parts[0]}")
                continue
            rows[parts[0]] = (dict(zip(self.columns, map(int, parts[1:]))), note)
        return rows, directives, problems

    def render(self, rows: Dict[str, Tuple[Counts, str]], directives: Sequence[str] = ()) -> str:
        out = [f"# {self.gate} baseline (scripts/lib/ratchet.py): shrink-only per-file counts, regenerate with --update-baseline.",
               "# path\t" + "\t".join(self.columns)]
        out += list(directives)
        for path in sorted(rows):
            counts, note = rows[path]
            out.append("\t".join([path] + [str(counts[c]) for c in self.columns] + ([note] if note else [])))
        return "\n".join(out) + "\n"

    def _tree_text(self) -> Optional[str]:
        try:
            with open(os.path.join(self.root, self.baseline), encoding="utf-8") as f:
                return f.read()
        except OSError:
            return None

    def _git(self, *args: str) -> Optional[str]:
        try:
            r = subprocess.run(["git", "-C", self.root, *args], capture_output=True, text=True)
        except OSError:
            return None
        return r.stdout if r.returncode == 0 else None

    def base(self) -> Tuple[Optional[str], Optional[str], str]:
        """`(base_text, base_sha, note)`. `base_text` is the baseline committed at the merge base, None when it cannot
        be read; `note` says why when it is None (or which base was used)."""
        if os.environ.get("RATCHET_NO_BASE"):
            return None, None, "base compare skipped (RATCHET_NO_BASE)"
        sha = self.base_ref
        if not sha:
            for ref in ("origin/main", "main", "origin/master"):
                out = self._git("merge-base", "HEAD", ref)
                if out and out.strip():
                    sha = out.strip()
                    break
        if not sha:
            return None, None, "ratchet base unavailable (no merge base with origin/main), compared with the tree only"
        text = self._git("show", f"{sha}:{self.baseline}")
        if text is None:
            return None, sha, (f"{self.baseline} does not exist at the merge base {sha[:10]}: the gate is being "
                               f"introduced, compared with the tree only")
        return text, sha, f"compared with the baseline at the merge base {sha[:10]}"

    # ------------------------------------------------------------------------------------------ helpers
    def _zero(self) -> Counts:
        return {c: 0 for c in self.columns}

    def _clean(self, measured: Dict[str, Counts]) -> Dict[str, Counts]:
        out = {}
        for path, c in measured.items():
            full = {col: int(c.get(col, 0)) for col in self.columns}
            if any(full.values()):
                out[path] = full
        return out

    def _total(self, rows: Dict[str, Counts]) -> int:
        return sum(self.total(c) for c in rows.values()) if self.total else 0

    @staticmethod
    def _delta(old: Counts, new: Counts) -> str:
        return ", ".join(f"{c} {old[c]}->{new[c]}" for c in new if new[c] != old[c])

    # ------------------------------------------------------------------------------------------ check
    def check(self, measured: Dict[str, Counts], *, stale: bool = True) -> Report:
        rep = Report()
        gate = self.gate
        M = self._clean(measured)
        text = self._tree_text()
        base_text, sha, base_note = self.base()
        if text is None:
            if base_text is not None:
                rep.errors.append(f"{gate}: {self.baseline} is missing but the merge base {sha[:10]} has it: restore it "
                                  f"(deleting the baseline is not a way to pass)")
            else:
                rep.errors.append(f"{gate}: {self.baseline} is missing: run --update-baseline")
            return rep
        rows, directives, problems = self.parse(text)
        rep.errors += [f"{gate}: {p}" for p in problems]
        T = {p: c for p, (c, _) in rows.items()}
        notes = {p: n for p, (_, n) in rows.items()}

        # 1. measured against the tree's baseline
        for path in sorted(set(M) | set(T)):
            m, t = M.get(path, self._zero()), T.get(path, self._zero())
            rose = [c for c in self.columns if m[c] > t[c]]
            fell = [c for c in self.columns if m[c] < t[c]]
            if rose:
                what = "new file with a non-zero count" if path not in T else "count rose above the baseline"
                rep.errors.append(f"{gate}: {path}: {what} ({self._delta(t, m)}). {self.advice}".rstrip())
            elif fell:
                if path not in M:
                    if stale:
                        rep.errors.append(f"{gate}: {path}: stale baseline row (the file is gone or clean, was "
                                          f"{self._delta(t, m)}): run --update-baseline to lock the drop in")
                else:
                    rep.errors.append(f"{gate}: {path}: count fell ({self._delta(t, m)}) and the baseline still allows "
                                      f"the old number: run --update-baseline to lock the drop in")

        # 2. the tree's baseline against the merge base's
        if base_text is not None:
            brows, bdirs, _ = self.parse(base_text, label="base ")
            B = {p: c for p, (c, _) in brows.items()}
            bnotes = {p: n for p, (_, n) in brows.items()}
            for path in sorted(T):
                t, b = T[path], B.get(path, self._zero())
                rose = [c for c in self.columns if t[c] > b[c]]
                if not rose:
                    continue
                note = notes.get(path, "")
                if RAISE_NOTE.match(note) and note != bnotes.get(path, ""):
                    rep.raises.append(f"{path}: {self._delta(b, t)}   {note}")
                else:
                    why = ("it has no `# raised: row N <reason>` note" if not note else
                           "its `# raised:` note is malformed" if not RAISE_NOTE.match(note) else
                           "its `# raised:` note is the one the merge base already has")
                    rep.errors.append(f"{gate}: {path}: the baseline row rises above the merge base "
                                      f"({self._delta(b, t)}) and {why}. A raise is the owner's decision: lower the "
                                      f"count, or write the row with `--update-baseline --allow-raise \"row N reason\"`. "
                                      f"{self.advice}".rstrip())
            # 3. the total
            if self.total:
                tt, tb = self._total(T), self._total(B)
                if tt > tb:
                    new = [d for d in directives if TOTAL_NOTE.match(d) and d not in bdirs]
                    if new:
                        rep.raises.append(f"TOTAL: {tb} -> {tt}   {new[0]}")
                    else:
                        rep.errors.append(f"{gate}: the total rose above the merge base ({tb} -> {tt}) and the baseline "
                                          f"has no new `#! total-raised: row N <reason>` line. Moving code from one "
                                          f"file to another must still lower the total. {self.advice}".rstrip())
                elif tt < tb:
                    rep.notes.append(f"total {tb} -> {tt} against the merge base")
        rep.notes.append(base_note)
        return rep

    def format(self, rep: Report) -> str:
        lines: List[str] = []
        if rep.raises:
            lines.append(f"RATCHET RAISES ({self.gate}): {len(rep.raises)} deliberate raise(s) over the merge base, "
                         f"for the owner to see:")
            lines += ["  " + r for r in rep.raises]
        lines += [f"{self.gate}: {n}" for n in rep.notes if n]
        lines += rep.errors
        return "\n".join(lines)

    # ------------------------------------------------------------------------------------------ update
    def update(self, measured: Dict[str, Counts], raise_reason: Optional[str] = None) -> Tuple[List[str], bool]:
        """Rewrite the baseline from `measured`. Returns `(errors, wrote)`. Shrink-only: a rise is refused unless
        `raise_reason` (which must name a row: `row N ...`) is given, and then it is written into the note."""
        M = self._clean(measured)
        text = self._tree_text()
        old_rows, old_dirs = ({}, [])
        if text is not None:
            old_rows, old_dirs, problems = self.parse(text)
            if problems:
                return problems, False
        T = {p: c for p, (c, _) in old_rows.items()}
        errors: List[str] = []
        if raise_reason is not None and not re.match(r"row\s+\d+\b.*\S", raise_reason):
            return [f"{self.gate}: --allow-raise needs a reason that names a backlog row: \"row 434 <why>\""], False
        base_text, _, _ = self.base()
        B: Dict[str, Counts] = {p: c for p, (c, _) in self.parse(base_text)[0].items()} if base_text else {}
        new_rows: Dict[str, Tuple[Counts, str]] = {}
        for path, m in M.items():
            t = T.get(path, self._zero())
            note = old_rows[path][1] if path in old_rows else ""
            rose = [c for c in self.columns if m[c] > t[c]]
            if rose and text is not None:
                if raise_reason is None:
                    errors.append(f"{self.gate}: {path}: {self._delta(t, m)} rises above the baseline; --update-baseline "
                                  f"is shrink-only. Remove the JS, or raise on purpose with --allow-raise "
                                  f"\"row N reason\". {self.advice}".rstrip())
                    continue
                note = f"# raised: {raise_reason} ({self._delta(t, m)})"
            elif base_text is not None and note and not any(m[c] > B.get(path, self._zero())[c] for c in self.columns):
                note = ""  # back at or under the merge base: the note has done its job
            new_rows[path] = (m, note)
        if errors:
            return errors, False
        dirs = list(old_dirs)
        if self.total and text is not None:
            tn, to = self._total(M), self._total({p: c for p, c in T.items()})
            if tn > to:
                if raise_reason is None:
                    return [f"{self.gate}: the total rises ({to} -> {tn}); --update-baseline is shrink-only"], False
                dirs = [d for d in dirs if not d.startswith(_DIRECTIVE)] + [f"{_DIRECTIVE} {raise_reason} (total {to}->{tn})"]
            elif base_text is not None:
                bt = self._total(B)
                if tn <= bt:
                    dirs = [d for d in dirs if not d.startswith(_DIRECTIVE)]
        os.makedirs(os.path.dirname(os.path.join(self.root, self.baseline)), exist_ok=True)
        with open(os.path.join(self.root, self.baseline), "w", encoding="utf-8") as f:
            f.write(self.render(new_rows, dirs))
        return [], True
