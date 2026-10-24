#!/usr/bin/env python3
"""history-names.py -- the engine behind scripts/check-history-names.sh (backlog row 384).

The codebase describes only its current state. check-history-comments.sh keeps that true of comments; this keeps
it true of NAMES, the part of a test that stays visible in every report: a Playwright `test(` / `test.describe(` /
`describe(` / `it(` / `test.step(` title, a Rust `#[test]` function name and a spec file name. A name says what
the test asserts; the story of the bug that produced it lives in the backlog row and is linked from a comment.

Unlike the comment gate this one has no ratchet baseline: a name is clean or it is not, so a hit fails the run,
and the only exemption is a reasoned ALLOW entry (path, name substring, reason) that goes stale the moment the
name changes. The marker table reuses scripts/comment-scan.py's `history_reason` (ISO dates 2026-08+, SHAs,
"used to", "previously", "before the fix", "no longer", "round5", "batch 13", "pre-fix", "owner report", ...)
and adds the forms that only a name carries (NAME_MARKERS below).

Usage: python3 -I scripts/history-names.py [--root DIR] [--inventory] [--self-test]
Exit status: 0 clean, 1 a name carries history or an ALLOW entry is stale, 2 bad usage.
"""
import importlib.util
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

_spec = importlib.util.spec_from_file_location("comment_scan", os.path.join(HERE, "comment-scan.py"))
cs = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cs)

# Where the names live: spec files and Rust sources. The vendored APG reference tree and the fixtures are skipped
# like the comment gate skips them (cs.SKIP_PREFIXES); so is this gate's own self-test text.
SPEC_RE = re.compile(r"\.(?:spec|test)\.(?:ts|tsx|js|mjs|cjs)$")
SKIP_FILES = ("scripts/history-names.py", "scripts/check-history-names.sh")

# Forms only a name carries, matched on the lower-cased name with `_` and `-` read as spaces (a Rust fn name and a
# file name are slugs; a title is prose). Each entry: (regex, label).
NAME_MARKERS = [
    (re.compile(r"\(regression guard\)|\bregression guard\b|\bguard for\b"), "regression guard"),
    (re.compile(r"\bregressions?\b"), "regression"),
    (re.compile(r"\b(?:round|batch|wave|lane|phase|stage)\s?\d+\b|\b(?:parity|parts|batch|round|wave|lane|phase|stage)\d+\b"), "round/batch n"),
    (re.compile(r"\b(?:user|owner|bug|live[- ]site)s?[- ]report(?:s|ed)?\b|\bowner'?s\b|\bowners\b"), "reported by"),
    (re.compile(r"\b(?:pre|post)[- ]fix\b|\b(?:before|after|without|with) the (?:fix|patch|change|rewrite)\b|\bfix(?:ed)? (?:in|by) (?:row|batch|lane)\b"), "pre-fix"),
    (re.compile(r"\bold (?:behaviou?r|way|bug)\b|\bformerly\b"), "the old"),
    (re.compile(r"\bno longer\b|\bany more\b|\banymore\b"), "no longer"),
    (re.compile(r"\([^)]*\blane\b[^)]*\)|\b[\w-]+ lane (?:sweep|report|fix)\b"), "lane"),
    (re.compile(r"\bhotfix\b|\bfollow[- ]?ups?\b"), "follow-up"),
    (re.compile(r"(?<![\d])2026[ _-](?:0[89]|1[0-2])\b|(?<![\d])20(?:2[7-9]|[3-9]\d)[ _-](?:0[1-9]|1[0-2])\b"), "date"),
]
# A slug read as prose: `-`/`_` are spaces, camelCase and digits stay.
_SLUG = re.compile(r"[-_]+")

