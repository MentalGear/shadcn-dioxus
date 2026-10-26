#!/usr/bin/env bash
# check-gate-kit.sh   (backlog row 429)
#
# The gate kit keeps its promises: `scripts/lib/css.py`, the one reader every CSS-reading gate imports, understands
# escapes in quoted strings, nested at-rules, comments with newlines (line numbers survive), `url(data:...;...{)` and
# nested rules, and `scripts/lib/allow.py`, the one allowlist engine, demands a reason, reports a stale or duplicate
# entry and marks what a site used. Fixtures run against the real files, each fixture group is mutation-tested (a
# broken copy of the library must turn it red), and no `scripts/check-*.sh` may define its own comment stripper or
# selector splitter again.
#
# THE CLASS (row 429): eighteen gates each embedded their own CSS reader inside a heredoc, and three of them read the
# same input three ways (a backslash in a quoted string ended the quote in one, comments were stripped without their
# newlines in two so reported lines were off, a flat brace pattern could not see a rule beyond a `{` in a string).
# Two gates that read one stylesheet then disagreed by accident, and a fix had to be made in each copy.
#
# THE CONSTRUCTION: a gate imports `from css import rules, split_top, ...` and `from allow import Allowlist` (after
# `sys.path.insert(0, "scripts/lib")`), never writes its own scanner. This gate holds the library to its contract:
#   1. css.py fixtures: escapes, nested at-rules, comments, `url(...)`, nested rules, parentheses, unbalanced braces,
#      fragment mode (CSS inside Rust or HTML), `split_top`, `subject_compound`, declaration lines; and a smoke over
#      every real stylesheet (no unbalanced brace, rules found);
#   2. allow.py fixtures: a reason is mandatory, a duplicate and a stale entry are reported, `find`/`find_all` mark
#      used, a malformed entry is refused, `--list` text;
#   3. mutation self-test: each of ~30 one-line breaks of css.py and allow.py is applied to a copy in memory and the
#      fixtures must turn red for it (an anchor that no longer exists in the source fails the gate);
#   4. the grep rule: no `scripts/check-*.sh` defines `strip_comments`, `strip_css_comments`, `blank_comments`,
#      `split_top` or `split_top_level` (a reasoned exception goes on GREP_ALLOW below, stale entries fail), and a gate
#      that imports the library inserts `scripts/lib` on its path the one way;
#   5. the library imports only the standard library;
#   6. reuse stays honest: no cargo gate script (the `scripts/check-*.sh` that `scripts/cargo-inputs-hash.sh` hashes) may
#      import the kit while the hash leaves `scripts/lib` out, because a stored cargo PASS is only valid for the inputs
#      its gates can read. None does today, so a library edit does not invalidate a stored cargo result.
#   7. the pipefail rule (row 437): in a shell script that sets `pipefail` (every `*.sh` of the repo outside
#      node_modules), no early-exit reader may sit on the right of an unquoted pipe: `grep -q`/`-qE`/`-qF`/
#      `--quiet`/`-m N`/`-l`, `head`, `sed ... q`, `awk ... exit`. The reader quits at its first match, the left side
#      then dies of SIGPIPE (141) and pipefail turns the match into a failure (`cmd | grep -q x || fail`, which is
#      intermittent: a gate red on a correct tree) or, under `if ! cmd | grep -q x`, a failure into a pass.
#      Read a variable (`grep -q PAT <<<"$var"`), a file, or take a prefix with `sed -n '1,Np'` (reads to the end).
#      A `pipe-ok` marker comment with a reason, on the line or the one above, exempts a site whose left side is
#      known to be tiny (an orphan marker or one without a reason fails). The scanner is quote-aware, joins
#      continuation lines, skips comments and heredoc bodies, and is fixture- and mutation-tested like the library.
# Rust-source and shell-only gates keep their own scanning (they do not read CSS); `comment-scan.py` is the shared
# Rust comment reader.
#
# Usage: scripts/check-gate-kit.sh      (a few seconds, no network, no cargo)
# Exit status: 0 = every fixture green and every mutation red; 1 = a promise broke; 2 = bad usage.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

[ "$#" -eq 0 ] || { echo "check-gate-kit: takes no arguments" >&2; exit 2; }

python3 -I - <<'PYEOF'
import ast
import glob
import os
import re
import sys
import types

sys.path.insert(0, "scripts/lib")
from allow import Allowlist  # noqa: E402

CSS_PATH = "scripts/lib/css.py"
ALLOW_PATH = "scripts/lib/allow.py"

# (script, defined name, reason): a gate that may define its own scanner. None today: the library is the one reader.
GREP_ALLOW = Allowlist("check-gate-kit", [], fields=("script", "name"))
BANNED_DEFS = re.compile(
    r"^[ \t]*def[ \t]+(strip_comments|strip_css_comments|blank_comments|split_top|split_top_level)\b", re.M)
LIB_IMPORT = re.compile(r"^[ \t]*from[ \t]+(css|allow)[ \t]+import\b", re.M)
LIB_PATH_LINE = 'sys.path.insert(0, "scripts/lib")'


def load(path, text=None):
    """Execute a library file (optionally a mutated text) into a fresh module."""
    src = open(path, encoding="utf-8").read() if text is None else text
    mod = types.ModuleType("gatekit_" + os.path.basename(path)[:-3])
    exec(compile(src, path, "exec"), mod.__dict__)
    return mod


