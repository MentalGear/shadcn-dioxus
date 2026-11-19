#!/usr/bin/env python3
"""report-receipts.py -- check the receipts behind a delegated report's findings, mechanically.

    scripts/report-receipts.py LEDGER.jsonl [LEDGER2.jsonl ...] [--repo PATH]
    scripts/report-receipts.py --self-test

A delegated report's findings sit in a ledger beside it, one JSON object per line: `id`, `claim`, `verdict` (CONFIRMED,
PARTLY, REFUTED or UNVERIFIABLE), `actual` and `receipts`. A receipt is a `quote` (file, line, text copied verbatim;
read in the working tree, a moved line is accepted and reported), a `count` (a ripgrep `pattern`, `globs`, `mode`
"matches" or "files", the `expected` number; re-run, and a failing one prints the per-file numbers for up to five
files), a `run` (the absolute path of a log with the command and its unedited output; present, not re-run) or a
`judgement` (the claim is an opinion; a top-level `"kind": "judgement"` says the same). The rule is
agent-rules/delegation-and-models.md; the main loop runs this before it relays a report.

Statuses: VERIFIED (every receipt holds), LOGGED-ONLY (only `run` receipts), OPINION (marked judgement), UNBACKED (no
receipts), BROKEN (a receipt fails, or the line is malformed), UNVERIFIABLE (the report says so).
Exit status: 0 when no claim is BROKEN or UNBACKED, 1 otherwise, 2 when the repo or a ledger cannot be read.
`--self-test` runs the fixtures against a throwaway repo (scripts/check-report-receipts.sh runs it as a gate).
"""
import argparse
import bisect
import collections
import json
import os
import re
import subprocess
import sys
import tempfile

VERDICTS = ("CONFIRMED", "PARTLY", "REFUTED", "UNVERIFIABLE")
STATUSES = ("VERIFIED", "LOGGED-ONLY", "OPINION", "UNBACKED", "BROKEN", "UNVERIFIABLE")
RG_TIMEOUT_S = 120
PER_FILE_LIMIT = 5
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def norm(text):
    return " ".join(text.split())


def read_text(path):
    with open(path, encoding="utf-8", errors="replace") as fh:
        return fh.read()


def check_quote(r, repo):
    file_ = r.get("file")
    line = r.get("line")
    text = r.get("text")
    if not isinstance(file_, str) or not file_:
        return "FAIL", "quote: 'file' must be a non-empty string"
    if type(line) is not int or line < 1:
        return "FAIL", "quote: 'line' must be an integer >= 1"
    if not isinstance(text, str) or not norm(text):
        return "FAIL", "quote: 'text' must be a non-empty string"
    path = file_ if os.path.isabs(file_) else os.path.join(repo, file_)
    if not os.path.isfile(path):
        return "FAIL", f"quote: file not found: {path}"
    try:
        lines = read_text(path).split("\n")
    except OSError as exc:
        return "FAIL", f"quote: cannot read {path}: {exc}"
    needle = norm(text)
    span = len(text.splitlines())
    lo = max(1, line - 3)
    hi = min(len(lines), line + span + 3)
    if lo <= hi and needle in norm(" ".join(lines[lo - 1:hi])):
        return "PASS", ""
    # Whole-file search. The non-empty normalised lines joined by single
    # spaces equal norm(whole file); map the match offset back to a line.
    parts, starts, owners, offset = [], [], [], 0
    for number, raw in enumerate(lines, start=1):
        seg = norm(raw)
        if not seg:
            continue
        if parts:
            offset += 1
        parts.append(seg)
        starts.append(offset)
        owners.append(number)
        offset += len(seg)
    hit = " ".join(parts).find(needle)
    if hit < 0:
        return "FAIL", (f"quote: text not in {file_} at lines {lo}-{hi} "
                        f"or anywhere else in the file")
    found = owners[bisect.bisect_right(starts, hit) - 1]
    return "MOVED", f"quote: found at line {found} in {file_} (claimed line {line})"


