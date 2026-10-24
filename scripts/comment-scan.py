#!/usr/bin/env python3
"""comment-scan.py -- the ONE comment extractor behind two gates (backlog row 266 (a)).

    scripts/check-history-comments.sh   per-file ratchet on comment lines that retell history
    scripts/check-comment-refs.sh       every `row N` and every repo path cited in a comment resolves

Both are thin bash wrappers over this file (`python3 -I scripts/comment-scan.py <history|refs> ...`), so what
counts as "a comment" is decided in exactly one place: Rust (`//`, `///`, `//!`, nested `/* */`), CSS (`/* */`),
TS/JS (`//`, `/* */`, regex literals skipped so `/[/*]/` opens nothing), Python (`#` tokens and docstrings, via
`tokenize`/`ast`; a triple-quoted string that is not a docstring is data) and shell (`#`, not in a string, not
`$#`/`${#x}`). A heredoc body is data (a generated file's text) unless the command line names an interpreter
(`python3 - <<'PY'`, `node <<`, `bash <<`), in which case it is read as that language. A comment marker inside a
string literal is not a comment, except in Rust: a string literal that reads as code (JS with a statement shape and a
JS word, CSS with a `{ prop: value }` rule and a `/* */`) is scanned for the `//` and `/* */` comments inside it
(`extract_rust`, `_embedded_comments`), reported at their Rust source line; the string text around them is data.

Scanned: `git ls-files` with the extensions above, minus SKIP_PREFIXES (dev-docs/archive/ is where history belongs, the
prose under dev-docs/ is not code so no extension picks it up, but its code files are scanned;
docs/ is the deployed site; the APG reference tree is vendored; generated primitives JS), the gates'
own sources (this file, the history and refs wrappers, history-names.py and its wrapper), lock files.

Exit status: 0 clean, 1 violation, 2 bad usage.
"""
import ast
import io
import os
import re
import subprocess
import sys
import tempfile
import tokenize

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

EXT_KIND = {
    ".rs": "rust",
    ".css": "css",
    ".ts": "js", ".tsx": "js", ".js": "js", ".mjs": "js", ".cjs": "js",
    ".sh": "shell",
    ".py": "python",
}
SKIP_PREFIXES = (
    "dev-docs/archive/",
    "docs/",
    "playwright/oracle/reference/",
    "playwright/fixtures/",
    "primitives/src/js/",
    "node_modules/",
    "target/",
    "scripts/comment-scan.py",
    "scripts/check-history-comments.sh",
    "scripts/check-comment-refs.sh",
    "scripts/history-names.py",
    "scripts/check-history-names.sh",
)
SKIP_PARTS = ("/node_modules/", "/vendor/", "/vendored/")
SKIP_SUFFIXES = (".min.js", ".lock")

HISTORY_BASELINE = "scripts/check-history-comments.baseline.tsv"
REFS_BASELINE = "scripts/check-comment-refs.baseline.tsv"


# ----------------------------------------------------------------------------- files
def tracked_files(root):
    out = subprocess.run(["git", "-C", root, "ls-files", "-z"], capture_output=True, check=True).stdout
    files = []
    for p in out.decode().split("\0"):
        if not p:
            continue
        if p.startswith(SKIP_PREFIXES) or any(s in "/" + p for s in SKIP_PARTS) or p.endswith(SKIP_SUFFIXES):
            continue
        if os.path.splitext(p)[1] in EXT_KIND and os.path.isfile(os.path.join(root, p)):
            files.append(p)
    return sorted(files)


# ----------------------------------------------------------------------------- comment extraction
# Every extractor yields (line_number, body). A block comment yields one entry per physical line.
def _block_lines(start_line, raw):
    """Split a block comment body into per-line entries, dropping a leading ` * `."""
    for k, ln in enumerate(raw.split("\n")):
        yield start_line + k, re.sub(r"^\s*\*+ ?", "", ln)


def _rust_unescape(raw, first_line):
    """A normal Rust string's value with the source line of every value character: `\\n`, `\\"` and the like become
    one character, a `\\` + newline continuation drops the newline and the next line's indent."""
    out, lines = [], []
    i, n, line = 0, len(raw), first_line
    simple = {"n": "\n", "r": "\r", "t": "\t", "0": "\0", "\\": "\\", '"': '"', "'": "'"}
    while i < n:
        c = raw[i]
        if c == "\\" and i + 1 < n:
            d = raw[i + 1]
            if d in simple:
                out.append(simple[d])
                lines.append(line)
                i += 2
            elif d == "\n":
                line += 1
                i += 2
                while i < n and raw[i] in " \t\r\n":
                    line += raw[i] == "\n"
                    i += 1
            else:  # \x41, \u{..}: one placeholder character
                m = re.match(r"x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}", raw[i + 1:i + 12])
                out.append("?")
                lines.append(line)
                i += 1 + (m.end() if m else 1)
        else:
            out.append(c)
            lines.append(line)
            if c == "\n":
                line += 1
            i += 1
    return "".join(out), lines


# A Rust string literal can hold the page's JS or CSS (eval scripts, `style { r#"..."# }`), and a comment in that code
# is a comment: it retells history like any other. Only a string that reads as code is scanned (a prose string with
# a `//` in it is data): JS needs a statement or block shape plus a JS word, CSS needs a `{ prop: value }` rule and a
# `/* */`. A CSS-only string yields only its block comments.
_EMBED_JS = re.compile(r"=>|\)\s*\{|\b(?:function|const|let|var|return|typeof|document|window|addEventListener|querySelector"
                       r"|getElementById|setTimeout|requestAnimationFrame|classList|dataset|new [A-Z]\w+)\b")
_EMBED_CSS = re.compile(r"\{[^{}]*:[^{}]*\}")


def _embedded_kind(content):
    if "/*" not in content and "//" not in content:
        return None
    if re.search(r"[;{}]", content) and _EMBED_JS.search(content):
        return "js"
    if "/*" in content and _EMBED_CSS.search(content):
        return "css"
    return None


def _embedded_comments(content, char_lines):
    kind = _embedded_kind(content)
    if not kind:
        return
    if kind == "js":
        for off, body in extract_js(content, True):
            yield char_lines[min(off, len(char_lines) - 1)], body
    else:  # CSS: line numbers relative to the string, mapped through the start of that content line
        starts = [0] + [k + 1 for k, ch in enumerate(content) if ch == "\n"]
        for rel, body in extract_css(content):
            yield char_lines[min(starts[min(rel, len(starts)) - 1], len(char_lines) - 1)], body


def extract_rust(text):
    i, n, line = 0, len(text), 1
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            yield line, re.sub(r"^//[/!]?", "", text[i:j])
            i = j
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            body = text[i + 2:max(i + 2, j - 2)]
            yield from _block_lines(line, body)
            line += text.count("\n", i, j)
            i = j
        elif c == '"' or (c in "rb" and re.match(r'b?r#*"', text[i:i + 12]) and not (i and (text[i - 1].isalnum() or text[i - 1] == "_"))):
            m = re.match(r'b?r(#*)"', text[i:i + 12]) if c in "rb" else None
            if m:  # raw string
                end = '"' + m.group(1)
                j = text.find(end, i + m.end())
                j = n if j < 0 else j + len(end)
                body_start, body_end = i + m.end(), max(i + m.end(), j - len(end))
                content = text[body_start:body_end]
                char_lines, ln = [], line
                for ch in content:
                    char_lines.append(ln)
                    ln += ch == "\n"
            else:
                j = i + 1
                while j < n and text[j] != '"':
                    j += 2 if text[j] == "\\" else 1
                j += 1
                content, char_lines = _rust_unescape(text[i + 1:max(i + 1, j - 1)], line)
            if len(content) > 8:
                yield from _embedded_comments(content, char_lines)
            line += text.count("\n", i, j)
            i = j
        elif c == "'":
            if text.startswith("\\", i + 1):
                j = text.find("'", i + 3)
                j = i + 1 if j < 0 else j + 1
            elif i + 2 < n and text[i + 2] == "'":
                j = i + 3
            else:
                j = i + 1  # a lifetime
            i = j
        else:
            i += 1


def extract_css(text):
    i, n, line = 0, len(text), 1
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j
            yield from _block_lines(line, text[i + 2:j])
            line += text.count("\n", i, j + 2)
            i = j + 2
        elif c in "\"'":
            j = i + 1
            while j < n and text[j] != c and text[j] != "\n":
                j += 2 if text[j] == "\\" else 1
            i = j + 1
        else:
            i += 1


_JS_REGEX_AFTER_KEYWORD = re.compile(r"(?:^|[^\w$.])(?:return|typeof|case|in|of|else|do|void|delete|throw|new|yield|await)$")


def _js_regex_end(text, i, n):
    """End index (past the flags) of a regex literal opening at text[i] == '/', or -1 when it is a division."""
    k = i - 1
    while k >= 0 and text[k] in " \t\r\n":
        k -= 1
    if k >= 0 and (text[k].isalnum() or text[k] in "_$)]"):
        if not _JS_REGEX_AFTER_KEYWORD.search(text[max(0, k - 11):k + 1]):
            return -1  # an operand precedes: this slash divides
    j, in_class = i + 1, False
    while j < n and text[j] != "\n":
        c = text[j]
        if c == "\\":
            j += 2
            continue
        if c == "[":
            in_class = True
        elif c == "]":
            in_class = False
        elif c == "/" and not in_class:
            j += 1
            while j < n and (text[j].isalpha()):
                j += 1
            return j
        j += 1
    return -1