# ------------------------------------------------------------------------------------------------ css.py fixtures
def css_fixtures(c):
    """Failure messages (empty = green) of the css.py fixtures against module `c`."""
    fails = []

    def check(label, got, want):
        if got != want:
            fails.append(f"{label}: expected {want!r}, got {got!r}")

    def pre(rs):
        return [r.prelude for r in rs]

    def attempt(label, fn):
        try:
            fn()
        except Exception as e:  # a crash is a red fixture, never a green gate
            fails.append(f"{label}: raised {e!r}")

    # --- escapes inside a quoted string do not end the quote
    def escapes():
        rs = c.rules('.a { content: "\\""; color: red }\n.b { content: \'\\\'\'; }\n.c { color: blue }\n')
        check("escaped quotes: preludes", pre(rs), [".a", ".b", ".c"])
        check("escaped quotes: lines", [r.line for r in rs], [1, 2, 3])
        check("escaped quotes: decls", rs[0].decls, ['content: "\\""', "color: red"])
    attempt("escapes", escapes)

    # --- a backslash outside a string takes the next character literally
    def escape_anywhere():
        rs = c.rules(".a\\{b { c: d }\n.e { f: g }\n")
        check("escape in a selector", pre(rs), [".a\\{b", ".e"])
    attempt("escape_anywhere", escape_anywhere)

    # --- a stray apostrophe ends at its line, the rest of the file is still read
    def bad_string():
        sheet = c.parse(".a { x: it's }\n.b { c: d }\n.c { e: f }\n")
        check("an apostrophe does not swallow the file", ".c" in pre(sheet.rules()), True)
        check("the malformed block is reported, not hidden", [m for _, m in sheet.problems], ["the block `.a` is never closed"])
    attempt("bad_string", bad_string)

    # --- nested at-rules keep their chain, outermost first
    def nested_at():
        rs = c.rules("@media (a) {\n  @supports (b) {\n    .x { c: d }\n  }\n  .y { e: f }\n}\n.z { g: h }\n")
        check("nested at: preludes", pre(rs), [".x", ".y", ".z"])
        check("nested at: chains", [r.at for r in rs], [["@media (a)", "@supports (b)"], ["@media (a)"], []])
        check("nested at: lines", [r.line for r in rs], [3, 5, 7])
    attempt("nested_at", nested_at)

    # --- keyframe steps are children of their at-rule
    def keyframes():
        sheet = c.parse("@keyframes k { from { a: b } 50% { c: d } to { e: f } }")
        check("keyframes: one at-rule", [(n.kind, n.prelude) for n in sheet.nodes], [("at", "@keyframes k")])
        check("keyframes: steps", [(n.kind, n.prelude) for n in sheet.nodes[0].children],
              [("rule", "from"), ("rule", "50%"), ("rule", "to")])
        check("keyframes: step chain", [r.at for r in sheet.rules()], [["@keyframes k"]] * 3)
    attempt("keyframes", keyframes)

    # --- url(data:...) with `;`, `{`, `}` and quotes is one token
    def url_data():
        css = ".a { background: url(data:image/svg+xml;utf8,<svg xmlns='http://x'>{</svg>) no-repeat; color: red }\n.b { c: d }\n"
        rs = c.rules(css)
        check("url data: preludes", pre(rs), [".a", ".b"])
        check("url data: decl count", len(rs[0].decls), 2)
        check("url data: second line", rs[1].line, 2)
        rs = c.rules('.a { b: url("x;{y") ; c: d }\n.e { f: g }\n')
        check("quoted url: preludes", pre(rs), [".a", ".e"])
        check("quoted url: decls", len(rs[0].decls), 2)
    attempt("url_data", url_data)

    # --- comments with newlines keep every later line number
    def comments():
        text = "/* one\n   two\n   three */\n.a { c: d }\n.b /* inline\n multi */ { e: f }\n.c { g: h }\n"
        rs = c.rules(text)
        check("comments: lines", [(r.prelude, r.line) for r in rs], [(".a", 4), (".b", 5), (".c", 7)])
        check("comments: brace line", rs[1].brace_line, 6)
        check("strip_comments keeps the newline count", c.strip_comments(text).count("\n"), text.count("\n"))
        check("blank_comments keeps every offset", len(c.blank_comments(text)), len(text))
        check("a comment inside a string is text",
              c.strip_comments('a { content: "/* x */" } /* y */'), 'a { content: "/* x */" } ')
        check("a comment inside url() is text", c.strip_comments("a { b: url(x/*y*/z) }"), "a { b: url(x/*y*/z) }")
        check("strings=False strips everywhere", c.strip_comments('a "/* x */" b', strings=False), 'a "" b')
        check("an unclosed comment runs to the end", c.strip_comments("a /* b"), "a ")
    attempt("comments", comments)

    # --- a multi-line selector reports where it starts and where its brace is
    def lines():
        rs = c.rules(".a,\n.b,\n.c {\n  x: y }\n")
        check("selector start line / brace line", (rs[0].line, rs[0].brace_line), (1, 3))
        rs = c.rules(".a {\n  c: d;\n  e: f\n}\n")
        check("declaration lines", rs[0].decl_lines, [2, 3])
    attempt("lines", lines)

    # --- `;`, `{` and `}` inside parentheses are not structure
    def parens():
        rs = c.rules(".a { width: calc(1px + var(--x, {)); c: d }\n.b:is(.c, .d) { e: f }\n")
        check("parens: preludes", pre(rs), [".a", ".b:is(.c, .d)"])
        check("parens: decls", rs[0].decls, ["width: calc(1px + var(--x, {))", "c: d"])
    attempt("parens", parens)

    # --- an at-rule statement ends at its `;`, even with `;` and `{` in its string
    def statements():
        sheet = c.parse('@import "a;{b.css";\n.x { c: d }\n')
        check("statement: rules", [(r.prelude, r.line) for r in sheet.rules()], [(".x", 2)])
        check("statement: no problems", sheet.problems, [])
    attempt("statements", statements)

    # --- a rule nested in a rule is its own rule; the parent keeps its own declarations
    def nested_rule():
        rs = c.rules(".p { color: red; .q { c: d } e: f }")
        check("nested rule: preludes", pre(rs), [".p", ".q"])
        check("nested rule: flags", [r.nested for r in rs], [True, False])
        check("nested rule: own decls", rs[0].decls, ["color: red", "e: f"])
        check("nested rule: parent body holds the child", ".q { c: d }" in rs[0].body, True)
    attempt("nested_rule", nested_rule)

    # --- unbalanced braces are reported, never silently accepted
    def problems():
        check("unclosed block", [m for _, m in c.parse(".a { c: d").problems], ["the block `.a` is never closed"])
        check("stray close", [m for _, m in c.parse(".a { c: d } }").problems], ["a `}` with no open block"])
        check("balanced", c.parse(".a { c: d }").problems, [])
    attempt("problems", problems)

    # --- fragment mode reads the CSS inside a Rust raw string; the host's quotes are not CSS's
    def fragment():
        rs = c.rules('const X: &str = r#"\n.dx-a {{\n  width: 100%;\n}}\n"#;\nfn f() { let s = "it\'s"; }\n.b { c: d }\n',
                     fragment=True)
        outer = next(r for r in rs if r.prelude.endswith(".dx-a"))
        check("fragment: the outer rule nests the inner block", outer.nested, True)
        check("fragment: the inner declaration", [r.decls for r in rs if r.prelude == ""], [["width: 100%"]])
        check("fragment: a rule after Rust code with an apostrophe", [r.line for r in rs if r.prelude == ".b"], [7])
        # a lifetime apostrophe before a brace must not hide the brace the way a CSS string would
        sheet = c.parse("fn f<'a>() {\n x }\n.b { c: d }\n", fragment=True)
        check("fragment: host quotes are plain characters", (sheet.problems, pre(sheet.rules())),
              ([], ["fn f<'a>()", ".b"]))
    attempt("fragment", fragment)

    # --- split_top
    def splits():
        check("split_top: commas outside parens, brackets and quotes",
              c.split_top('a, b:not(.c, .d), [x="1,2"], e'), ["a", " b:not(.c, .d)", ' [x="1,2"]', " e"])
        check("split_top: an escaped comma", c.split_top(".a\\,b, .c"), [".a\\,b", " .c"])
        check("split_top: whitespace tokens", c.split_top('calc(1px + 2px) 3px "x y"', " \t"),
              ["calc(1px + 2px)", "3px", '"x y"'])
        check("split_top: quotes are text with strings=False", c.split_top('r#" body, x', strings=False),
              ['r#" body', " x"])
        check("split_top: an opened quote runs to the line end", c.split_top('r#" body, x'), ['r#" body, x'])
        check("split_top: empty pieces are kept", c.split_top("a,,b"), ["a", "", "b"])
    attempt("splits", splits)

    # --- subject_compound and split_decl
    def subjects():
        check("subject: child combinator", c.subject_compound(".a > .b:not(.c d)"), ".b:not(.c d)")
        check("subject: attribute with a space", c.subject_compound('.a   .b[x="p q"]'), '.b[x="p q"]')
        check("subject: no whitespace around the combinator", [c.subject_compound(x) for x in (".a>.b", ".a+.b", ".a~.b")],
              [".b", ".b", ".b"])
        check("subject: lone compound", c.subject_compound("html"), "html")
        check("subject: empty", c.subject_compound(""), "")
        check("subject: newline is a combinator", c.subject_compound("html\n  body"), "body")
        check("split_decl: lower-cased property", c.split_decl("Color : Red "), ("color", "Red"))
        check("split_decl: no colon", c.split_decl("nocolon"), ("", "nocolon"))
    attempt("subjects", subjects)

    return fails


