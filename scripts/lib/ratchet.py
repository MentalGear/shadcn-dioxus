"""The one shrink-only ratchet of the gate scripts, with the grant rule (backlog rows 434 and 444).

A ratchet gate keeps a per-file table of counts (JS sinks, history comments, ...) in a committed baseline TSV and
fails when a count rises. Comparing the measurement with the SAME tree's baseline is not enough: an agent that raises
a count edits the baseline in the same commit and the gate is green. So this class compares the tree's baseline with
the baseline committed at the BASE (read through `git show <base>:<path>`), and a raise passes only against a GRANT:
a backlog row that exists and is open AT THE BASE and whose front matter says `js-grant: <column>=<n>` with n at least
the raise (scripts/lib/rows.py reads it). A lane cannot grant itself: a grant written in the lane's own diff is not at
the base. The owner grants; the red text says so.

    import sys
    sys.path.insert(0, "scripts/lib")
    from ratchet import Ratchet

    R = Ratchet("check-js-volume", "scripts/check-js-volume.baseline.tsv", ("sink", "js_lit"),
                total=lambda c: c["js_lit"], advice="use the bridge: dev-docs/js-interop.md")
    report = R.check({"a.rs": {"sink": 2, "js_lit": 10}})         # measured counts per file (zeros may be omitted)
    print(R.format(report)); sys.exit(1 if report.errors else 0)
    # --update-baseline:  errors, wrote = R.update(measured, raise_reason=None)

The baseline file (`#! columns: c1 c2 ...` header, `path<TAB>count<TAB>count...[<TAB># raised: row N <reason>]`,
sorted, rows for non-zero files only, no totals line, so lanes that lower different files merge without conflicts) is
checked in four steps:
  1. measured against the tree's baseline: a rise (or a new file with a non-zero count) fails; a fall that is not
     written back fails with a hint to run `--update-baseline`; a row whose file is gone or clean is stale and fails;
  2. the tree's baseline against the base's: a rise needs a `# raised: row N <reason>` note that the base's row does
     not carry (a note already merged cannot be spent twice) AND row N must be a grant at the base; the rises that
     cite one row add up per column and must stay within its `js-grant`. Own raises (this diff's) and inherited ones
     (notes the base already carries) are listed separately under `RATCHET RAISES`. A row carries across a rename
     (`git diff -M`), notes included. When two granted rows raise the same file, the note is a CHAIN of steps, each
     with its own row and its own from -> to, joined by `; `:
         # raised: row 454 <reason> (js_file_bytes 19956->22234); row 451 <reason> (js_file_bytes 22234->22558)
     The chain must run without a gap from the base's number to the tree's, per column, and each step is charged to
     ITS row only (that row's grant must cover that step, summed over every file it cites): one row's grant never
     pays for another row's step, and two grants never add up to cover one oversized step. A note of one step needs
     no from -> to; the file's whole raise is charged to its row;
  3. the optional total (sum of `total(row)` over all rows) against the base's: a rise needs a new
     `#! total-raised: row N total=<the tree's total> <reason>` line; the number is compared with the tree's total, and
     the rise must stay within the sum of the row's `js-grant` numbers. Several granted rows chain the same way, one
     line per row, each ending `(total A->B)`: the first A is the base's total, each A is the previous B, the last B
     is the tree's total, and each step stays within the grant sum of its own row;
  4. a column the base's `#! columns:` header lacks (a baseline without the header has the gate's columns) is
     compared with the tree only and printed as "introduced".
With no base available (no `origin/main` or `origin/queue/*`, a shallow clone, the file is not in the base yet because
the gate is being introduced) the compare with the tree alone is the check and the report says so; with
`RATCHET_REQUIRE_BASE=1` (a full gate run) "no base" is red instead. A baseline that exists at the base and is missing
in the tree fails (deleting it is not a pass).

The base is the nearest of `origin/main` and `origin/queue/*`: the merge base with HEAD that has the fewest commits
between it and HEAD. `--update-baseline` is shrink-only; `--allow-raise "row N reason"` is only a writer of the note
and refuses unless the grant at the base already covers the raise, so it cannot pass on its own. When a file's note
(or the total line) is already a step made in this diff, a second `--allow-raise` APPENDS its step from the number
the chain reaches to the tree's number; a note the base already carries is replaced, as is one whose row is back at or
under the base (dropped). A refused write leaves the file untouched. A chain is trimmed when a count falls again.

Environment: `RATCHET_BASE_REF=<commit>` pins the base (the self-test, or a CI that knows it); `RATCHET_NO_BASE=1`
skips the base compare; `RATCHET_REQUIRE_BASE=1` makes an unavailable base red. `--self-test` proves the rules.
The module uses only the standard library and runs git without a shell.
"""
import contextlib
import os
import re
import subprocess
import sys
import tempfile
from typing import Callable, Dict, List, Optional, Sequence, Tuple

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rows import row_at, parse_columns, row_problems, GRANT_COLUMNS  # noqa: E402

__all__ = ["Ratchet", "Report", "Step", "note_steps", "RAISE_NOTE", "TOTAL_NOTE"]

RAISE_NOTE = re.compile(r"^#\s*raised:\s*row\s+(\d+)\b.*\S")
TOTAL_NOTE = re.compile(r"^#!\s*total-raised:\s*row\s+(\d+)\s+total=(\d+)\b.*\S")
_STEP_SPLIT = re.compile(r";\s*(?=row\s+\d+\b)")
_STEP_HEAD = re.compile(r"row\s+(\d+)\b\s*(.*)$", re.S)
_DELTA_TAIL = re.compile(r"\(\s*((?:[A-Za-z_]\w*\s+\d+->\d+\s*,?\s*)+)\)\s*$")
_DELTA_PART = re.compile(r"([A-Za-z_]\w*)\s+(\d+)->(\d+)")
_TOTAL_TAIL = re.compile(r"\(\s*total\s+(\d+)->(\d+)\s*\)\s*$")
_DIRECTIVE = "#! total-raised:"
_COLUMNS = "#! columns:"
ASK = ("A raise is the owner's decision: ask the owner for a grant row (an open backlog row at the merge base whose "
       "front matter says `js-grant: <column>=<n>`, n at least the raise); a grant written in this diff does not count")

CHAIN_HELP = ("Each step of a chain is one row's own raise: run `--update-baseline --allow-raise \"row N reason\"` once "
              "per row, in merge order, on a tree where the earlier rows' raises are already written")

Counts = Dict[str, int]


class Step:
    """One step of a raise note: the row that grants it, its reason and, per column, `(from, to)` (empty when the
    note names none). `raw` is the step's own text, kept so that an untouched note is written back unchanged."""

    def __init__(self, row: int, reason: str, deltas: Dict[str, Tuple[int, int]], raw: Optional[str] = None):
        self.row, self.reason, self.deltas, self.raw = row, reason, deltas, raw

    @property
    def text(self) -> str:
        if self.raw is not None:
            return self.raw
        d = ", ".join(f"{c} {a}->{b}" for c, (a, b) in self.deltas.items())
        return f"row {self.row} {self.reason}" + (f" ({d})" if d else "")

    def delta_text(self) -> str:
        return ", ".join(f"{c} {a}->{b}" for c, (a, b) in self.deltas.items())