def extract_js(text, embedded=False):
    """`embedded`: the text is JS held in a Rust string, so a `//` only opens a comment at a line start or after a
    code character (`https://x` and `a//b` are data); it then yields each comment line's character OFFSET in `text`
    instead of a line number, so the caller can map it back to the Rust source line."""
    i, n, line = 0, len(text), 1
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif embedded and text.startswith("//", i) and i and text[i - 1] not in " \t\n;{}),":
            i += 2
        elif text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            yield (i if embedded else line), text[i + 2:j]
            i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j
            if embedded:
                off = i + 2
                for k, b in _block_lines(0, text[i + 2:j]):
                    yield off, b
                    off += len(text[i + 2:j].split("\n")[k]) + 1
            else:
                yield from _block_lines(line, text[i + 2:j])
            line += text.count("\n", i, j + 2)
            i = j + 2
        elif c == "/":
            j = _js_regex_end(text, i, n)
            i = i + 1 if j < 0 else j
        elif c in "\"'`":
            j = i + 1
            while j < n and text[j] != c and (c == "`" or text[j] != "\n"):
                j += 2 if text[j] == "\\" else 1
            line += text.count("\n", i, j)
            i = j + 1
        elif c == "\\":
            i += 2
        else:
            i += 1


def extract_python(text):
    """`#` comments from the token stream (a `#` in a string is not one) plus docstrings; any other string is data."""
    try:
        for tok in tokenize.generate_tokens(io.StringIO(text).readline):
            if tok.type == tokenize.COMMENT:
                yield tok.start[0], tok.string[1:]
        tree = ast.parse(text)
    except (tokenize.TokenError, SyntaxError, ValueError):
        for lineno, ln in enumerate(text.split("\n"), 1):
            body, _ = _shell_line_comment(ln, None)
            if body is not None:
                yield lineno, body
        return
    for node in ast.walk(tree):
        if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) and node.body:
            first = node.body[0]
            if isinstance(first, ast.Expr) and isinstance(first.value, ast.Constant) and isinstance(first.value.value, str):
                for k, ln in enumerate(first.value.value.split("\n")):
                    yield first.lineno + k, ln


_HEREDOC = re.compile(r"(?<!<)<<-?\s*(['\"]?)([A-Za-z_][A-Za-z_0-9]*)\1")


def _shell_line_comment(ln, in_quote):
    """Return (comment_body | None, in_quote) for one line; the quote state carries over."""
    i, n = 0, len(ln)
    while i < n:
        c = ln[i]
        if in_quote:
            if in_quote == '"' and c == "\\":
                i += 2
                continue
            if c == in_quote:
                in_quote = None
        elif c == "\\":
            i += 2
            continue
        elif c in "\"'":
            in_quote = c
        elif c == "#" and (i == 0 or ln[i - 1] in " \t;|&("):
            return ln[i + 1:], in_quote
        i += 1
    return None, in_quote


_INTERPRETER = re.compile(r"(?:^|[\s|;(&`])(python3?|node|bash|sh|zsh)(?=[\s'\"|;)&`]|$)")


_HEREDOC_TARGET = re.compile(r">\s*['\"]?[^\s'\"]*\.(py|m?js|cjs|ts|sh)['\"]?(?:\s|$)")
_HEREDOC_TAG = re.compile(r"^(py(?:thon)?|js|node|javascript|sh|bash)(?:_?eof|_?end)?$", re.I)


def _heredoc_language(ln, m):
    """The language of a heredoc body when it is code, else None (data: a generated file's text is not a comment).
    Code is named by the interpreter on the command line, by a `PY`/`JS`-style delimiter, or by a `> file.py` target."""
    im = _INTERPRETER.search(ln[:m.start()]) or _INTERPRETER.search(ln[m.end():])
    name = im.group(1) if im else None
    if not name:
        tag = _HEREDOC_TAG.match(m.group(2))
        name = tag.group(1).lower() if tag else None
    if not name:
        tm = _HEREDOC_TARGET.search(ln)
        name = {"py": "python", "sh": "bash"}.get(tm.group(1), "node") if tm else None
    if not name:
        return None
    return {"python": "python", "python3": "python", "py": "python", "node": "js", "js": "js", "javascript": "js"}.get(name, "shell")


def extract_shell(text):
    in_quote, heredoc = None, None
    lines = text.split("\n")
    body_start, body_lang, body_lines = 0, None, []
    for lineno, ln in enumerate(lines, 1):
        if heredoc:
            if ln.strip() == heredoc:
                heredoc = None
                if body_lang:  # an interpreter's heredoc is code: read it as that language, lines offset by its start
                    sub = "\n".join(body_lines)
                    ex = {"python": extract_python, "js": extract_js, "shell": extract_shell}[body_lang]
                    for k, b in ex(sub):
                        yield body_start + k - 1, b
                body_lang, body_lines = None, []
            else:
                body_lines.append(ln)
            continue
        if lineno == 1 and ln.startswith("#!"):
            continue
        body, in_quote = _shell_line_comment(ln, in_quote)
        m = _HEREDOC.search(ln if body is None else ln[:ln.rfind("#" + body)])
        if m and not in_quote:
            heredoc = m.group(2)
            body_start, body_lines = lineno + 1, []
            body_lang = _heredoc_language(ln, m)
        if body is not None:
            yield lineno, body


EXTRACTORS = {"rust": extract_rust, "css": extract_css, "js": extract_js, "shell": extract_shell, "python": extract_python}


def comments_of(root, path):
    with open(os.path.join(root, path), encoding="utf-8", errors="replace") as fh:
        text = fh.read()
    return list(EXTRACTORS[EXT_KIND[os.path.splitext(path)[1]]](text))


# ----------------------------------------------------------------------------- history detection
# The repo's first commit is 2026-08-29, so only a date from 2026-08 on is a retelling; a 2024 or 2026-05 date in a
# comment is test data (a calendar fixture), not history. The same floor applies to numeric and month-name dates.
DATE = re.compile(r"(?<![\w.-])(?:2026-(?:0[89]|1[0-2])|20(?:2[7-9]|[3-9]\d)-(?:0[1-9]|1[0-2]))-(?:0[1-9]|[12]\d|3[01])(?![\w-])")
# "(2026-08)": a month alone dates a sentence as well as a day does.
MONTH_ISO = re.compile(r"(?<![\w.-])(?:2026-(?:0[89]|1[0-2])|20(?:2[7-9]|[3-9]\d)-(?:0[1-9]|1[0-2]))(?![\w-]|\.\d)")
# 02/03/2026, 2.3.2027 (day/month order is unknowable, so any month): the year alone decides.
NUMERIC_DATE = re.compile(r"(?<![\w./-])\d{1,2}([/.])\d{1,2}\1(?:2026|20(?:2[7-9]|[3-9]\d))(?![\w/-])")
HEX = re.compile(r"(?<![\w#.\-/$@])(?:[0-9a-f]{7,40})(?![\w\-/])")
MONTHS = ("January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
          "December")
_MONTH = "(?:%s)" % "|".join(MONTHS)
# This is a calendar library: a month name is data ("30 days in November", "May 2026" in a fixture, a chart's January
# bar), so a month name is history only in a dating frame. "since March", "as of October", "last January", "early
# March" say when something changed; "in/on/during/from/by <Month> <year>" does too once the year is inside the
# repo's lifetime (2026-08 or later, the DATE floor). A bare "March 2025" or "in October" is content, not flagged.
MONTH_FRAME = re.compile(r"\b(?:since|as of|last|early|late|mid-?)\s+(?:in\s+)?%s\b" % _MONTH, re.I)
MONTH_YEAR_FRAME = re.compile(r"\b(?:in|on|during|from|by|until|before|after)\s+(?:(?:early|late|mid-?)\s+)?(%s)\s+(?:\d{1,2}(?:st|nd|rd|th)?,?\s+)?(20\d\d)\b" % _MONTH)
_MONTH_FRAME_CASE = re.compile(r"\b(?:since|as of|last|early|late|mid-?)\s+(?:in\s+)?(%s)\b" % _MONTH, re.I)


def _month_dated(body):
    for m in _MONTH_FRAME_CASE.finditer(body):
        if m.group(1)[0].isupper():  # "since march" lower case is not a month name worth trusting
            return True
    for m in MONTH_YEAR_FRAME.finditer(body):
        year, month = int(m.group(2)), MONTHS.index(m.group(1)) + 1
        if year > 2026 or (year == 2026 and month >= 8):
            return True
    return False