def real_tree_smoke(c):
    """Every real stylesheet parses with balanced braces and yields rules; the corpus is not empty."""
    fails, total, sheets = [], 0, 0
    for root in ("preview/src", "preview/assets"):
        for path in sorted(glob.glob(root + "/**/*.css", recursive=True)):
            if "node_modules" in path:
                continue
            sheets += 1
            text = open(path, encoding="utf-8").read()
            sheet = c.parse(text)
            n = len(sheet.rules())
            total += n
            if sheet.problems:
                fails.append(f"{path}: {sheet.problems[0][1]} (line {sheet.problems[0][0]})")
            if "{" in c.strip_comments(text) and n == 0:
                fails.append(f"{path}: has braces but the reader found no rule")
    if sheets < 50 or total < 1000:
        fails.append(f"only {sheets} stylesheets / {total} rules found: the smoke scan is not reading the tree")
    return fails, sheets, total


# ---------------------------------------------------------------------------------------------- allow.py fixtures
def allow_fixtures(a):
    fails = []

    def check(label, got, want):
        if got != want:
            fails.append(f"{label}: expected {want!r}, got {got!r}")

    def attempt(label, fn):
        try:
            fn()
        except Exception as e:
            fails.append(f"{label}: raised {e!r}")

    def reasons():
        al = a.Allowlist("g", [("a.css", "x*", "because"), ("b.css", "y", "  ")], fields=("path", "pat"))
        probs = al.problems(stale=False)
        check("a blank reason is reported", [("b.css, y" in p and "no reason" in p) for p in probs], [True])
        check("a reason is not demanded twice", len(probs), 1)
    attempt("reasons", reasons)

    def stale_and_used():
        al = a.Allowlist("g", [("a.css", "x*", "r1"), ("b.css", "y", "r2")], fields=("path", "pat"))
        check("everything is stale before a site is seen", len(al.problems()), 2)
        hit = al.find(lambda e: e.path == "a.css")
        check("find returns the entry", hit and hit.pat, "x*")
        check("find marks it used", (al.used(hit), len(al.unused())), (True, 1))
        stale = al.problems()
        check("only the unused entry is stale", [("b.css, y" in p and "stale" in p) for p in stale], [True])
        check("stale=False skips the stale report", al.problems(stale=False), [])
        check("find misses", al.find(lambda e: e.path == "nope"), None)
    attempt("stale_and_used", stale_and_used)

    def find_all():
        al = a.Allowlist("g", [("a", "1", "r"), ("a", "2", "r"), ("b", "3", "r")], fields=("p", "q"))
        got = al.find_all(lambda e: e.p == "a")
        check("find_all returns every match", len(got), 2)
        check("find_all marks every match", [e.q for e in al.unused()], ["3"])
        al.mark(al.entries[2])
        check("mark", al.unused(), [])
    attempt("find_all", find_all)

    def duplicates():
        al = a.Allowlist("g", [("a", "1", "r"), ("a", "1", "other")], fields=("p", "q"))
        check("a duplicate is reported", [("duplicate" in p) for p in al.problems(stale=False)], [True])
    attempt("duplicates", duplicates)

    def arity():
        try:
            a.Allowlist("g", [("only-one",)], fields=("p", "q"))
        except ValueError:
            return
        fails.append("arity: an entry without its reason was accepted")
    attempt("arity", arity)

    def listing():
        text = a.Allowlist("g", [("a.css", "x", "the reason")], fields=("path", "pat")).listing()
        check("--list text", ("1 allowlist entry" in text and "a.css | x" in text and "the reason" in text), True)
        check("empty list", "0 allowlist entries" in a.Allowlist("g", [], fields=("p",)).listing(), True)
    attempt("listing", listing)

    return fails


