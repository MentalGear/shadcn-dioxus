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
string literal is not a comment. Not covered (known gap, ratchet-neutral): CSS or JS that lives inside a Rust
string literal.

Scanned: `git ls-files` with the extensions above, minus SKIP_PREFIXES (dev-docs/archive/ is where history belongs, the
prose under dev-docs/ is not code so no extension picks it up, but its code files are scanned;
docs/ is the deployed site; the APG reference tree is vendored; generated primitives JS), the two gates'
own sources, lock files.

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
            else:
                j = i + 1
                while j < n and text[j] != '"':
                    j += 2 if text[j] == "\\" else 1
                j += 1
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


def extract_js(text):
    i, n, line = 0, len(text), 1
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            yield line, text[i + 2:j]
            i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j
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
    (re.compile(r"\bsince (?:been )?(?:removed|replaced|renamed|dropped|fixed)\b|\bhas since\b|\buntil recently\b|\bback when\b", re.I), "since removed"),
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
                    "key", "name", "label", "scope", "calculate", "show", "pass", "feed", "fill", "generate", "assert", "count"}
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
NO_LONGER = re.compile(r"\bno longer\b\s*(\w*)", re.I)
_SUBORD_BEFORE = re.compile(r"\b(?:that|which|who|once|when|until|if|while|whenever|whether|unless|after|because|where|so) +(?:[\w'`-]+ +){0,5}$", re.I)
_AUX_BEFORE = re.compile(r"\b(?:can|could|would|will|may|might|must|should|cannot|is|are|was|were|be|been|being|has|have|does|do|did) +(?:[\w'`-]+ +){0,2}$", re.I)
_STATE_AFTER = re.compile(r"^(?:in|at|on|over|against|under|inside|within|visible|open|present|connected|there|the|a|an|any)$", re.I)
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


def history_reason(body):
    """Why a comment line retells history, or None. One reason per line is enough: the unit is the line."""
    if "history-ok:" in body:
        return None
    if DATE.search(body) or NUMERIC_DATE.search(body) or _month_dated(body):
        return "date"
    for m in HEX.finditer(body):
        tok = m.group(0)
        if re.search(r"\d", tok) and len(re.findall(r"[a-f]", tok)) >= 2 and not re.fullmatch(r"\d+e\d+", tok):
            if any(rev.startswith(tok) for rev in PINNED_REVS) or body[:m.start()].endswith("@ "):
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
