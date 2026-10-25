"""The one CSS reader of the gate scripts (backlog row 429).

Every `scripts/check-*.sh` that reads a stylesheet imports this module instead of embedding its own comment
stripper, brace walker or selector splitter, so two gates that read one sheet read it one way, by construction.
A gate's Python heredoc runs from the repository root and imports it with

    import sys
    sys.path.insert(0, "scripts/lib")
    from css import rules as css_rules, split_top      # `from`, never `import css`: gates keep a local `css`

(`python3 -I` ignores PYTHONPATH, hence the explicit insert.) `scripts/check-gate-kit.sh` holds the fixtures below
as tests, mutation-tests them against this file, and fails a gate that defines its own comment stripper or splitter.

What the scanner understands:
  - comments, replaced by their newlines so a line number is still the line of the source;
  - quoted strings with backslash escapes (`content: "\\""` does not end at the escaped quote), and an unescaped
    newline ends a string the way CSS ends a bad string, so one stray apostrophe cannot swallow the rest of a file;
  - an unquoted `url(...)`, taken whole: `url(data:image/svg+xml;utf8,<svg ... {>)` contains `;`, `{` and quotes that
    are not CSS structure;
  - parentheses: `;`, `{` and `}` inside `(...)` are not structure (`:is(...)`, `calc(...)`, `var(--a, {})`);
  - a backslash escape anywhere (`.a\\{b`, `.sm\\:flex`), which takes the next character literally;
  - nested at-rules (`@media { @supports { rule } }`), reported in each rule's `at` chain, outermost first;
  - rules nested in rules (CSS nesting): each is its own Rule, the parent's `nested` flag is set and its `body`
    holds the raw text including the nested block, its `decls` only the declarations written outside it.

Two modes read text that is not a stylesheet: `strings=False` (comment stripping and splitting treat quotes as plain
characters) and `fragment=True` (`parse`/`rules`: the CSS inside a Rust raw string, an HTML `<style>`, or the Rust
source around it, read by braces and `;` alone, the way the host's quotes require).

The offsets of `Sheet.text` are valid for every node, so a gate can slice a body without re-scanning.
"""
import re
from typing import List, NamedTuple, Optional, Tuple

__all__ = [
    "strip_comments", "blank_comments", "split_top", "subject_compound", "parse", "rules", "split_decl", "Rule", "Node",
    "Sheet", "line_of",
]

_IDENT = re.compile(r"[\w-]")
_URL = re.compile(r"url\(", re.I)


def _atom_end(s: str, i: int) -> Optional[int]:
    """If a string, a comment or an unquoted `url(...)` starts at `s[i]`, the index just after it, else None.

    A string stops at its closing quote, and also before an unescaped newline (CSS: a bad string). A comment or an
    unquoted url that never closes runs to the end of the text.
    """
    n = len(s)
    c = s[i]
    if c == "/" and s.startswith("/*", i):
        j = s.find("*/", i + 2)
        return n if j < 0 else j + 2
    if c in "\"'":
        j = i + 1
        while j < n:
            d = s[j]
            if d == "\\" and j + 1 < n:
                j += 2
                continue
            if d == c:
                return j + 1
            if d == "\n":
                return j
            j += 1
        return n
    if c in "uU" and (i == 0 or not _IDENT.match(s[i - 1])) and _URL.match(s, i):
        j = i + 4
        while j < n and s[j] in " \t\r\n\f":
            j += 1
        if j < n and s[j] in "\"'":
            return None  # a quoted url: the string and the parenthesis are scanned the ordinary way
        while j < n:
            d = s[j]
            if d == "\\" and j + 1 < n:
                j += 2
                continue
            if d == ")":
                return j + 1
            j += 1
        return n
    return None


def strip_comments(css: str, *, strings: bool = True) -> str:
    """`css` with every comment replaced by its own newlines: a line number of the result is a line of the source.

    A comment opener inside a string or an unquoted `url(...)` is text, not a comment. `strings=False` is for text
    that is not a stylesheet (the CSS in a Rust string, an HTML `<style>`): every `/* ... */` goes, wherever it sits.
    """
    return _remove_comments(css, lambda comment: "\n" * comment.count("\n"), strings)