# ------------------------------------------------------------------------------- pipefail rule (row 437)
# The scanner is kept as source text so the mutation run can break one line of it and require its fixtures to turn
# red, the way the library is held. Never write the three quote characters of a raw string inside it.
PIPEFAIL_SRC = r"""
import re

PIPEFAIL_SET = re.compile(r"^[ \t]*set[ \t]+[^#\n]*\bpipefail\b", re.M)
HEREDOC = re.compile(r"(?<!<)<<(-?)[ \t]*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
MARKER = re.compile(r"#[ \t]*pipe-ok:(.*)$")
GREP_NAMES = ("grep", "egrep", "fgrep")
GREP_LONG = ("--quiet", "--silent", "--max-count", "--files-with-matches")
SED_Q = re.compile(r"(?:^|[\s'\";{}/0-9$])q(?:[\s'\";}0-9]|$)")
AWK_EXIT = re.compile(r"\bexit\b")
QUOTED = re.compile(r"'[^']*'|\"(?:\\.|[^\"\\])*\"")


def strip_comment(line):
    return re.sub(r"(?:^|[ \t])#[^'\"]*$", "", line)


def logical_lines(text, body):
    # Yields (first line, last line, code). Comment lines and heredoc bodies are skipped (their 1-based line numbers
    # go into `body`), a line ending in a pipe or a backslash is joined with the next one.
    lines = text.split("\n")
    i, n = 0, len(lines)
    while i < n:
        raw = lines[i]
        if raw.lstrip().startswith("#"):
            i += 1
            continue
        start, code = i, strip_comment(raw).rstrip()
        while i + 1 < n and (re.search(r"(?<![|&>])\|&?$", code) or code.endswith("\\")):
            code = code[:-1].rstrip() if code.endswith("\\") else code
            i += 1
            code = code + " " + strip_comment(lines[i]).strip()
            code = code.rstrip()
        yield start + 1, i + 1, code
        m = HEREDOC.search(code)
        if m:
            delim, dash = m.group(3), m.group(1)
            i += 1
            while i < n and (lines[i].lstrip("\t") if dash else lines[i]).rstrip() != delim:
                body.add(i + 1)
                i += 1
        i += 1


def unquoted_pipe_segments(code):
    # The command text to the right of every unquoted single pipe (`||` and `>|` are not pipes), found through a
    # quote stack: a `$(...)` or `(...)` inside double quotes is code again, so `x="$(cmd | head -n 3)"` is seen.
    segs, open_segs = [], []
    stack = []  # "'" and '"' for quotes, "(" for a command substitution or subshell
    i, n = 0, len(code)

    def close(pred):
        for seg in [o for o in open_segs if pred(o[1])]:
            open_segs.remove(seg)
            segs.append(code[seg[0]:i].strip())

    while i < n:
        c, top = code[i], (stack[-1] if stack else "")
        if top == "'":
            if c == "'":
                stack.pop()
        elif c == "\\":
            i += 1
        elif top == '"' and c == '"':
            stack.pop()
        elif c == "$" and code[i + 1:i + 2] == "(":
            stack.append("(")
            i += 1
        elif top != '"' and c in "'\"(":
            stack.append(c)
        elif c == ")" and top == "(":
            stack.pop()
            close(lambda base: base > len(stack))
        elif top != '"' and c in "|;&":
            close(lambda base: base == len(stack))
            if c == "|" and code[i + 1:i + 2] == "|":
                i += 1
            elif c == "|" and code[i - 1:i] != ">":
                amp = 1 if code[i + 1:i + 2] == "&" else 0
                open_segs.append((i + 1 + amp, len(stack)))
                i += amp
        i += 1
    i = n
    close(lambda base: True)
    return segs


def early_exit_reader(seg):
    # The name of the early-exit reader this segment is, or None.
    skeleton = QUOTED.sub(lambda m: "Q" * len(m.group(0)), seg)
    tokens = skeleton.split()
    while tokens and tokens[0] in ("command", "builtin", "exec", "nice"):
        tokens = tokens[1:]
    if not tokens:
        return None
    cmd = tokens[0].rsplit("/", 1)[-1]
    if cmd == "head":
        return "head"
    if cmd in GREP_NAMES:
        for t in tokens[1:]:
            if t.startswith("--"):
                if t.startswith(GREP_LONG):
                    return "grep " + t.split("=")[0]
            elif t.startswith("-") and any(ch in "qlm" for ch in t[1:]):
                return "grep " + t
        return None
    if cmd == "sed" and SED_Q.search(seg):
        return "sed q"
    if cmd == "awk" and AWK_EXIT.search(seg):
        return "awk exit"
    return None


def scan(text):
    # (problems, markers_used_ok): problems are (line, message) for a pipefail script, [] for any other.
    if not PIPEFAIL_SET.search(text):
        return []
    lines = text.split("\n")
    body = set()
    logical = list(logical_lines(text, body))
    markers = {}
    for k, raw in enumerate(lines, 1):
        m = MARKER.search(raw)
        if m and k not in body:
            markers[k] = m.group(1).strip()
    used, problems = set(), []
    for first, last, code in logical:
        for seg in unquoted_pipe_segments(code):
            reader = early_exit_reader(seg)
            if not reader:
                continue
            covering = [k for k in markers if first - 1 <= k <= last]
            if covering:
                used.update(covering)
                if all(markers[k] for k in covering):
                    continue
                problems.append((covering[0], "a pipe-ok marker needs a reason"))
                continue
            problems.append((first, "`| %s` reads early on the right of a pipe in a pipefail script: %s" % (reader, seg[:70])))
    for k, reason in sorted(markers.items()):
        if k not in used:
            problems.append((k, "a pipe-ok marker with no early-exit reader on its line or the next (stale)"))
    return problems
"""

