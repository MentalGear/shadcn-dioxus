#!/usr/bin/env bash
#
# check-decision-kernel.sh   (backlog row 509)
#
# The decision kernel (`decision-kernel/`, the pure Rust `decide` of every named behaviour) is a COMMITTED wasm module
# (`primitives/src/kernel/`), written only by `scripts/build-kernel.sh`. This gate holds it true: `kernel.manifest` hashes
# the crate's sources, the `[profile.kernel]` table and `rust-toolchain.toml` and a rebuild on the pinned toolchain gives
# identical bytes; the module exports exactly five functions and its `dx_kinds` hash is the generated kind table's; a
# gzip-size ratchet bounds the module; and the crate's `clippy.toml` bans the transcendental float methods.
#
# Committing a generated binary makes one class possible, the sources and the module drifting apart, and the page then
# silently never decides (a stale module answers a function the sources no longer hold; a table that differs answers nothing). The checks:
#   1. THE MANIFEST. `kernel.manifest` hashes every input of the module: the crate's `Cargo.toml` and sources, the
#      `[profile.kernel]` table of the root `Cargo.toml` (the table, not the file: an unrelated crate added to the
#      workspace stales nothing) and `rust-toolchain.toml`, and records `rustc -Vv` without its `host:` line. An edited
#      source, a changed profile or a changed toolchain pin without a rebuild is red; so is a module or base64 text that
#      is not the one the manifest names.
#   2. THE EXPORTS AND THE TABLE. The module exports exactly `dx_abi`, `dx_kinds`, `dx_input`, `dx_decide`, `dx_output` and
#      its memory (a behaviour never adds an export: a kind is an index into the kind table), and `dx_kinds()` equals the
#      `KERNEL_HASH` of the generated `behaviours.gen.ts`. The generated `KINDS` are the kinds `decision-kernel/src/kinds.rs`
#      registers, in order, so a kind removed from the kernel and left in `KINDS` is red. The crate has no dependency and
#      its `ffi.rs` declares those five `extern "C"` functions and no other.
#   3. THE REBUILD. On the pinned toolchain (the `rustc -Vv` of the manifest, with the wasm32 target) a full gate run (or
#      `--rebuild`) rebuilds into a scratch directory and requires identical bytes for the module, the base64 and the
#      manifest. On any other toolchain the rebuild half is reported as skipped, as `check-js-bundle` does for `bun`; the
#      pin makes that the exception. A fast run (`run-gates.sh --fast`) skips it too, because it builds with cargo.
#   4. THE SIZE BUDGET. `scripts/check-decision-kernel.budgets.tsv` is a shrink-only ratchet (scripts/lib/ratchet.py, column
#      `gzip_bytes`) on `gzip -9` of the module: a rise passes only against a `js-grant: gzip_bytes=<n>` row at the merge
#      base that names the behaviour paying for it. The raw size is reported, not gated.
#   5. THE FLOAT CONTRACT. `decision-kernel/clippy.toml` bans the transcendental float methods (`exp`, `ln`, `powf`, `powi`,
#      `sin`, `cos`, `tan`, `atan2`, `hypot`, `cbrt`, in their `f64` and `f32` forms): they come from Rust's bundled libm on
#      wasm32 and from the system libm natively, so a threshold could flip by one ulp between web and Blitz. The clippy arms
#      enforce the list; this gate holds the list itself.
#
# Usage: scripts/check-decision-kernel.sh [--rebuild] [--self-test] [--update-baseline [--allow-raise "row N why"]]
#   --rebuild          run the rebuild half even without a full gate run (needs CARGO_TARGET_DIR, absolute; wrap it in
#                      scripts/lane-lock.sh like any cargo command)
#   --self-test        apply one-line breaks to an in-memory copy of the tree and require each to turn the gate red
# Exit status: 0 when clean.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

mode=run rebuild=0 raise_reason=""
while [ $# -gt 0 ]; do
  case "$1" in
    --rebuild) rebuild=1 ;;
    --self-test) mode=self-test ;;
    --update-baseline) mode=update ;;
    --allow-raise) shift; raise_reason="${1:-}" ;;
    *) echo "check-decision-kernel: unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done