_AUX = r"(?:the|a|an|its|their|any|whichever|whatever|other|of|to|from|with|by|for|on|in|at|as|is|are|be|been|being|and|or|not|only|also|already)"
PHRASES = [
    (re.compile(r"\b(?:(?:before|after) (?:the|this|that|a) (?:fix|change|refactor|migration|rename|revert|patch|bug ?fix|port)|before (?:the|this|that|a) rewrite)\b", re.I), "before the fix"),
    (re.compile(r"\bformerly\b", re.I), "formerly"),
    (re.compile(r"\b(?:the|an?|its|their|this|that) old(?:-style)? (?!(?:value|values|state|year|height|width|size|index|key|keys|one|ones|element|elements|"
                r"listeners?|entry|entries|item|items|selection|text|content|position|offset|rect|props|data|slot|task|body|scope|"
                r"and|or|page|focus|id|ids|node|nodes|parent|child|children|row|rows|column|cells?|tab|panel|handle|time|range|date|"
                r"anchor|target|origin|count|length|index|list|order|set|map|string|name)\b)\w", re.I), "old behaviour"),
    (re.compile(r"\bold(?:-style)? (?:behaviou?r|implementation|approach|name|way|test|tests|engine|primitive|design|sweep|layout|pattern|"
                r"rule|variant|shape|style|api|version|default|defaults|ramp|prefix|method|logic|model|scheme|build|"
                r"selector|stylesheet|setup|hook|component|helper|function|bug|fix|workaround|algorithm|renderer|machinery|"
                r"mechanism)\b", re.I), "old behaviour"),
    (re.compile(r"\b(?:(?:was|were|got) (?:removed|deleted|added|introduced|merged|extracted|reverted|retired|deprecated)|(?:was|were|got|has been|have been) (?:changed|renamed|rewritten))\b", re.I), "was changed"),
    (re.compile(r"\bwe (?:now|used to|no longer|previously)\b", re.I), "we now"),
    (re.compile(r"\bnow (?:uses|reads|writes|goes|lives|comes|calls|returns|lets|owns|runs|derives|has|takes|gets|keeps|"
                r"carries|wraps|handles|follows|emits|renders|needs|does|sets|ships)\b", re.I), "now uses"),
    (re.compile(r"\bno longer (?:uses|reads|writes|carries|ships|sets|renders|needs|does|do|owns|"
                r"holds|lives|calls|emits|paints|draws|returns|takes)", re.I), "no longer"),
    (re.compile(r"\bsince (?:been )?(?:removed|replaced|renamed|dropped|fixed)\b|\bhas since (?:been )?(?:removed|replaced|renamed|dropped|fixed|wired|landed|changed|added|moved|become|gained|lost)\b|\buntil recently\b|\bback when\b", re.I), "since removed"),
    # a backlog row as a point in time: "before row 169 flipped", "since row 303", "row 8 landed"
    (re.compile(r"\b(?:before|after|until|since|once) (?:backlog )?rows? \d+|\brows? \d+ (?:\w+ )?(?:landed|lands|merged|shipped)\b", re.I), "before row N"),
    # a lane, batch or round told as history: "batch 13", "batch-2 integration", "round 1's migration", "lane/p5tabs",
    # "the STYLE-DELIVERY lane found"; a bare lower-case "lane" is a layout term (masonry lanes), so it needs a hyphenated ALL-CAPS name (`SSG lane` is a test target)
    (re.compile(r"\b(?:batch|round|wave)[- ]\d+\b|\blane/(?!spec\b)[\w-]+|\b[A-Z][A-Z0-9]+(?:-[A-Z0-9]+)+ lane\b"), "lane/batch"),
    # how a finding was made: "found twice", "seen twice", "caught by the foo spec", "found by check-bar"
    (re.compile(r"\b(?:found|seen|hit|reproduced|caught|flagged) (?:twice|three times)\b|"
                r"\b(?:caught|found|flagged|surfaced|revealed) by (?:the )?`?[\w./-]*(?:spec|gate|test|oracle|check-)", re.I), "found by"),
    # earlier drafts of the code or of this very comment
    (re.compile(r"\bthe first (?:version|draft)\b|"
                r"\bearlier (?:note|version|comment|draft|text|revision|fix|report|claim)s?\b", re.I), "first version"),
    # a work round as a point in time: "this round", "the previous round's", "(round5)"; not "round trip"/"round robin"
    (re.compile(r"\b(?:this|last|next|previous|earlier|prior) round(?![-\w]|\s+(?:trip|robin|up|down|off))|\bround\d+\b", re.I), "this round"),
    # the work stream that wrote the code, told in the code: "this lane" (not "against this lane": the SSG lane is a test
    # target), "another lane owns it", "the lane's own report", "lane T", "s2-polar", "Migration A slice 3/3", "this
    # session", "this fix", "this port", "the task brief"; a bare "slice", "lane" or "fix" is a data or infra word
    (re.compile(r"(?<!against )(?<!on )\bthis lane\b|\b(?:another|other) lanes? (?:owns?|edits?|is editing|touch(?:es)?|may)\b|"
                r"\b(?:owned|edited|touched) by (?:another|other) lanes?\b|\blane(?:'s)?(?: own)? (?:brief|report|agent|prompt)\b|"
                r"(?<![\w$-])s\d-[a-z]+\b|\bslice \d+/\d+|\bthis session\b|\bsession's (?:report|brief|diagnosis)\b|"
                r"\bthis fix\b|\bthis port\b(?! (?:is|number|serves|listens|binds|to|from|for|on|in|must|can|may|and|or|will|so|before|until|after|if)\b)(?![,.;)])|\b(?:task|per[- ]file|agent) brief\b|\bfetched fresh\b|(?:before|after|until|since) this (?:slice|lane|session|task|PR|commit|batch)\b", re.I), "lane/session"),
    # how a finding was made, told as a story: "confirmed live", "found by execution", "verified at write time",
    # "red-then-green", "RED on the unmodified tree", "pre-fix", "STATUS AT WRITE TIME", "dated addendum"
    (re.compile(r"\b(?:confirmed|verified|found|reproduced|observed|proven|seen) (?:live|empirically|by execution(?!')|in this session)\b|"
                r"\bverified at write time\b|\bat write time\b|\bdated addendum\b|\bred[- ](?:then|before)[- ]green\b|\bgreen-after\b|"
                r"\b(?:pre|post)-(?:fix|migration|rewrite|port|refactor|stage-?\d\w*|phase-?[\d.]*|axis-fix)\b|"
                r"\bconfirmed (?:\w+ )?(?:there )?by execution\b(?!')|\bunmodified tree\b|\bhas been observed to\b|\bhave been observed to\b|"
                r"\bwithout the fix\b|\bintermittently red\b|\bwent red\b|\bturned red\b", re.I), "evidence"),
    # a status header, an upper-case marker (case matters: `RED` and `UPDATE` are shouted when they narrate)
    (re.compile(r"\bSTATUS AT\b|\blane [TFS]\b|\bMigration [A-Z] slice\b|\bHISTORY[,:]|\bUPDATE\s*[(:]|\bRED[- ](?:FIRST|BEFORE|ON|AGAINST)\b|\bWritten RED\b|\bAt HEAD\b|\bhad NO\b|\bCorrection, filed\b"), "status header"),
    # past framing: "tried first", "the hard way", "renamed from X", "not the previous 2.5", "every generation above"
    (re.compile(r"\btried first\b|\bthe hard way\b|\bthe original v\d|\bevery generation\b|\brenamed (?:from|to)\b|"
                r"\bnot the previous \d|\bthe previous \d[\d.]*(?:px|ms|rem|em|%)|\b(?:exactly|just|same|unchanged|works?|closes|behaves?|stays?) as before\b|"
                r"\bunchanged from before\b|"
                # the code's own earlier shape: "the original asserted N", "the previous engine kept", "pre-shadcn-parity", "historical
                # defect", "Phase 4.4 solved", "before Phase 1.1", "as it did before", "before this prop existed", "not clamped any more"
                r"\bthe original (?:asserted|asserts|assertion|defect|report|draft|implementation|engine|walk|construction|hazard|bug|v\d)\b|"
                r"\bthe original \w+ (?:walk|engine|loop|pass)\b|\bprevious (?:engine|implementation|design|approach|algorithm|construction|walk)\b|"
                r"\b(?:pre|post)-(?:this-[\w-]+|shadcn[\w-]*)|\bhistorical (?:[\w-]+ )?(?:defect|hazard|bug|incident|regression|shape|cascade|behaviou?r|context|failure|class)\b|"
                r"\b(?:Phase|Stage) [\d.]+[a-z]? (?:removed|dropped|replaced|landed|migrated|introduced|added|flipped|solved|hit|adds)\b|"
                r"\b(?:before|after|until|since|once|when) (?:PATH )?(?:Phase|Stage) [\d.]+|\b(?:as|like|way) (?:it|they) did before\b|"
                r"\bbefore (?:this|that|the) (?:prop|feature|option|attribute|variant|rule|construction|wait|call|type|page|component|check|gate) existed\b|"
                r"\bnot (?:\w+ )?\w+ed any ?more\b", re.I), "past framing"),
    # who asked: "owner report", "the user report", "live-site report", "bug report"
    (re.compile(r"\b(?:owner|user|bug|live-site|incident)[- ](?:user )?reports?\b|\blive-site\b|\bowner-report\b|"
                r"\bowner(?:'s)?(?: real)?[- ](?:[\w-]+ )?(?:report|session)\b|\bowner's (?:bench|lab)\b|\blab session\b|"
                r"\bincident behind\b|\bthe incident (?:itself|that|which)\b|\bproduction incident\b|"
                r"(?<!upstream )(?<!upstream Dioxus )\bPR #\d+|\bmotivated PR #\d+", re.I), "report"),
    # a pointer into a dead session's scratchpad
    (re.compile(r"\$S/"), "scratchpad path"),
]
USED_TO = re.compile(r"(?:\b([\w']+)\s+)?\bused to\b", re.I)
# "is used to X", "can be used to X", "used to" after these is the passive/instrument sense, not history. "that" and
# "which" are NOT here: "the block that used to live here" is the habitual-past sense (a `that's`/`it's used to` is).
PASSIVE_PREV = {"is", "are", "be", "been", "being", "was", "were", "get", "gets", "got", "it's", "that's", "also", "only",
                "then", "often", "just", "can", "could", "may", "might", "must", "should", "will", "would",
                "not", "if", "when", "and", "or", "both", "either", "so", "thus", "than"}