def per_file_numbers(rows, mode):
    """The first PER_FILE_LIMIT rg rows as `file=n` (matches) or `file` (files), and how many more there are."""
    shown = []
    for row in rows[:PER_FILE_LIMIT]:
        if mode == "matches":
            name, _, number = row.rpartition(":")
            shown.append(f"{name}={number}")
        else:
            shown.append(row)
    more = len(rows) - len(shown)
    return ", ".join(shown) + (f", ... {more} more files" if more > 0 else "")


def check_count(r, repo):
    pattern = r.get("pattern")
    globs = r.get("globs", [])
    mode = r.get("mode")
    expected = r.get("expected")
    if not isinstance(pattern, str) or not pattern:
        return "FAIL", "count: 'pattern' must be a non-empty string"
    if not isinstance(globs, list) or not all(isinstance(g, str) for g in globs):
        return "FAIL", "count: 'globs' must be a list of strings"
    if mode not in ("matches", "files"):
        return "FAIL", "count: 'mode' must be 'matches' or 'files'"
    if type(expected) is not int or expected < 0:
        return "FAIL", "count: 'expected' must be an integer >= 0"
    args = ["rg", "--no-config", "--no-messages"]
    for glob in globs:
        args += ["-g", glob]
    args += ["--count-matches" if mode == "matches" else "--files-with-matches",
             "-e", pattern, "."]
    try:
        proc = subprocess.run(args, cwd=repo, capture_output=True, text=True,
                              timeout=RG_TIMEOUT_S, stdin=subprocess.DEVNULL,
                              check=False)
    except subprocess.TimeoutExpired:
        return "FAIL", f"count: rg timed out after {RG_TIMEOUT_S} s"
    except FileNotFoundError:
        return "FAIL", "count: rg is not on PATH"
    if proc.returncode not in (0, 1):  # 1 means zero matches, not an error
        first = (proc.stderr.strip().splitlines() or ["no stderr"])[0]
        return "FAIL", f"count: rg exited {proc.returncode}: {first}"
    rows = sorted(row[2:] if row.startswith("./") else row for row in proc.stdout.splitlines() if row.strip())
    if mode == "matches":
        got = 0
        for row in rows:
            try:
                got += int(row.rsplit(":", 1)[1])
            except (IndexError, ValueError):
                return "FAIL", f"count: unparseable rg output line {row!r}"
    else:
        got = len(rows)
    if got == expected:
        return "PASS", ""
    message = (f"count: got {got}, expected {expected} "
               f"(pattern {pattern!r}, globs {globs}, mode {mode})")
    if rows:
        message += f"; per file: {per_file_numbers(rows, mode)}"
    return "FAIL", message


def check_run(r):
    log = r.get("log")
    if not isinstance(log, str) or not os.path.isabs(log):
        return "FAIL", "run: 'log' must be an absolute path"
    if not os.path.isfile(log):
        return "FAIL", f"run: log missing: {log}"
    try:
        content = read_text(log)
    except OSError as exc:
        return "FAIL", f"run: cannot read {log}: {exc}"
    lines = content.splitlines()
    if not content.strip() or len(lines) < 2:
        return "FAIL", (f"run: log needs at least 2 lines (command, output), "
                        f"has {len(lines)}: {log}")
    return "LOGGED", f"run: not re-run; {log} first line: {lines[0]}"


def is_judgement(receipt):
    return isinstance(receipt, dict) and receipt.get("kind") == "judgement"


def check_receipt(r, repo):
    if not isinstance(r, dict):
        return "FAIL", "malformed receipt: not a JSON object"
    kind = r.get("kind")
    if kind == "quote":
        return check_quote(r, repo)
    if kind == "count":
        return check_count(r, repo)
    if kind == "run":
        return check_run(r)
    return "FAIL", f"unknown receipt kind {kind!r}"