# Before/after CONTRAST (row 384): a title that says what a change did not move ("selecting a sub-item still closes the
# whole menu", "keyboard is unchanged: ArrowRight ...", "hovering still scrolls") compares the code with its earlier
# shape, which is history even with no date in it. The rules are narrow on purpose; a title that states a guarantee
# under a condition or after an operation is a present-tense assertion and stays:
#   still       `still` followed by a finite verb (`still closes`, `still scrolls`, `still drag`: a word in -s, or a base
#               verb from _STILL_VERBS) unless the title names a condition or an operation the guarantee holds under
#               (while / when / once / until / if / under / with / without / inside / after / afterwards / during /
#               across ...: "keeps its background while the pointer is still over it", "under RTL, Rewind still
#               wraps", "after prepending history the fade still follows" are guarantees under a condition) or a
#               determiner stands right before it (`a still shape`, `the still matching item`
#               are adjectives), or it continues a predicate (`... and still pages correctly`: a conjunction or comma
#               right before it), or it stands in a LATER clause (after " and ", " but ", ";" or ", ") whose earlier
#               clause has an action verb (`... and a second open still does`, `survives a reload and the jump
#               target still exists`); a `still` in the first clause (`a scroll still closes an open card`) flags.
#               `still` before an adjective, preposition or participle (`still popover-open`, `still in
#               flight`, `is still opaque`, `is still a click`) describes a state the test observes and stays
#   unchanged   (a) `is|are unchanged:` -- the claim as a label for the cases after the colon; (b) the title LEADS with
#               `unchanged` (after an optional short `label: `); (c) a parenthetical `(unchanged)`; (d) `is|are
#               unchanged` with no operation named in the title (nothing was clicked, scrolled, cancelled, reset ... so
#               what it is compared with can only be the old build). `the order is unchanged` after `Escape cancels a
#               lift`, `offset ... is unchanged after scrolling` and a Rust `..._is_unchanged` (an identity test:
#               output equals input) are assertions and stay; the unchanged rules do not read Rust or file names
#   regained / restored / again   `regained`, `is restored` as the end of a clause, and `again` only after a
#               works/passes/green verb (`works again`, `is green again`), each unless a condition or operation word
#               is in the title; `closing it again restores X`, `focus restored` and `scroll is locked while open and
#               restored after Escape` repeat or undo an action the test performs, which is behaviour
_COND = re.compile(r"\b(?:while|whilst|when|whenever|once|until|if|unless|as long as|so long as|provided)\b")
_DETERMINER_BEFORE = re.compile(r"\b(?:a|an|the|its|their|this|that|any|each|every|no|one|my|our)\s+$")
_CONJ_BEFORE = re.compile(r"(?:\b(?:and|but|or|yet|so|also)|,)\s+$")
# a later clause starts after " and ", " but ", ";" or ", "; its `still` continues a guarantee when an earlier clause has a
# verb of action or event (a verb of _STILL_VERBS in -s/-es/-ed, or renders/exists/survives/...; a bare noun such as `Escape`
# in `Escape and Close still close it` is a compound subject, not a clause)
_CLAUSE_BREAK = re.compile(r"\s(?:and|but)\s|;|,\s")
_OPERATION = re.compile(
    r"\b(?:after|before|across|during|past|following|then|cancel\w*|escape|esc|leav\w+|reset|reload\w*|click\w*|press\w*|"
    r"typ(?:e|es|ed|ing)|hover\w*|scroll\w*|drag\w*|pick\w*|stag\w+|toggl\w+|clos\w+|open\w+|switch\w*|select\w*|"
    r"resiz\w+|blur\w*|focus\w*|revert\w*|undo\w*|reject\w*|block\w*|clamp\w*|apply|applied|commit\w*|submit\w*|"
    r"navigat\w+|re-?render\w*|re-?mount\w*|paging|page|flick\w*|tap\w*|key(?:s|stroke\w*)?|enter|arrow\w*)\b")
_STILL_COND = re.compile(r"\b(?:while|whilst|when|whenever|once|until|if|unless|as long as|so long as|provided|under|with|without|inside|"
                         r"within|after|afterwards|following|during|across|then|later|past)\b")
