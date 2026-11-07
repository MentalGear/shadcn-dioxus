#!/usr/bin/env python3
"""The engine of scripts/check-dependency-register.sh (read that header first).

Usage: python3 -I scripts/dependency-register.py [--self-test | --list | --closure-update]
  (no flag)        check the tree against dev-docs/dependency-register.tsv and the closure baseline
  --self-test      prove every failure class turns red on in-memory fixtures; no cargo, no tree reads
  --list           print the direct dependencies the manifests declare as register-shaped rows
  --closure-update rewrite the closure baseline from the tree (a shrink is the normal case; a growth is a decision)
Environment: DEP_REGISTER (register path), DEP_CLOSURE (closure baseline path), DEP_REGISTER_TODAY (YYYY-MM-DD).
Exit status: 0 clean, 1 a violation, 2 an unreadable source or a failed cargo call.
"""
import datetime
import json
import os
import re
import subprocess
import sys
import tomllib

ROOT = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
REGISTER = os.environ.get("DEP_REGISTER", os.path.join(ROOT, "dev-docs/dependency-register.tsv"))
CLOSURE = os.environ.get("DEP_CLOSURE", os.path.join(ROOT, "dev-docs/dependency-register.closure.tsv"))

COLUMNS = ["name", "ecosystem", "version", "outcome", "reach", "scope", "evidence_date", "exception_end", "note"]
ECOSYSTEMS = {"cargo", "npm", "tool"}
OUTCOMES = {"keep", "lift", "drop", "exception"}
REACH = {"compile-time-code", "unsafe", "untrusted-input", "fs-network", "none"}
SCOPES = {"user-closure", "site", "cli", "lab", "dev", "tool"}
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
CLOSURE_KINDS = {"build-script", "proc-macro"}
SKIP_MANIFEST_PREFIXES = ("templates/",)


class Fail(Exception):
    """An unreadable source or a failed cargo call (exit 2)."""