def judge(entry, repo):
    """Return (status, notes); notes is a list of (outcome, message)."""
    if entry["verdict"] == "UNVERIFIABLE":
        return "UNVERIFIABLE", []
    receipts = entry.get("receipts", [])
    if not isinstance(receipts, list):
        return "BROKEN", [("FAIL", "'receipts' must be a list")]
    opinion = entry.get("kind") == "judgement" or any(is_judgement(r) for r in receipts)
    notes = [check_receipt(r, repo) for r in receipts if not is_judgement(r)]
    if any(outcome == "FAIL" for outcome, _ in notes):
        return "BROKEN", notes
    if opinion:
        return "OPINION", notes
    if not notes:
        return "UNBACKED", notes
    if all(outcome == "LOGGED" for outcome, _ in notes):
        return "LOGGED-ONLY", notes
    return "VERIFIED", notes


def process_ledger(path, repo):
    rows = []
    for number, raw in enumerate(read_text(path).split("\n"), start=1):
        if not raw.strip():
            continue
        cid = f"line {number}"
        try:
            entry = json.loads(raw)
        except json.JSONDecodeError as exc:
            rows.append(("BROKEN", cid, "-", "(malformed JSON)",
                         [("FAIL", f"line {number}: malformed JSON: {exc.msg}")]))
            continue
        if not isinstance(entry, dict):
            rows.append(("BROKEN", cid, "-", "(not a JSON object)",
                         [("FAIL", f"line {number}: not a JSON object")]))
            continue
        if isinstance(entry.get("id"), str):
            cid = entry["id"]
        claim = entry.get("claim") if isinstance(entry.get("claim"), str) else ""
        verdict = entry.get("verdict")
        if verdict not in VERDICTS:
            rows.append(("BROKEN", cid, str(verdict), claim,
                         [("FAIL", "'verdict' must be one of " + ", ".join(VERDICTS))]))
            continue
        status, notes = judge(entry, repo)
        rows.append((status, cid, verdict, claim, notes))
    return rows


def report(path, rows):
    print(f"== {path}")
    counts = collections.Counter(row[0] for row in rows)
    for status, cid, verdict, claim, notes in rows:
        print(f"{status:<12}  {cid}  {verdict}  {' '.join(claim.split())[:100]}")
        for outcome, message in notes:
            if outcome != "PASS":
                print(f"    {outcome:<6} {message}")
    print(f"  totals ({len(rows)} claims): " + "  ".join(f"{s}={counts[s]}" for s in STATUSES))
    return counts


def failing(counts):
    return counts["BROKEN"] > 0 or counts["UNBACKED"] > 0


# ---- self-test -------------------------------------------------------------------------------------------------

STATUS_ROW = re.compile(r"^(" + "|".join(STATUSES) + r")\s+(line \d+|\S+)\s")
A_TXT = ["alpha one", "beta two", "gamma  three   spaced", "delta four", "epsilon five", "zeta six", "eta seven",
         "theta eight", "iota nine", "kappa ten", "lambda eleven", "mu twelve", "nu thirteen", "xi fourteen",
         "omicron fifteen", "pi sixteen", "rho seventeen", "sigma eighteen", "tau nineteen", "upsilon twenty"]
B_TXT = ["bravo one", "bravo two", "bravo bravo"]


def _quote(file_, line, text):
    return {"kind": "quote", "file": file_, "line": line, "text": text}


def _count(pattern, globs, mode, expected):
    return {"kind": "count", "pattern": pattern, "globs": globs, "mode": mode, "expected": expected}


def _claim(cid, verdict, receipts, **extra):
    entry = {"id": cid, "claim": f"{cid} fixture", "verdict": verdict, "actual": "fixture", "receipts": receipts}
    entry.update(extra)
    return entry


def _write(path, text):
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def _statuses(stdout):
    found = {}
    for row in stdout.splitlines():
        match = STATUS_ROW.match(row)
        if match:
            found[match.group(2)] = match.group(1)
    return found


def _run_tool(*args):
    return subprocess.run([sys.executable, os.path.abspath(__file__), *args], capture_output=True, text=True,
                          timeout=300, check=False)


