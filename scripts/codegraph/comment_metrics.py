# Per-file comment metrics M1-M5 for the code graph (row 486), from the repo's one comment tokenizer.
# Imported by build_graph.py through importlib (a script run as `python3 -I` has no sibling imports); stdlib only.
#
# measure(repo, paths) -> {path: (counts, detail)}
#   counts  CM_FIELDS order, raw counts; the page sums them over files, so a component or crate ratio is the ratio of
#           the sums, never an average of ratios.
#   detail  {"und": [[line, name]], "todo": [[line, word]], "hist": [[line, reason]]}, only the non-empty keys.
# A Rust file gets every field. Any other file the comment scanner reads (TS, CSS, shell, ...) gets only `todo` and
# `hist`: its code lines are not tokenized here, so a ratio over it would be meaningless.
import importlib.util, os, re

CM_FIELDS = ("code", "com", "doc", "pubs", "doced", "fields", "fdoced", "todo", "hist", "js")
_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("comment_scan", os.path.join(_HERE, "..", "comment-scan.py"))
cs = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cs)

_ITEM = re.compile(r"^\s*pub\s+(?:(?:async|unsafe|const)\s+)*(fn|struct|enum|trait|const|static|type|mod)\s+(\w+)")
_FIELD = re.compile(r"^\s*pub\s+(\w+)\s*:")
_MARK = re.compile(r"\b(TODO|FIXME|HACK|XXX)\b")
_TEST_PATH = re.compile(r"(^|/)tests?/|(^|/)tests?\.rs$|_tests?\.rs$|\.spec\.ts$|/ts-test/")


def is_test_path(path):
    return bool(_TEST_PATH.search(path))


def scannable_paths(repo):
    """The files the history gate reads (tracked, minus its skip lists); the metrics use the same universe."""
    return set(cs.tracked_files(repo))


def _documented(k, lines, code_lines, doc_lines):
    """Is the item on line k (1-based) preceded by a `///` block, attributes and `//` notes skipped?"""
    q = k - 1
    while q >= 1:
        if q in doc_lines:
            return True
        s = code_lines[q - 1].strip()
        if s.startswith("#[") or s.startswith("#!["):
            q -= 1
        elif s.endswith("]") or s.endswith(")]"):  # the tail of a multi-line attribute: climb to its `#[`
            while q > 1 and not code_lines[q - 1].strip().startswith("#["):
                q -= 1
            q -= 1
        elif s == "" and lines[q - 1].strip().startswith("//"):  # a `//` note line between doc and item
            q -= 1
        else:
            return False
    return False


def _rust(text):
    lines = text.split("\n")
    code_lines = cs.code_of(text, keep_strings=True).split("\n")
    com, doc, hist, js = set(), set(), [], 0
    todo = []
    for kind, i, _j, p in cs.walk_rust(text):
        if kind == "comment":
            is_doc = text.startswith("///", i) or text.startswith("//!", i)
            for ln, body in p:
                com.add(ln)
                if is_doc:
                    doc.add(ln)
                m = _MARK.search(body)
                if m:
                    todo.append([ln, m.group(1)])
                why = cs.history_reason(body)
                if why:
                    hist.append([ln, why])
        elif len(p[3]) > 8:  # JS or CSS held in a Rust string: counted for the gate's history rule, apart from M1
            for ln, body in cs.embedded_comments(p[3], p[4]):
                js += 1
                why = cs.history_reason(body)
                if why:
                    hist.append([ln, why])
    pubs = doced = fields = fdoced = 0
    und = []
    for k, ln in enumerate(code_lines, 1):
        m = _ITEM.match(ln)
        f = None if m else _FIELD.match(ln)
        if not (m or f):
            continue
        if m and m.group(1) == "mod" and ln.rstrip().endswith(";"):
            continue  # `pub mod x;` is documented by the `//!` header of x itself
        has = _documented(k, lines, code_lines, doc)
        if m:
            pubs += 1
            doced += has
        else:
            fields += 1
            fdoced += has
        if not has:
            und.append([k, (m or f).group(2 if m else 1)])
    code = sum(1 for c in code_lines if c.strip())
    counts = (code, len(com), len(doc), pubs, doced, fields, fdoced, len(todo), len(hist), js)
    detail = {k: v for k, v in (("und", und), ("todo", todo), ("hist", sorted(hist))) if v}
    return counts, detail


def _other(path, text):
    ext = os.path.splitext(path)[1]
    todo, hist = [], []
    for ln, body in cs.EXTRACTORS[cs.EXT_KIND[ext]](text):
        m = _MARK.search(body)
        if m:
            todo.append([ln, m.group(1)])
        why = cs.history_reason(body)
        if why:
            hist.append([ln, why])
    counts = (0, 0, 0, 0, 0, 0, 0, len(todo), len(hist), 0)
    detail = {k: v for k, v in (("todo", todo), ("hist", hist)) if v}
    return counts, detail


def measure(repo, paths):
    scan = scannable_paths(repo)
    out = {}
    for p in paths:
        if p not in scan:
            continue
        with open(os.path.join(repo, p), encoding="utf-8", errors="replace") as fh:
            text = fh.read()
        out[p] = _rust(text) if p.endswith(".rs") else _other(p, text)
    return out
