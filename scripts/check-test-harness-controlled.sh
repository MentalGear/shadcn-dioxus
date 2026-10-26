#!/usr/bin/env bash
#
# check-test-harness-controlled.sh  (backlog row 431)
#
# A concern implemented once in `primitives/src` is used, not copied: the test runtime harness is `with_runtime` in `primitives/src/test_support.rs`, and controlled-or-uncontrolled state is `use_controlled` / `use_controlled_state` in `primitives/src/lib.rs`. Eight private `with_runtime` copies (each with its own `TestHarnessProps`) and three hand-rolled "internal signal, memo of `prop.or(internal)`, callback that sets one and notifies" copies had grown because nothing looked for the second one; this gate fails the ninth harness and the fourth hand-rolled controlled memo.
#
# THE RULE (every `.rs` under `primitives/src` and `preview/src`, comments and string contents ignored):
#   1. `struct TestHarnessProps` and `fn with_runtime` are declared only in `primitives/src/test_support.rs`; a test imports `crate::test_support::with_runtime` instead of writing a harness.
#   2. a `use_memo(...)` whose closure resolves a controlled prop with `.unwrap_or_else(&*signal)` or `.or_else(&*signal)` -- the memo half of a hand-rolled controlled state -- appears only inside `use_controlled_state`; every other component takes `use_controlled` (or `use_controlled_state` when it also needs the internal signal).
#   3. the one harness and the one implementation still exist where this gate expects them.
# A site that cannot go through the helper takes a reasoned entry in ALLOW (`scripts/lib/allow.py`: a blank reason, a duplicate or an entry that matches nothing fails), the way `drawer.rs` does. A bare `.unwrap_or_else(&*x)` outside a `use_memo` (a different thing) is not flagged. `--self-test` runs mutations that must turn red.
#
# Usage: scripts/check-test-harness-controlled.sh [--self-test] [--list]
# Exit status: 0 when clean, 1 on a violation, 2 on a self-test failure.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

exec python3 -I - "$@" <<'PYEOF'
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402

HARNESS_HOME = "primitives/src/test_support.rs"
CONTROLLED_HOME = "primitives/src/lib.rs"

# (path, pattern, reason): a controlled-prop memo that is deliberately not `use_controlled`.
ALLOW_ENTRIES = [
    (CONTROLLED_HOME, "use_controlled_state",
     "the one implementation: `use_controlled_state` is where the memo is written, `use_controlled` wraps it"),
    ("primitives/src/drawer.rs", "active_snap_point",
     "the snap-point state is an `Option<SnapPoint>` whose uncontrolled start is `default_active_snap_point` even when "
     "the prop is `Some` at mount, and `reset_active_snap_point` writes that default back without notifying; "
     "`use_controlled_state` seeds the fallback from the controlled value and has no silent reset, so a migration would "
     "change what an uncontrolled-after-controlled drawer shows and add a prop-wrapping memo, not provably identical"),
]

ROOTS = ["primitives/src", "preview/src"]


def blank(text):
    """Comments and string contents replaced by spaces (newlines kept), so a doc example or a message never matches."""
    out, i, n = [], 0, len(text)
    while i < n:
        two = text[i:i + 2]
        if two == "//":
            while i < n and text[i] != "\n":
                out.append(" "); i += 1
            continue
        if two == "/*":
            while i < n and text[i:i + 2] != "*/":
                out.append("\n" if text[i] == "\n" else " "); i += 1
            out.append("  "); i += 2
            continue
        m = re.match(r'[bB]?r(#*)"', text[i:])
        if m:
            closer = '"' + m.group(1)
            end = text.find(closer, i + m.end())
            end = n if end == -1 else end + len(closer)
            out.extend("\n" if ch == "\n" else " " for ch in text[i:end]); i = end
            continue
        m = re.match(r"'(?:\\.[^']*|[^\\'])'", text[i:])
        if m:   # a char literal such as '"'; a lifetime `'a` has no closing quote right after one char
            out.extend(" " * m.end()); i += m.end()
            continue
        if text[i] == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2; continue
                if text[j] == '"':
                    j += 1; break
                j += 1
            out.extend("\n" if ch == "\n" else " " for ch in text[i:j]); i = j
            continue
        out.append(text[i]); i += 1
    return "".join(out)


HARNESS = re.compile(r"\bstruct\s+TestHarnessProps\b|\bfn\s+with_runtime\b")
# the memo half of a hand-rolled controlled state: a use_memo closure (no `;` before it ends) that falls back to a signal
MEMO = re.compile(r"\buse_memo\s*\(\s*(?:move\s*)?\|\|[^;]*?\.(?:unwrap_or_else|or_else)\(\s*&\*\s*\w+\s*\)")


def scan(root: Path, allow: Allowlist):
    errors, scanned = [], 0
    for base in ROOTS:
        for p in sorted((root / base).rglob("*.rs")):
            rel = p.relative_to(root).as_posix()
            scanned += 1
            text = blank(p.read_text(encoding="utf-8", errors="replace"))
            if rel != HARNESS_HOME:
                for m in HARNESS.finditer(text):
                    line = text.count("\n", 0, m.start()) + 1
                    errors.append(f"{rel}:{line}: `{m.group(0)}` -- the one test runtime harness is `with_runtime` in "
                                  f"{HARNESS_HOME}; import `crate::test_support::with_runtime`")
            for m in MEMO.finditer(text):
                line = text.count("\n", 0, m.start()) + 1
                snippet = " ".join(m.group(0).split())
                if allow.find(lambda e: e.path == rel and e.pattern in snippet + " " + enclosing(text, m.start())):
                    continue
                errors.append(f"{rel}:{line}: a hand-rolled controlled state (`{snippet[:90]}`); take "
                              f"`use_controlled` (or `use_controlled_state` for the internal signal too) from {CONTROLLED_HOME}")
    return errors, scanned


