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
      f"{n_gates} gate scripts, {n_importing} of them on the shared kit")
PYEOF