_STILL_VERBS = ("be do have hold work apply drag drive glide close open fire run move show hide match render pass keep follow "
                "dismiss flip count ring band win fill lose clip trap turn mask read stack scroll submit select toggle resolve "
                "swap restore reorder paint center centre stay remain reach cover exist use get take make give let set sit "
                "start stop fit land jump snap survive resume animate persist propagate respond behave update clear cancel "
                "block fail fall flow grow pick point press reject reveal rest return sort split step switch trigger wrap")
_ACTION_VERB = re.compile(r"\b(?:%s|focus|dismiss)(?:s|es|ed)?\b(?!-)|\b(?:renders?|exists?|survives?|produces?|draws?|lands?|glides?)\b" % "|".join(_STILL_VERBS.split()))
# `still` + verb: a 3rd-person -s word (not an adverb or noun in -ss/-us/-ous) or a base verb for a plural subject
_STILL = re.compile(r"\bstill\s+(?!(?:less|across|status|focus|process|always|thus|plus|various|previous|its|this|us|as)\b)"
                    r"(?:\w{2,}s|%s)\b(?!-)" % "|".join(_STILL_VERBS.split()))
_UNCHANGED_LABEL = re.compile(r"\b(?:is|are) unchanged\s*:")
_UNCHANGED_LEAD = re.compile(r"^\W*(?:[\w ,()/.'+-]{1,40}:\s*)?unchanged\b")
_UNCHANGED_PAREN = re.compile(r"\(\s*unchanged\s*\)")
_UNCHANGED_IS = re.compile(r"\b(?:is|are) unchanged\b")
_REGAINED = re.compile(r"\bregained\b")
_RESTORED_END = re.compile(r"\b(?:is|are|was|were|has been|have been|got) (?:now |fully )?restored\s*(?:[:;,)]|$)")
_AGAIN = re.compile(r"\b(?:works?|passes|is green|are green|green|recovers?) again\b|\bagain (?:works|passes|green)\b")


def _continues_earlier_action(before):
    breaks = list(_CLAUSE_BREAK.finditer(before))
    if not breaks:
        return False
    earlier = before[:breaks[-1].start()]
    return bool(_ACTION_VERB.search(earlier))


def contrast_reason(text, slug=False):
    """A before/after contrast in a lower-cased title or slug (see the table above), or None."""
    for m in _STILL.finditer(text):
        before = text[:m.start()]
        if _STILL_COND.search(text) or _DETERMINER_BEFORE.search(before) or _CONJ_BEFORE.search(before) \
                or _continues_earlier_action(before):
            continue
        return "still (before/after)"
    if not slug:
        if _UNCHANGED_LABEL.search(text) or _UNCHANGED_LEAD.search(text) or _UNCHANGED_PAREN.search(text):
            return "unchanged (before/after)"
        if _UNCHANGED_IS.search(text) and not _OPERATION.search(text) and not _COND.search(text):
            return "unchanged (before/after)"
    if not _COND.search(text) and not _OPERATION.search(text):
        if _REGAINED.search(text) or _RESTORED_END.search(text) or _AGAIN.search(text):
            return "regained/restored/again"
    return None


def reason_for(name, slug=False):
    """Why `name` carries history, or None."""
    name = cs._PATH.sub("PATH", name)  # a cited repo path is a link, not a date
    text = name.lower()
    if slug:
        text = _SLUG.sub(" ", text).strip()
    for rx, why in NAME_MARKERS:
        if rx.search(text):
            return why
    why = contrast_reason(text, slug)
    if why:
        return why
    why = cs.history_reason(name if not slug else _SLUG.sub(" ", name))
    return why


# ----------------------------------------------------------------------------- extraction
_TITLE_CALL = re.compile(r"(?<![\w$.])(?:test|it|describe)(?:\.(?:describe|only|skip|fixme|fail|slow|serial|parallel|configure|step))*\s*\(\s*")
_STEP_CALL = re.compile(r"(?<![\w$.])test\.step\s*\(\s*")