INSTRUMENT_VERBS = {"prove", "compute", "size", "measure", "check", "verify", "test", "detect", "decide", "determine", "derive",
                    "compare", "identify", "distinguish", "tell", "mark", "signal", "pin", "anchor", "position", "place", "locate",
                    "find", "pick", "select", "build", "make", "create", "seed", "drive", "simulate", "emulate", "mimic",
                    "represent", "express", "indicate", "avoid", "prevent", "ensure", "force", "hold", "store", "track", "record",
                    "key", "name", "label", "scope", "calculate", "show", "pass", "feed", "fill", "generate", "assert", "count",
                    "exclude", "include", "filter", "skip", "ignore", "group", "sort", "split", "join", "parse", "resolve", "hide",
                    "reveal", "trigger", "wrap", "capture", "bound", "limit", "cap", "gate", "mock", "stub", "tag", "flag", "cache",
                    "carry", "encode", "decode", "dispatch", "reach", "address", "index", "order", "match", "separate", "lift"}
PREVIOUSLY = re.compile(r"\bpreviously\b(?!-)(?:\s+(\w+))?", re.I)
ORIGINALLY = re.compile(r"\boriginally\b(?!-)", re.I)
# Upstream revisions the repo pins on purpose (provenance, not history): a comment naming one stays unflagged.
PINNED_REVS = ("7e4034b262bc0d25332e330d8a582aaf34113829",  # w3c/aria-practices, playwright/oracle/reference/
               "295a1f114a138f23b5dfee0e0c6812394dfeb90c")  # shadcn-ui/ui, the style-nova reference
_ADJ_PREV = re.compile(r"(?:^|\W)%s[\s\"'`(\[]*$" % _AUX, re.I)
_PARTICIPLE = re.compile(r"\w+(?:ed|en)$|^(?:set|read|run|put|shown|sent|hidden|held|made|built|kept|given|taken|written|left|seen|known|lost|found|shut|done|hit|cut|spent|drawn|thrown)$", re.I)
# a present-tense passive description of behaviour ("changed by the caller", "removed from the payload by the server")
# is not a story: a `by <agent>` later in the same clause clears the was-X-ed phrases.
_PASSIVE_BY = re.compile(r"\bby\b")
_WAS_ED = re.compile(r"\b(?:(?:was|were|got) (?:removed|deleted|added|introduced|merged|extracted|reverted|retired|deprecated)|(?:was|were|got|has been|have been) (?:changed|renamed|rewritten))\b([^.;:]*)", re.I)
# "no longer X" is mostly a state the code handles ("an entry that no longer matches", "once it is no longer open"),
# so the generic form is history only when no condition word precedes it in the clause, or what follows is a role
# word for the code itself ("no longer used/needed/supported").
NO_LONGER = re.compile(r"\bno longer\b\s*`?(\w*)", re.I)
_SUBORD_BEFORE = re.compile(r"(?:\(|\b(?:that|which|who|whose|once|when|until|if|while|whenever|whether|unless|after|because|where|so)\b) *(?:[\w'`-]+ +){0,5}$", re.I)
_AUX_BEFORE = re.compile(r"\b(?:can|could|would|will|may|might|must|should|cannot|is|are|was|were|be|been|being|has|have|does|do|did) +(?:[\w'`-]+ +){0,2}$", re.I)
_STATE_AFTER = re.compile(r"^(?:in|at|on|over|against|under|inside|within|visible|open|present|connected|there|the|a|an|any)$", re.I)
# a state verb about the subject under test ("Duplicate no longer matches", "an API its primitive no longer has")
_STATE_VERB = re.compile(r"^(?:matches|match|has|have|exists|exist|applies|apply|intersects|overlaps|fits|resolves|resolve|contains|"
                         r"contain|equals|counts|count|appears|appear|hits|hit|fires|fire|reaches|reach|covers|cover)$", re.I)
_CONDITION_AFTER = re.compile(r"\b(?:once|when|whenever|while|if|after|because|unless)\b")
_ROLE_WORD = re.compile(r"^(?:used|needed|required|supported|referenced|called|imported|emitted|generated|wired|loaded|shipped|"
                        r"exposed|necessary|part|retries|relies|depends|applies|applicable|calls|supports)$", re.I)


def _adjective_use(body, m):
    """`previously`/`originally` right after a determiner or preposition ("the previously focused element") names a
    state, not a story. A lower-case word at the start of a line that is followed by a participle continues a sentence."""
    before = body[:m.start()]
    if _ADJ_PREV.search(before):
        return True
    if not before.strip() and body[m.start()].islower():
        nxt = body[m.start():].split(None, 2)[1:2]
        return not nxt or bool(_PARTICIPLE.search(nxt[0].strip(".,;:`\"'()")))
    return False


# A path is a link, which the rule wants: `dev-docs/research/round3-2026-10-05.md` is not "this round" or a date.
_PATH = re.compile(r"(?<![\w$])(?:[\w.@~-]+/)+[\w.@~\[\]*{}$-]*\.[A-Za-z][A-Za-z0-9]{0,5}\b|"
                   r"\b(?:%s)/[\w./@~\[\]*{}$-]+" % "|".join(re.escape(t) for t in ("dev-docs", "scripts", "primitives", "preview", "playwright",
                                                                                 "templates", "labs", "cli", "test-harness")))


def _provenance(body, m):
    """A pinned upstream revision names where code was ported from: `<repo> commit <sha>`, `<sha> of <repo>`,
    `pinned/upstream ... <sha>`; the gate leaves those alone (a `history-ok` marker is not needed)."""
    before, after = body[:m.start()], body[m.end():]
    if re.search(r"[\w.-]+/[\w.-]+`?\)?\s*(?:@|commit|rev(?:ision)?|at)\s*`?$", before, re.I):
        return True  # `radix-ui/primitives` commit <sha>, recharts/recharts @ <sha>
    if re.match(r"`?\s+of\s+`?[\w.-]+/[\w.-]+", after):
        return True  # <sha> of `w3c/aria-practices`
    return bool(len(m.group(0)) >= 12 and re.search(r"\b(?:pinned|upstream|vendored)\b", before, re.I))


def history_reason(body):
    """Why a comment line retells history, or None. One reason per line is enough: the unit is the line."""
    if "history-ok:" in body:
        return None
    body = _PATH.sub("PATH", body)
    if DATE.search(body) or MONTH_ISO.search(body) or NUMERIC_DATE.search(body) or _month_dated(body):
        return "date"
    for m in HEX.finditer(body):
        tok = m.group(0)
        if re.search(r"\d", tok) and len(re.findall(r"[a-f]", tok)) >= 2 and not re.fullmatch(r"\d+e\d+", tok):
            if any(rev.startswith(tok) for rev in PINNED_REVS) or body[:m.start()].endswith("@ ") or _provenance(body, m):
                continue  # a pinned upstream revision, or the provenance form `<path> @ <sha>`
            return "sha"
    for rx, why in PHRASES:
        m = rx.search(body)
        if m:
            if why == "was changed":
                w = _WAS_ED.search(body)
                if w and _PASSIVE_BY.search(w.group(1)):
                    continue  # `was changed by the caller`: behaviour, not history
            return why
    for m in USED_TO.finditer(body):
        prev = (m.group(1) or "").lower()
        if prev in PASSIVE_PREV:
            continue
        nxt = re.match(r"\s+(\w+)", body[m.end():])
        if prev not in ("that", "which", "who") and nxt and nxt.group(1).lower() in INSTRUMENT_VERBS:
            continue  # "a stub used to prove the gate red", "a value used to compute it": the instrument sense
        return "used to"
    for m in PREVIOUSLY.finditer(body):
        if re.search(r"\b(?:whatever|what|that|which)\s+(?:was|were|is)\s+$", body[:m.start()], re.I):
            continue
        if not _adjective_use(body, m):
            return "previously"
    for m in ORIGINALLY.finditer(body):
        if not _adjective_use(body, m):
            return "originally"
    for m in NO_LONGER.finditer(body):
        before = body[:m.start()]
        if _SUBORD_BEFORE.search(before):
            continue  # "an entry that no longer matches", "once it is no longer needed": a condition the code handles
        if _STATE_VERB.match(m.group(1)) or _CONDITION_AFTER.search(re.split(r"[.;:]", body[m.end():], maxsplit=1)[0]):
            continue  # "Duplicate no longer matches", "no longer applies once the element ...": a state, not a story
        if _ROLE_WORD.match(m.group(1)) or not (_AUX_BEFORE.search(before) or _STATE_AFTER.match(m.group(1))):
            return "no longer"
    return None