PIPEFAIL_HEAD = "#!/usr/bin/env bash\nset -euo pipefail\n"
PIPEFAIL_RED = [
    ("grep -q", "printf '%s\\n' \"$x\" | grep -q foo"),
    ("grep -qE", "printf '%s\\n' \"$x\" | grep -qE 'a|b'"),
    ("grep -qF", "cmd | grep -qF 'x'"),
    ("grep -Eq", "cmd | grep -Eq 'x'"),
    ("grep -qx in an if", "if ! cmd | grep -qx \"$f\"; then"),
    ("grep --quiet", "cmd | grep --quiet x"),
    ("grep --silent", "cmd | grep --silent x"),
    ("grep -m1", "cmd | grep -m1 x"),
    ("grep -m 1 after a flag", "cmd | grep -F -m 1 x"),
    ("grep --max-count", "cmd | grep --max-count=1 x"),
    ("egrep -q", "cmd | egrep -q x"),
    ("head -n", "cmd | head -n 3"),
    ("head -1 mid-pipe", "x=$(cmd | head -1 | cut -d: -f1)"),
    ("head glued to the pipe", "cmd |head -2"),
    ("sed q", "cmd | sed -n '1p;q'"),
    ("sed Nq", "cmd | sed 3q"),
    ("awk exit", "cmd | awk 'NR==1{print; exit}'"),
    ("inside $(...)", "x=$(cmd | grep -q y && echo 1)"),
    ("pipe-and-stderr", "cmd |& grep -q x"),
    ("pipe at the end of a line", "cmd |\n  grep -q x"),
    ("pipe after a backslash", "cmd \\\n  | grep -q x"),
    ("flags on a continuation line", "cmd | grep \\\n  -q x"),
    ("pipe after a quoted pipe", "cmd | sed 's/a|b/c/' | grep -q x"),
    ("inside a double-quoted substitution", "x=\"$(cmd | grep -q y)\""),
    ("head inside a quoted substitution with an or-list", "x=\"$(cmd | head -n 3 || true)\""),
    ("a nested substitution", "echo \"a $(cmd | sed 3q) b\""),
    ("a substitution after quoted text", "echo 'x' \"$(cmd | head -1)\" done"),
    ("a pipe after a closed substitution", "x=$(a) ; b | grep -q y"),
    ("after a heredoc opener", "cat <<EOF | grep -q x\nbody\nEOF"),
    ("command prefix", "cmd | command grep -q x"),
    ("a marker without a reason", "cmd | grep -q x  # pipe-ok:"),
    ("a stale marker", "# pipe-ok: nothing here\ncmd | grep x"),
]
PIPEFAIL_GREEN = [
    ("a here-string", "grep -q foo <<<\"$x\""),
    ("a file argument", "grep -q foo file | wc -l"),
    ("grep -c on the right", "cmd | grep -c foo"),
    ("grep -v on the right", "cmd | grep -v foo"),
    ("grep -E with q in the pattern", "cmd | grep -E 'quit|q'"),
    ("grep -n", "cmd | grep -n foo"),
    ("sed prefix", "cmd | sed -n '1,5p'"),
    ("sed substitute", "cmd | sed -E 's/x/y/'"),
    ("awk without exit", "cmd | awk -F'\\t' '$1 == \"C\" { print $2 }'"),
    ("an or-list", "cmd || grep -q x"),
    ("a clobber redirect", "cmd >| out"),
    ("a clobber redirect onto a reader's name", "cmd >| head -n 3"),
    ("a quoted pipe", "echo 'a | grep -q b'"),
    ("a double-quoted pipe", "echo \"use | head -n 3\""),
    ("a double-quoted pipe next to a substitution", "echo \"a | grep -q b $(cmd)\""),
    ("a flag after a substitution closes", "x=\"$(cmd | grep -E y)\" ; echo -q"),
    ("an escaped quote", "echo \"a \\\" | head\""),
    ("a comment line", "# cmd | grep -q x"),
    ("a comment line with a quote", "# cmd | grep -q x, don't do it"),
    ("a flag after a command separator", "cmd | grep -E foo && echo -q done"),
    ("a flag after a semicolon", "cmd | grep -E foo; grep -m1 x file"),
    ("a trailing comment", "cmd  # | grep -q x"),
    ("a heredoc body", "python3 - <<'EOF'\nx | grep -q y\nz | head\nEOF"),
    ("a dash heredoc body", "cat <<-EOF\n\tx | head\n\tEOF"),
    ("grep -q on the left of a pipe", "grep -q foo file && cmd | wc -l"),
    ("head on the left of a pipe", "head -n 3 file | sort"),
    ("a marker with a reason", "cmd | grep -q x  # pipe-ok: the left side is one short line"),
    ("a marker above with a reason", "# pipe-ok: the left side is one short line\ncmd | grep -q x"),
]


