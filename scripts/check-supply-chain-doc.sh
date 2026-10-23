#!/usr/bin/env bash
#
# check-supply-chain-doc.sh
#
# `dev-docs/supply-chain/README.md` is the repository's supply-chain playbook (owner ask 2026-10-23: document the
# dependency and repo security properly and separately, so that other projects can inspect and lift the approach).
# A playbook that names controls is only worth copying while it is true, so the controls table is made a property of
# the tree, not of a reviewer's memory. By construction this gate parses the table (`## Controls`, one row per
# control: ID, Control, Threat it stops, Enforcing script, Config, Runs in, Status, Adopt) and fails on:
#   1. an `enforced` or `manual` row that names no repository path, or names a script or config path (a backticked
#      token with a `/` or a file extension; a `*` is a glob) that does not exist;
#   2. a `planned` row whose script already exists (the row must be flipped to `enforced` or `manual`: a planned
#      control that landed silently is documentation drift);
#   3. a script in SUPPLY_CHAIN_SCRIPTS (the declared set kept at the top of this file) that no row names, or one
#      that no longer exists: a new supply-chain script cannot land without a table row, and a deleted one cannot
#      stay documented. Adding a supply-chain script means adding it to that list and to the table in one commit;
#   4. a row whose "Runs in" is `run-gates` and whose `scripts/check-*.sh` is not in scripts/run-gates.sh's gate list;
#   5. a status other than enforced / manual / planned, a repeated or malformed id, a row with the wrong number of
#      cells, an id (`C01`..) used anywhere in the document that has no row, and a "Counts" line that disagrees with
#      the table;
#   6. a missing required heading, or a SECURITY.md that is missing or does not link the playbook.
# `--self-test` mutates an in-memory copy of the table (and a throwaway tree that mirrors which paths exist) twelve
# ways, every one of which must turn red, and requires the unmutated table to stay green. No network, no cargo.
#
# Usage: scripts/check-supply-chain-doc.sh [--self-test]     (a plain run also runs the self-test, quietly)
# Overrides for tests: SCD_DOC (the playbook path). Exit status: 0 when true, 1 on drift, 2 when a source is unreadable.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# The declared supply-chain set: every script here must be named in the controls table (any status), and must exist.
SUPPLY_CHAIN_SCRIPTS=(
  scripts/check-cargo-locks.sh
  scripts/check-cargo-dep-versions.sh
  scripts/check-markdown-parser.sh
  scripts/check-primitives-features.sh
  scripts/check-installed-paths.sh
  scripts/check-js-bundle.sh
  scripts/build-js-bundle.sh
  scripts/check-registry-url.sh
  scripts/check-demo-remote-assets.sh
  scripts/registry-install-smoke.sh
  scripts/stamp-release-rev.sh
  scripts/dep-report.sh
  scripts/session-setup.sh
  scripts/check-supply-chain-doc.sh
)

SCD_SET="$(printf '%s\n' "${SUPPLY_CHAIN_SCRIPTS[@]}")" \
SCD_ARGS="${1:-}" \
python3 -I - <<'PYEOF'
import glob
import os
import re
import shutil
import sys
import tempfile

DOC = os.environ.get("SCD_DOC", "dev-docs/supply-chain/README.md")
DECLARED = [l for l in os.environ["SCD_SET"].splitlines() if l]
STATUSES = ("enforced", "manual", "planned")
COLUMNS = ["id", "control", "threat it stops", "enforcing script", "config", "runs in", "status", "adopt"]
HEADINGS = [
    "## Scope", "## Threat model", "### What we do not defend against", "## Controls",
    "## Decisions recorded", "## Adopt this in your project", "## Keeping this document true",
]
PLAYBOOK_LINK = "dev-docs/supply-chain/README.md"

def read(path):
    try:
        return open(path, encoding="utf-8").read()
    except OSError as err:
        print(f"check-supply-chain-doc: cannot read {path}: {err}", file=sys.stderr)
        sys.exit(2)

def path_tokens(cell):
    """Backticked repository paths in a table cell: a `/` or a file extension, no spaces."""
    out = []
    for tok in re.findall(r"`([^`]+)`", cell):
        if " " in tok:
            continue
        if "/" in tok or re.search(r"\.[A-Za-z0-9]+$", tok):
            out.append(tok)
    return out

def exists(root, path):
    full = os.path.join(root, path)
    return bool(glob.glob(full)) if "*" in path else os.path.exists(full)