def blank_comments(css: str, *, strings: bool = True) -> str:
    """Like `strip_comments`, but every other character of a comment becomes a space too: offsets and columns of
    the result are those of the source (for a gate that reports `file:line:col` or slices by position)."""
    return _remove_comments(css, lambda comment: re.sub(r"[^\n]", " ", comment), strings)


def _remove_comments(css: str, fill, strings: bool) -> str:
    if not strings:
        return re.sub(r"/\*.*?\*/", lambda m: fill(m.group(0)), css, flags=re.S)
    out: List[str] = []
    i, n, start = 0, len(css), 0
    while i < n:
        c = css[i]
        if c == "\\" and i + 1 < n:
            i += 2
            continue
        if c in "/\"'uU":
            j = _atom_end(css, i)
            if j is not None:
                if c == "/":
                    out.append(css[start:i])
                    out.append(fill(css[i:j]))
                    start = j
                i = j
                continue
        i += 1
    out.append(css[start:])
    return "".join(out)


def split_top(s: str, seps: str = ",", *, strings: bool = True) -> List[str]:
    """Split `s` on any character of `seps` outside quotes, `url(...)`, parentheses and square brackets, honouring
    backslash escapes. The pieces are raw (not stripped, empty ones kept): `"a, b,"` gives `['a', ' b', '']`.
    `strings=False` is for text that is not CSS (the Rust code in front of a CSS fragment): quotes and backslashes are
    plain characters there."""
    out: List[str] = []
    depth, start, i, n = 0, 0, 0, len(s)
    while i < n:
        c = s[i]
        if strings:
            if c == "\\" and i + 1 < n:
                i += 2
                continue
            if c in "\"'uU":
                j = _atom_end(s, i)
                if j is not None:
                    i = j
                    continue
        if c in "([":
            depth += 1
        elif c in ")]":
            depth = max(0, depth - 1)
        elif c in seps and depth == 0:
            out.append(s[start:i])
            start = i + 1
        i += 1
    out.append(s[start:])
    return out


def subject_compound(selector: str, *, strings: bool = True) -> str:
    """The rightmost compound selector of a complex selector (the element a rule styles): split on whitespace and
    `>`, `+`, `~` outside quotes, `url(...)`, parentheses and brackets. `".a > .b:not(.c d)"` gives `".b:not(.c d)"`.
    `strings=False`: see `split_top`."""
    parts = [p for p in split_top(selector.strip(), " \t\n\r\f>+~", strings=strings) if p]
    return parts[-1] if parts else ""


def split_decl(decl: str) -> Tuple[str, str]:
    """`(property, value)` of one declaration text (`Rule.decls` item): property lower-cased, value stripped. A text
    with no colon gives `("", decl)`."""
    if ":" not in decl:
        return "", decl.strip()
    prop, value = decl.split(":", 1)
    return prop.strip().lower(), value.strip()


def line_of(text: str, index: int) -> int:
    """1-based line number of `text[index]`."""
    return text.count("\n", 0, index) + 1


class Node:
    """One block of a stylesheet. `kind` is "at" (`@media ...`, `@keyframes x`, `@font-face`) or "rule" (a style rule
    or a keyframe selector such as `from`, `50%`)."""

    __slots__ = ("kind", "prelude", "line", "brace_line", "body_start", "body_end", "decls", "decl_lines", "children",
                 "at", "parent", "text")

    def __init__(self, kind, prelude, line, brace_line, body_start, at, parent, text):
        self.kind = kind
        self.prelude = prelude          # the text before `{`, stripped (not whitespace-collapsed)
        self.line = line                # line the prelude starts on
        self.brace_line = brace_line    # line of the `{`
        self.body_start = body_start    # index in Sheet.text just after `{`
        self.body_end = body_start      # index of the matching `}` (the end of the text when it never closes)
        self.decls: List[str] = []      # own declarations, stripped, empty ones dropped
        self.decl_lines: List[int] = [] # the line each declaration starts on
        self.children: List["Node"] = []
        self.at: Tuple[str, ...] = at   # enclosing at-rule preludes, outermost first
        self.parent = parent
        self.text = text

    @property
    def body(self) -> str:
        """Raw text between the braces, nested blocks included."""
        return self.text[self.body_start:self.body_end]

    @property
    def nested(self) -> bool:
        """The block holds another block (CSS nesting inside a rule, or the rules of an at-rule)."""
        return bool(self.children)

    def __repr__(self):
        return f"Node({self.kind} {self.prelude!r} line {self.line})"