def note_steps(note: str) -> Optional[List[Step]]:
    """The steps of a `# raised:` note (`row N <reason> (col A->B)`, further steps after `; `), None when the note
    is not one."""
    if not RAISE_NOTE.match(note):
        return None
    body = re.sub(r"^#\s*raised:\s*", "", note).strip()
    steps: List[Step] = []
    for seg in _STEP_SPLIT.split(body):
        m = _STEP_HEAD.match(seg.strip())
        if not m:
            return None
        rest, deltas = m.group(2).strip(), {}
        tail = _DELTA_TAIL.search(rest)
        if tail:
            deltas = {c: (int(a), int(b)) for c, a, b in _DELTA_PART.findall(tail.group(1))}
            rest = rest[:tail.start()].strip()
        steps.append(Step(int(m.group(1)), rest, deltas, seg.strip()))
    return steps


class Report:
    """What `Ratchet.check` found: `errors` fail the gate, `raises` are this diff's own granted raises and `inherited`
    the raises the base already carries (both always printed), `notes` are informational lines (the base used, why
    the base compare was skipped, the columns introduced)."""

    def __init__(self):
        self.errors: List[str] = []
        self.raises: List[str] = []
        self.inherited: List[str] = []
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
    @staticmethod
    def header_columns(text: str) -> Optional[Tuple[str, ...]]:
        """The columns a baseline text declares with its `#! columns:` line, None when it has no such line."""
        for ln in text.split("\n"):
            if ln.startswith(_COLUMNS):
                return tuple(ln[len(_COLUMNS):].split())
        return None

    def parse(self, text: str, label: str = "", columns: Optional[Sequence[str]] = None
              ) -> Tuple[Dict[str, Tuple[Counts, str]], List[str], List[str]]:
        """`(rows, directives, problems)` of a baseline text: rows maps path -> (counts, note), directives are the
        `#! total-raised:` lines, problems are malformed lines. `columns` are the columns the text's rows carry
        (default: the gate's)."""
        cols = tuple(columns) if columns else self.columns
        rows: Dict[str, Tuple[Counts, str]] = {}
        directives: List[str] = []
        problems: List[str] = []
        for n, ln in enumerate(text.split("\n"), 1):
            if not ln.strip():
                continue
            if ln.startswith(_COLUMNS):
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
            if len(parts) != len(cols) + 1 or not all(p.isdigit() for p in parts[1:]):
                problems.append(f"{label}{self.baseline}:{n}: malformed row (want path and {len(cols)} counts: "
                                f"{', '.join(cols)})")
                continue
            if parts[0] in rows:
                problems.append(f"{label}{self.baseline}:{n}: duplicate row for {parts[0]}")
                continue
            rows[parts[0]] = (dict(zip(cols, map(int, parts[1:]))), note)
        return rows, directives, problems

    def render(self, rows: Dict[str, Tuple[Counts, str]], directives: Sequence[str] = ()) -> str:
        out = [f"# {self.gate} baseline (scripts/lib/ratchet.py): shrink-only per-file counts, regenerate with --update-baseline.",
               f"{_COLUMNS} " + " ".join(self.columns),
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

    def _nearest_base(self) -> Optional[str]:
        """The merge base of HEAD with the nearest of `origin/main` and `origin/queue/*` (fewest commits between the
        merge base and HEAD; `origin/main` wins a tie)."""
        refs = self._git("for-each-ref", "--format=%(refname:short)", "refs/remotes/origin/main",
                         "refs/remotes/origin/queue")
        best: Optional[Tuple[int, int, str]] = None
        for rank, ref in enumerate(sorted((refs or "").split(), key=lambda r: (r != "origin/main", r))):
            mb = (self._git("merge-base", "HEAD", ref) or "").strip()
            if not mb:
                continue
            dist = (self._git("rev-list", "--count", f"{mb}..HEAD") or "").strip()
            key = (int(dist) if dist.isdigit() else 1 << 30, rank, mb)
            if best is None or key < best:
                best = key
        return best[2] if best else None

    def base(self) -> Tuple[Optional[str], Optional[str], str]:
        """`(base_text, base_sha, note)`. `base_text` is the baseline committed at the base, None when it cannot
        be read; `note` says why when it is None (or which base was used)."""
        if os.environ.get("RATCHET_NO_BASE"):
            return None, None, "ratchet base unavailable: base compare skipped (RATCHET_NO_BASE)"
        sha = self.base_ref or self._nearest_base()
        if not sha:
            return None, None, ("ratchet base unavailable (no merge base with origin/main or origin/queue/*), "
                                "compared with the tree only")
        text = self._git("show", f"{sha}:{self.baseline}")
        if text is None:
            return None, sha, (f"{self.baseline} does not exist at the base {sha[:10]}: the gate is "
                               f"being introduced, compared with the tree only")
        return text, sha, f"compared with the baseline at the base {sha[:10]}"

    def _renames(self, sha: str) -> Dict[str, str]:
        """new path -> old path of the files the tree renamed since `sha` (`git diff -M`)."""
        out = self._git("diff", "-M", "--name-status", "--diff-filter=R", sha) or ""
        pairs = (ln.split("\t") for ln in out.split("\n") if ln.startswith("R"))
        return {p[2]: p[1] for p in pairs if len(p) == 3}

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

    def _grant_problem(self, sha: str, n: int, need: Counts, total_need: int = 0) -> Optional[str]:
        """Why row `n` at `sha` does not grant `need` (per column) and `total_need` (against the sum of its grant
        numbers), None when it does."""
        row = row_at(self.root, sha, n)
        if row is None:
            return f"row {n} does not exist at the base {sha[:10]}"
        if not row.open:
            return f"row {n} is closed at the base ({row.why_closed()})"
        valid = GRANT_COLUMNS + tuple(c for c in self.columns if c not in GRANT_COLUMNS)
        bad = row_problems(row.fm, valid)
        if bad:
            return f"row {n} ({row.path}): {bad[0]}"
        if not row.grant:
            return f"row {n} has no `js-grant: <column>=<n>` at the base"
        for c, d in sorted(need.items()):
            if row.grant.get(c, 0) < d:
                return f"row {n} grants {c}={row.grant.get(c, 0)} at the base, the raise is {d}"
        if total_need and sum(row.grant.values()) < total_need:
            return f"row {n} grants {sum(row.grant.values())} in all at the base, the total raise is {total_need}"
        return None

    def _chain_problem(self, steps: List[Step], b: Counts, t: Counts) -> Optional[str]:
        """Why the steps of a note of several steps do not run, per column, from the base's number `b` to the tree's
        `t` without a gap, None when they do."""
        for s in steps:
            if not s.deltas:
                return (f"step `row {s.row}` names no `(column A->B)`: a note of several steps gives each step its "
                        f"own from -> to")
        listed = {c for s in steps for c in s.deltas}
        for c in sorted(listed):
            if c not in self.columns:
                return f"names `{c}`, which is not a column ({', '.join(self.columns)})"
            cur = b[c]
            for s in steps:
                if c not in s.deltas:
                    continue
                a, to = s.deltas[c]
                if a != cur:
                    return (f"has a gap in {c}: step `row {s.row}` starts at {a} but the chain before it is at {cur} "
                            f"(the base's number is the first start)")
                cur = to
            if cur != t[c]:
                return f"ends at {c}={cur} but the tree has {c}={t[c]}"
        for c in self.columns:
            if t[c] > b[c] and c not in listed:
                return f"has no step for {c} ({b[c]}->{t[c]})"
        return None

    @staticmethod
    def _step_need(steps: List[Step], rose_total: Optional[Counts] = None) -> Dict[int, Counts]:
        """Per row, per column, what its steps raise (a step that falls costs nothing). A note of one step without
        its own from -> to is charged with `rose_total`, the file's whole raise."""
        need: Dict[int, Counts] = {}
        if len(steps) == 1:
            for c, d in (rose_total or {}).items():
                need.setdefault(steps[0].row, {})[c] = need.get(steps[0].row, {}).get(c, 0) + d
            return need
        for s in steps:
            for c, (a, b) in s.deltas.items():
                if b > a:
                    need.setdefault(s.row, {})[c] = need.get(s.row, {}).get(c, 0) + b - a
        return need

    def _total_chain(self, new: List[str], tb: int, tt: int) -> Tuple[Dict[int, int], Optional[str]]:
        """`(per row, what its steps raise the total, problem)` of the new `#! total-raised:` lines. One line is
        charged the whole rise (its `(total A->B)` is optional); several chain from the base's total to the tree's."""
        rows = [int(TOTAL_NOTE.match(d).group(1)) for d in new]
        if len(new) == 1:
            return {rows[0]: tt - tb}, None
        need: Dict[int, int] = {}
        cur = tb
        for d, n in zip(new, rows):
            tail = _TOTAL_TAIL.search(d)
            if not tail:
                return need, f"the total step `row {n}` names no `(total A->B)`: several steps each give their own"
            a, to = int(tail.group(1)), int(tail.group(2))
            if int(TOTAL_NOTE.match(d).group(2)) != to:
                return need, f"the total step `row {n}` says total={TOTAL_NOTE.match(d).group(2)} but ends at {to}"
            if a != cur:
                return need, f"the total chain has a gap: step `row {n}` starts at {a} but the chain is at {cur}"
            if to > a:
                need[n] = need.get(n, 0) + to - a
            cur = to
        if cur != tt:
            return need, f"the total chain ends at {cur} but the tree's total is {tt}"
        return need, None

    # ------------------------------------------------------------------------------------------ check
    def check(self, measured: Dict[str, Counts], *, stale: bool = True) -> Report:
        rep = Report()
        gate = self.gate
        M = self._clean(measured)
        text = self._tree_text()
        base_text, sha, base_note = self.base()
        if sha is None and os.environ.get("RATCHET_REQUIRE_BASE"):
            rep.errors.append(f"{gate}: RATCHET_REQUIRE_BASE is set and there is no base ({base_note}): a full run "
                              f"needs origin/main or an origin/queue/* ref; fetch it, or pin RATCHET_BASE_REF")
        if text is None:
            if base_text is not None:
                rep.errors.append(f"{gate}: {self.baseline} is missing but the base {sha[:10]} has it: restore it "
                                  f"(deleting the baseline is not a way to pass)")
            else:
                rep.errors.append(f"{gate}: {self.baseline} is missing: run --update-baseline")
            return rep
        tree_cols = self.header_columns(text)
        if tree_cols is not None and tree_cols != self.columns:
            rep.errors.append(f"{gate}: {self.baseline} declares `#! columns: {' '.join(tree_cols)}` but the gate "
                              f"counts {' '.join(self.columns)}: run --update-baseline")
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

        # 2. the tree's baseline against the base's
        if base_text is not None:
            bcols = self.header_columns(base_text) or self.columns
            brows, bdirs, _ = self.parse(base_text, label="base ", columns=bcols)
            introduced = [c for c in self.columns if c not in bcols]
            if introduced:
                rep.notes.append(f"column(s) {', '.join(introduced)} introduced (not in the base's `#! columns:`): "
                                 f"compared with the tree only")
            renames = self._renames(sha)
            # the base's row of every tree path, carried across a rename; an introduced column has no base number, so
            # it takes the tree's own (a zero delta) and only the tree compare above sees it
            B: Dict[str, Counts] = {}
            bnotes: Dict[str, str] = {}
            for path in T:
                src = path if path in brows else renames.get(path, path)
                counts, note = brows.get(src, ({}, ""))
                B[path] = {c: (T[path][c] if c in introduced else counts.get(c, 0)) for c in self.columns}
                bnotes[path] = note
            need: Dict[int, Counts] = {}
            lines: Dict[int, List[str]] = {}
            for path in sorted(T):
                t, b = T[path], B[path]
                note = notes.get(path, "")
                tag = f" (carried from {renames[path]})" if path in renames else ""
                rose = [c for c in self.columns if t[c] > b[c]]
                if not rose:
                    if note and RAISE_NOTE.match(note) and note == bnotes[path]:
                        rep.inherited.append(f"{path}{tag}: {note}")
                    continue
                if RAISE_NOTE.match(note) and note != bnotes[path]:
                    steps = note_steps(note) or []
                    if len(steps) > 1:
                        bad = self._chain_problem(steps, b, t)
                        if bad:
                            rep.errors.append(f"{gate}: {path}: the `# raised:` chain {bad}. {CHAIN_HELP}. "
                                              f"{self.advice}".rstrip())
                            continue
                    for n, cols in self._step_need(steps, {c: t[c] - b[c] for c in rose}).items():
                        for c, d in cols.items():
                            need.setdefault(n, {}).setdefault(c, 0)
                            need[n][c] += d
                    if len(steps) > 1:
                        for s in steps:
                            lines.setdefault(s.row, []).append(f"{path}{tag}: {s.delta_text()}   # raised: {s.text}")
                    else:
                        lines.setdefault(steps[0].row, []).append(f"{path}{tag}: {self._delta(b, t)}   {note}")
                else:
                    why = ("it has no `# raised: row N <reason>` note" if not note else
                           "its `# raised:` note is malformed" if not RAISE_NOTE.match(note) else
                           "its `# raised:` note is the one the base already has")
                    rep.errors.append(f"{gate}: {path}: the baseline row rises above the base "
                                      f"({self._delta(b, t)}) and {why}. {ASK}. {self.advice}".rstrip())
            for n in sorted(need):
                why = self._grant_problem(sha, n, need[n])
                if why:
                    rep.errors.append(f"{gate}: the raise cited as row {n} ({', '.join(f'{c} +{d}' for c, d in need[n].items())}) "
                                      f"is not granted: {why}. {ASK}. {self.advice}".rstrip())
                else:
                    rep.raises += lines[n]
            # 3. the total
            if self.total:
                tt = self._total(T)
                tb = self._base_total(brows, T, renames, introduced)
                new = [d for d in directives if TOTAL_NOTE.match(d) and d not in bdirs]
                for d in (new if len(new) == 1 else []):
                    m = TOTAL_NOTE.match(d)
                    if int(m.group(2)) != tt:
                        rep.errors.append(f"{gate}: `{d[:70]}` carries total={m.group(2)} but the tree's total is {tt}: "
                                          f"the directive's number must be the tree's total")
                if tt > tb:
                    if not new:
                        rep.errors.append(f"{gate}: the total rose above the base ({tb} -> {tt}) and the baseline "
                                          f"has no new `#! total-raised: row N total={tt} <reason>` line. Moving code "
                                          f"from one file to another must still lower the total. {ASK}. "
                                          f"{self.advice}".rstrip())
                    else:
                        tneed, bad = self._total_chain(new, tb, tt)
                        if bad:
                            rep.errors.append(f"{gate}: {bad}. {CHAIN_HELP}. {self.advice}".rstrip())
                        bad_rows = []
                        for n in sorted(tneed):
                            why = self._grant_problem(sha, n, {}, tneed[n])
                            if why:
                                bad_rows.append(n)
                                rep.errors.append(f"{gate}: the total raise {tb} -> {tt} cited as row {n} is not "
                                                  f"granted: {why}. {ASK}. {self.advice}".rstrip())
                        if not bad and not bad_rows and new:
                            rep.raises += [f"TOTAL: {tb} -> {tt}   {d}" for d in new]
                else:
                    for d in directives:
                        if TOTAL_NOTE.match(d) and d in bdirs:
                            rep.inherited.append(f"TOTAL (the base's own): {d}")
                    if tt < tb:
                        rep.notes.append(f"total {tb} -> {tt} against the base")
        rep.notes.append(base_note)
        return rep

    def _base_total(self, brows, T, renames, introduced) -> int:
        """The base's total over the rows it has (the tree's own number stands in for an introduced column)."""
        total = 0
        for path, (counts, _) in brows.items():
            full = {c: counts.get(c, 0) for c in self.columns}
            tp = next((p for p in T if p == path or renames.get(p) == path), None)
            for c in introduced:
                full[c] = T[tp][c] if tp else 0
            total += self.total(full)
        return total

    def format(self, rep: Report) -> str:
        lines: List[str] = []
        if rep.raises:
            lines.append(f"RATCHET RAISES ({self.gate}): own, {len(rep.raises)} granted raise(s) in this diff over "
                         f"the base, for the owner to see:")
            lines += ["  " + r for r in rep.raises]
        if rep.inherited:
            lines.append(f"RATCHET RAISES ({self.gate}): inherited, {len(rep.inherited)} raise(s) the base already "
                         f"carries (merged earlier, not spent again):")
            lines += ["  " + r for r in rep.inherited]
        lines += [f"{self.gate}: {n}" for n in rep.notes if n]
        lines += rep.errors
        return "\n".join(lines)

    # ------------------------------------------------------------------------------------------ update
    @staticmethod
    def _trim(steps: List[Step], m: Counts) -> List[Step]:
        """The steps with every column capped at the new number `m` (a count fell again): a step that no longer
        raises anything in a column loses it, a step with no column left goes."""
        out: List[Step] = []
        for s in steps:
            d = {c: (a, min(b, m[c])) for c, (a, b) in s.deltas.items() if a < m.get(c, 0)}
            if d == s.deltas:
                out.append(s)
            elif d:
                out.append(Step(s.row, s.reason, d))
        return out

    def _trim_total(self, own: List[str], tn: int) -> List[str]:
        out: List[str] = []
        for d in own:
            m, tail = TOTAL_NOTE.match(d), _TOTAL_TAIL.search(d)
            if len(own) == 1 or not tail or not m:
                out.append(re.sub(r"total=\d+", f"total={tn}", d, count=1))
                continue
            a, b = int(tail.group(1)), int(tail.group(2))
            if a >= tn:
                continue
            if b > tn:
                head = d[:tail.start()]
                d = re.sub(r"total=\d+", f"total={tn}", head, count=1) + f"(total {a}->{tn})"
            out.append(d)
        return out

    def update(self, measured: Dict[str, Counts], raise_reason: Optional[str] = None) -> Tuple[List[str], bool]:
        """Rewrite the baseline from `measured`. Returns `(errors, wrote)`. Shrink-only: a rise is refused unless
        `raise_reason` (`row N ...`) is given AND the grant at the base already covers it; then a step with the
        reason is written into the file's note: a new note, or appended to the chain when the note is already a step
        made in this diff (a note the base carries is replaced). The writer never grants: `check` reads the grant at
        the base again. A refusal writes nothing."""
        M = self._clean(measured)
        text = self._tree_text()
        old_rows, old_dirs = ({}, [])
        if text is not None:
            old_rows, old_dirs, problems = self.parse(text)
            if problems:
                return problems, False
        T = {p: c for p, (c, _) in old_rows.items()}
        rm = re.match(r"row\s+(\d+)\b.*\S", raise_reason) if raise_reason is not None else None
        if raise_reason is not None and not rm:
            return [f"{self.gate}: --allow-raise needs a reason that names a backlog row: \"row 434 <why>\""], False
        if raise_reason is not None and _STEP_SPLIT.search(raise_reason):
            return [f"{self.gate}: --allow-raise reason must not contain `; row N` (it separates the steps of a note)"], False
        row_n = int(rm.group(1)) if rm else 0
        base_text, sha, _ = self.base()
        bcols = (self.header_columns(base_text) or self.columns) if base_text else self.columns
        B: Dict[str, Counts] = {}
        bnote: Dict[str, str] = {}
        brows: Dict[str, Tuple[Counts, str]] = {}
        bdirs: List[str] = []
        renames: Dict[str, str] = {}
        introduced = [c for c in self.columns if c not in bcols]
        if base_text:
            renames = self._renames(sha)
            brows, bdirs, _ = self.parse(base_text, columns=bcols)
            for path in set(M) | set(T):
                src = path if path in brows else renames.get(path, "")
                counts, bnote[path] = brows.get(src, ({}, ""))
                B[path] = {c: (T[path][c] if c in T.get(path, {}) and c in introduced else counts.get(c, 0))
                           for c in self.columns}
        errors: List[str] = []
        new_rows: Dict[str, Tuple[Counts, str]] = {}
        need: Dict[int, Counts] = {}
        for path, m in M.items():
            t = T.get(path, self._zero())
            bt = B.get(path, t) if base_text is not None else t
            note = old_rows[path][1] if path in old_rows else ""
            own = bool(note) and RAISE_NOTE.match(note) is not None and (base_text is None or note != bnote.get(path))
            rose = [c for c in self.columns if m[c] > t[c]]
            if rose and text is not None:
                if raise_reason is None:
                    errors.append(f"{self.gate}: {path}: {self._delta(t, m)} rises above the baseline; --update-baseline "
                                  f"is shrink-only. Remove the JS. {ASK}. {self.advice}".rstrip())
                    continue
                note, err = self._next_note(path, note if own else "", bt, m, raise_reason, row_n)
                if err:
                    errors.append(err)
                    continue
            elif base_text is not None and note and not any(m[c] > B[path][c] for c in self.columns):
                note = ""  # back at or under the base: the note has done its job
            elif own and any(m[c] < t[c] for c in self.columns):
                steps = note_steps(note) or []
                if len(steps) > 1:
                    note = "# raised: " + "; ".join(s.text for s in self._trim(steps, m)) if steps else note
            own = bool(note) and RAISE_NOTE.match(note) is not None and (base_text is None or note != bnote.get(path))
            if own and raise_reason is not None:
                steps = note_steps(note) or []
                for n, cols in self._step_need(steps, {c: m[c] - bt[c] for c in self.columns if m[c] > bt[c]}).items():
                    for c, d in cols.items():
                        need.setdefault(n, {})[c] = need.get(n, {}).get(c, 0) + d
            new_rows[path] = (m, note)
        if errors:
            return errors, False
        dirs = list(old_dirs)
        tn = self._total(M)
        total_need = 0
        if self.total and text is not None:
            to = self._total(T)
            tb = self._base_total(brows, T, renames, introduced) if base_text else None
            own_dirs = [d for d in dirs if TOTAL_NOTE.match(d) and d not in bdirs]
            rest = [d for d in dirs if not d.startswith(_DIRECTIVE)]
            if tn > to and raise_reason is None:
                return [f"{self.gate}: the total rises ({to} -> {tn}); --update-baseline is shrink-only. {ASK}"], False
            if tn > to:
                reason = raise_reason[rm.end(1):].strip() or "raise"
                start = tb if tb is not None else to
                if own_dirs:
                    tail = _TOTAL_TAIL.search(own_dirs[-1])
                    start = int(tail.group(2)) if tail else to
                dirs = rest + own_dirs + [f"{_DIRECTIVE} row {row_n} total={tn} {reason} (total {start}->{tn})"]
                total_need = tn - (tb if tb is not None else to)
                if base_text is not None:
                    tneed, bad = self._total_chain(dirs[len(rest):], tb, tn)
                    if bad:
                        return [f"{self.gate}: --allow-raise \"{raise_reason}\" refused: {bad}. {CHAIN_HELP}"], False
                    total_need = tneed.get(row_n, 0)
            elif base_text is not None:
                if tn <= tb:
                    dirs = rest
                else:  # a standing directive keeps naming the tree's current total
                    dirs = rest + self._trim_total(own_dirs, tn) + [d for d in dirs if d.startswith(_DIRECTIVE)
                                                                  and d not in own_dirs]
        if raise_reason is not None and (need.get(row_n) or total_need > 0):
            if not sha or base_text is None:
                return [f"{self.gate}: --allow-raise needs a base to read the grant from (origin/main or "
                        f"origin/queue/*); none is available. {ASK}"], False
            why = self._grant_problem(sha, row_n, need.get(row_n, {}), max(total_need, 0))
            if why:
                return [f"{self.gate}: --allow-raise \"{raise_reason}\" refused: {why}. {ASK}"], False
        os.makedirs(os.path.dirname(os.path.join(self.root, self.baseline)), exist_ok=True)
        with open(os.path.join(self.root, self.baseline), "w", encoding="utf-8") as f:
            f.write(self.render(new_rows, dirs))
        return [], True

    def _next_note(self, path: str, own_note: str, bt: Counts, m: Counts, reason: str, row_n: int
                   ) -> Tuple[str, Optional[str]]:
        """`(note, error)` of a path whose count rose over the baseline: `own_note` (a step made in this diff, or "")
        trimmed to the new counts, plus a step for `row_n` from the number its chain reaches (the base's, in a column
        the chain does not name) to `m`. Only columns above the base's number are raised; none: no note."""
        steps = self._trim(note_steps(own_note) or [], m) if own_note else []
        rising: Dict[str, Tuple[int, int]] = {}
        for c in self.columns:
            ends = [s.deltas[c][1] for s in steps if c in s.deltas]
            end = ends[-1] if ends else bt[c]
            if m[c] > end:
                rising[c] = (end, m[c])
        if rising:
            steps.append(Step(row_n, reason[re.match(r"row\s+\d+", reason).end():].strip(), rising))
            if len(steps) > 1:
                bad = self._chain_problem(steps, bt, m)
                if bad:
                    return "", (f"{self.gate}: --allow-raise \"{reason}\" refused: the note of {path} {bad}. "
                                f"Rewrite it by hand as steps `row N reason (column A->B)` joined by `; `")
        return ("# raised: " + "; ".join(s.text for s in steps)) if steps else "", None


# ---------------------------------------------------------------------------------------------------- self-test
ENV_KEYS = ("RATCHET_BASE_REF", "RATCHET_NO_BASE", "RATCHET_REQUIRE_BASE")


@contextlib.contextmanager
def selftest_env(**overrides: str):
    """The environment of one self-test scenario: the three base-policy variables are cleared (a self-test proves
    the gate's logic and never inherits the caller's `RATCHET_REQUIRE_BASE=1` of a full run or a pinned base), then
    `overrides` are set; the caller's values come back on exit. Every ratchet gate's `--self-test` builds its
    scenarios inside this."""
    saved = {k: os.environ.pop(k, None) for k in ENV_KEYS}
    try:
        os.environ.update(overrides)
        yield
    finally:
        for k in ENV_KEYS:
            os.environ.pop(k, None)
        os.environ.update({k: v for k, v in saved.items() if v is not None})


_GIT_ENV = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid", "GIT_COMMITTER_NAME": "t",
            "GIT_COMMITTER_EMAIL": "t@example.invalid", "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_SYSTEM": os.devnull}