def self_test():
    checks = []

    def check(label, ok, detail=""):
        checks.append((label, bool(ok), detail))

    with tempfile.TemporaryDirectory() as tmp:
        repo = os.path.join(tmp, "repo")
        os.mkdir(repo)
        _write(os.path.join(repo, "a.txt"), "\n".join(A_TXT) + "\n")
        _write(os.path.join(repo, "b.txt"), "\n".join(B_TXT) + "\n")
        for number in range(7):
            _write(os.path.join(repo, f"many{number}.txt"), "needle\n" * (number + 1))
        run_ok = os.path.join(tmp, "run_ok.log")
        _write(run_ok, "$ pytest -q\n3 passed\n")
        run_missing = os.path.join(tmp, "run_missing.log")

        entries = [
            _claim("Q-pass", "CONFIRMED", [_quote("a.txt", 2, "beta two")]),
            _claim("Q-moved", "CONFIRMED", [_quote("a.txt", 15, "alpha one")]),
            _claim("Q-fail", "CONFIRMED", [_quote("a.txt", 5, "nonexistent phrase")]),
            _claim("Q-ws", "CONFIRMED", [_quote("a.txt", 3, "gamma three spaced")]),
            _claim("Q-nofile", "CONFIRMED", [_quote("missing.txt", 1, "alpha one")]),
            _claim("C-match", "CONFIRMED", [_count("bravo", ["b.txt"], "matches", 4)]),
            _claim("C-off", "CONFIRMED", [_count("bravo", ["b.txt"], "matches", 5)]),
            _claim("C-zero", "CONFIRMED", [_count("zzz-no-such-token", ["b.txt"], "matches", 0)]),
            _claim("C-files", "CONFIRMED", [_count("bravo", ["b.txt"], "files", 1)]),
            _claim("C-many", "CONFIRMED", [_count("needle", ["many*.txt"], "matches", 1)]),
            _claim("C-many-files", "CONFIRMED", [_count("needle", ["many*.txt"], "files", 1)]),
            _claim("R-ok", "CONFIRMED", [{"kind": "run", "log": run_ok}]),
            _claim("R-missing", "CONFIRMED", [{"kind": "run", "log": run_missing}]),
            _claim("J-top", "CONFIRMED", [_quote("a.txt", 2, "beta two")], kind="judgement"),
            _claim("J-top-bare", "CONFIRMED", [], kind="judgement"),
            _claim("J-receipt", "PARTLY", [{"kind": "judgement"}]),
            _claim("J-mixed", "CONFIRMED", [_quote("a.txt", 2, "beta two"), {"kind": "judgement"}]),
            _claim("J-broken", "CONFIRMED", [_quote("a.txt", 5, "nonexistent phrase"), {"kind": "judgement"}]),
            _claim("C-none", "PARTLY", []),
            _claim("U1", "UNVERIFIABLE", []),
            _claim("U-judge", "UNVERIFIABLE", [{"kind": "judgement"}]),
            _claim("R-ref", "REFUTED", [_quote("a.txt", 1, "alpha one")]),
        ]
        lines = [json.dumps(entry) for entry in entries]
        lines.append('{"id": "M1", "claim": "truncated')  # malformed JSON
        malformed_id = f"line {len(lines)}"
        ledger1 = os.path.join(tmp, "ledger1.jsonl")
        _write(ledger1, "\n".join(lines) + "\n")

        ledger2 = os.path.join(tmp, "ledger2.jsonl")
        _write(ledger2, json.dumps(_claim("X1", "CONFIRMED", [_quote("b.txt", 2, "bravo two")])) + "\n")

        ledger3 = os.path.join(tmp, "ledger3.jsonl")
        _write(ledger3, "\n".join(json.dumps(e) for e in (
            _claim("OP", "CONFIRMED", [{"kind": "judgement"}]),
            _claim("LG", "CONFIRMED", [{"kind": "run", "log": run_ok}]),
            _claim("UV", "UNVERIFIABLE", []))) + "\n")

        ledger4 = os.path.join(tmp, "ledger4.jsonl")
        _write(ledger4, json.dumps(_claim("NB", "CONFIRMED", [])) + "\n")

        run_a = _run_tool(ledger1, ledger2, "--repo", repo)
        run_b = _run_tool(ledger2, "--repo", repo)
        run_c = _run_tool(ledger3, "--repo", repo)
        run_d = _run_tool(ledger4, "--repo", repo)
        statuses = _statuses(run_a.stdout)

        expected = {
            "Q-pass": "VERIFIED", "Q-moved": "VERIFIED", "Q-fail": "BROKEN",
            "Q-ws": "VERIFIED", "Q-nofile": "BROKEN", "C-match": "VERIFIED",
            "C-off": "BROKEN", "C-zero": "VERIFIED", "C-files": "VERIFIED",
            "C-many": "BROKEN", "C-many-files": "BROKEN",
            "R-ok": "LOGGED-ONLY", "R-missing": "BROKEN",
            "J-top": "OPINION", "J-top-bare": "OPINION", "J-receipt": "OPINION", "J-mixed": "OPINION",
            "J-broken": "BROKEN", "C-none": "UNBACKED", "U1": "UNVERIFIABLE", "U-judge": "UNVERIFIABLE",
            "R-ref": "VERIFIED", malformed_id: "BROKEN", "X1": "VERIFIED",
        }
        for cid, want in expected.items():
            got = statuses.get(cid)
            check(f"status {cid}", got == want, f"got {got}, want {want}")

        check("exit 1 with BROKEN claims", run_a.returncode == 1, f"got {run_a.returncode}")
        check("exit 0, clean ledger", run_b.returncode == 0, f"got {run_b.returncode}")
        check("exit 0, only OPINION, LOGGED-ONLY, UNVERIFIABLE", run_c.returncode == 0, f"got {run_c.returncode}")
        check("statuses of the exit-0 ledger", _statuses(run_c.stdout) ==
              {"OP": "OPINION", "LG": "LOGGED-ONLY", "UV": "UNVERIFIABLE"}, str(_statuses(run_c.stdout)))
        check("exit 1, UNBACKED alone", run_d.returncode == 1 and _statuses(run_d.stdout).get("NB") == "UNBACKED",
              f"got {run_d.returncode}")
        check("totals line has OPINION", "OPINION=4" in run_a.stdout, "totals line")
        check("MOVED reports the found line", "found at line 1 " in run_a.stdout)
        check("count FAIL shows got vs expected", "got 4, expected 5" in run_a.stdout)
        check("count FAIL lists per-file numbers", "per file: many0.txt=1" in run_a.stdout
              and "many1.txt=2" in run_a.stdout)
        check("count FAIL caps the per-file list at five", "many4.txt=5, ... 2 more files" in run_a.stdout)
        check("files-mode FAIL lists file names", "per file: many0.txt, many1.txt" in run_a.stdout)
        check("malformed JSON is reported", "malformed JSON" in run_a.stdout)

    passed = sum(1 for _, ok, _ in checks if ok)
    for label, ok, detail in checks:
        if not ok:
            print(f"MISMATCH {label}: {detail}")
    print(f"selftest: {passed}/{len(checks)} passed")
    return 0 if passed == len(checks) else 1


def main(argv=None):
    parser = argparse.ArgumentParser(description="Mechanically check a delegated report's finding receipts.")
    parser.add_argument("ledgers", nargs="*", help="JSONL ledger files")
    parser.add_argument("--repo", default=ROOT,
                        help="repository for relative quote paths and rg (default: this checkout)")
    parser.add_argument("--self-test", action="store_true", help="run the fixtures and exit")
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()
    if not args.ledgers:
        parser.error("a ledger is required (or --self-test)")
    repo = os.path.abspath(args.repo)
    if not os.path.isdir(repo):
        print(f"error: repo is not a directory: {repo}", file=sys.stderr)
        return 2
    overall = collections.Counter()
    for path in args.ledgers:
        try:
            rows = process_ledger(path, repo)
        except OSError as exc:
            print(f"error: cannot read ledger {path}: {exc}", file=sys.stderr)
            return 2
        overall.update(report(path, rows))
    print(f"== overall ({sum(overall.values())} claims): "
          + "  ".join(f"{s}={overall[s]}" for s in STATUSES))
    return 1 if failing(overall) else 0


if __name__ == "__main__":
    sys.exit(main())