def pipefail_fixtures(m):
    # Failure messages (empty = green) of the pipefail scanner `m` (a module loaded from PIPEFAIL_SRC).
    fails = []
    for label, body in PIPEFAIL_RED:
        try:
            got = m.scan(PIPEFAIL_HEAD + body + "\n")
        except Exception as e:
            fails.append("%s: raised %r" % (label, e))
            continue
        if not got:
            fails.append("pipefail: `%s` was not reported" % label)
    for label, body in PIPEFAIL_GREEN:
        try:
            got = m.scan(PIPEFAIL_HEAD + body + "\n")
        except Exception as e:
            fails.append("%s: raised %r" % (label, e))
            continue
        if got:
            fails.append("pipefail: `%s` was reported: %s" % (label, got))
    # A script that does not set pipefail is out of scope (a lone `cmd | grep -q x` is fine there).
    for body in ("cmd | grep -q x", "cmd | head -n 1"):
        if m.scan("#!/usr/bin/env bash\nset -eu\n" + body + "\n"):
            fails.append("pipefail: a script without pipefail was reported")
        if m.scan("#!/usr/bin/env bash\n# set -o pipefail\n" + body + "\n"):
            fails.append("pipefail: a commented-out `set -o pipefail` counted")
    for spelling in ("set -o pipefail", "set -Eeuo pipefail", "set -euo pipefail", "set -uo pipefail", "  set -e -o pipefail"):
        if not m.scan("#!/usr/bin/env bash\n" + spelling + "\ncmd | grep -q x\n"):
            fails.append("pipefail: `%s` was not recognised" % spelling)
    # The line number is the logical line's first line.
    got = m.scan(PIPEFAIL_HEAD + "echo a\ncmd |\n  grep -q x\n")
    if [k for k, _ in got] != [4]:
        fails.append("pipefail: a joined line reports line %s, not its first line 4" % [k for k, _ in got])
    return fails


def pipefail_repo_scan(m):
    # Every shell script of the repository: (path, line, message) for each site, and the number of scripts that set pipefail.
    skip_dirs = {"node_modules", ".git", "target", "dist", "public", ".cargo"}
    found, n_scripts, n_pipefail = [], 0, 0
    for dirpath, dirnames, filenames in os.walk("."):
        dirnames[:] = sorted(d for d in dirnames if d not in skip_dirs)
        for name in sorted(filenames):
            if not name.endswith(".sh"):
                continue
            path = os.path.join(dirpath, name)[2:]
            text = open(path, encoding="utf-8").read()
            n_scripts += 1
            if m.PIPEFAIL_SET.search(text):
                n_pipefail += 1
            for line, msg in m.scan(text):
                found.append((path, line, msg))
    return found, n_scripts, n_pipefail


PIPEFAIL_MUTATIONS = [
    ("head is not an early reader", 'if cmd == "head":', 'if cmd == "headx":'),
    ("grep -q is not an early reader", 'any(ch in "qlm" for ch in t[1:])', 'any(ch in "lm" for ch in t[1:])'),
    ("grep -m is not an early reader", 'any(ch in "qlm" for ch in t[1:])', 'any(ch in "ql" for ch in t[1:])'),
    ("grep long options are ignored", 'if t.startswith(GREP_LONG):', 'if False:'),
    ("egrep is not a grep", 'GREP_NAMES = ("grep", "egrep", "fgrep")', 'GREP_NAMES = ("grep",)'),
    ("sed q is not an early reader", 'if cmd == "sed" and SED_Q.search(seg):', 'if False:'),
    ("awk exit is not an early reader", 'if cmd == "awk" and AWK_EXIT.search(seg):', 'if False:'),
    ("a script without pipefail is scanned", 'if not PIPEFAIL_SET.search(text):\n        return []', 'if False:\n        return []'),
    ("a commented-out pipefail counts", r'"^[ \t]*set[ \t]+[^#\n]*\bpipefail\b"', r'"set[ \t]+[^#\n]*\bpipefail\b"'),
    ("a marker never exempts", 'if covering:\n                used.update(covering)', 'if False:\n                used.update(covering)'),
    ("a marker needs no reason", 'if all(markers[k] for k in covering):', 'if True:'),
    ("a stale marker is not reported", 'if k not in used:', 'if False:'),
    ("a marker above is not seen", 'first - 1 <= k <= last', 'first <= k <= last'),
    ("continuation lines are not joined", 'while i + 1 < n and (re.search(r"(?<![|&>])\\|&?$", code) or code.endswith("\\\\")):', 'while False:'),
    ("a backslash continuation is not joined", 'or code.endswith("\\\\")):', '):'),
    ("comment lines are scanned", 'if raw.lstrip().startswith("#"):', 'if False:'),
    ("trailing comments are scanned", 'return re.sub(r"(?:^|[ \\t])#[^\'\\"]*$", "", line)', 'return line'),
    ("heredoc bodies are scanned", 'if m:\n            delim, dash', 'if False:\n            delim, dash'),
    ("single quotes are not tracked", 'if top == "\'":\n            if c == "\'":', 'if False:\n            if c == "\'":'),
    ("a substitution in double quotes is not code", 'elif c == "$" and code[i + 1:i + 2] == "(":', 'elif False:'),
    ("a double quote is not tracked", 'elif top == \'"\' and c == \'"\':', 'elif False:'),
    ("a pipe inside double quotes is a pipe", 'elif top != \'"\' and c in "|;&":', 'elif c in "|;&":'),
    ("|| is a pipe", 'if c == "|" and code[i + 1:i + 2] == "|":\n                i += 1', 'if c == "|" and code[i + 1:i + 2] == "|":\n                open_segs.append((i + 2, len(stack)))\n                i += 1'),
    ("a clobber redirect is a pipe", 'elif c == "|" and code[i - 1:i] != ">":', 'elif c == "|":'),
    ("the pipe-and-stderr form is not a pipe", 'amp = 1 if code[i + 1:i + 2] == "&" else 0', 'amp = 0'),
    ("a segment runs past a command separator", 'close(lambda base: base == len(stack))\n            if c', 'if c'),
    ("a segment runs past the end of its substitution", 'close(lambda base: base > len(stack))', 'pass'),
    ("a command prefix hides the reader", 'while tokens and tokens[0] in ("command", "builtin", "exec", "nice"):', 'while False:'),
    ("the reported line is the last line", 'problems.append((first, "`| %s`', 'problems.append((last, "`| %s`'),
]