def scan_history(root, files):
    """{path: [(line, reason, text)]} for every file with at least one retelling comment line."""
    result = {}
    for p in files:
        hits = []
        for lineno, body in comments_of(root, p):
            why = history_reason(body)
            if why:
                hits.append((lineno, why, body.strip()))
        if hits:
            result[p] = hits
    return result


# ----------------------------------------------------------------------------- baselines
def read_baseline(path):
    base = {}
    if os.path.exists(path):
        for ln in open(path, encoding="utf-8").read().splitlines():
            if ln and not ln.startswith("#"):
                k, _, v = ln.partition("\t")
                base[k] = v
    return base


def usage(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)


def cmd_history(root, opts):
    baseline_path = os.path.join(root, HISTORY_BASELINE)
    files = opts["files"] or tracked_files(root)
    hits = scan_history(root, files)
    counts = {p: len(h) for p, h in hits.items()}
    if opts["inventory"]:
        for p in sorted(hits):
            for lineno, why, text in hits[p]:
                print(f"{p}:{lineno}\t{why}\t{text[:140]}")
        print(f"check-history-comments: {sum(counts.values())} lines in {len(counts)} files", file=sys.stderr)
        return 0
    base = {k: int(v) for k, v in read_baseline(baseline_path).items()}
    if opts["update"]:
        grow = {p: (base.get(p, 0), c) for p, c in counts.items() if c > base.get(p, 0)}
        if grow and os.path.exists(baseline_path) and not opts["allow_grow"]:
            print("check-history-comments: refusing to GROW the baseline (shrink-only; fix the comments or add\n"
                  "`history-ok: <reason>`; --allow-grow overrides, say why in the commit):", file=sys.stderr)
            for p, (a, b) in sorted(grow.items()):
                print(f"  {p}: {a} -> {b}", file=sys.stderr)
            return 1
        with open(baseline_path, "w", encoding="utf-8") as fh:
            fh.write("# check-history-comments baseline: comment lines that retell history, per file (shrink-only).\n")
            fh.write("# Regenerate with scripts/check-history-comments.sh --update-baseline; see dev-docs/history-in-comments.md.\n")
            for p in sorted(counts):
                fh.write(f"{p}\t{counts[p]}\n")
        print(f"check-history-comments: baseline written ({len(counts)} files, {sum(counts.values())} lines)")
        return 0
    rc = 0
    rose = [(p, base.get(p, 0), c) for p, c in sorted(counts.items()) if c > base.get(p, 0)]
    if rose:
        rc = 1
        print("check-history-comments: comment lines that retell history (dates, SHAs, \"used to\", \"previously\", \"no longer\" ...)\n"
              "were ADDED. A comment says what the code is and why; history lives in the backlog row and is linked\n"
              "(`dev-docs/backlog.md row N`). Rewrite the line, or mark a deliberate one `history-ok: <reason>`.\n"
              "See dev-docs/history-in-comments.md.", file=sys.stderr)
        for p, was, now in rose:
            print(f"  {p}: {was} -> {now}" + ("  (new file)" if p not in base else ""), file=sys.stderr)
            for lineno, why, text in hits[p][:12]:
                print(f"      {p}:{lineno} [{why}] {text[:110]}", file=sys.stderr)
    stale = [(p, int(v), counts.get(p, 0)) for p, v in sorted(base.items()) if counts.get(p, 0) < int(v)]
    if stale:
        rc = 1
        print("check-history-comments: the count FELL (good) but the baseline still allows the old number, which would\n"
              "let history creep back in. Lock the drop in: scripts/check-history-comments.sh --update-baseline",
              file=sys.stderr)
        for p, was, now in stale:
            print(f"  {p}: baseline {was}, now {now}" + ("  (file gone or clean)" if now == 0 else ""), file=sys.stderr)
    if rc == 0:
        print(f"check-history-comments: OK ({sum(counts.values())} baselined history lines in {len(counts)} files, none new)")
    return rc


# ----------------------------------------------------------------------------- reference detection
REPO_TOPS = ("dev-docs", "scripts", "primitives", "preview", "playwright", "templates", "labs", "docs", "cli",
             "dioxus-attributes", "test-harness", ".github")
PATH_RX = re.compile(r"(?<![\w./@~-])(?:%s)/[A-Za-z0-9_.\-/\[\]*<>{}$\u2026]*" % "|".join(re.escape(t) for t in REPO_TOPS))
ROW_RX = re.compile(r"\brows?\s+(\d{1,4})(?!\d|-\d\d-)[a-z]?((?:\s*(?:,|/|&|and|-|–|to)\s*\d{1,4}(?!\d|-\d\d-)[a-z]?)*)", re.I)
ROW_NUM = re.compile(r"\d{1,4}")
PLACEHOLDER = re.compile(r"[<>{}*$|]|NNN|XXX|NAME|\.\.\.|…|<name>|\bN\b")
GENERATED_OK = ("preview/assets/tailwind", "preview/public", "target/")


def backlog_ids(root):
    ids = set()
    for d in ("dev-docs/backlog", "dev-docs/backlog/done"):
        full = os.path.join(root, d)
        if os.path.isdir(full):
            for f in os.listdir(full):
                m = re.match(r"(\d{3})[a-z]?-.*\.md$", f)
                if m:
                    ids.add(int(m.group(1)))
    return ids


def refs_in(blocks, ids):
    """[(line, kind, ref)] unresolved in a list of (line, body) comment lines (consecutive lines share a block)."""
    bad = []
    prev = None
    # A comment block is a run of consecutive comment lines; "backlog" anywhere in it makes `row N` a backlog row.
    spans, cur = [], []
    for ln, body in blocks:
        if cur and ln != prev + 1:
            spans.append(cur)
            cur = []
        cur.append((ln, body))
        prev = ln
    if cur:
        spans.append(cur)
    for span in spans:
        block_has_backlog = any(re.search(r"backlog", b, re.I) for _, b in span)
        for ln, body in span:
            if "ref-ok:" in body:
                continue
            for m in ROW_RX.finditer(body):
                nums = [m.group(1)] + ROW_NUM.findall(m.group(2))
                # "backlog" in the block, or a 3-digit id in range (a grid's "row 998" is a data row, not a backlog row)
                strong = block_has_backlog or (len(nums[0]) >= 3 and int(nums[0]) <= max(ids, default=0) + 25)
                if not strong:
                    continue
                if re.search(r"\brows?\s+\d{1,3}\s+(?:of|in)\b", body[m.start():m.end() + 8], re.I):
                    continue
                for x in nums:
                    if int(x) not in ids:
                        bad.append((ln, "row", str(int(x))))
            for m in PATH_RX.finditer(body):
                if m.end() >= len(body.rstrip()) and m.group(0).endswith("-"):
                    continue  # hyphen-wrapped onto the next comment line: the rest of the name is not on this one
                tok = m.group(0).rstrip(".-")
                if PLACEHOLDER.search(tok) or tok.startswith(GENERATED_OK) or "node_modules" in tok:
                    continue
                bad.append((ln, "path", tok))
    return bad


def scan_refs(root, files):
    ids = backlog_ids(root)
    out = {}
    for p in files:
        cands = refs_in(comments_of(root, p), ids)
        res = []
        for ln, kind, ref in cands:
            if kind == "path":
                tok = ref
                tok = re.sub(r":\d+(?:-\d+)?$", "", tok)
                if os.path.exists(os.path.join(root, tok)) or os.path.exists(os.path.join(root, tok.rstrip("/"))):
                    continue
                ref = tok
            res.append((ln, kind, ref))
        if res:
            out[p] = res
    return out


