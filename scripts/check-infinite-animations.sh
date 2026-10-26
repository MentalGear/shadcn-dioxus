#!/usr/bin/env bash
#
# check-infinite-animations.sh
#
# An `infinite` CSS animation may only animate COMPOSITOR properties -- `transform`, `translate`, `scale`,
# `rotate`, `opacity` -- unless its @keyframes is allowlisted below with a reason. Invisible to cargo and
# clippy, and with CI frozen (dev-docs/backlog.md row 33) nothing else would notice.
#
# THE CLASS (row 174): an animation that runs forever is main-thread work on
# EVERY vsync for as long as its element exists, unless the browser can hand it to the compositor. Only
# `transform`/`translate`/`scale`/`rotate`/`opacity` (and `filter`/`backdrop-filter`, which this gate does not
# allow: they can move pixels) are eligible; anything else (`background-position`, `width`, `color`,
# `clip-path`, `box-shadow`, a registered custom property ...) re-runs style, and usually paint, 60 times a
# second per instance. A skeleton, spinner or text shimmer made that way keeps the main thread awake through
# every scroll (row 174 measured 630-730 "Animation" style recalcs per scroll from the skeletons alone). A finite animation is
# bounded, so it is not this class; only `infinite` is checked.
#
# HOW IT READS THE CSS: every `*.css` under `preview/src` and `preview/assets` (node_modules and build output
# skipped). A rule is "infinite" when its declaration block contains the word `infinite` inside an `animation`
# or `animation-iteration-count` declaration. Its keyframes are the names in that block's `animation` /
# `animation-name` declarations -- and, because a rule that only SWAPS the name (`animation-name: x-rtl`)
# inherits the iteration count from another rule, every `animation-name` longhand in a file that also has an
# infinite rule counts as infinite too (conservative: it can only add checks). Each such keyframe name must be
# defined, and every property in its frames must be a compositor property, or the (stylesheet, keyframes)
# pair must be in ALLOW with a reason. An ALLOW entry that no longer matches an infinite keyframe with a
# non-compositor property fails the run, so the list cannot rot into blanket permission.
#
# SECOND RULE -- REDUCED MOTION (row 176): every `infinite` animation must be stopped or replaced under
# `@media (prefers-reduced-motion: reduce)`. Covered when ANY of these holds, else the run fails:
#   1. the infinite rule is itself inside `@media (prefers-reduced-motion: no-preference)`;
#   2. the infinite rule is itself inside `@media (prefers-reduced-motion: reduce)` (it IS the replacement);
#   3. a reduced-motion rule in the same sheet (or in preview/assets/dx-components-theme.css) names the SAME
#      selector and stops or replaces the animation: `animation: none` / a shorthand without `infinite`,
#      `animation-name: none`, or a finite `animation-iteration-count`. A duration-only rule
#      (`animation-duration: var(--dx-motion-duration-reduced)`) is REJECTED: an infinite loop of 0.01 ms
#      does not stop, it samples a random phase every frame and flickers;
#   4. the sheet is a component sheet whose component applies `use_motion*` (the same wiring
#      check-motion-gating.sh requires), because the theme carries ONE reduced-motion rule for every
#      `[data-dx-motion-key]` host that sets `animation: none !important` on the host and everything under it.
#      That theme rule must exist and must stop animations, or case 4 does not count.
# A looping component therefore gets reduced motion by calling `use_motion`; only a loop outside that
# convention (the OTP caret, the shimmer utility) writes its own block.
#
# Usage: scripts/check-infinite-animations.sh
# Exit 0: clean. Exit 1: a violation, with file:line detail on stderr.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

python3 - <<'PYEOF'
import fnmatch
import re
import sys
from pathlib import Path

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402
from css import parse, split_top as split_top_raw  # noqa: E402

COMPOSITOR = {"transform", "translate", "scale", "rotate", "opacity", "animation-timing-function", "offset-path"}
COMPOSITOR.discard("offset-path")  # not compositor-driven everywhere; keep the list to what every engine accelerates

# (stylesheet path, fnmatch pattern on the @keyframes name, reason). Keep the reason about WHY this one cannot
# be a transform/opacity animation -- "it is old" is not a reason.
ALLOW = Allowlist("check-infinite-animations", [
    ("preview/assets/dx-effects.css", "dx-shimmer-sweep",
     "the highlight is painted INSIDE the glyphs with `background-clip: text`, so the only thing that can move it "
     "is the background itself; a translated pseudo-element would move the glyphs with it. Paint-only (no style "
     "cascade or layout), opt-in per element, and `prefers-reduced-motion` turns it off"),
], fields=("path", "pattern"))