# a full run-gates.sh run (no --fast, --only or --skip) sets RATCHET_REQUIRE_BASE=1: that is the run that rebuilds
[ "${RATCHET_REQUIRE_BASE:-}" = 1 ] && rebuild=1

command -v node >/dev/null || { echo "check-decision-kernel: node is required (it reads the module's exports)" >&2; exit 1; }

python3 -I - "$mode" "$rebuild" "$raise_reason" <<'PYEOF'
import base64
import copy
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, "scripts/lib")
from ratchet import Ratchet  # noqa: E402

MODE, REBUILD, RAISE_REASON = sys.argv[1], sys.argv[2] == "1", sys.argv[3] or None
GATE = "check-decision-kernel"

KERNEL_DIR = Path("primitives/src/kernel")
WASM = str(KERNEL_DIR / "decision-kernel.wasm")
B64 = str(KERNEL_DIR / "decision-kernel.b64")
MANIFEST = str(KERNEL_DIR / "kernel.manifest")
ROOT_CARGO = "Cargo.toml"
TOOLCHAIN = "rust-toolchain.toml"
CRATE = "decision-kernel"
CRATE_CARGO = "decision-kernel/Cargo.toml"
KINDS_RS = "decision-kernel/src/kinds.rs"
FFI_RS = "decision-kernel/src/ffi.rs"
CLIPPY = "decision-kernel/clippy.toml"
GEN_TS = "primitives/src/ts/behaviours/behaviours.gen.ts"
PRIMITIVES_CARGO = "primitives/Cargo.toml"
BUDGETS = "scripts/check-decision-kernel.budgets.tsv"
EXPORTS = ["dx_abi", "dx_decide", "dx_input", "dx_kinds", "dx_output", "memory"]
FFI_FUNCTIONS = ["dx_abi", "dx_decide", "dx_input", "dx_kinds", "dx_output"]
FLOATS = ["exp", "ln", "powf", "powi", "sin", "cos", "tan", "atan2", "hypot", "cbrt"]
BUDGET_ADVICE = ("the module grows only by a grant: ask the owner for a `js-grant: gzip_bytes=<n>` row at the merge base "
                 "that names the behaviour paying for it, or shrink the module and run --update-baseline")
REPORT = []

NODE_EXPORTS = r"""
const fs = require("fs");
const bytes = fs.readFileSync(process.argv[1]);
try {
  const module = new WebAssembly.Module(bytes);
  const out = { exports: WebAssembly.Module.exports(module).map((e) => e.name).sort() };
  const exp = new WebAssembly.Instance(module, {}).exports;
  out.abi = typeof exp.dx_abi === "function" ? exp.dx_abi() : null;
  out.kinds = typeof exp.dx_kinds === "function" ? exp.dx_kinds() >>> 0 : null;
  console.log(JSON.stringify(out));
} catch (error) {
  console.log(JSON.stringify({ error: String(error) }));
}
"""


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def blob_id(data):
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()


def profile_of(cargo_text):
    """The `[profile.kernel]` table as build-kernel.sh writes it into the manifest: the header and its non-blank lines."""
    out, on = [], False
    for line in cargo_text.split("\n"):
        if line.startswith("[profile.kernel]"):
            on = True
            out.append(line)
        elif line.startswith("["):
            on = False
        elif on and line.strip():
            out.append(line)
    return "\n".join(out)