def _literal(text, i):
    """The static text of the string literal starting at text[i] (a quote), or None. `${..}` becomes `X`."""
    q = text[i]
    if q not in "\"'`":
        return None, i
    out, j, n = [], i + 1, len(text)
    while j < n:
        c = text[j]
        if c == "\\" and j + 1 < n:
            out.append(text[j + 1])
            j += 2
            continue
        if q == "`" and c == "$" and text[j + 1:j + 2] == "{":
            depth, j = 1, j + 2
            while j < n and depth:
                depth += (text[j] == "{") - (text[j] == "}")
                j += 1
            out.append("X")
            continue
        if c == q:
            return "".join(out), j + 1
        if c == "\n" and q != "`":
            return None, j
        out.append(c)
        j += 1
    return None, n


def _in_comment(text, pos):
    ls = text.rfind("\n", 0, pos) + 1
    prefix = text[ls:pos]
    return prefix.lstrip().startswith(("//", "*", "/*")) or "//" in prefix.replace("://", "")


def ts_names(text):
    """[(line, name)] of every static test, describe and step title in a spec file."""
    found = []
    for m in list(_TITLE_CALL.finditer(text)):
        if _in_comment(text, m.start()):
            continue
        lit, _ = _literal(text, m.end()) if m.end() < len(text) else (None, 0)
        if lit:
            found.append((text.count("\n", 0, m.start()) + 1, lit))
    return found


_RS_TEST_ATTR = re.compile(r"#\[(?:[\w:]+::)?(?:test|wasm_bindgen_test|rstest)(?:\([^)]*\))?\]")
_RS_ATTR = re.compile(r"\s*(?:#\[[^\]]*\]|///[^\n]*|//[^\n]*)")
_RS_FN = re.compile(r"\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)")


def rs_names(text):
    """[(line, fn name)] of every `#[test]` function."""
    found = []
    for m in _RS_TEST_ATTR.finditer(text):
        if _in_comment(text, m.start()):
            continue
        pos = m.end()
        while True:
            a = _RS_ATTR.match(text, pos)
            if not a:
                break
            pos = a.end()
        f = _RS_FN.match(text, pos)
        if f:
            found.append((text.count("\n", 0, f.start(1)) + 1, f.group(1)))
    return found


# ----------------------------------------------------------------------------- the allowlist
# (path, a substring of the flagged name, reason). Stale entries (nothing flagged there any more) fail.
ALLOW = [
]


def tracked(root):
    out = subprocess.run(["git", "-C", root, "ls-files", "-z"], capture_output=True, check=True).stdout.decode()
    for p in sorted(x for x in out.split("\0") if x):
        if p.startswith(cs.SKIP_PREFIXES) or any(s in "/" + p for s in cs.SKIP_PARTS) or p in SKIP_FILES:
            continue
        if os.path.isfile(os.path.join(root, p)):
            yield p


def scan(root, allow=None):
    """([(path, line, kind, name, why)], [stale ALLOW entry text])."""
    allow = ALLOW if allow is None else allow
    used = [0] * len(allow)
    hits = []

    def consider(path, line, kind, name, slug):
        why = reason_for(name, slug)
        if not why:
            return
        for k, (p, sub, _reason) in enumerate(allow):
            if p == path and sub in name:
                used[k] += 1
                return
        hits.append((path, line, kind, name, why))

    for p in tracked(root):
        base = os.path.basename(p)
        is_spec = bool(SPEC_RE.search(base))
        if is_spec:
            consider(p, 0, "spec file", base[:base.index(".")], True)
        if is_spec or (p.startswith("playwright/") and p.endswith((".ts", ".mjs"))):
            with open(os.path.join(root, p), encoding="utf-8", errors="replace") as fh:
                for line, name in ts_names(fh.read()):
                    consider(p, line, "title", name, False)
        elif p.endswith(".rs"):
            with open(os.path.join(root, p), encoding="utf-8", errors="replace") as fh:
                text = fh.read()
            for line, name in rs_names(text):
                consider(p, line, "rust test", name, True)
    stale = []
    for k, (p, sub, reason) in enumerate(allow):
        if not reason.strip():
            stale.append(f"ALLOW entry ({p}, {sub!r}) has no reason")
        if not used[k]:
            stale.append(f"stale ALLOW entry ({p}, {sub!r}): no such flagged name any more -- remove it")
    return hits, stale