stylesheets = []
for root in ("preview/src", "preview/assets"):
    for p in sorted(Path(root).rglob("*.css")):
        if "node_modules" in p.parts or "target" in p.parts:
            continue
        stylesheets.append(p)

errors = []
keyframes = {}  # name -> list of (path, line, set(properties))
KF_RE = re.compile(r"@keyframes\s+([\w-]+)\s*$")
DECL_RE = re.compile(r"(?<![\w-])([a-z-]+)\s*:\s*([^;{}]+)")

inf_rule_sels = []   # (path, line, selector_text, media_chain)  -- one per rule that declares `infinite`
reduced_rules = []   # (path, selector_text, {prop: [values]})   -- every rule inside a reduced-motion media

RED_RE = re.compile(r"prefers-reduced-motion\s*:\s*reduce")
NOPREF_RE = re.compile(r"prefers-reduced-motion\s*:\s*no-preference")

def walk(css: Path, nodes, infinite_rules, namers, media=()):
    """The blocks of one nesting level of the shared reader's tree (scripts/lib/css.py); a reported line is the
    line of the block's `{`."""
    for node in nodes:
        sel, body = node.prelude, node.body
        m = KF_RE.search(sel)
        if m:
            props = set()
            for frame in node.children:
                for d in DECL_RE.finditer(frame.body):
                    props.add(d.group(1))
            keyframes.setdefault(m.group(1), []).append((css.as_posix(), node.brace_line, props))
        elif sel.startswith("@"):
            walk(css, node.children, infinite_rules, namers, media + (sel,))
        else:
            decls = {}
            for d in DECL_RE.finditer(body):
                decls.setdefault(d.group(1), []).append(d.group(2).strip())
            line = node.brace_line
            anim = decls.get("animation", []) + decls.get("-webkit-animation", [])
            count = decls.get("animation-iteration-count", [])
            names_long = decls.get("animation-name", [])
            is_inf = any(re.search(r"(?<![\w-])infinite(?![\w-])", v) for v in anim + count)
            names = set()
            for v in anim:
                for tok in re.split(r"[\s,]+", strip_funcs(v)):
                    if re.fullmatch(r"-?[a-zA-Z_][\w-]*", tok) and tok not in KW:
                        names.add(tok)
            for v in names_long:
                for tok in re.split(r"[\s,]+", v):
                    if re.fullmatch(r"-?[a-zA-Z_][\w-]*", tok) and tok not in ("none", "initial", "inherit", "unset", "revert"):
                        names.add(tok)
                        namers.append((css.as_posix(), line, tok))
            if is_inf:
                inf_rule_sels.append((css.as_posix(), line, sel, media))
                for nme in names:
                    infinite_rules.append((css.as_posix(), line, nme))
            if any(RED_RE.search(m) for m in media):
                reduced_rules.append((css.as_posix(), sel, decls))

def strip_funcs(v: str) -> str:
    prev = None
    while prev != v:
        prev = v
        v = re.sub(r"[\w-]*\([^()]*\)", " ", v)
    return v

KW = {
    "infinite", "none", "normal", "reverse", "alternate", "alternate-reverse", "forwards", "backwards", "both",
    "running", "paused", "linear", "ease", "ease-in", "ease-out", "ease-in-out", "step-start", "step-end",
    "initial", "inherit", "unset", "revert",
}

infinite_refs = []   # (path, line, name)
n_sheets_with = 0
for css in stylesheets:
    sheet = parse(css.read_text())
    src = sheet.text
    inf, namers = [], []
    walk(css, sheet.nodes, inf, namers)
    if inf:
        n_sheets_with += 1
        infinite_refs += inf
        # a name-only rule in a file with an infinite rule inherits `infinite` (see header)
        infinite_refs += namers
    parsed = len(re.findall(r"@keyframes\s+[\w-]+\s*\{", src))
    stray = len(re.findall(r"@keyframes\b", src)) - parsed
    if stray:
        errors.append(f"{css}: {stray} `@keyframes` token(s) this script could not parse -- fix the parse, do not skip them")

n_allowed = 0
checked = set()
for path, line, name in infinite_refs:
    if name in checked:
        continue
    checked.add(name)
    defs = keyframes.get(name)
    if not defs:
        errors.append(f"{path}:{line}: infinite animation `{name}` has no @keyframes in preview/src or preview/assets -- "
                      f"define it there (or this gate cannot see what it animates)")
        continue
    bad = set()
    for kpath, kline, props in defs:
        bad |= {p for p in props if p not in COMPOSITOR}
    if not bad:
        continue
    if ALLOW.find(lambda e: any(kp == e.path for kp, _, _ in defs) and fnmatch.fnmatchcase(name, e.pattern)):
        n_allowed += 1
        continue
    kpath, kline, _ = defs[0]
    errors.append(
        f"{kpath}:{kline}: @keyframes {name} runs `infinite` (from {path}:{line}) but animates {sorted(bad)} -- "
        f"only transform/translate/scale/rotate/opacity run on the compositor; anything else costs the main thread "
        f"on every vsync for as long as the element exists. Rewrite it with those properties, or allowlist it in "
        f"scripts/check-infinite-animations.sh with the reason it cannot be"
    )