def load_tree():
    tree = {"files": {}, "bytes": {}, "src": {}}
    for p in [ROOT_CARGO, TOOLCHAIN, CRATE_CARGO, KINDS_RS, FFI_RS, CLIPPY, GEN_TS, PRIMITIVES_CARGO, MANIFEST, B64]:
        pp = Path(p)
        tree["files"][p] = pp.read_text(encoding="utf-8") if pp.is_file() else None
    tree["bytes"][WASM] = Path(WASM).read_bytes() if Path(WASM).is_file() else None
    # the inputs the manifest names: the crate's manifest and sources, and the toolchain pin
    for p in [Path(CRATE_CARGO)] + sorted(Path(CRATE, "src").rglob("*")) + [Path(TOOLCHAIN)]:
        if p.is_file():
            tree["src"][p.as_posix()] = p.read_bytes()
    tree["primitives_build_rs"] = Path("primitives/build.rs").exists()
    return tree


def read_manifest(text):
    m = {"rustc": [], "src": {}, "profile": [], "profile_sha": None, "target": None, "wasm": None}
    for line in text.split("\n"):
        if line.startswith("rustc: "):
            m["rustc"].append(line[len("rustc: "):])
        elif line.startswith("target: "):
            m["target"] = line[len("target: "):]
        elif line.startswith("profile-sha256 "):
            m["profile_sha"] = line.split()[1]
        elif line.startswith("profile> "):
            m["profile"].append(line[len("profile> "):])
        elif line.startswith("src "):
            parts = line.split(" ", 3)
            if len(parts) == 4:
                m["src"][parts[3]] = (parts[1], parts[2])
        elif line.startswith("wasm "):
            mm = re.match(r"wasm ([0-9a-f]{64})\s+decision-kernel\.wasm \((\d+) bytes\)", line)
            if mm:
                m["wasm"] = (mm.group(1), int(mm.group(2)))
    return m


def gen_values(text):
    kinds = re.findall(r'"([^"]+)"', (re.search(r"export const KINDS = \[([^\]]*)\]", text or "") or [None, ""])[1])
    h = re.search(r"export const KERNEL_HASH = (\d+);", text or "")
    table = re.findall(r'\{ kind: "([^"]+)"', text or "")
    return kinds, int(h.group(1)) if h else None, table


def registered_kinds(text):
    code = "\n".join(l for l in (text or "").split("\n") if not l.lstrip().startswith("//"))
    return re.findall(r'"([a-z0-9-]+)"\s*=>\s*[a-z0-9_]+', code)


def node_exports(data):
    with tempfile.TemporaryDirectory() as tmp:
        wasm = Path(tmp, "k.wasm")
        wasm.write_bytes(data)
        r = subprocess.run(["node", "-e", NODE_EXPORTS, str(wasm)], capture_output=True, text=True)
    try:
        return json.loads(r.stdout)
    except ValueError:
        return {"error": (r.stdout + r.stderr)[-300:]}


def gzip9(data):
    return len(subprocess.run(["gzip", "-9c"], input=data, capture_output=True, check=True).stdout)