def cmd_refs(root, opts):
    baseline_path = os.path.join(root, REFS_BASELINE)
    files = opts["files"] or tracked_files(root)
    found = scan_refs(root, files)
    keys = sorted({f"{p}\t{kind}\t{ref}" for p, rs in found.items() for _, kind, ref in rs})
    if opts["inventory"]:
        for p in sorted(found):
            for ln, kind, ref in found[p]:
                print(f"{p}:{ln}\t{kind}\t{ref}")
        print(f"check-comment-refs: {len(keys)} unresolved reference(s)", file=sys.stderr)
        return 0
    base_lines = set()
    if os.path.exists(baseline_path):
        base_lines = {ln for ln in open(baseline_path, encoding="utf-8").read().splitlines() if ln and not ln.startswith("#")}
    if opts["update"]:
        new = [k for k in keys if k not in base_lines]
        if new and os.path.exists(baseline_path) and not opts["allow_grow"]:
            print("check-comment-refs: refusing to GROW the baseline (shrink-only; fix the reference or add\n"
                  "`ref-ok: <reason>`; --allow-grow overrides, say why in the commit):", file=sys.stderr)
            for k in new:
                print(f"  {k}", file=sys.stderr)
            return 1
        with open(baseline_path, "w", encoding="utf-8") as fh:
            fh.write("# check-comment-refs baseline: dead `row N` / repo-path citations in code comments (shrink-only debt).\n")
            fh.write("# file<TAB>kind<TAB>ref. Fix the citation, then scripts/check-comment-refs.sh --update-baseline.\n")
            for k in keys:
                fh.write(k + "\n")
        print(f"check-comment-refs: baseline written ({len(keys)} dead references)")
        return 0
    rc = 0
    new = [k for k in keys if k not in base_lines]
    if new:
        rc = 1
        print("check-comment-refs: a code comment cites a backlog row or a repo path that does not exist\n"
              "(`row N` needs dev-docs/backlog/NNN-*.md or done/NNN-*.md; a path must exist in the tree):", file=sys.stderr)
        for k in new:
            p, kind, ref = k.split("\t")
            where = next(ln for ln, kd, rf in found[p] if kd == kind and rf == ref)
            print(f"  {p}:{where}: {kind} {ref}", file=sys.stderr)
        print("Fix the citation (history is linked by row id; a moved doc has a new path), or mark a deliberate one\n"
              "`ref-ok: <reason>`. See dev-docs/history-in-comments.md.", file=sys.stderr)
    stale = [k for k in sorted(base_lines) if k not in keys]
    if stale:
        rc = 1
        print("check-comment-refs: baseline entries that now resolve (or whose comment is gone); lock it in with\n"
              "scripts/check-comment-refs.sh --update-baseline:", file=sys.stderr)
        for k in stale:
            print(f"  {k.replace(chr(9), '  ')}", file=sys.stderr)
    if rc == 0:
        print(f"check-comment-refs: OK ({len(keys)} baselined dead references, none new)")
    return rc


# ----------------------------------------------------------------------------- self-tests
def _run_in_fixture(kind, files, baseline_lines, extra=()):
    """Build a throwaway git repo from {path: text}, write the baseline, run the gate; return (rc, stderr)."""
    with tempfile.TemporaryDirectory(prefix="comment-scan-") as d:
        subprocess.run(["git", "init", "-q", d], check=True)
        for p, text in files.items():
            full = os.path.join(d, p)
            os.makedirs(os.path.dirname(full), exist_ok=True)
            with open(full, "w", encoding="utf-8") as fh:
                fh.write(text)
        os.makedirs(os.path.join(d, "scripts"), exist_ok=True)
        base = HISTORY_BASELINE if kind == "history" else REFS_BASELINE
        with open(os.path.join(d, base), "w", encoding="utf-8") as fh:
            fh.write("\n".join(baseline_lines) + ("\n" if baseline_lines else ""))
        subprocess.run(["git", "-C", d, "add", "-A"], check=True)
        r = subprocess.run([sys.executable, "-I", os.path.abspath(__file__), kind, "--root", d, *extra],
                           capture_output=True, text=True)
        return r.returncode, r.stdout + r.stderr


def self_test_history():
    clean_rs = "// the panel is positioned from the trigger rect\nfn a() {}\n"
    base = ["src/a.rs\t1"]
    one = "// used to be 3px\n// the panel is positioned; changed on 2026-10-19\nfn a() {}\n"
    cases = [
        # (name, files, baseline, expected rc)
        ("clean fixture passes", {"src/a.rs": "// used to center\nfn a() {}\n"}, base, 0),
        ("dated comment added to a clean file", {"src/a.rs": clean_rs.replace("trigger rect", "trigger rect 2026-10-19")}, [], 1),
        ("one more line in a baselined file", {"src/a.rs": one}, base, 1),
        ("new file with a history line", {"src/b.css": "/* previously, this was 2px */\na{}\n"}, [], 1),
        ("sha in a comment", {"src/c.ts": "// fixed in 98e7ca1b\nconst x = 1;\n"}, [], 1),
        ("shell comment", {"src/d.sh": "# we now run this first\necho hi\n"}, [], 1),
        ("block comment line", {"src/e.rs": "/*\n * no longer uses the cache since the rewrite\n */\nfn a(){}\n"}, [], 1),
        ("history-ok exempts the line", {"src/a.rs": "// used to be 3px, 2026-10-19 history-ok: quoted upstream changelog\nfn a() {}\n"}, [], 0),
        ("stale baseline (count fell) fails", {"src/a.rs": clean_rs}, base, 1),
        ("baselined file gone fails", {"src/z.rs": "fn z(){}\n"}, ["src/gone.rs\t2"], 1),
        ("colour in code is not a SHA", {"src/f.css": "a { color: #abcdef1; }\n/* border colour is #abc1234 */\n"}, [], 0),
        ("date in a string is not a comment", {"src/g.rs": 'const D: &str = "2026-10-19 // not a comment";\n'}, [], 0),
        ("`is used to` is not history", {"src/h.rs": "// is used to size the panel; can be used to anchor it\nfn a(){}\n"}, [], 0),
        ("skipped dir is ignored", {"dev-docs/archive/x.rs": "// 2026-10-19 used to\n", "docs/y.js": "// 2026-10-19\n"}, [], 0),
        ("a code file under dev-docs/ is scanned", {"dev-docs/supply-chain/z.sh": "# used to be 3px\n"}, [], 1),
        ("a dated comment inside JS in a rust string is counted", {"src/j.rs": 'const J: &str = r#"\n// 2026-10-19 fix\nconst a = 1;\n"#;\n'}, [], 1),
        ("a clean comment inside JS in a rust string passes", {"src/j.rs": 'const J: &str = r#"\n// anchors to the trigger\nconst a = 1;\n"#;\n'}, [], 0),
        ("history-ok exempts a comment inside a rust string", {"src/j.rs": 'const J: &str = r#"\n// 2026-10-19 fix history-ok: quoted\nconst a = 1;\n"#;\n'}, [], 0),
        ("a path with a date is not a date", {"src/p.sh": "# see dev-docs/research/round3-2026-10-05.md\n"}, [], 0),
    ]
    fails = 0
    for name, files, bl, want in cases:
        rc, out = _run_in_fixture("history", files, bl)
        ok = rc == want
        fails += not ok
        print(f"  {'ok  ' if ok else 'FAIL'} {name} (rc {rc}, want {want})")
        if not ok:
            print("    " + out.replace("\n", "\n    "))
    fails += _self_test_lines()
    fails += _self_test_extraction()
    fails += _self_test_embedded_lines()
    # a real file, mutated: copy this very script's neighbour with a dated comment appended (must turn red)
    real = os.path.join(ROOT, "scripts/lane-touch.sh")
    if os.path.exists(real):
        text = open(real, encoding="utf-8").read()
        rc0, _ = _run_in_fixture("history", {"scripts/lane-touch.sh": text}, [])
        rc1, out = _run_in_fixture("history", {"scripts/lane-touch.sh": text + "\n# changed on 2026-10-19, used to differ\n"},
                                   [f"scripts/lane-touch.sh\t{len(scan_history_text('scripts/lane-touch.sh', text))}"])
        rc0b, _ = _run_in_fixture("history", {"scripts/lane-touch.sh": text},
                                  [f"scripts/lane-touch.sh\t{len(scan_history_text('scripts/lane-touch.sh', text))}"])
        ok = rc0b == 0 and rc1 == 1
        fails += not ok
        print(f"  {'ok  ' if ok else 'FAIL'} real-file mutation (lane-touch.sh + dated comment): baseline rc {rc0b}, mutated rc {rc1}")
    print(f"check-history-comments --self-test: {'OK' if not fails else str(fails) + ' FAILED'}")
    return 1 if fails else 0