errors += ALLOW.problems()

# ---- reduced motion: every infinite rule is stopped or replaced ---------------------------------------------
THEME = "preview/assets/dx-components-theme.css"
HOST = "[data-dx-motion-key]"

def norm(sel: str) -> str:
    return re.sub(r"\s+", " ", sel.replace("'", '"')).strip()

def selector_list(sel: str):
    return [norm(x) for x in split_top_raw(sel) if x.strip()]

def stops(decls) -> bool:
    for v in decls.get("animation", []) + decls.get("-webkit-animation", []):
        if not re.search(r"(?<![\w-])infinite(?![\w-])", v):
            return True
    if any(v.strip() == "none" for v in decls.get("animation-name", [])):
        return True
    return any(not re.search(r"(?<![\w-])infinite(?![\w-])", v) for v in decls.get("animation-iteration-count", []))

def duration_only(decls) -> bool:
    return bool(decls.get("animation-duration")) and not stops(decls)

# The theme's host rule: a reduced-motion rule whose selector list names the host and stops animations.
host_rule = any(
    p == THEME and HOST in selector_list(sel) and stops(decls)
    for p, sel, decls in reduced_rules
)

def component_wired(name: str) -> bool:
    gate = re.compile(r"use_motion\w*")
    cands = [Path(f"preview/src/components/{name}/component.rs"), Path(f"primitives/src/{name}.rs")]
    pd = Path(f"primitives/src/{name}")
    if pd.is_dir():
        cands += sorted(pd.rglob("*.rs"))
    pv = Path(f"preview/src/components/{name}")
    if pv.is_dir():
        cands += sorted(pv.rglob("*.rs"))
    return any(c.is_file() and gate.search(c.read_text()) for c in cands)

covered_by_host, covered_by_own = set(), set()
for path, line, sel, media in inf_rule_sels:
    if any(NOPREF_RE.search(m) or RED_RE.search(m) for m in media):
        continue
    pp = Path(path)
    wired_host = (
        host_rule
        and pp.name == "style.css"
        and pp.parent.parent.as_posix() == "preview/src/components"
        and component_wired(pp.parent.name)
    )
    for one in selector_list(sel):
        own = [d for p, rs, d in reduced_rules if p in (path, THEME) and one in selector_list(rs)]
        if any(stops(d) for d in own):
            covered_by_own.add((path, one))
        elif wired_host:
            covered_by_host.add((path, one))
        elif any(duration_only(d) for d in own):
            errors.append(
                f"{path}:{line}: `{one}` loops forever and its prefers-reduced-motion rule only shortens "
                f"`animation-duration`. An infinite 0.01 ms loop does not stop, it flickers: use `animation: none` "
                f"(or a finite replacement) instead"
            )
        else:
            errors.append(
                f"{path}:{line}: `{one}` loops forever (`infinite`) with no prefers-reduced-motion answer. Either "
                f"apply `use_motion` in its component (the theme's `{HOST}` rule then stops it under reduced motion), "
                f"or add `@media (prefers-reduced-motion: reduce) {{ {one} {{ animation: none; }} }}` to the sheet"
            )
if covered_by_host and not host_rule:
    errors.append(f"{THEME}: no `@media (prefers-reduced-motion: reduce)` rule with `{HOST}` and `animation: none` -- "
                  f"the theme rule that stops every use_motion host is missing")

if not stylesheets or not keyframes or not infinite_refs:
    errors.append("no stylesheet, no @keyframes or no infinite animation was found -- the scan in this script is broken")

if errors:
    for e in errors:
        print(f"check-infinite-animations: {e}", file=sys.stderr)
    print(f"check-infinite-animations: FAILED -- {len(errors)} problem(s)", file=sys.stderr)
    sys.exit(1)
print(
    f"check-infinite-animations: OK -- {len(checked)} infinite @keyframes ({', '.join(sorted(checked))}) in "
    f"{n_sheets_with} stylesheets; compositor-only except {n_allowed} allowlisted; reduced motion: "
    f"{len(covered_by_host)} loop selector(s) via the theme `{HOST}` rule, {len(covered_by_own)} via their own block"
)
PYEOF