def enclosing(text, pos):
    """The text of the `fn` item around `pos` (name included), so an allowlist pattern can name the function or the binding."""
    start = text.rfind("\nfn ", 0, pos)
    start = max(start, text.rfind("\npub fn ", 0, pos), text.rfind("\npub(crate) fn ", 0, pos))
    let = text.rfind("let ", 0, pos)
    return text[max(start, 0):pos] if start >= 0 else text[let:pos]


def positives(root: Path):
    errs = []
    ts = (root / HARNESS_HOME).read_text(encoding="utf-8") if (root / HARNESS_HOME).exists() else ""
    if not re.search(r"\bpub fn with_runtime\b", blank(ts)):
        errs.append(f"{HARNESS_HOME}: `pub fn with_runtime`, the one test runtime harness, is gone")
    lib = (root / CONTROLLED_HOME).read_text(encoding="utf-8") if (root / CONTROLLED_HOME).exists() else ""
    lib = blank(lib)
    if not MEMO.search(lib) or not re.search(r"\bfn\s+use_controlled\b", lib) or not re.search(r"\bfn\s+use_controlled_state\b", lib):
        errs.append(f"{CONTROLLED_HOME}: `use_controlled` / `use_controlled_state` (the one controlled-state implementation) is gone")
    return errs


def run(root: Path):
    allow = Allowlist("check-test-harness-controlled", ALLOW_ENTRIES, fields=("path", "pattern"))
    errors, scanned = scan(root, allow)
    errors += positives(root)
    errors += [f"ALLOW: {p}" for p in allow.problems()]
    return errors, scanned, allow


def self_test():
    files = {
        HARNESS_HOME: "pub fn with_runtime<R>(f: impl Fn() -> R) -> R { f() }\n",
        CONTROLLED_HOME: "pub fn use_controlled() {}\npub(crate) fn use_controlled_state() {\n"
                         "    let value = use_memo(move || prop.cloned().unwrap_or_else(&*internal_value));\n}\n",
        "primitives/src/drawer.rs": "pub fn Drawer() {\n    let active_snap_point = use_memo(move || controlled().or_else(&*internal_active));\n}\n",
        "primitives/src/ok.rs": "fn f() { let was_open = open.write().take().unwrap_or_else(&*is_open); }\n"
                                "// let x = use_memo(move || a.unwrap_or_else(&*b));\n",
    }
    mutations = {
        "a copied harness struct": ("primitives/src/slider.rs", "mod tests {\n struct TestHarnessProps { f: u8 }\n}\n"),
        "a copied with_runtime fn": ("preview/src/x.rs", "fn with_runtime(f: impl Fn()) {}\n"),
        "a hand-rolled controlled memo": ("primitives/src/alert_dialog.rs",
                                          "fn A() {\n    let open = use_memo(move || (props.open)().unwrap_or_else(&*open_signal));\n}\n"),
        "a multi-line controlled memo": ("primitives/src/accordion.rs",
                                         "fn A() {\n    let v = use_memo(move || {\n        c.cloned().unwrap_or_else(&*internal)\n    });\n}\n"),
    }

    def build(extra):
        d = Path(tempfile.mkdtemp(prefix="check-thc-"))
        for rel, body in {**files, **extra}.items():
            (d / rel).parent.mkdir(parents=True, exist_ok=True)
            (d / rel).write_text(body)
        (d / "preview/src").mkdir(parents=True, exist_ok=True)
        return d

    failures = []
    errors, _, _ = run(build({}))
    if errors:
        failures.append(f"clean fixture reported: {errors}")
    for name, (rel, body) in mutations.items():
        errors, _, _ = run(build({rel: body}))
        if not errors:
            failures.append(f"mutation `{name}` was not caught")
    errors, _, _ = run(build({HARNESS_HOME: "// gone\n"}))
    if not any("test runtime harness" in e for e in errors):
        failures.append("a missing harness was not caught")
    errors, _, _ = run(build({"primitives/src/drawer.rs": "pub fn Drawer() {}\n"}))
    if not any(e.startswith("ALLOW:") for e in errors):
        failures.append("a stale allowlist entry was not caught")
    return failures


if "--self-test" in sys.argv:
    f = self_test()
    if f:
        print("\n".join(f), file=sys.stderr)
        sys.exit(2)
    print("check-test-harness-controlled: self-test OK")
    sys.exit(0)

errors, scanned, allow = run(Path("."))
if "--list" in sys.argv:
    print(allow.listing())
    sys.exit(0)
fail = bool(errors)
for e in errors:
    print(e)
if fail:
    print(f"\ncheck-test-harness-controlled: {len(errors)} problem(s). See the header of scripts/check-test-harness-controlled.sh.",
          file=sys.stderr)
    sys.exit(1)
st = self_test()
if st:
    print("check-test-harness-controlled: self-test FAILED\n" + "\n".join(st), file=sys.stderr)
    sys.exit(2)
print(f"check-test-harness-controlled: OK ({scanned} file(s): one test harness, controlled state through use_controlled)")
PYEOF