def check(tree, rebuild, rebuild_fn=None, virtual=False):
    errors = []
    F, B = tree["files"], tree["bytes"]
    for p in [ROOT_CARGO, TOOLCHAIN, CRATE_CARGO, KINDS_RS, FFI_RS, CLIPPY, GEN_TS, MANIFEST, B64]:
        if F.get(p) is None:
            errors.append(f"{p} is missing")
    if B.get(WASM) is None:
        errors.append(f"{WASM} is missing")
    if errors:
        return errors
    wasm = B[WASM]

    # 1. the manifest
    m = read_manifest(F[MANIFEST])
    advice = "run scripts/build-kernel.sh and commit primitives/src/kernel/"
    profile = profile_of(F[ROOT_CARGO])
    if not profile:
        errors.append(f"{ROOT_CARGO}: no [profile.kernel] table")
    elif "\n".join(m["profile"]) != profile or m["profile_sha"] != sha256((profile + "\n").encode()):
        errors.append(f"{MANIFEST}: the [profile.kernel] table of {ROOT_CARGO} changed since the module was built: {advice}")
    if set(m["src"]) != set(tree["src"]):
        errors.append(f"{MANIFEST}: lists {sorted(set(m['src']) ^ set(tree['src']))} differently from the tree: a source file was "
                      f"added or removed since the module was built: {advice}")
    for path, data in sorted(tree["src"].items()):
        if path in m["src"] and m["src"][path] != (sha256(data), blob_id(data)):
            errors.append(f"{path}: changed since the module was built ({MANIFEST} holds another hash): {advice}")
    if m["wasm"] != (sha256(wasm), len(wasm)):
        errors.append(f"{WASM} is not the module {MANIFEST} names: {advice}")
    if F[B64] != base64.b64encode(wasm).decode():
        errors.append(f"{B64} is not the base64 of {WASM}: {advice}")
    if m["target"] != "wasm32-unknown-unknown" or not m["rustc"]:
        errors.append(f"{MANIFEST}: no target or no rustc -Vv record")

    # 2. the exports and the kind table
    info = node_exports(wasm)
    if "error" in info:
        errors.append(f"{WASM}: not a loadable module ({info['error']})")
    else:
        if info["exports"] != EXPORTS:
            errors.append(f"{WASM}: exports {info['exports']}, expected exactly {EXPORTS}: the export set is fixed, a behaviour "
                          f"adds a row to the kind table and never an export")
        kinds, h, table = gen_values(F[GEN_TS])
        if h is None:
            errors.append(f"{GEN_TS}: no KERNEL_HASH")
        elif info.get("kinds") != h:
            errors.append(f"{WASM}: dx_kinds() is {info.get('kinds')} but {GEN_TS} says KERNEL_HASH {h}: the kernel's kind table "
                          f"and the page's differ; regenerate (`DX_BLESS=1 cargo test -p dioxus-primitives behaviours`) and rebuild")
    kinds, h, table = gen_values(F[GEN_TS])
    reg = registered_kinds(F[KINDS_RS])
    if kinds != reg:
        errors.append(f"{GEN_TS}: KINDS {kinds} is not the kinds {KINDS_RS} registers, in order ({reg}): a kind was removed "
                      f"from the kernel (or added) without regenerating")
    if table != kinds:
        errors.append(f"{GEN_TS}: KERNEL_KINDS {table} is not KINDS {kinds}")
    cargo = re.sub(r"^#.*$", "", F[CRATE_CARGO], flags=re.M)
    dep = re.search(r"^\[dependencies\][ \t]*\n((?:(?!\[).*\n?)*)", cargo, re.M)
    if dep and dep.group(1).strip():
        errors.append(f"{CRATE_CARGO}: the decision kernel has no dependency (bytes, build time and the unsafe surface of a "
                      f"dependency are all paid by every page): {dep.group(1).strip().splitlines()[0]}")
    funcs = sorted(re.findall(r'^\s*pub\s+extern\s+"C"\s+fn\s+(\w+)', F[FFI_RS], re.M))
    if funcs != FFI_FUNCTIONS:
        errors.append(f"{FFI_RS}: `extern \"C\"` functions {funcs}, expected exactly {FFI_FUNCTIONS}")
    if tree.get("primitives_build_rs") or re.search(r"^\s*build\s*=", F.get(PRIMITIVES_CARGO) or "", re.M):
        errors.append("primitives/: a build script: dioxus-primitives has none (a consumer's build would run it); the module is committed")

    # 5. the float contract
    for ty in ("f64", "f32"):
        for name in FLOATS:
            if not re.search(r'path\s*=\s*"%s::%s"' % (ty, name), F[CLIPPY]):
                errors.append(f"{CLIPPY}: `{ty}::{name}` is not in disallowed-methods: the same threshold must not flip by an ulp "
                              f"between the page and Blitz")

    # 3. the rebuild
    if rebuild:
        note = rebuild_fn(tree) if rebuild_fn else None
        if note and note[0] == "skip":
            REPORT.append(f"{GATE}: rebuild half skipped: {note[1]}")
        elif note and note[0] == "bytes":
            outputs = note[1]
            for name, want in ((WASM, wasm), (B64, F[B64].encode()), (MANIFEST, F[MANIFEST].encode())):
                if outputs.get(name) != want:
                    errors.append(f"{name}: a rebuild on the pinned toolchain gives other bytes than the committed file: "
                                  f"{advice} (the sources and the manifest agree, so the committed bytes were not built by this toolchain)")
    else:
        REPORT.append(f"{GATE}: rebuild half not run (a full gate run or --rebuild does it)")

    # 4. the size budget (the real file, or the virtual one a self-test case carries)
    measured = {WASM: {"gzip_bytes": gzip9(wasm)}}
    keys = ("RATCHET_NO_BASE", "RATCHET_REQUIRE_BASE")
    saved = {k: os.environ.get(k) for k in keys}
    tmp = None
    try:
        if virtual:
            os.environ.pop("RATCHET_REQUIRE_BASE", None)
            os.environ["RATCHET_NO_BASE"] = "1"
            tmp = tempfile.TemporaryDirectory()
            if tree["budgets"] is not None:
                target = Path(tmp.name) / BUDGETS
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(tree["budgets"], encoding="utf-8")
            R = Ratchet(GATE, BUDGETS, ("gzip_bytes",), root=tmp.name, advice=BUDGET_ADVICE)
        else:
            R = Ratchet(GATE, BUDGETS, ("gzip_bytes",), advice=BUDGET_ADVICE)
        rep = R.check(measured)
    finally:
        for k, v in saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
    errors += list(rep.errors)
    rep.errors = []
    if not virtual:
        REPORT.append(R.format(rep))
        REPORT.append(f"{GATE}: {WASM} is {len(wasm)} bytes raw, {measured[WASM]['gzip_bytes']} gzip -9 (the raw size is reported, not gated)")
    if tmp is not None:
        tmp.cleanup()
    return errors