# ------------------------------------------------------------------------------------------- mutation self-test
# (label, text in the library source, replacement). The fixtures must turn red for every one.
CSS_MUTATIONS = [
    ("a quoted string ignores backslash escapes",
     '            if d == "\\\\" and j + 1 < n:\n                j += 2\n                continue\n            if d == c:',
     '            if d == c:'),
    ("a string no longer ends at a newline", '            if d == "\\n":\n                return j\n', ''),
    ("an unquoted url(...) is not read whole", 'and _URL.match(s, i):', 'and False:'),
    ("comments lose their newlines", 'lambda comment: "\\n" * comment.count("\\n")', 'lambda comment: ""'),
    ("comment stripping ignores strings", '    if not strings:\n        return re.sub(', '    if True:\n        return re.sub('),
    ("the at-rule chain is dropped", 'at = tuple(s.prelude for s in stack if s.kind == "at")', 'at = ()'),
    ("parentheses are not structure-proof",
     '            if c == "(":\n                paren += 1', '            if c == "(":\n                paren += 0'),
    ("a rule's line is its brace line", 'line_of(text, buf_start + lead)', 'line_of(text, i)'),
    ("a rule never reports nesting", 'return bool(self.children)', 'return False'),
    ("a stray close brace is not reported",
     'sheet.problems.append((line_of(text, i), "a `}` with no open block"))', 'pass'),
    ("an unclosed block is not reported",
     'sheet.problems.append((node.brace_line, f"the block `{node.prelude[:60]}` is never closed"))', 'pass'),
    ("a declaration's line is the block end", 'line_of(text, start + len(raw) - len(raw.lstrip()))', 'line_of(text, end)'),
    ("fragment mode still reads strings", '        if not fragment:\n            if c == "\\\\"', '        if True:\n            if c == "\\\\"'),
    ("split_top ignores brackets and parentheses", '        if c in "([":\n            depth += 1', '        if c in "([":\n            depth += 0'),
    ("split_top ignores escapes", '        if strings:\n            if c == "\\\\" and i + 1 < n:', '        if strings:\n            if False:'),
    ("split_top reads quotes in host text", '''    out: List[str] = []
    depth, start, i, n = 0, 0, 0, len(s)
    while i < n:
        c = s[i]
        if strings:''', '''    out: List[str] = []
    depth, start, i, n = 0, 0, 0, len(s)
    while i < n:
        c = s[i]
        if True:'''),
    ("subject_compound ignores `>`", '" \\t\\n\\r\\f>+~"', '" \\t\\n\\r\\f"'),
    ("split_decl keeps the property case", 'return prop.strip().lower(), value.strip()', 'return prop.strip(), value.strip()'),
    ("a declaration after a nested block is lost", '                top = stack.pop()\n                _add_decl(top, text, buf_start, i)', '                top = stack.pop()'),
    ("a statement's `;` does not end it", '            if stack:\n                _add_decl(stack[-1], text, buf_start, i)\n            buf_start = i + 1', '            if stack:\n                _add_decl(stack[-1], text, buf_start, i)'),
]
ALLOW_MUTATIONS = [
    ("a blank reason is accepted", 'if not str(e.reason).strip():', 'if False:'),
    ("a duplicate entry is accepted", '            if key in seen:', '            if False:'),
    ("a stale entry is not reported", '        if stale:\n            for e in self.unused():', '        if False:\n            for e in self.unused():'),
    ("find does not mark used", '                self._used[k] = True\n                return e', '                return e'),
    ("find_all does not mark used", '                self._used[k] = True\n                out.append(e)', '                out.append(e)'),
    ("a malformed entry is accepted", 'if len(raw) != len(self.fields) + 1:', 'if False:'),
]


def mutation_run(path, mutations, fixtures_for, label):
    errs = []
    src = open(path, encoding="utf-8").read()
    for name, old, new in mutations:
        if src.count(old) != 1:
            errs.append(f"{label} mutation `{name}`: its anchor is not exactly once in {path} "
                        f"(found {src.count(old)}); update the mutation list with the source")
            continue
        try:
            mutant = load(path, src.replace(old, new))
        except Exception as e:
            continue  # a mutant that does not even load is red
        if not fixtures_for(mutant):
            errs.append(f"{label} mutation `{name}` stayed GREEN: no fixture notices it")
    return errs