def cmd_check(root, inventory):
    hits, stale = scan(root)
    if inventory:
        for p, line, kind, name, why in hits:
            print(f"{p}:{line}\t{kind}\t{why}\t{name[:120]}")
        print(f"check-history-names: {len(hits)} names", file=sys.stderr)
        return 0
    rc = 0
    if hits:
        rc = 1
        print("check-history-names: a test name tells history instead of behaviour (a date, \"regression\", \"round 5\",\n"
              "\"batch 3\", \"user-reported\", \"owner's\", \"pre-fix\", \"before the fix\", \"the old ...\", \"no longer\"). A name says\n"
              "what the test asserts, in the present tense; the story is the backlog row's, linked from a comment.\n"
              "Rename it, or add a reasoned ALLOW entry in scripts/history-names.py (for a name that is data).",
              file=sys.stderr)
        for p, line, kind, name, why in hits:
            print(f"  {p}:{line} [{kind}, {why}] {name[:110]}", file=sys.stderr)
    for s in stale:
        rc = 1
        print(f"check-history-names: {s}", file=sys.stderr)
    if rc == 0:
        print("check-history-names: OK -- no spec title, Rust test name or spec file name carries history")
    return rc


# ----------------------------------------------------------------------------- self-test
MUST_FLAG = [
    ("title", "regression: the panel stays open", "regression"),
    ("title", "the panel stays open (regression guard)", "regression guard"),
    ("title", "round 5: tooltip delay", "round/batch n"),
    ("title", "round5 tooltip delay", "round/batch n"),
    ("title", "batch 13 demos open", "round/batch n"),
    ("title", "user-reported: the caret jumps", "reported by"),
    ("title", "the owner's report of a clipped label", "reported by"),
    ("title", "owner's lab session: slider", "reported by"),
    ("title", "pre-fix the thumb overflowed", "pre-fix"),
    ("title", "before the fix the menu closed", "pre-fix"),
    ("title", "the old behaviour keeps focus", "the old"),
    ("title", "the toast no longer steals focus", "no longer"),
    ("title", "a closed popup no longer matches :popover-open", "no longer"),
    ("title", "fixed 2026-10-19: the date is shown", "date"),
    ("title", "calendar since 2026-09-02", "date"),
    ("title", "closes on Escape (row 77 follow-up)", "follow-up"),
    ("title", "used to scroll on open", "used to"),
    ("title", "active tab keeps its color (calendar-state lane sweep)", "lane"),
    ("title", "previously the label wrapped", "previously"),
    ("slug", "demos-batch3", "round/batch n"),
    ("slug", "demos-parity3", "round/batch n"),
    ("slug", "demos-parts188", "round/batch n"),
    ("slug", "tooltip_follow_up_fix", "follow-up"),
    ("slug", "dialog_regression", "regression"),
    ("slug", "popover_2026_10_19", "date"),
    ("slug", "tooltip_no_longer_flickers", "no longer"),
    ("slug", "select_pre_fix_overflow", "pre-fix"),
    ("slug", "menu_before_the_fix", "pre-fix"),
    ("slug", "owners_report_label", "reported by"),
    # round 3 (row 384): before/after contrast
    ("title", "selecting a sub-item still closes the whole menu", "still (before/after)"),
    ("title", "hovering still scrolls", "still (before/after)"),
    ("title", "keyboard: ArrowDown still opens the menu and focuses its first item", "still (before/after)"),
    ("title", "the older ones still glide", "still (before/after)"),
    ("title", "Escape and Close still close it", "still (before/after)"),
    ("title", "a scroll still closes an open card", "still (before/after)"),
    ("title", "keyboard still opens and Escape closes", "still (before/after)"),
    ("title", "scroll-up release and jump-to-latest still work on a long transcript", "still (before/after)"),
    ("slug", "duplicate_protections_still_hold", "still (before/after)"),
    ("title", "`.dark` among other classes still counts, a look-alike class does not", "still (before/after)"),
    ("slug", "caller_style_still_overrides_the_gap_prop", "still (before/after)"),
    ("title", "keyboard is unchanged: ArrowRight moves to the next item", "unchanged (before/after)"),
    ("title", "wcag, dark: unchanged, a bare ratio is still held", "unchanged (before/after)"),
    ("title", "unchanged: the thumb keeps its size", "unchanged (before/after)"),
    ("title", "neither: prefers-color-scheme decides (unchanged)", "unchanged (before/after)"),
    ("title", "desktop is unchanged on the routes that had no phone entry", "unchanged (before/after)"),
    ("title", "the keyboard counterpart is unchanged", "unchanged (before/after)"),
    ("title", "the thumb regained its ring", "regained/restored/again"),
    ("title", "the divider is restored", "regained/restored/again"),
    ("title", "the divider works again", "regained/restored/again"),
    ("title", "the suite is green again", "regained/restored/again"),
]
MUST_NOT_FLAG = [
    ("title", "opens on click and closes on Escape"),
    ("title", "the selected day is highlighted"),
    ("title", "wraps a long label onto a second line"),
    ("title", "restores the old value on cancel"),
    ("title", "a dataset round-trips through the serializer"),
    ("title", "formats 2024-10-19 as 19 Oct 2024"),
    ("title", "the calendar shows October 2026"),
    ("title", "the owner element receives the click"),
    ("title", "masonry lanes keep equal gaps"),
    ("title", "no role=application (dev-docs/research/chart-2026-09-19.md section 2.1)"),
    ("title", "a programmatic open returns focus to the previously focused element"),
    ("slug", "dialog-prefix-match"),
    ("title", "initial sizes give the leftover to panels without a default"),
    ("title", "Tab moves to the next batch of rows"),
    ("title", "the thumb sits at 3 of 5 steps"),
    ("title", "linear scale maps 0..1 to 0..100%"),
    ("slug", "dialog-non-modal-aria-and-dismissal"),
    ("slug", "demos-menu-overlay-floating"),
    ("slug", "user-select"),
    ("slug", "carousel_wheel_replay"),
    ("slug", "popover_stays_anchored"),
    ("slug", "parses_iso_dates_2024_10_19"),
    # round 3: the same words as a guarantee under a condition, an observed state, or an action the test repeats
    ("title", "the scrim blocks pointer events while disabled"),
    ("title", "a selected day keeps its background while the pointer is still over it"),
    ("title", "clicking a tab keeps its active text color while still hovered"),
    ("title", "under RTL, loop_mode Rewind still wraps both directions"),
    ("title", "a click inside a slide still works when the pointer never crosses the drag threshold"),
    ("title", "after prepending history the fade still follows the scroll position"),
    ("title", "light/dark toggle still flips with a preset active"),
    ("title", "a mid-range pointer drag produces no transform and still pages correctly"),
    ("title", "first mouse-click open focuses the command input, and a second open still does"),
    ("title", "no chart-swatch element renders, but the row's name and value still do"),
    ("title", "expanded, close button on the newest: nothing below it exists, the older ones still glide"),
    ("title", "the order chosen through the menu survives a reload and the jump target still exists"),
    ("title", "content fades out (opacity -> 0, still popover-open) before unmounting"),
    ("title", "a press on the close button is still a click, not a swipe"),
    ("title", "the scrollbar is still opaque"),
    ("title", "reduced motion stops every loop and keeps a still 'loading' shape"),
    ("slug", "a_narrowing_search_keeps_the_still_matching_current_item"),
    ("slug", "a_close_voids_a_measurement_still_in_flight"),
    ("slug", "key_effects_stillness_is_measured"),
    ("title", "Escape cancels a lift and the order is unchanged"),
    ("title", "offset to trigger is unchanged after scrolling"),
    ("title", "picking a day stages it: the calendar shows it, the input is unchanged, the popover stays open"),
    ("title", "a non-digit keystroke is blocked and leaves the segment's value unchanged"),
    ("slug", "clamp_selected_within_bounds_is_unchanged"),
    ("slug", "resolve_horizontal_ltr_unchanged"),
    ("title", "closing it again restores hidden=until-found"),
    ("title", "pressing the active role again keeps it selected"),
    ("title", "a reveal after the user closed it opens it again"),
    ("title", "mobile: opens as a sheet and closes with Escape (focus restored)"),
    ("title", "scroll is locked while open and restored after Escape"),
    ("title", "Back restores the saved offset in one step"),
    ("title", "a revealed placeholder is hidden again, and the real row is marked as the match"),
    ("title", "the thumb regains focus when Escape closes the popup"),
]