def real_rebuild(tree):
    """The rebuild half on the real tree: skipped unless the toolchain is the manifest's, else the rebuilt bytes."""
    m = read_manifest(tree["files"][MANIFEST])
    have = [l for l in subprocess.run(["rustc", "-Vv"], capture_output=True, text=True).stdout.split("\n")
            if l and not l.startswith("host:")]
    if have != m["rustc"]:
        return ("skip", f"the toolchain is not the pinned one ({have[0] if have else 'no rustc'}; the manifest records "
                        f"{m['rustc'][0] if m['rustc'] else 'none'}), so only the hashes above are held")
    target_dir = os.environ.get("CARGO_TARGET_DIR", "")
    cleanup = None
    if not target_dir.startswith("/"):
        cleanup = tempfile.TemporaryDirectory()
        target_dir = cleanup.name
    libdir = subprocess.run(["rustc", "--print", "target-libdir", "--target", "wasm32-unknown-unknown"], capture_output=True, text=True)
    if libdir.returncode != 0 or not Path(libdir.stdout.strip()).is_dir():
        return ("skip", "the wasm32-unknown-unknown target is not installed")
    with tempfile.TemporaryDirectory() as out:
        r = subprocess.run(["scripts/build-kernel.sh", "--out", out], env={**os.environ, "CARGO_TARGET_DIR": target_dir},
                           capture_output=True, text=True)
        if r.returncode != 0:
            return ("bytes", {})
        outputs = {WASM: Path(out, "decision-kernel.wasm").read_bytes(), B64: Path(out, "decision-kernel.b64").read_bytes(),
                   MANIFEST: Path(out, "kernel.manifest").read_bytes()}
    if cleanup:
        cleanup.cleanup()
    return ("bytes", outputs)


def mutate(tree, fn):
    t = copy.deepcopy(tree)
    fn(t)
    return t


def set_text(path, old, new, section="files"):
    def f(t):
        text = t[section][path]
        assert old in text, f"self-test anchor missing in {path}: {old!r}"
        t[section][path] = text.replace(old, new, 1)
    return f