# Comment bodies that MUST be flagged (the history forms a reviewer found the scanner missing) ...
MUST_FLAG = [
    "dating: since March the panel reads the registered item",
    "dating: since March 2025 the ring is 2px",
    "dating: this changed in October 2026 after the report",
    "dating: as of October the helper owns it",
    "dating: last January we dropped the wrapper",
    "dating: early March, before the rewrite",
    "numeric date 02/03/2026 in a comment",
    "numeric date 2.3.2027 in a comment",
    "Previously it returned nil",
    "it previously returned nil here",
    "each one previously asserted the absence of an input",
    "Previously, the caller wrote this by hand",
    "this is no longer used by the renderer",
    "no longer used by the renderer",
    "the helper no longer retries on 429",
    "callers no longer wrap it in a <main>",
    "the timer, which used to be a timer per row",
    "the value that used to live here",
    "originally a one-shot measurement",
    "was originally written against the main fixture",
    "the old engine rebuilt the whole geometry",
    "the old test set it before load",
    "the new rule matches what holds after the fix",
    "the same shape as before this change",
    "before row 169 landed the popover was modal",
    "default since row 303",
    "until row 228 this was ambient",
    "row 8 landed and the locator changed",
    "this round's regression guard",
    "the previous round's click-to-caret fix",
    "round5's own fixed bug",
    "this file was added when the gate was introduced",
    "the table was added",
    "a value that was changed in row 12",
    "which has since been wired",
    "back when Navbar wrapped it",
    "the wrapper was renamed from Foo",
    "The STYLE-DELIVERY lane found 208 sheet pairs",
    "batch 13 added it",
    "the batch-2 integration server",
    "round 1's migration moved it",
    "see lane/p5tabs for the evidence",
    "Found twice in review before it was fixed",
    "Seen twice: the carousel dot lost its colour",
    "caught by first-paint-styles.spec.ts",
    "the first version of this gate banned it",
    "Earlier note in this file claimed a cold cache self-heals",
    "the theme now has ONE zero-specificity rule",
    "same as before this round",
    "`width: 100%` was added next to it",
    "the old calendar rule used a hand-written ring",
    "the `5fc1439`-shaped collision shape",
    "a stub used to call this before",
    # round 2: month-only dates, work-stream words, evidence narration, status headers, past framing, reports
    "(2026-08) the ring was 3px",
    "the audit of 2027-01 moved it",
    "see dev-docs/research/a.md; this round's regression guard",
    "assertion for the bug this lane fixed",
    "another lane owns it, so this file only reads",
    "see this lane's own report for the table",
    "(`virtual/window.rs`, read-only to this lane)",
    "Viewports: the six the lane brief itself named",
    "phase C lane F moved it to the theme",
    "RadialBar is still `s2-refactor`'s placeholder",
    "Rule 10 (Migration A slice 3/3, final) stacks toasts",
    "see this session's report for the full ledger",
    "roving-tabstop design this fix deliberately preserves",
    "the thing this port needs to match for the angle",
    "per the task brief, dark is where this is worst",
    "from that pinned commit's own file (fetched fresh for this task)",
    "demo (`$S/refs/ui/apps/v4/registry/charts/chart-bar-*.tsx`)",
    "the contract (`$S/chart-api.md`)",
    "confirmed live: 3 page-wide vs. 1 once scoped",
    "Found live: the mount measured 0px",
    "found by execution: main's own content sat low",
    "Verified live: FocusNext from the day segment",
    "this was verified at write time against the page",
    "confirmed empirically, not assumed",
    "Confirmed by execution: before this the anchor never resolved",
    "a small box (confirmed there by execution",
    "also confirmed necessary by execution, and the rest follows",
    "a live RED-then-GREEN result on the Playwright side",
    "the pre-fix code could still have passed by coincidence",
    "byte-for-byte the pre-Phase-4.2 path",
    "the pre-migration contract this asserts",
    "matching the pre-refactor flat argument list",
    "this pipeline has been observed to silently drop a rule",
    "went red 3 times in 20 under --workers=4",
    "(intermittently red without the fix):",
    "so the pressed background already survived hover on the unmodified tree",
    "STATUS AT WRITE TIME: RED. No scroll lock exists",
    "**UPDATE (row 46, construction landed):**",
    "HISTORY, AND WHY the gate exists",
    "RED-FIRST: every test below was run against the old file",
    "Written RED first, then fixed",
    "At HEAD a fresh 100 ms interval was installed per call",
    "30 controls on 23 components had NO focus indicator",
    "had NO hover companion at all",
    "Correction, filed 2026-10-19, to the paragraph above",
    "row 91's dated addendum has the report",
    "found live, the hard way: every variant shares the host",
    "an index counter was tried first; it drifted",
    "the original v1 construction kept a Vec",
    "every generation above has kept it",
    "One dot inside an indicators list. Renamed from `CarouselTab`.",
    "the farthest distance (not the previous 2.5) is the one",
    "keeps winning exactly as before, with no change to this rule",
    "is unchanged from before this slice",
    "threshold (the original asserted N=3)",
    "The previous engine kept a Vec of items",
    "the pre-shadcn-parity `multiple` variant",
    "a pre-this-feature hardcoded value",
    "the fix for the historical stranded-signal defect",
    "given when Phase 4.4 landed",
    "Before Phase 1.1 this test passed by accident",
    "renders exactly the markup it did before this prop existed",
    "deliberately NOT clamped any more",
    "the owner report: vertical slider css is messed up",
    "Live-site user report: opening the palette by click",
    "the owner's real-trackpad report (MacBook, Firefox)",
    "the owner's lab session: pinned for a minute",
    "the incident behind that choice",
    "(the upstream regression that motivated PR #47)",
    "scrolled) from PR #107. A comment-only sheet is exempt",
    "the literal case the bug report names",
]
# ... and present-tense sentences that must NOT be (the false positives to keep out).
MUST_NOT_FLAG = [
    "returns a value that was changed by the caller",
    "the field was removed from the payload by the server",
    "a row that was deleted by another tab is skipped",
    "restores the previously focused element",
    "keeps focus on a previously tapped text field",
    "the previously-selected pill is still narrowing",
    "cancels whatever was previously running first",
    "an entry that no longer matches anything fails",
    "once the item is no longer needed it is dropped",
    "the scroller is no longer at the slice edge",
    "regions without toasts, or no longer in the document, are skipped",
    "30 days in November + 31 in December",
    "opens on May 2026 with the range demo",
    "the caption reads March 2025 in the fixture",
    "dates like 02/03/2025 and 12/25/2024 format per locale",
    "the leap day falls in February and the week holds January 1st",
    "the plain text \"October 2026\" between the arrows",
    "the walk starts in October",
    "is used to size the panel and can be used to anchor it",
    "the width that's used to size the popup",
    "one round trip to the server per frame, a round-robin pick",
    "compares the old value with the new value and the old and new props",
    "the originally requested width wins over the measured one",
    "state remembered before the swap is put back",
    "a stub used to prove the gate red",
    "a value used to compute the width",
    "the lane/spec builds, row 152a",
    "runs on the SSG lane against a prerendered page",
    "masonry lanes: each card goes to the shortest lane",
    "accepts the first attempt that clears the threshold; the first pass measures",
    "a batch size of 13 and a round of 3 samples",
    "the old year, the old element and the old index",
    "history-ok: used to be 3px since March, 2026-10-19 (a quoted changelog)",
    # round 2: a path is a link, pinned upstream revisions are provenance, "no longer" can be a condition
    "see dev-docs/research/round3-2026-10-05.md for the numbers",
    "THE DEFECT (dev-docs/research/2026-10-05.md section 2): a sheet",
    "`radix-ui/primitives` commit `f7ecd5ab16f5e1e820eb5786a1419a98a2d594ae`",
    "`recharts/recharts` commit 86ad3632ff3f83a742001ae6fc079dd27961a2a4",
    "`primitives/src/scroll_lock.rs` @ 5af3cc292559a0e8d73c7b9a827c4ca08ef34d99,",
    "7e4034b262bc0d25332e330d8a582aaf34113829 of w3c/aria-practices -- see that",
    "pinned to the upstream tree at a82254af78f08799c71d7ab25df557c4872a3c51",
    "buffer \"de\" -- Duplicate no longer matches, Delete does",
    "whether the element was unmounted (no longer `exists`) before the cap",
    "a spec whose titles no longer match fails",
    "a file that no longer has a hit fails (stale)",
    "a component calling an API its primitive no longer has",
    "once it is no longer referenced it is dropped",
    "the constraint no longer applies once the element is out of flow",
    "a closing toast no longer counts toward the pile",
    "used to exclude `.use_something(` method calls",
    "a regex used to filter the rows and to skip the header",
    "when running against this lane, also serve the fixtures",
    "Queued debug SSG build for a lane worktree",
    "other lanes are compiling on the same box; the report prints both",
    "Draw this slice active: 10px further out",
    "the server listens on this port, the dev proxy forwards to it",
    "Confirmed necessary by execution's design: the trace shows it",
    "This is checked by execution, not by hand math",
    "measured live from a rendered slide's own box rather than assumed",
    "RED/GREEN: tests 1 and 3 are red on a build without the bundle and green with it",
    "Press left arrow to move focus back to the original day",
    "the original touch coordinates are restored after the drag",
    "Upstream PR #293 ever got a chance to play",
    "upstream Dioxus PR #255 (the lifted input)",
    "Files that are not in the registry any more but that you edited; kept.",
    "nothing is animating on it any more",
    "Nothing is hovered or focused any more: close the tooltip",
    "scroll back to where it did before the click",
    "files created before this file existed have no baseline",
    "remains the owner's real-device check",
    "sweeping this class one incident at a time only catches the sites",
    "(row 374 has the incident)",
    "historical data in the chart is clipped to the window",
    "the previous version of the file is kept as .orig",
    "pre-existing baseline entries do not fail the gate",
    "the dialog is gone (unmounted) or has no scrim left",
    "`:popover-open` is gone once Escape closes it",
    "state carried over from the previous render",
    "a drag that started as a touch keeps the touch id",
    "the handler was never attached, so the click is dropped",
    "see Phase 1.3 and the form page for the fixture",
    "the previous attempt's own gesture is cancelled",
    "the first digit of the previous point is kept",
]


def _self_test_lines():
    fails = 0
    for want, bodies in ((True, MUST_FLAG), (False, MUST_NOT_FLAG)):
        for body in bodies:
            got = history_reason(body) is not None
            ok = got == want
            fails += not ok
            if not ok:
                print(f"  FAIL line {'must flag' if want else 'must not flag'}: {body!r} -> {history_reason(body)}")
    print(f"  {'ok  ' if not fails else 'FAIL'} {len(MUST_FLAG)} lines that must flag, {len(MUST_NOT_FLAG)} that must not")
    return fails