def parse_table(doc, problems):
    lines = doc.splitlines()
    start = next((i for i, l in enumerate(lines) if l.strip() == "## Controls"), None)
    if start is None:
        problems.append("no `## Controls` heading")
        return []
    rows, header = [], None
    for l in lines[start + 1:]:
        if l.startswith("## "):
            break
        if not l.startswith("|"):
            continue
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if header is None:
            header = [c.lower() for c in cells]
            if header != COLUMNS:
                problems.append(f"controls table header is {header}, want {COLUMNS}")
                return []
            continue
        if set("".join(cells)) <= set("-: "):
            continue
        if len(cells) != len(COLUMNS):
            problems.append(f"table row has {len(cells)} cells, want {len(COLUMNS)} (a `|` inside a cell?): {l[:70]}...")
            continue
        rows.append(dict(zip(COLUMNS, cells)))
    if header is None:
        problems.append("`## Controls` has no table")
    return rows

def check(root, doc, declared):
    problems = []
    for h in HEADINGS:
        if not any(l.strip() == h for l in doc.splitlines()):
            problems.append(f"missing heading `{h}`")
    rows = parse_table(doc, problems)
    gates = ""
    rg = os.path.join(root, "scripts/run-gates.sh")
    if os.path.exists(rg):
        gates = open(rg, encoding="utf-8").read()
    else:
        problems.append("scripts/run-gates.sh does not exist")
    ids, named_scripts, counts = set(), set(), {s: 0 for s in STATUSES}
    for r in rows:
        rid = r["id"]
        if not re.fullmatch(r"C\d\d", rid):
            problems.append(f"{rid!r}: id must look like C01")
        if rid in ids:
            problems.append(f"{rid}: id repeated")
        ids.add(rid)
        status = r["status"]
        if status not in STATUSES:
            problems.append(f"{rid}: status {status!r} is not one of {STATUSES}")
            continue
        counts[status] += 1
        scripts, configs = path_tokens(r["enforcing script"]), path_tokens(r["config"])
        named_scripts.update(scripts)
        if status == "planned":
            for s in scripts:
                if exists(root, s):
                    problems.append(f"{rid}: planned, but `{s}` exists now; flip the row to enforced or manual")
            continue
        if not scripts and not configs:
            problems.append(f"{rid}: {status} row names no script or config path")
        for p in scripts + configs:
            if not exists(root, p):
                problems.append(f"{rid}: {status} row names `{p}`, which does not exist")
        if r["runs in"] == "run-gates":
            if status != "enforced":
                problems.append(f"{rid}: runs in run-gates but status is {status}")
            for s in scripts:
                if re.fullmatch(r"scripts/check-[a-z0-9-]+\.sh", s) and f"|{s}\"" not in gates:
                    problems.append(f"{rid}: `{s}` is not in scripts/run-gates.sh's gate list")
    for s in declared:
        if not exists(root, s):
            problems.append(f"declared supply-chain script `{s}` does not exist (remove it from SUPPLY_CHAIN_SCRIPTS and the table)")
        if s not in named_scripts:
            problems.append(f"declared supply-chain script `{s}` is in no controls-table row")
    used = set(re.findall(r"\bC\d\d\b", doc))
    for u in sorted(used - ids):
        problems.append(f"{u} is used in the document but has no controls-table row")
    m = re.search(r"Counts[^:\n]*: (\d+) enforced, (\d+) manual, (\d+) planned", doc)
    if not m:
        problems.append("no `Counts ...: N enforced, N manual, N planned` line")
    elif tuple(int(x) for x in m.groups()) != tuple(counts[s] for s in STATUSES):
        problems.append(f"Counts line says {m.groups()} but the table has {tuple(counts[s] for s in STATUSES)}")
    sec = os.path.join(root, "SECURITY.md")
    if not os.path.exists(sec):
        problems.append("SECURITY.md does not exist")
    elif PLAYBOOK_LINK not in open(sec, encoding="utf-8").read():
        problems.append(f"SECURITY.md does not link {PLAYBOOK_LINK}")
    return problems, counts