def wasm_with_exports(names):
    """A minimal module exporting memory plus the named functions (each `() -> i32` returning 0)."""
    def leb(n):
        out = bytearray()
        while True:
            b = n & 0x7F
            n >>= 7
            out.append(b | (0x80 if n else 0))
            if not n:
                return bytes(out)
    def section(i, body):
        return bytes([i]) + leb(len(body)) + body
    def name(s):
        return leb(len(s)) + s.encode()
    exports = leb(len(names) + 1) + name("memory") + b"\x02\x00"
    for i, n in enumerate(names):
        exports += name(n) + b"\x00" + leb(i)
    bodies = b"".join(b"\x04\x00\x41\x00\x0b" for _ in names)
    return (b"\x00asm\x01\x00\x00\x00" + section(1, b"\x01\x60\x00\x01\x7f") + section(3, leb(len(names)) + b"\x00" * len(names))
            + section(5, b"\x01\x00\x01") + section(7, exports) + section(10, leb(len(names)) + bodies))


def self_test(tree):
    tree["budgets"] = Path(BUDGETS).read_text(encoding="utf-8") if Path(BUDGETS).is_file() else None
    same = lambda t: ("bytes", {WASM: t["bytes"][WASM], B64: t["files"][B64].encode(), MANIFEST: t["files"][MANIFEST].encode()})
    base = check(tree, True, same, virtual=True)
    if base:
        print(f"{GATE}: the unmutated tree is not green, the self-test cannot start:", file=sys.stderr)
        for e in base:
            print("  " + e, file=sys.stderr)
        return 1
    src = "decision-kernel/src/fixture.rs"
    muts = [
        ("a kernel source edited without a rebuild", lambda t: t["src"].__setitem__(src, t["src"][src] + b"\n// edit\n"), "changed since the module was built"),
        ("a kernel source added without a rebuild", lambda t: t["src"].__setitem__("decision-kernel/src/new.rs", b"//"), "differently from the tree"),
        ("a kernel source removed without a rebuild", lambda t: t["src"].pop(src), "differently from the tree"),
        ("the crate manifest edited without a rebuild", lambda t: t["src"].__setitem__(CRATE_CARGO, t["src"][CRATE_CARGO] + b"\n# x\n"), "changed since"),
        ("[profile.kernel] changed without a rebuild", set_text(ROOT_CARGO, 'opt-level = "z"\ndebug = false\ndebug-assertions = false', 'opt-level = "s"\ndebug = false\ndebug-assertions = false'), "[profile.kernel]"),
        ("the toolchain pin changed without a rebuild", lambda t: t["src"].__setitem__(TOOLCHAIN, t["src"][TOOLCHAIN].replace(b"1.97.0", b"1.98.0")), "changed since"),
        ("the module changed", lambda t: t["bytes"].__setitem__(WASM, t["bytes"][WASM] + b"\x00"), "not the module"),
        ("the base64 not the module's", set_text(B64, "A", "B", "files"), "not the base64"),
        ("the manifest without its toolchain record", lambda t: t["files"].__setitem__(MANIFEST, re.sub(r"^rustc: .*\n", "", t["files"][MANIFEST], flags=re.M)), "no target or no rustc"),
        ("an export added to the module", lambda t: (t["bytes"].__setitem__(WASM, wasm_with_exports(FFI_FUNCTIONS + ["dx_decide_fixture"]))), "exports"),
        ("an export missing from the module", lambda t: (t["bytes"].__setitem__(WASM, wasm_with_exports(FFI_FUNCTIONS[:-1]))), "exports"),
        ("the generated hash changed", lambda t: t["files"].__setitem__(GEN_TS, re.sub(r"KERNEL_HASH = \d+", "KERNEL_HASH = 1", t["files"][GEN_TS])), "dx_kinds()"),
        ("a kind removed from the kernel and left in KINDS", set_text(KINDS_RS, '"fixture" => fixture', '// "fixture" => fixture'), "removed from the kernel"),
        ("a kind added to the kernel and not generated", set_text(KINDS_RS, '"fixture" => fixture,', '"fixture" => fixture,\n    "ghost" => ghost,'), "registers"),
        ("a dependency in the kernel crate", set_text(CRATE_CARGO, "[dependencies]", "[dependencies]\nserde = \"1\""), "no dependency"),
        ("a sixth extern C function", set_text(FFI_RS, "pub extern \"C\" fn dx_abi", "pub extern \"C\" fn dx_extra() -> u32 { 0 }\n#[no_mangle]\npub extern \"C\" fn dx_abi"), "extern"),
        ("a build script for the primitives", lambda t: t.__setitem__("primitives_build_rs", True), "build script"),
        ("powi allowed", set_text(CLIPPY, '{ path = "f64::powi"', '{ path = "f64::powi_"'), "f64::powi"),
        ("f32 sin allowed", set_text(CLIPPY, '{ path = "f32::sin"', '{ path = "f32::sin_"'), "f32::sin"),
        ("atan2 allowed", set_text(CLIPPY, '{ path = "f64::atan2"', '{ path = "f64::atan2_"'), "f64::atan2"),
        ("the size budget lower than the module", lambda t: t.__setitem__("budgets", re.sub(r"(decision-kernel\.wasm\t)\d+", r"\g<1>1000", t["budgets"])), "rose above"),
        ("the size budget row missing", lambda t: t.__setitem__("budgets", None), ""),
        ("a rebuild that gives other bytes", None, "rebuild on the pinned toolchain"),
    ]
    failed = 0
    for name, fn, needle in muts:
        if fn is None:
            other = lambda t: ("bytes", {WASM: t["bytes"][WASM] + b"\x00", B64: t["files"][B64].encode(), MANIFEST: t["files"][MANIFEST].encode()})
            errs = check(copy.deepcopy(tree), True, other, virtual=True)
        else:
            errs = check(mutate(tree, fn), True, same, virtual=True)
        if not errs:
            print(f"{GATE}: self-test: `{name}` did NOT turn the gate red", file=sys.stderr)
            failed += 1
        elif needle and not any(needle in e for e in errs):
            print(f"{GATE}: self-test: `{name}` turned the gate red, but not by the rule under test ({needle!r}): {errs[0][:160]}", file=sys.stderr)
            failed += 1
    skip = check(copy.deepcopy(tree), True, lambda t: ("skip", "another toolchain"), virtual=True)
    greens = [
        ("a file the manifest does not name (the crate's tests)", lambda t: None),
        ("another crate added to the root Cargo.toml", set_text(ROOT_CARGO, '"decision-kernel",', '"decision-kernel",\n    "another",')),
        ("a rebuild on another toolchain (skipped)", None),
    ]
    for name, fn in greens:
        errs = skip if fn is None else check(mutate(tree, fn), True, same, virtual=True)
        if errs:
            print(f"{GATE}: self-test: `{name}` must stay green but reported: {errs[0]}", file=sys.stderr)
            failed += 1
    if failed:
        return 1
    print(f"{GATE}: self-test OK ({len(muts)} mutations red, {len(greens)} non-mutations green)")
    return 0


tree = load_tree()
if MODE == "update":
    R = Ratchet(GATE, BUDGETS, ("gzip_bytes",), advice=BUDGET_ADVICE)
    errs, wrote = R.update({WASM: {"gzip_bytes": gzip9(tree["bytes"][WASM])}}, RAISE_REASON)
    for e in errs:
        print(e, file=sys.stderr)
    print(f"{GATE}: {'wrote ' + BUDGETS if wrote else 'baseline unchanged'}")
    sys.exit(1 if errs else 0)
if MODE == "self-test":
    sys.exit(self_test(tree))
tree["budgets"] = None
errors = check(tree, REBUILD, real_rebuild)
for line in REPORT:
    if line:
        print(line)
if errors:
    for e in errors:
        print(e, file=sys.stderr)
    print(f"\n{GATE}: {len(errors)} problem(s). See the header of scripts/{GATE}.sh.", file=sys.stderr)
    sys.exit(1)
print(f"{GATE}: OK -- the manifest matches the sources, the profile, the toolchain pin and the module; the module exports the five "
      "functions and its kind table is the page's; the size is within its budget; the float ban list is whole.")
PYEOF