# (name, file name, source, number of comment lines that must be flagged): the extractors decide what is a comment.
EXTRACTION_CASES = [
    ("js regex with a class holding /* is not a block comment", "a.js", 'const r = /[/*]/;\nconst s = "used to be";\n', 0),
    ("js regex ending in /*$/ then a real comment", "a.js", 'x = p.replace(/\\/*$/, "");\nconst k = 1; // used to be 3\n', 1),
    ("js division is not a regex", "a.js", "const h = w / 2; /* no longer uses it */ const k = a / b;\n", 1),
    ("js regex holding a backtick hides nothing", "a.ts", "const r = /`/g;\n// used to be x\nconst t = `a`;\n", 1),
    ("js regex after return", "a.js", "function f(s) { return /[/*]/.test(s); }\nconst s = 'used to';\n", 0),
    ("python triple-quoted non-docstring is data", "a.py", 'x = 1\ns = """\n# used to be\nused to be\n"""\n', 0),
    ("python # comment is read", "a.py", "x = 1  # used to be 3px\n", 1),
    ("python # in a string is not a comment", "a.py", 's = "# used to be"\n', 0),
    ("python docstring is read", "a.py", 'def f():\n    """Returns x; used to return y."""\n', 1),
    ("shell heredoc that writes a file is data", "a.sh", "cat > f <<'EOF'\n# used to be 3px\nEOF\n", 0),
    ("shell heredoc into an interpreter is code", "a.sh", "python3 - <<'PY'\n# used to be 3px\nPY\n", 1),
    ("shell heredoc tagged PYEOF is code", "a.sh", "cat >\"$c\" <<'PYEOF'\n# used to be 3px\nPYEOF\n", 1),
    ("shell heredoc written to a .py file is code", "a.sh", "cat > check.py <<'EOF'\n# used to be 3px\nEOF\n", 1),
    ("shell comment after a heredoc is read", "a.sh", "cat <<EOF\nx\nEOF\n# used to be 3px\n", 1),
    ("shell here-string is not a heredoc", "a.sh", "cat <<<EOF\n# used to be 3px\n", 1),
    # JS and CSS held in Rust string literals: a comment in the code is a comment, the string text around it is data
    ("rust raw-string JS: a // comment is read", "a.rs", 'const J: &str = r#"\n  // used to be 3\n  const a = 1;\n"#;\n', 1),
    ("rust raw-string JS: a trailing // comment is read", "a.rs", 'const J: &str = r#"const a = 1; // 2026-09-02 iOS fix\nfoo();"#;\n', 1),
    ("rust raw-string JS: a /* */ comment is read", "a.rs", 'const J: &str = r#"function f() { /* no longer uses it */ return 1; }"#;\n', 1),
    ("rust normal-string JS: \\n and \\\" are unescaped", "a.rs", 'let s = "const a = \\"x\\";\\n// previously x\\nfoo();";\n', 1),
    ("rust string JS with a line continuation", "a.rs", 'let w = "const a = 1; \\\n        // previously x";\n', 1),
    ("rust string CSS: a /* */ comment is read", "a.rs", 'let c = "a { color: red; } /* filed 2026-09-03 */";\n', 1),
    ("rust string CSS: // is not a comment", "a.rs", 'let c = "a { background: url(//cdn/used to.png); color: red; }";\n', 0),
    ("rust prose string with // is data", "a.rs", 'let p = "2026-10-19 // used to be 3px";\n', 0),
    ("rust string JS: a URL is not a comment", "a.rs", 'let u = "const u = \'https://x.dev/used to be\'; if (a) { b(); }";\n', 0),
    ("rust string JS: // inside a JS string is not a comment", "a.rs", 'let u = "const m = \'// used to be\'; function f() { return m; }";\n', 0),
    ("rust string with no code shape is data", "a.rs", 'let u = "see /* used to be */ above";\n', 0),
    ("rust byte string is skipped", "a.rs", 'let u = b"const a = 1; // used to be";\n', 1),
    ("rust string JS with a clean comment", "a.rs", 'const J: &str = r#"\n  // the panel is positioned from the trigger\n  const a = 1;\n"#;\n', 0),
]


def _self_test_extraction():
    fails = 0
    for name, path, text, want in EXTRACTION_CASES:
        got = len(scan_history_text(path, text))
        ok = got == want
        fails += not ok
        if not ok:
            print(f"  FAIL extraction {name}: {got} flagged, want {want}")
    print(f"  {'ok  ' if not fails else 'FAIL'} {len(EXTRACTION_CASES)} extraction cases (regex literals, python strings, heredocs)")
    return fails


def _self_test_embedded_lines():
    """A comment in a Rust string is reported at its source line: raw, escaped (`\\n`) and continued strings."""
    src = ('fn a() {\n'                                   # 1
           '    let r = r#"\n'                               # 2
           '        const a = 1;\n'                         # 3
           '        // raw comment\n'                       # 4
           '    "#;\n'                                      # 5
           '    let e = "const b = 2;\\n// escaped comment\\nfoo();";\n'  # 6
           '    let w = "const c = 3; \\\n'                # 7
           '        // continued comment";\n'               # 8
           '    // plain comment\n'                         # 9
           '}\n')
    got = {(ln, b.strip()) for ln, b in extract_rust(src)}
    want = {(4, "raw comment"), (6, "escaped comment"), (8, "continued comment"), (9, "plain comment")}
    ok = got == want
    if not ok:
        print(f"  FAIL embedded line numbers: got {sorted(got)}, want {sorted(want)}")
    print(f"  {'ok  ' if ok else 'FAIL'} embedded comment line numbers (raw, escaped, continued)")
    return 0 if ok else 1


def scan_history_text(path, text):
    ext = os.path.splitext(path)[1]
    return [1 for _, b in EXTRACTORS[EXT_KIND[ext]](text) if history_reason(b)]


def self_test_refs():
    rows = {"dev-docs/backlog/266-x.md": "x", "dev-docs/backlog/done/012-y.md": "y", "scripts/real.sh": "# hi\n", "dev-docs/backlog.md": "x"}
    cases = [
        ("existing row and path pass",
         {**rows, "src/a.rs": "// dev-docs/backlog.md row 266 and row 12; see scripts/real.sh:3\n"}, [], 0),
        ("dead row fails", {**rows, "src/a.rs": "// backlog row 999 explains it\n"}, [], 1),
        ("dead path fails", {**rows, "src/a.rs": "// see docs/research/nowhere.md for the numbers\n"}, [], 1),
        ("dead path in css fails", {**rows, "src/a.css": "/* see scripts/gone.sh */\n"}, [], 1),
        ("dead path in ts fails", {**rows, "src/a.ts": "// see playwright/missing.spec.ts\n"}, [], 1),
        ("dead path in shell fails", {**rows, "src/a.sh": "# see primitives/src/missing.rs\n"}, [], 1),
        ("ref-ok exempts", {**rows, "src/a.rs": "// see docs/old.md ref-ok: quoted from the upstream issue\n"}, [], 0),
        ("a date is not a row id", {**rows, "src/a.rs": "// backlog row 266 (row 2026-10-05, owner)\n"}, [], 0),
        ("4-digit dead row fails", {**rows, "src/a.rs": "// backlog row 9999\n"}, [], 1),
        ("grid row is not a backlog row", {**rows, "src/a.rs": "// row 5 of the grid is focused\n"}, [], 0),
        ("placeholder path is skipped", {**rows, "src/a.rs": "// preview/src/components/<name>/style.css and scripts/*.sh\n"}, [], 0),
        ("baselined dead path passes", {**rows, "src/a.rs": "// see docs/old.md\n"}, ["src/a.rs\tpath\tdocs/old.md"], 0),
        ("stale baseline fails", {**rows, "src/a.rs": "// see scripts/real.sh\n"}, ["src/a.rs\tpath\tdocs/old.md"], 1),
        ("path in a string is not a comment", {**rows, "src/a.rs": 'const P: &str = "docs/gone/x.md";\n'}, [], 0),
    ]
    fails = 0
    for name, files, bl, want in cases:
        rc, out = _run_in_fixture("refs", files, bl)
        ok = rc == want
        fails += not ok
        print(f"  {'ok  ' if ok else 'FAIL'} {name} (rc {rc}, want {want})")
        if not ok:
            print("    " + out.replace("\n", "\n    "))
    print(f"check-comment-refs --self-test: {'OK' if not fails else str(fails) + ' FAILED'}")
    return 1 if fails else 0


def main(argv):
    if len(argv) < 2 or argv[1] not in ("history", "refs"):
        usage("usage: comment-scan.py <history|refs> [--root DIR] [--inventory|--update-baseline [--allow-grow]|--self-test] [FILE...]")
    kind = argv[1]
    opts = {"inventory": False, "update": False, "allow_grow": False, "files": []}
    root, selftest = ROOT, False
    it = iter(argv[2:])
    for a in it:
        if a == "--inventory":
            opts["inventory"] = True
        elif a == "--update-baseline":
            opts["update"] = True
        elif a == "--allow-grow":
            opts["allow_grow"] = True
        elif a == "--self-test":
            selftest = True
        elif a == "--root":
            root = next(it)
        elif a.startswith("-"):
            usage(f"comment-scan: unknown option {a}")
        else:
            opts["files"].append(a)
    if selftest:
        return self_test_history() if kind == "history" else self_test_refs()
    return (cmd_history if kind == "history" else cmd_refs)(root, opts)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