# ------------------------------------------------------------------------------------------------------ run it
errors = []

css = load(CSS_PATH)
allow = load(ALLOW_PATH)

for f in css_fixtures(css):
    errors.append("css.py fixture: " + f)
smoke_fails, n_sheets, n_rules = real_tree_smoke(css)
for f in smoke_fails:
    errors.append("css.py smoke: " + f)
for f in allow_fixtures(allow):
    errors.append("allow.py fixture: " + f)

# The fixtures only mean something when green on the real file; then each mutant must turn them red.
if not errors:
    errors += mutation_run(CSS_PATH, CSS_MUTATIONS, css_fixtures, "css.py")
    errors += mutation_run(ALLOW_PATH, ALLOW_MUTATIONS, allow_fixtures, "allow.py")

# The pipefail scanner: fixtures on the real source, a break of each line must turn them red, then the repository.
pipefail = load("pipefail-scanner", PIPEFAIL_SRC)
fx = pipefail_fixtures(pipefail)
errors += ["pipefail fixture: " + f for f in fx]
if not fx:
    for name, old, new in PIPEFAIL_MUTATIONS:
        if PIPEFAIL_SRC.count(old) != 1:
            errors.append(f"pipefail mutation `{name}`: its anchor is not exactly once in PIPEFAIL_SRC "
                          f"(found {PIPEFAIL_SRC.count(old)}); update the mutation list with the source")
            continue
        try:
            mutant = load("pipefail-scanner", PIPEFAIL_SRC.replace(old, new))
            red = bool(pipefail_fixtures(mutant))
        except Exception:
            red = True  # a mutant that does not load or raises is red
        if not red:
            errors.append(f"pipefail mutation `{name}` stayed GREEN: no fixture notices it")
pf_sites, n_shell, n_pipefail = pipefail_repo_scan(pipefail)
for path, line, msg in pf_sites:
    errors.append(f"{path}:{line}: {msg}. THE CLASS (row 437): the reader exits on its first match, the left side dies "
                  f"of SIGPIPE and pipefail turns that into a failure; read a variable (`grep -q PAT <<<\"$var\"`), a "
                  f"file, or use `sed -n '1,Np'`; a short left side takes `# pipe-ok: <reason>`")

# The grep rule and the import rule over every gate script.
n_gates = n_importing = 0
for script in sorted(glob.glob("scripts/check-*.sh")):
    if script.endswith("check-gate-kit.sh"):
        continue
    n_gates += 1
    text = open(script, encoding="utf-8").read()
    for m in BANNED_DEFS.finditer(text):
        name = m.group(1)
        if GREP_ALLOW.find(lambda e: e.script == script and e.name == name):
            continue
        line = text.count("\n", 0, m.start()) + 1
        errors.append(f"{script}:{line}: defines `{name}`; read CSS with scripts/lib/css.py (`from css import "
                      f"strip_comments, blank_comments, split_top`), the one reader every gate shares (row 429)")
    if LIB_IMPORT.search(text):
        n_importing += 1
        if LIB_PATH_LINE not in text:
            errors.append(f"{script}: imports the gate kit but never runs `{LIB_PATH_LINE}` (python3 -I ignores "
                          f"PYTHONPATH; the heredoc runs from the repository root)")
errors += [f"GREP_ALLOW: {p}" for p in GREP_ALLOW.problems()]

# Reuse: a cargo gate that read the kit would make `scripts/lib` a cargo input.
hash_text = open("scripts/cargo-inputs-hash.sh", encoding="utf-8").read()
cargo_scripts = sorted(set(re.findall(r"scripts/check-[a-z-]+\.sh", hash_text)))
for script in cargo_scripts:
    if os.path.exists(script) and LIB_IMPORT.search(open(script, encoding="utf-8").read()) and "scripts/lib" not in hash_text:
        errors.append(f"{script} is hashed by scripts/cargo-inputs-hash.sh and imports the gate kit, but the hash does "
                      f"not include scripts/lib: add it to WORKSPACE_FILES (a stored cargo PASS would outlive a library edit)")
if not cargo_scripts:
    errors.append("scripts/cargo-inputs-hash.sh names no gate script: the reuse rule of this gate reads nothing")

# The library is standard library only: a gate must stay runnable on a bare container.
STDLIB = {"re", "typing", "collections", "types", "os", "sys"}
for path in (CSS_PATH, ALLOW_PATH):
    for node in ast.walk(ast.parse(open(path, encoding="utf-8").read())):
        names = []
        if isinstance(node, ast.Import):
            names = [a.name.split(".")[0] for a in node.names]
        elif isinstance(node, ast.ImportFrom) and node.module:
            names = [node.module.split(".")[0]]
        for n in names:
            if n not in STDLIB:
                errors.append(f"{path}: imports `{n}`; the gate kit is standard library only")

if errors:
    for e in errors:
        print(f"check-gate-kit: {e}", file=sys.stderr)
    print(f"check-gate-kit: FAILED -- {len(errors)} problem(s)", file=sys.stderr)
    sys.exit(1)
print(f"check-gate-kit: OK -- css.py and allow.py fixtures green on the real files ({n_rules} rules in {n_sheets} "
      f"stylesheets read cleanly), red on {len(CSS_MUTATIONS) + len(ALLOW_MUTATIONS)}/"
      f"{len(CSS_MUTATIONS) + len(ALLOW_MUTATIONS)} mutations; no definition of a comment stripper or splitter in "
      f"{n_gates} gate scripts, {n_importing} of them on the shared kit; no early-exit reader on the right of a pipe in "
      f"{n_pipefail} of {n_shell} shell scripts that set pipefail (red on {len(PIPEFAIL_MUTATIONS)}/{len(PIPEFAIL_MUTATIONS)} "
      f"scanner mutations)")
PYEOF