def read(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError as e:
        raise Fail(f"cannot read {path}: {e}")


def git_files(pattern):
    out = subprocess.run(["git", "ls-files", pattern], cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return [p for p in out.splitlines() if p and not p.startswith(SKIP_MANIFEST_PREFIXES)]


# ---------------------------------------------------------------- the dependencies the tree declares

def parse_lock(path):
    data = tomllib.loads(read(path))
    return data.get("package", [])


def lock_version(lock, member, dep):
    """The version of `dep` that `member` (a lock package name) resolves to, or None."""
    entries = []
    for p in lock:
        if p["name"] == member and "source" not in p:
            entries = p.get("dependencies", [])
            break
    cands = []
    for e in entries:
        parts = e.split(" ")
        if parts[0] == dep:
            cands.append(parts[1] if len(parts) > 1 else None)
    for c in cands:
        if c:
            return c.split("+")[0]
    if cands:
        versions = [p["version"] for p in lock if p["name"] == dep]
        if len(versions) == 1:
            return versions[0].split("+")[0]
    return None


def cargo_deps(root_workspace_deps, manifest, lock):
    """Yield (name, version-or-None, kind) for each registry dependency one Cargo.toml declares."""
    data = tomllib.loads(read(os.path.join(ROOT, manifest)))
    pkg = data.get("package", {}).get("name")
    if pkg is None:
        return
    tables = []
    for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
        tables.append((kind, data.get(kind, {})))
        for tgt in data.get("target", {}).values():
            tables.append((kind, tgt.get(kind, {})))
    for kind, table in tables:
        for key, spec in table.items():
            spec = spec if isinstance(spec, dict) else {"version": spec}
            if spec.get("workspace"):
                base = root_workspace_deps.get(key, {})
                base = base if isinstance(base, dict) else {"version": base}
                spec = {**base, **{k: v for k, v in spec.items() if k != "workspace"}}
            if "path" in spec or "git" in spec:
                continue
            name = spec.get("package", key)
            yield name, lock_version(lock, pkg, name), kind


def npm_deps(manifest):
    """Yield (name, pinned version, specifier-or-None-when-lock-disagrees) for one package.json."""
    d = os.path.dirname(os.path.join(ROOT, manifest))
    data = json.loads(read(os.path.join(ROOT, manifest)))
    lock = read(os.path.join(d, "pnpm-lock.yaml")).splitlines()
    locked = {}
    in_importers = in_root = False
    name = None
    for line in lock:
        if re.match(r"^\S", line):
            in_importers = line.startswith("importers:")
            in_root = False
            continue
        if in_importers and re.match(r"^  \S", line):
            in_root = line.strip() == ".:"
            continue
        if in_root:
            m = re.match(r"^      ['\"]?([^'\":]+)['\"]?:\s*$", line)
            if m:
                name = m.group(1)
                continue
            m = re.match(r"^        version:\s*['\"]?([^'\"(\s]+)", line)
            if m and name:
                locked[name] = m.group(1)
    for section in ("dependencies", "devDependencies", "optionalDependencies"):
        for n, spec in data.get(section, {}).items():
            yield n, locked.get(n), spec


def tool_versions():
    m = re.search(r'^DX_VERSION="([^"]+)"', read(os.path.join(ROOT, "scripts/session-setup.sh")), re.M)
    if not m:
        raise Fail("scripts/session-setup.sh has no DX_VERSION=")
    return {"dx": m.group(1)}


def actual_dependencies():
    """{(ecosystem, name): {"versions": set, "errors": [str]}} from the manifests and their locks."""
    out = {}
    errors = []

    def add(eco, name, version, where):
        e = out.setdefault((eco, name), {"versions": set(), "where": set()})
        if version:
            e["versions"].add(version)
        e["where"].add(where)

    root_ws = tomllib.loads(read(os.path.join(ROOT, "Cargo.toml"))).get("workspace", {}).get("dependencies", {})
    root_lock = parse_lock(os.path.join(ROOT, "Cargo.lock"))
    for manifest in git_files("*Cargo.toml"):
        own = os.path.join(ROOT, os.path.dirname(manifest), "Cargo.lock")
        lock = parse_lock(own) if os.path.exists(own) else root_lock
        for name, version, _kind in cargo_deps(root_ws, manifest, lock):
            if version is None:
                errors.append(f"{manifest}: {name} is declared but the lock has no resolved version for it (stale lock)")
            add("cargo", name, version, manifest)
    for manifest in git_files("*package.json"):
        for name, version, spec in npm_deps(manifest):
            if version is None:
                errors.append(f"{manifest}: {name} is declared but pnpm-lock.yaml has no resolved version for it")
            elif spec != version:
                errors.append(f"{manifest}: {name} is pinned '{spec}' but pnpm-lock.yaml resolves {version}")
            add("npm", name, version or spec, manifest)
    for name, version in tool_versions().items():
        add("tool", name, version, "scripts/session-setup.sh")
    return out, errors


# ---------------------------------------------------------------- the register

def parse_register(text):
    rows, errors = {}, []
    header_seen = False
    for n, line in enumerate(text.splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        cells = [c.strip() for c in line.split("\t")]
        if not header_seen:
            header_seen = True
            if cells != COLUMNS:
                errors.append(f"register line {n}: the header must be {' '.join(COLUMNS)}")
            continue
        if len(cells) != len(COLUMNS):
            errors.append(f"register line {n}: {len(cells)} cells, expected {len(COLUMNS)}")
            continue
        r = dict(zip(COLUMNS, cells))
        key = (r["ecosystem"], r["name"])
        if key in rows:
            errors.append(f"register line {n}: {r['name']} ({r['ecosystem']}) appears twice")
        rows[key] = r
        where = f"register {r['name']} ({r['ecosystem']})"
        for col, allowed in (("ecosystem", ECOSYSTEMS), ("outcome", OUTCOMES), ("reach", REACH), ("scope", SCOPES)):
            if r[col] not in allowed:
                errors.append(f"{where}: {col} '{r[col]}' is not one of {sorted(allowed)}")
        if not r["name"] or not r["version"]:
            errors.append(f"{where}: name and version are required")
        if not DATE_RE.match(r["evidence_date"]):
            errors.append(f"{where}: evidence_date '{r['evidence_date']}' is not YYYY-MM-DD")
        if r["outcome"] == "exception" and not DATE_RE.match(r["exception_end"]):
            errors.append(f"{where}: an exception needs an exception_end date (YYYY-MM-DD)")
        if r["outcome"] != "exception" and r["exception_end"]:
            errors.append(f"{where}: exception_end is set but the outcome is '{r['outcome']}'")
    if not header_seen:
        errors.append("register has no header row")
    return rows, errors


def compare(actual, rows, today):
    """Every violation between the declared dependencies and the register, as a list of messages."""
    errors = []
    for key in sorted(actual):
        if key not in rows:
            a = actual[key]
            errors.append(f"not in the register: {key[1]} ({key[0]}) {','.join(sorted(a['versions']))} "
                          f"declared in {', '.join(sorted(a['where']))}; evaluate it against C23 D1-D6 and add a row")
    for key in sorted(rows):
        r = rows[key]
        if key not in actual:
            errors.append(f"stale row: {key[1]} ({key[0]}) is in the register but no manifest declares it any more")
            continue
        want = ",".join(sorted(actual[key]["versions"]))
        have = ",".join(sorted(v.strip() for v in r["version"].split(",")))
        if want and want != have:
            errors.append(f"wrong version: {key[1]} ({key[0]}) is {want} in the lock or manifest pin but "
                          f"{have} in the register; re-evaluate the dependency and update the row with a new evidence_date")
        if DATE_RE.match(r["evidence_date"]) and r["evidence_date"] > today:
            errors.append(f"{key[1]} ({key[0]}): evidence_date {r['evidence_date']} is in the future")
        if r["outcome"] == "exception" and DATE_RE.match(r["exception_end"]) and r["exception_end"] < today:
            errors.append(f"expired exception: {key[1]} ({key[0]}) ended {r['exception_end']}; "
                          f"re-check D1-D5, then lift the exception, replace the dependency or record a new end date")
    return errors


# ---------------------------------------------------------------- the closure a user installs

def cargo(args):
    p = subprocess.run(["cargo", *args], cwd=ROOT, capture_output=True, text=True)
    if p.returncode != 0:
        raise Fail(f"cargo {' '.join(args)} failed:\n{p.stderr.strip()}")
    return p.stdout


def component_crates(primitives_direct):
    """Registry crates the component manifests name that primitives does not already carry, as name -> version."""
    names = set()
    for f in git_files("preview/src/components/*/component.json"):
        for dep in json.loads(read(os.path.join(ROOT, f))).get("cargoDependencies", []):
            if "git" not in dep:
                names.add(dep["name"])
    return sorted(n for n in names if n not in primitives_direct)


def closure_from_tree():
    """{crate name: set of kinds} for every crate with a build script or a proc-macro in a user's closure."""
    meta = json.loads(cargo(["metadata", "--offline", "--locked", "--format-version", "1"]))
    build_script = {p["name"] for p in meta["packages"] if any("custom-build" in t["kind"] for t in p["targets"])}
    lock = parse_lock(os.path.join(ROOT, "Cargo.lock"))
    prim = tomllib.loads(read(os.path.join(ROOT, "primitives/Cargo.toml")))
    direct = set(prim.get("dependencies", {}))
    roots = []
    for target in ("wasm32-unknown-unknown", None):
        a = ["tree", "--offline", "--locked", "-p", "dioxus-primitives", "--all-features", "-e", "normal,build",
             "--prefix", "none"]
        roots.append(a + (["--target", target] if target else []))
    for name in component_crates(direct):
        versions = sorted({p["version"] for p in lock if p["name"] == name})
        if len(versions) != 1:
            raise Fail(f"component.json names {name}, which the lock resolves to {versions or 'nothing'}")
        roots.append(["tree", "--offline", "--locked", "-p", f"{name}@{versions[0]}", "-e", "normal,build",
                      "--prefix", "none"])
    found = {}
    for a in roots:
        for line in cargo(a).splitlines():
            m = re.match(r"^(\S+) v\S+(?: \(proc-macro\))?", line)
            if not m:
                continue
            kinds = found.setdefault(m.group(1), set())
            if "(proc-macro)" in line:
                kinds.add("proc-macro")
            if m.group(1) in build_script:
                kinds.add("build-script")
    return {n: k for n, k in found.items() if k}


def parse_closure(text):
    rows, errors = {}, []
    header = False
    for n, line in enumerate(text.splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        cells = [c.strip() for c in line.split("\t")]
        if not header:
            header = True
            if cells != ["crate", "kinds"]:
                errors.append(f"closure baseline line {n}: the header must be crate<TAB>kinds")
            continue
        if len(cells) != 2:
            errors.append(f"closure baseline line {n}: expected 2 cells")
            continue
        kinds = set(cells[1].split(","))
        if not kinds <= CLOSURE_KINDS:
            errors.append(f"closure baseline line {n}: kinds must be build-script and/or proc-macro")
        rows[cells[0]] = kinds
    return rows, errors


def compare_closure(found, baseline):
    errors = []
    for name in sorted(found):
        new = found[name] - baseline.get(name, set())
        if new:
            errors.append(f"new in a user's install closure: {name} brings {'+'.join(sorted(new))}; "
                          f"a user's build now runs that code. Decide it (C23), then add it with --closure-update")
    for name in sorted(baseline):
        gone = baseline[name] - found.get(name, set())
        if gone:
            errors.append(f"stale closure baseline entry: {name} ({'+'.join(sorted(gone))}) is no longer in the closure; "
                          f"shrink the baseline with --closure-update")
    return errors


def closure_text(found):
    lines = ["# Crates with a build script or a proc-macro in the closure a user installs (dioxus-primitives with all features on",
             "# wasm32 and the host, plus every registry crate a component.json names). A shrink-only ratchet read by",
             "# scripts/check-dependency-register.sh: a crate or kind that is not listed is red, a listed one that is gone is red.",
             "# Regenerate with scripts/dependency-register.py --closure-update.",
             "crate\tkinds"]
    lines += [f"{n}\t{','.join(sorted(found[n]))}" for n in sorted(found)]
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------- list, check, self-test

def list_rows():
    actual, errors = actual_dependencies()
    for e in errors:
        print(f"# {e}", file=sys.stderr)
    print("\t".join(COLUMNS))
    for (eco, name) in sorted(actual, key=lambda k: (k[0], k[1])):
        print("\t".join([name, eco, ",".join(sorted(actual[(eco, name)]["versions"])), "", "", "", "", "", ""]))


def check():
    today = os.environ.get("DEP_REGISTER_TODAY") or datetime.date.today().isoformat()
    actual, errors = actual_dependencies()
    rows, rerrors = parse_register(read(REGISTER))
    errors += rerrors + compare(actual, rows, today)
    baseline, cerrors = parse_closure(read(CLOSURE))
    try:
        errors += cerrors + compare_closure(closure_from_tree(), baseline)
    except Fail as e:
        for m in errors:
            print(f"check-dependency-register: {m}", file=sys.stderr)
        raise Fail(f"the closure check could not run (a manifest the lock does not cover is the usual cause): {e}")
    return errors, len(rows), len(baseline)


def self_test():
    today = "2026-11-07"
    head = "\t".join(COLUMNS)

    def row(name, eco="cargo", version="1.0.0", outcome="keep", exc="", ev="2026-11-06"):
        return "\t".join([name, eco, version, outcome, "none", "dev", ev, exc, ""])

    def reg(*rows):
        return head + "\n" + "\n".join(rows) + "\n"

    def actual(**kw):
        return {("cargo", n): {"versions": set(v.split(",")), "where": {"x/Cargo.toml"}} for n, v in kw.items()}

    base_rows, errs = parse_register(reg(row("a"), row("b", version="2.0.0")))
    assert not errs, errs
    ok = actual(a="1.0.0", b="2.0.0")
    cases = [
        ("clean", ok, base_rows, False),
        ("new dependency", actual(a="1.0.0", b="2.0.0", c="3.0.0"), base_rows, True),
        ("stale row", actual(a="1.0.0"), base_rows, True),
        ("wrong pin", actual(a="1.0.1", b="2.0.0"), base_rows, True),
    ]
    exp_rows, errs = parse_register(reg(row("a", outcome="exception", exc="2026-11-06"), row("b", version="2.0.0")))
    assert not errs, errs
    cases.append(("expired exception", ok, exp_rows, True))
    live_rows, errs = parse_register(reg(row("a", outcome="exception", exc="2026-11-07"), row("b", version="2.0.0")))
    assert not errs, errs
    cases.append(("exception ending today is live", ok, live_rows, False))
    future_rows, errs = parse_register(reg(row("a", ev="2026-12-01"), row("b", version="2.0.0")))
    cases.append(("future evidence date", ok, future_rows, True))
    failures = []
    for label, act, rws, want_red in cases:
        red = bool(compare(act, rws, today))
        if red != want_red:
            failures.append(f"{label}: expected {'red' if want_red else 'green'}")
    for label, text in (
        ("exception without end", reg(row("a", outcome="exception"))),
        ("end without exception", reg(row("a", exc="2027-01-01"))),
        ("bad outcome", reg(row("a", outcome="maybe"))),
        ("duplicate row", reg(row("a"), row("a"))),
        ("short row", head + "\na\tcargo\n"),
        ("bad date", reg(row("a", ev="yesterday"))),
    ):
        if not parse_register(text)[1]:
            failures.append(f"{label}: expected the register parser to report it")
    base = {"x": {"build-script"}, "y": {"proc-macro"}}
    for label, found, want_red in (
        ("closure unchanged", dict(base), False),
        ("new build script", {**base, "z": {"build-script"}}, True),
        ("new proc-macro on a known crate", {**base, "x": {"build-script", "proc-macro"}}, True),
        ("closure shrank", {"x": {"build-script"}}, True),
    ):
        if bool(compare_closure(found, base)) != want_red:
            failures.append(f"{label}: expected {'red' if want_red else 'green'}")
    if not parse_closure("crate\tkinds\nx\tbuild-script\n")[1] == []:
        failures.append("closure parser rejects a valid file")
    if not parse_closure("crate\tkinds\nx\tmystery\n")[1]:
        failures.append("closure parser accepts an unknown kind")
    return failures


def main():
    arg = sys.argv[1] if len(sys.argv) > 1 else ""
    try:
        if arg == "--self-test":
            failures = self_test()
            for f in failures:
                print(f"self-test FAILED: {f}", file=sys.stderr)
            if not failures:
                print("self-test ok")
            return 1 if failures else 0
        if arg == "--list":
            list_rows()
            return 0
        if arg == "--closure-update":
            with open(CLOSURE, "w", encoding="utf-8") as f:
                f.write(closure_text(closure_from_tree()))
            return 0
        if arg:
            print(__doc__, file=sys.stderr)
            return 2
        failures = self_test()
        if failures:
            for f in failures:
                print(f"self-test FAILED: {f}", file=sys.stderr)
            return 1
        errors, nrows, nclosure = check()
    except Fail as e:
        print(f"check-dependency-register: {e}", file=sys.stderr)
        return 2
    for e in errors:
        print(f"check-dependency-register: {e}", file=sys.stderr)
    if errors:
        print(f"check-dependency-register: {len(errors)} violation(s)", file=sys.stderr)
        return 1
    print(f"check-dependency-register: OK -- {nrows} dependencies registered, {nclosure} build-script or proc-macro crates in the closure baseline")
    return 0


if __name__ == "__main__":
    sys.exit(main())