class Sheet:
    """A parsed stylesheet: `text` (comments stripped, newlines kept), top-level `nodes`, and `problems`, a list of
    `(line, message)` for braces that do not balance (a stray `}`, or a block still open at the end)."""

    def __init__(self, text: str):
        self.text = text
        self.nodes: List[Node] = []
        self.problems: List[Tuple[int, str]] = []

    def walk(self):
        """Every node, parents before children, in source order."""
        stack = list(reversed(self.nodes))
        while stack:
            node = stack.pop()
            yield node
            stack.extend(reversed(node.children))

    def rules(self) -> List["Rule"]:
        return [Rule(n.prelude, list(n.decls), n.body, n.line, n.brace_line, list(n.at), n.nested, list(n.decl_lines))
                for n in self.walk() if n.kind == "rule"]


class Rule(NamedTuple):
    prelude: str            # selector text, stripped
    decls: List[str]        # own declarations
    body: str               # raw text between the braces (nested blocks included)
    line: int               # line the selector starts on
    brace_line: int         # line of the `{` (the line of the first declaration when the brace is on its own line)
    at: List[str]           # enclosing at-rule preludes, outermost first
    nested: bool            # the body holds a nested block
    decl_lines: List[int]   # the line each item of `decls` starts on


def parse(css: str, *, fragment: bool = False) -> Sheet:
    """Parse `css` (comments are stripped here) into a `Sheet`.

    `fragment=True` reads CSS that sits inside another language's text (a Rust raw string, an HTML `<style>`, or the
    Rust source around it): the quote characters there are the host's, not CSS's, so strings, `url(...)`, escapes and
    parentheses are plain characters and the structure is braces and `;` only, the way a flat brace scan reads it."""
    text = strip_comments(css, strings=not fragment)
    sheet = Sheet(text)
    stack: List[Node] = []
    buf_start = 0           # start of the text before the next `{`, `;` or `}`
    paren = 0
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if not fragment:
            if c == "\\" and i + 1 < n:
                i += 2
                continue
            if c in "\"'uU":
                j = _atom_end(text, i)
                if j is not None:
                    i = j
                    continue
            if c == "(":
                paren += 1
            elif c == ")":
                paren = max(0, paren - 1)
        if paren:
            pass
        elif c == "{":
            raw = text[buf_start:i]
            prelude = raw.strip()
            lead = len(raw) - len(raw.lstrip())
            at = tuple(s.prelude for s in stack if s.kind == "at")
            node = Node("at" if prelude.startswith("@") else "rule", prelude, line_of(text, buf_start + lead),
                        line_of(text, i), i + 1, at, stack[-1] if stack else None, text)
            (stack[-1].children if stack else sheet.nodes).append(node)
            stack.append(node)
            buf_start = i + 1
        elif c == ";":
            if stack:
                _add_decl(stack[-1], text, buf_start, i)
            buf_start = i + 1
        elif c == "}":
            if not stack:
                sheet.problems.append((line_of(text, i), "a `}` with no open block"))
            else:
                top = stack.pop()
                _add_decl(top, text, buf_start, i)
                top.body_end = i
            buf_start = i + 1
        i += 1
    for node in stack:
        node.body_end = n
        sheet.problems.append((node.brace_line, f"the block `{node.prelude[:60]}` is never closed"))
    return sheet


def _add_decl(node: Node, text: str, start: int, end: int) -> None:
    raw = text[start:end]
    decl = raw.strip()
    if decl:
        node.decls.append(decl)
        node.decl_lines.append(line_of(text, start + len(raw) - len(raw.lstrip())))


def rules(css: str, *, fragment: bool = False) -> List[Rule]:
    """Every style rule of `css`, at-rule bodies descended into, nested rules included, in source order.
    `fragment=True` is for CSS inside another language's text, see `parse`."""
    return parse(css, fragment=fragment).rules()