_SOURCE = "".join(f"fn f{k}() -> u32 {{ {k} }}\n" for k in range(12))


def _row(n: int, grant: str, status: str = "APPROVED 2026-11-06") -> str:
    return f'---\nid: {n}\ntitle: "t"\nstatus: "{status}"\njs-grant: {grant}\nadds: []\n---\n\n# Row {n}\n'


def _git_run(root: str, *args: str) -> str:
    r = subprocess.run(["git", "-C", root, *args], capture_output=True, text=True, env={**os.environ, **_GIT_ENV})
    if r.returncode:
        raise RuntimeError(f"git {' '.join(args)}: {r.stderr}")
    return r.stdout.strip()


def _write(root: str, path: str, text: str) -> None:
    full = os.path.join(root, path)
    os.makedirs(os.path.dirname(full) or root, exist_ok=True)
    with open(full, "w", encoding="utf-8") as f:
        f.write(text)


def _self_test() -> int:
    fails: List[str] = []
    ok = 0
    cols = ("a", "b")
    tsv = "b.tsv"

    def total(c: Counts) -> int:
        return c["a"] + c["b"]

    def baseline(rows: Sequence[Tuple[str, int, int, str]], directives: Sequence[str] = (), header: str = "a b") -> str:
        out = [f"#! columns: {header}"] if header else []
        out += list(directives)
        out += ["\t".join([p, str(x), str(y)] + ([n] if n else [])) for p, x, y, n in rows]
        return "\n".join(out) + "\n"

    base_files = {
        tsv: baseline([("x.rs", 2, 0, ""), ("y.rs", 1, 1, "")]),
        "x.rs": _SOURCE, "y.rs": _SOURCE + "// y\n",
        "dev-docs/backlog/50-grant.md": _row(50, "a=10 (ceiling: three lines of witness)"),
        "dev-docs/backlog/52-small.md": _row(52, "a=1"),
        "dev-docs/backlog/53-none.md": _row(53, "none"),
        "dev-docs/backlog/54-landed.md": _row(54, "a=10", status="LANDED 2026-11-01"),
        "dev-docs/backlog/55-first.md": _row(55, "a=3"),
        "dev-docs/backlog/56-second.md": _row(56, "a=2"),
        "dev-docs/backlog/done/51-closed.md": _row(51, "a=10"),
    }
    measured = {"x.rs": {"a": 2, "b": 0}, "y.rs": {"a": 1, "b": 1}}

    def scenario(label: str, want: str, mutate, meas=None, env=None, pin=True, gate_cols=cols, files=None,
                 why: Optional[str] = None, show: Sequence[str] = ()) -> None:
        nonlocal ok
        with selftest_env(**(env or {})), tempfile.TemporaryDirectory(prefix="ratchet-st.") as root:
            _git_run(root, "init", "-q")
            for p, t in (files or base_files).items():
                _write(root, p, t)
            _git_run(root, "add", "-A")
            _git_run(root, "commit", "-q", "-m", "base")
            sha = _git_run(root, "rev-parse", "HEAD")
            mutate(root)
            _git_run(root, "add", "-A")
            R = Ratchet("selftest", tsv, gate_cols, root=root, total=total if gate_cols == cols else None,
                        base_ref=sha if pin else None)
            rep = R.check(meas if meas is not None else measured)
            detail = R.format(rep)
        got = "red" if rep.errors else "green"
        if got != want:
            fails.append(f"{label}: expected {want}, got {got}\n{detail}")
        elif why and why not in detail:
            fails.append(f"{label}: {want}, but {why!r} is not in:\n{detail}")
        else:
            for s in show:
                if s not in detail:
                    fails.append(f"{label}: {s!r} is not printed:\n{detail}")
                    break
            else:
                ok += 1

    def tree(rows, directives=(), header="a b"):
        return lambda root: _write(root, tsv, baseline(rows, directives, header))

    up = {"x.rs": {"a": 5, "b": 0}, "y.rs": {"a": 1, "b": 1}}   # x.rs: a 2 -> 5, the total 4 -> 7

    def raised(n: int, total_n: int = 7, directive: Optional[str] = None):
        d = directive if directive is not None else f"#! total-raised: row {n} total={total_n} witness lines"
        return tree([("x.rs", 5, 0, f"# raised: row {n} witness lines (a 2->5)"), ("y.rs", 1, 1, "")], [d])

    scenario("control: an unchanged tree", "green", lambda r: None)
    scenario("a grant at the base covers the raise", "green", raised(50), meas=up,
             show=("RATCHET RAISES (selftest): own, 2 granted", "x.rs: a 2->5", "TOTAL: 4 -> 7"))
    scenario("a raise with no note", "red", tree([("x.rs", 5, 0, ""), ("y.rs", 1, 1, "")]), meas=up, why="has no `# raised:")
    scenario("a self-service note (the row does not exist)", "red", raised(99), meas=up, why="row 99 does not exist at the base")
    scenario("a malformed note", "red", tree([("x.rs", 5, 0, "# raised: because"), ("y.rs", 1, 1, "")],
                                           ["#! total-raised: row 50 total=7 x"]), meas=up, why="malformed")

    def own_grant(root):
        _write(root, "dev-docs/backlog/60-mine.md", _row(60, "a=100"))
        _git_run(root, "add", "-A")
        _git_run(root, "commit", "-q", "-m", "a grant in the lane's own diff")
        raised(60)(root)
    scenario("a grant added only in the lane's own diff", "red", own_grant, meas=up, why="row 60 does not exist at the base")
    scenario("a closed row (under done/)", "red", raised(51), meas=up, why="is under done/")
    scenario("a closed row (its status says LANDED)", "red", raised(54), meas=up, why="status says it is finished")
    scenario("a row with `js-grant: none`", "red", raised(53), meas=up, why="has no `js-grant")
    scenario("a grant smaller than the delta", "red", raised(52), meas=up, why="grants a=1 at the base, the raise is 3")
    scenario("a grant of another column", "red", tree([("x.rs", 2, 3, "# raised: row 50 b (b 0->3)"), ("y.rs", 1, 1, "")],
                                                      ["#! total-raised: row 50 total=7 b"]),
             meas={"x.rs": {"a": 2, "b": 3}, "y.rs": {"a": 1, "b": 1}}, why="grants b=0 at the base, the raise is 3")
    scenario("a wrong total number", "red", raised(50, total_n=9), meas=up, why="carries total=9 but the tree's total is 7")
    scenario("a total raise with no directive", "red", raised(50, directive="# nothing"), meas=up, why="no new `#! total-raised")
    scenario("a total raise beyond the grant sum", "red", tree(
        [("x.rs", 2, 0, ""), ("y.rs", 1, 12, "# raised: row 52 b (b 1->12)")], ["#! total-raised: row 52 total=14 b"]),
        meas={"x.rs": {"a": 2, "b": 0}, "y.rs": {"a": 1, "b": 12}}, why="row 52 grants")
    scenario("no base under RATCHET_REQUIRE_BASE (no origin ref)", "red", lambda r: None, pin=False,
             env={"RATCHET_REQUIRE_BASE": "1"}, why="RATCHET_REQUIRE_BASE is set and there is no base")
    scenario("no base under RATCHET_REQUIRE_BASE (RATCHET_NO_BASE)", "red", lambda r: None, pin=False,
             env={"RATCHET_REQUIRE_BASE": "1", "RATCHET_NO_BASE": "1"}, why="RATCHET_REQUIRE_BASE")
    scenario("control: a base that exists under RATCHET_REQUIRE_BASE", "green", lambda r: None, env={"RATCHET_REQUIRE_BASE": "1"})
    scenario("control: no base without RATCHET_REQUIRE_BASE compares with the tree only", "green", lambda r: None,
             pin=False, show=("unavailable",))

    def rename(root):
        _git_run(root, "mv", "x.rs", "z.rs")
        _write(root, tsv, baseline([("z.rs", 2, 0, ""), ("y.rs", 1, 1, "")]))
    ren = {"z.rs": {"a": 2, "b": 0}, "y.rs": {"a": 1, "b": 1}}
    scenario("a renamed file carries its base row", "green", rename, meas=ren)

    def copy(root):
        _write(root, "z.rs", _SOURCE)
        _write(root, tsv, baseline([("x.rs", 2, 0, ""), ("z.rs", 2, 0, ""), ("y.rs", 1, 1, "")]))
    scenario("control: a copy (not a rename) is a new file and a raise", "red", copy,
             meas={"x.rs": {"a": 2, "b": 0}, "z.rs": {"a": 2, "b": 0}, "y.rs": {"a": 1, "b": 1}}, why="rises above the base")

    inherited = dict(base_files)
    inherited[tsv] = baseline([("x.rs", 5, 0, "# raised: row 50 earlier (a 2->5)"), ("y.rs", 1, 1, "")],
                              ["#! total-raised: row 50 total=7 earlier"])

    def rename_inherited(root):
        _git_run(root, "mv", "x.rs", "z.rs")
        _write(root, tsv, baseline([("z.rs", 5, 0, "# raised: row 50 earlier (a 2->5)"), ("y.rs", 1, 1, "")],
                                   ["#! total-raised: row 50 total=7 earlier"]))
    scenario("inherited raises print apart from own, and a rename carries them", "green", rename_inherited,
             meas={"z.rs": {"a": 5, "b": 0}, "y.rs": {"a": 1, "b": 1}}, files=inherited,
             show=("RATCHET RAISES (selftest): inherited, 2", "z.rs (carried from x.rs)"))
    scenario("a note the base already has cannot be spent again", "red", tree(
        [("x.rs", 8, 0, "# raised: row 50 earlier (a 2->5)"), ("y.rs", 1, 1, "")], ["#! total-raised: row 50 total=10 earlier"]),
        meas={"x.rs": {"a": 8, "b": 0}, "y.rs": {"a": 1, "b": 1}}, files=inherited, why="the base already has")

    c3 = ("a", "b", "c")
    scenario("a column the base lacks is compared with the tree only and printed as introduced", "green",
             lambda r: _write(r, tsv, "#! columns: a b c\nx.rs\t2\t0\t4\ny.rs\t1\t1\t0\n"),
             meas={"x.rs": {"a": 2, "b": 0, "c": 4}, "y.rs": {"a": 1, "b": 1}}, gate_cols=c3,
             show=("column(s) c introduced",))
    scenario("an introduced column that the tree does not match still fails", "red",
             lambda r: _write(r, tsv, "#! columns: a b c\nx.rs\t2\t0\t4\ny.rs\t1\t1\t0\n"),
             meas={"x.rs": {"a": 2, "b": 0, "c": 5}, "y.rs": {"a": 1, "b": 1}}, gate_cols=c3, why="count rose above the baseline")
    scenario("a tree header that differs from the gate's columns", "red",
             tree([("x.rs", 2, 0, ""), ("y.rs", 1, 1, "")], header="a c"), why="declares")

    # a note of several steps: two granted rows raise one file, each step is charged to its own row
    def x_to(a: int) -> Dict[str, Counts]:
        return {"x.rs": {"a": a, "b": 0}, "y.rs": {"a": 1, "b": 1}}

    def chain(a: int, steps: Sequence[str], dirs: Sequence[str] = ()):
        note = "# raised: " + "; ".join(steps) if steps else ""
        return tree([("x.rs", a, 0, note), ("y.rs", 1, 1, "")], list(dirs))

    def tot(*steps: Tuple[int, str, int, int]) -> List[str]:   # (row, reason, from, to) of the total
        return [f"#! total-raised: row {n} total={e} {w} (total {s0}->{e})" for n, w, s0, e in steps]

    scenario("a chain of two rows, each step inside its own grant", "green",
             chain(5, ["row 55 first (a 2->4)", "row 56 second (a 4->5)"],
                   tot((55, "first", 4, 6), (56, "second", 6, 7))), meas=x_to(5),
             show=("row 55 first (a 2->4)", "row 56 second (a 4->5)", "x.rs: a 2->4", "x.rs: a 4->5", "TOTAL: 4 -> 7"))
    scenario("a chain of three rows, each step inside its own grant", "green",
             chain(6, ["row 55 first (a 2->4)", "row 56 second (a 4->5)", "row 50 third (a 5->6)"],
                   tot((55, "first", 4, 6), (56, "second", 6, 7), (50, "third", 7, 8))), meas=x_to(6))
    scenario("a chain: step 2 over its row's grant is red and names that row", "red",
             chain(6, ["row 50 first (a 2->4)", "row 52 second (a 4->6)"], tot((50, "first", 4, 6), (52, "second", 6, 8))),
             meas=x_to(6), why="the raise cited as row 52 (a +2) is not granted: row 52 grants a=1 at the base, the raise is 2")
    scenario("a chain: step 1 over its row's grant, row 2 with room to spare, is red (no borrowing)", "red",
             chain(5, ["row 52 first (a 2->4)", "row 50 second (a 4->5)"], tot((52, "first", 4, 6), (50, "second", 6, 7))),
             meas=x_to(5), why="the raise cited as row 52 (a +2) is not granted")
    scenario("a chain with a gap (step 2 does not start where step 1 ends)", "red",
             chain(6, ["row 55 first (a 2->4)", "row 56 second (a 5->6)"], tot((55, "first", 4, 6), (56, "second", 6, 8))),
             meas=x_to(6), why="has a gap in a: step `row 56` starts at 5 but the chain before it is at 4")
    scenario("a chain that starts above the base's number", "red",
             chain(6, ["row 55 first (a 3->4)", "row 56 second (a 4->6)"], tot((55, "first", 4, 5), (56, "second", 5, 8))),
             meas=x_to(6), why="step `row 55` starts at 3 but the chain before it is at 2")
    scenario("a chain that ends below the tree's number", "red",
             chain(6, ["row 55 first (a 2->4)", "row 56 second (a 4->5)"], tot((55, "first", 4, 6), (56, "second", 6, 8))),
             meas=x_to(6), why="ends at a=5 but the tree has a=6")
    scenario("a chain whose first step has no from -> to", "red",
             chain(5, ["row 55 first", "row 56 second (a 4->5)"], tot((55, "first", 4, 6), (56, "second", 6, 7))),
             meas=x_to(5), why="names no `(column A->B)`")
    scenario("a chain citing a row that is closed at the base", "red",
             chain(5, ["row 55 first (a 2->4)", "row 51 second (a 4->5)"], tot((55, "first", 4, 6), (51, "second", 6, 7))),
             meas=x_to(5), why="row 51 is closed at the base")
    scenario("a chain citing a row that does not exist", "red",
             chain(5, ["row 55 first (a 2->4)", "row 99 second (a 4->5)"], tot((55, "first", 4, 6), (99, "second", 6, 7))),
             meas=x_to(5), why="row 99 does not exist at the base")

    def own_grant_chain(root):
        _write(root, "dev-docs/backlog/60-mine.md", _row(60, "a=100"))
        _git_run(root, "add", "-A")
        _git_run(root, "commit", "-q", "-m", "a grant in the lane's own diff")
        chain(5, ["row 55 first (a 2->4)", "row 60 second (a 4->5)"], tot((55, "first", 4, 6), (60, "second", 6, 7)))(root)
    scenario("a chain citing a grant written in the lane's own diff", "red", own_grant_chain, meas=x_to(5),
             why="row 60 does not exist at the base")
    scenario("a chain whose total steps leave a gap", "red",
             chain(5, ["row 55 first (a 2->4)", "row 56 second (a 4->5)"], tot((55, "first", 4, 6), (56, "second", 5, 7))),
             meas=x_to(5), why="the total chain has a gap")
    scenario("a chain whose total step is over its row's grant sum", "red",
             chain(6, ["row 55 first (a 2->4)", "row 56 second (a 4->6)"], tot((55, "first", 4, 5), (56, "second", 5, 8))),
             meas=x_to(6), why="the total raise 4 -> 8 cited as row 56 is not granted: row 56 grants 2 in all at the base, the total raise is 3")
    scenario("a chain whose total steps end at another number than the tree's", "red",
             chain(5, ["row 55 first (a 2->4)", "row 56 second (a 4->5)"], tot((55, "first", 4, 6), (56, "second", 6, 9))),
             meas=x_to(5), why="the total chain ends at 9 but the tree's total is 7")
    scenario("control: a note of one step keeps the old reading (no from -> to needed)", "green",
             chain(5, ["row 50 witness lines"], tot((50, "witness lines", 4, 7))), meas=x_to(5))

    # the writer: `--allow-raise` writes a note only where the grant at the base already covers it
    def writer(label: str, reason: str, want_written: bool, why: str = "") -> None:
        nonlocal ok
        with selftest_env(), tempfile.TemporaryDirectory(prefix="ratchet-st.") as root:
            _git_run(root, "init", "-q")
            for p, t in base_files.items():
                _write(root, p, t)
            _git_run(root, "add", "-A")
            _git_run(root, "commit", "-q", "-m", "base")
            R = Ratchet("selftest", tsv, cols, root=root, total=total, base_ref=_git_run(root, "rev-parse", "HEAD"))
            errors, wrote = R.update(up, reason)
            rep = R.check(up) if wrote else None
        if wrote != want_written or (why and not any(why in e for e in errors)) or (rep and rep.errors):
            fails.append(f"{label}: wrote={wrote}, errors={errors}, check={rep.errors if rep else None}")
        else:
            ok += 1
    writer("--allow-raise against a grant at the base writes a note the check accepts", "row 50 witness lines", True)
    writer("--allow-raise against a row that does not exist is refused", "row 99 self-service", False, "row 99 does not exist")
    writer("--allow-raise against a grant that is too small is refused", "row 52 small", False, "grants a=1")

    # the writer on a chain: each run adds the step of its own row, a refusal leaves the file as it was
    def writer_chain(label: str, runs: Sequence[Tuple[int, Optional[str], bool, str]], base_overrides=None,
                     want: Sequence[str] = (), absent: Sequence[str] = (), green: bool = True) -> None:
        nonlocal ok
        files = {**base_files, **(base_overrides or {})}
        problems: List[str] = []
        with selftest_env(), tempfile.TemporaryDirectory(prefix="ratchet-st.") as root:
            _git_run(root, "init", "-q")
            for p, t in files.items():
                _write(root, p, t)
            _git_run(root, "add", "-A")
            _git_run(root, "commit", "-q", "-m", "base")
            R = Ratchet("selftest", tsv, cols, root=root, total=total, base_ref=_git_run(root, "rev-parse", "HEAD"))
            last = x_to(2)
            for a, reason, want_written, why in runs:
                before = open(os.path.join(root, tsv), encoding="utf-8").read()
                errors, wrote = R.update(x_to(a), reason)
                after = open(os.path.join(root, tsv), encoding="utf-8").read()
                if wrote != want_written or (why and not any(why in e for e in errors)):
                    problems.append(f"run a={a} {reason!r}: wrote={wrote}, errors={errors}")
                if not wrote and after != before:
                    problems.append(f"run a={a} {reason!r}: refused but the file changed")
                if wrote:
                    last = x_to(a)
            body = open(os.path.join(root, tsv), encoding="utf-8").read()
            rep = R.check(last)
        problems += [f"missing {w!r} in:\n{body}" for w in want if w not in body]
        problems += [f"unexpected {w!r} in:\n{body}" for w in absent if w in body]
        if green and rep.errors:
            problems.append(f"check is red: {rep.errors}")
        if problems:
            fails.append(f"{label}: " + "; ".join(problems))
        else:
            ok += 1
    writer_chain("the writer appends the second row's step to the first's", [
        (4, "row 55 first", True, ""), (5, "row 56 second", True, "")],
        want=("# raised: row 55 first (a 2->4); row 56 second (a 4->5)",
              "total-raised: row 55 total=6 first (total 4->6)", "total-raised: row 56 total=7 second (total 6->7)"))
    writer_chain("the writer appends a third step", [
        (4, "row 55 first", True, ""), (5, "row 56 second", True, ""), (6, "row 50 third", True, "")],
        want=("; row 50 third (a 5->6)", "total=8 third (total 7->8)"))
    writer_chain("the writer refuses a step its row's grant does not cover, and leaves the file untouched", [
        (4, "row 55 first", True, ""), (7, "row 52 second", False, "row 52 grants a=1 at the base, the raise is 3")],
        want=("# raised: row 55 first (a 2->4)",), absent=("row 52",))
    writer_chain("the writer refuses a step over the row's grant even when another row has room (no borrowing)", [
        (4, "row 52 first", False, "row 52 grants a=1 at the base, the raise is 2")], absent=("row 52",), green=False)
    writer_chain("the writer refuses a step of a row that does not exist", [
        (4, "row 55 first", True, ""), (5, "row 99 second", False, "row 99 does not exist")], absent=("row 99",))
    writer_chain("the writer refuses a reason that holds a step separator", [
        (4, "row 55 first; row 56 sneaky", False, "must not contain")], green=False)
    writer_chain("a count that falls again trims the chain", [
        (4, "row 55 first", True, ""), (5, "row 56 second", True, ""), (4, None, True, "")],
        want=("# raised: row 55 first (a 2->4)",), absent=("row 56", "(a 4->5)", "total=7"))
    writer_chain("a count back at the base drops the note and the total line", [
        (4, "row 55 first", True, ""), (5, "row 56 second", True, ""), (2, None, True, "")], absent=("raised", "row 5"))
    inherited_chain = {tsv: baseline([("x.rs", 5, 0, "# raised: row 50 earlier (a 2->5)"), ("y.rs", 1, 1, "")],
                                     ["#! total-raised: row 50 total=7 earlier"])}
    with selftest_env(), tempfile.TemporaryDirectory(prefix="ratchet-st.") as root:
        _git_run(root, "init", "-q")
        for p, t in {**base_files, **inherited_chain}.items():
            _write(root, p, t)
        _git_run(root, "add", "-A")
        _git_run(root, "commit", "-q", "-m", "base")
        R = Ratchet("selftest", tsv, cols, root=root, total=total, base_ref=_git_run(root, "rev-parse", "HEAD"))
        errors, wrote = R.update(x_to(6), "row 56 later")
        body = open(os.path.join(root, tsv), encoding="utf-8").read()
        rep = R.check(x_to(6))
    if not wrote or "row 50" in body or "# raised: row 56 later (a 5->6)" not in body or rep.errors:
        fails.append(f"a note the base carries is replaced, not extended: wrote={wrote} {errors} {rep.errors}\n{body}")
    else:
        ok += 1

    # the grant reader, on the real forms
    from rows import parse_columns as pc, parse_list as pl
    for label, got, want in (
            ("js-grant with a remark", pc("js_file=360 (inferred ceiling: witness.ts about 300 that never ships; a=1)"), {"js_file": 360}),
            ("js-grant none", pc("none"), {}), ("two columns", pc("sink=1, js_lit=20"), {"sink": 1, "js_lit": 20}),
            ("adds", pl("[otp_filter, pan]"), ["otp_filter", "pan"]), ("empty adds", pl("[]"), [])):
        if got != want:
            fails.append(f"rows.py {label}: {got!r} != {want!r}")
        else:
            ok += 1
    row = row_at(".", "HEAD", 446)
    if row is not None:
        # The live row is read for its shape, never for today's number: an open row with a grant in a real column.
        if row.open and row.grant and not row.problems:
            ok += 1
        else:
            fails.append(f"rows.py: row 446 at HEAD reads as grant {row.grant}, open {row.open}, problems {row.problems}")

    for f in fails:
        print("ratchet --self-test FAIL: " + f, file=sys.stderr)
    if fails:
        return 2
    print(f"ratchet --self-test: OK ({ok} scenarios)")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        sys.exit(_self_test())
    print("usage: ratchet.py --self-test", file=sys.stderr)
    sys.exit(2)