def self_test(doc, quiet=False):
    problems, _ = check(".", doc, DECLARED)
    if problems:
        print("check-supply-chain-doc: self-test needs a green tree first:", *problems, sep="\n  - ", file=sys.stderr)
        return 1
    # A throwaway tree that mirrors which paths exist, so mutations can add or remove files without touching the repo.
    tmp = tempfile.mkdtemp(prefix="scd-selftest-")
    def put(rel, text=""):
        full = os.path.join(tmp, rel.replace("*", "x"))
        os.makedirs(os.path.dirname(full), exist_ok=True)
        open(full, "w", encoding="utf-8").write(text)
    probs = []
    rows = parse_table(doc, probs)
    for r in rows:
        for p in path_tokens(r["enforcing script"]) + path_tokens(r["config"]):
            if exists(".", p):
                put(p)
    for s in DECLARED:
        put(s)
    put("scripts/run-gates.sh", read("scripts/run-gates.sh"))
    put("SECURITY.md", read("SECURITY.md"))
    sec_ok = read("SECURITY.md")
    gates_ok = read("scripts/run-gates.sh")

    first = next(r for r in rows if r["status"] == "enforced" and path_tokens(r["enforcing script"]))
    first_script = path_tokens(first["enforcing script"])[0]
    cfg_row = next(r for r in rows if r["status"] == "enforced" and path_tokens(r["config"]))
    cfg_tok = path_tokens(cfg_row["config"])[0]
    planned = next(r for r in rows if r["status"] == "planned" and path_tokens(r["enforcing script"]))
    planned_script = path_tokens(planned["enforcing script"])[0]
    gate_row = next(r for r in rows if r["runs in"] == "run-gates" and
                    any(re.fullmatch(r"scripts/check-[a-z0-9-]+\.sh", s) for s in path_tokens(r["enforcing script"])))
    gate_script = next(s for s in path_tokens(gate_row["enforcing script"]) if re.fullmatch(r"scripts/check-[a-z0-9-]+\.sh", s))
    drop_set = [s for s in DECLARED if s != first_script][0]

    def run(name, mutate_doc=None, declared=None, files=None, remove=None, rewrite=None):
        d = mutate_doc(doc) if mutate_doc else doc
        for rel, text in (files or {}).items():
            put(rel, text)
        for rel in (remove or []):
            os.remove(os.path.join(tmp, rel))
        for rel, text in (rewrite or {}).items():
            put(rel, text)
        got, _ = check(tmp, d, DECLARED if declared is None else declared)
        # restore
        for rel in (files or {}):
            full = os.path.join(tmp, rel)
            if os.path.exists(full):
                os.remove(full)
        for rel in (remove or []):
            put(rel)
        put("scripts/run-gates.sh", gates_ok)
        put("SECURITY.md", sec_ok)
        return name, got

    cases = [
        ("unmutated table stays green", None),
        ("an enforced row's script is deleted", dict(remove=[first_script])),
        ("an enforced row names a config path that does not exist",
         dict(mutate_doc=lambda d: "\n".join(
             "|".join(c.replace(f"`{cfg_tok}`", "`no/such/config.toml`", 1) if k == 5 else c
                      for k, c in enumerate(l.split("|"))) if l.startswith(f"| {cfg_row['id']} |") else l
             for l in d.splitlines()) + "\n")),
        ("a declared script has no row",
         dict(mutate_doc=lambda d: "\n".join(l for l in d.splitlines() if f"`{drop_set}`" not in l) + "\n")),
        ("a planned row's script now exists", dict(files={planned_script: ""})),
        ("an unknown status", dict(mutate_doc=lambda d: d.replace("| enforced |", "| done |", 1))),
        ("a declared script that does not exist", dict(declared=DECLARED + ["scripts/check-no-such-thing.sh"])),
        ("a required heading is removed",
         dict(mutate_doc=lambda d: d.replace("## Adopt this in your project", "## Adoption", 1))),
        ("a control id with no row is referenced", dict(mutate_doc=lambda d: d + "\nSee C99.\n")),
        ("the Counts line is wrong", dict(mutate_doc=lambda d: re.sub(r"(Counts[^:\n]*: )\d+", r"\g<1>99", d, count=1))),
        ("a run-gates row's script is not in the gate list",
         dict(rewrite={"scripts/run-gates.sh": gates_ok.replace(f"|{gate_script}\"", "|scripts/other.sh\"")})),
        ("SECURITY.md does not link the playbook", dict(rewrite={"SECURITY.md": "# Security\n"})),
    ]
    bad = 0
    for i, (name, kw) in enumerate(cases):
        n, got = run(name, **(kw or {}))
        want_green = i == 0
        ok = (not got) if want_green else bool(got)
        if not quiet or not ok:
            print(f"  self-test {'ok  ' if ok else 'FAIL'} {n}" + ("" if ok or not want_green else f" -- {got[0]}"))
        if not ok:
            bad += 1
    shutil.rmtree(tmp, ignore_errors=True)
    if bad:
        print(f"check-supply-chain-doc: self-test FAIL ({bad} of {len(cases)} cases wrong)", file=sys.stderr)
        return 1
    if not quiet:
        print(f"check-supply-chain-doc: self-test OK ({len(cases) - 1} mutations red, the unmutated table green)")
    return 0

doc = read(DOC)
if os.environ.get("SCD_ARGS") == "--self-test":
    sys.exit(self_test(doc))
if os.environ.get("SCD_ARGS"):
    print(f"check-supply-chain-doc: unknown argument {os.environ['SCD_ARGS']!r}", file=sys.stderr)
    sys.exit(2)
problems, counts = check(".", doc, DECLARED)
if problems:
    print("check-supply-chain-doc: FAIL -- the playbook and the tree disagree:", file=sys.stderr)
    for p in problems:
        print(f"  - {p}", file=sys.stderr)
    sys.exit(1)
print(f"check-supply-chain-doc: OK -- {counts['enforced']} enforced, {counts['manual']} manual, "
      f"{counts['planned']} planned controls; {len(DECLARED)} declared scripts all documented")
sys.exit(self_test(doc, quiet=True))
PYEOF