def _fixture(files):
    d = tempfile.mkdtemp(prefix="history-names-")
    subprocess.run(["git", "init", "-q", d], check=True)
    for p, text in files.items():
        full = os.path.join(d, p)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as fh:
            fh.write(text)
    subprocess.run(["git", "-C", d, "add", "-A"], check=True)
    return d


def _run(files, extra=()):
    d = _fixture(files)
    r = subprocess.run([sys.executable, "-I", os.path.abspath(__file__), "--root", d, *extra], capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def self_test():
    fails = 0

    def report(ok, text):
        nonlocal fails
        fails += not ok
        print(f"  {'ok  ' if ok else 'FAIL'} {text}")

    for kind, name, want in MUST_FLAG:
        got = reason_for(name, kind == "slug")
        report(got is not None, f"flags {kind} {name!r} -> {got}")
        if got is not None and got != want:
            print(f"      (labelled {got!r}, expected {want!r}: any label fails the gate)")
    for kind, name in MUST_NOT_FLAG:
        got = reason_for(name, kind == "slug")
        report(got is None, f"leaves {kind} {name!r} alone" + (f" (got {got})" if got else ""))

    ts = ('import { test } from "./fixtures";\n'
          'test.describe(`menus, ${mode}`, () => {\n'
          '  test("opens and closes", async () => {});\n'
          "  test.skip('a skipped one', async () => {});\n"
          '  // test("a commented-out regression", () => {});\n'
          '  test(\n    "wraps onto the next line",\n    async () => {});\n'
          '});\n')
    names = [n for _, n in ts_names(ts)]
    report(names == ["menus, X", "opens and closes", "a skipped one", "wraps onto the next line"], f"ts extraction {names}")
    rs = ('#[cfg(test)]\nmod t {\n    #[test]\n    fn clean_name() {}\n    #[test]\n    #[should_panic]\n    /// doc\n'
          '    fn regression_bug() {}\n    fn helper_regression() {}\n    // #[test]\n    // fn commented() {}\n'
          '    #[wasm_bindgen_test]\n    async fn wasm_one() {}\n}\n')
    names = [n for _, n in rs_names(rs)]
    report(names == ["clean_name", "regression_bug", "wasm_one"], f"rust extraction {names}")

    # the gate on throwaway repos: each mutation must turn it red, each clean tree stays green
    cases = [
        ("clean tree passes", {"playwright/menu.spec.ts": 'test("opens", () => {});\n', "a/src/lib.rs": "#[test]\nfn opens() {}\n"}, 0),
        ("dated title fails", {"playwright/menu.spec.ts": 'test("opens, fixed 2026-10-19", () => {});\n'}, 1),
        ("describe with `regression` fails", {"playwright/menu.spec.ts": 'test.describe("menu regression", () => {});\n'}, 1),
        ("`(regression guard)` title fails", {"playwright/menu.spec.ts": 'test("opens (regression guard)", () => {});\n'}, 1),
        ("template title with `no longer` fails", {"playwright/menu.spec.ts": "test(`${x} no longer opens`, () => {});\n"}, 1),
        ("spec file named for a batch fails", {"playwright/demos-batch3.spec.ts": 'test("opens", () => {});\n'}, 1),
        ("spec file named for a date fails", {"playwright/menu-2026-10-19.spec.ts": 'test("opens", () => {});\n'}, 1),
        ("Rust test named for a regression fails", {"a/src/lib.rs": "#[test]\nfn regression_closes() {}\n"}, 1),
        ("a `still <verb>s` title fails", {"playwright/menu.spec.ts": 'test("selecting a sub-item still closes the whole menu", () => {});\n'}, 1),
        ("a template title with `is unchanged:` fails", {"playwright/menu.spec.ts": "test(`${x}: keyboard is unchanged: ArrowRight`, () => {});\n"}, 1),
        ("a `still` guarantee under a condition passes", {"playwright/menu.spec.ts": 'test("the scrim still blocks pointer events while disabled", () => {});\n'}, 0),
        ("Rust test named `..._still_works` fails", {"a/src/lib.rs": "#[test]\nfn unsorted_input_still_works() {}\n"}, 1),
        ("Rust identity test `..._is_unchanged` passes", {"a/src/lib.rs": "#[test]\nfn clamp_within_bounds_is_unchanged() {}\n"}, 0),
        ("Rust helper (no #[test]) is not a test name", {"a/src/lib.rs": "fn regression_closes() {}\n"}, 0),
        ("a commented-out history title is not a title", {"playwright/menu.spec.ts": '// test("round 5 menu", () => {});\n'}, 0),
        ("the vendored reference tree is skipped", {"playwright/oracle/reference/x/y.spec.ts": 'test("the old way", () => {});\n'}, 0),
    ]
    for name, files, want in cases:
        rc, out = _run(files)
        ok = rc == want
        report(ok, f"{name} (rc {rc}, want {want})")
        if not ok:
            print("    " + out.replace("\n", "\n    "))
    # a real spec of this tree, mutated: appending a history-titled test must turn the gate red
    real = os.path.join(ROOT, "playwright/slider.spec.ts")
    if os.path.exists(real):
        text = open(real, encoding="utf-8").read()
        rc0, _ = _run({"playwright/slider.spec.ts": text})
        rc1, _ = _run({"playwright/slider.spec.ts": text + '\ntest("the thumb (regression guard, round 5)", async () => {});\n'})
        report(rc0 == 0 and rc1 == 1, f"real-file mutation (slider.spec.ts + history title): clean rc {rc0}, mutated rc {rc1}")
    # stale ALLOW entries fail, a reasoned one exempts exactly its name
    hits, stale = scan(_fixture({"playwright/menu.spec.ts": 'test("opens", () => {});\n'}),
                       [("playwright/menu.spec.ts", "regression", "a statistics demo")])
    report(not hits and len(stale) == 1, "an ALLOW entry that matches nothing is stale")
    hits, stale = scan(_fixture({"playwright/menu.spec.ts": 'test("linear regression demo", () => {});\n'}),
                       [("playwright/menu.spec.ts", "linear regression", "a statistics demo")])
    report(not hits and not stale, "a reasoned ALLOW entry exempts its name")
    hits, stale = scan(_fixture({"playwright/menu.spec.ts": 'test("linear regression demo", () => {});\n'}),
                       [("playwright/menu.spec.ts", "linear regression", " ")])
    report(not hits and len(stale) == 1, "an ALLOW entry without a reason fails")
    print(f"check-history-names --self-test: {'OK' if not fails else str(fails) + ' FAILED'}")
    return 1 if fails else 0


def main(argv):
    root, inventory, selftest = ROOT, False, False
    it = iter(argv[1:])
    for a in it:
        if a == "--root":
            root = next(it)
        elif a == "--inventory":
            inventory = True
        elif a == "--self-test":
            selftest = True
        else:
            print("usage: history-names.py [--root DIR] [--inventory] [--self-test]", file=sys.stderr)
            return 2
    return self_test() if selftest else cmd_check(root, inventory)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
