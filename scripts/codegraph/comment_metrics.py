# Per-file comment metrics for the code graph (row 486), from the repo's one comment tokenizer.
# Imported by build_comments.py through importlib (a script run as `python3 -I` has no sibling imports); stdlib only.
#
# measure(repo, paths) -> {path: (counts, detail, items)}
#   counts  CM_FIELDS order, raw counts; the page sums them over files, so a component or crate ratio is the ratio of
#           the sums, never an average of ratios.
#   detail  {"und": [[line, name]], "todo": [[line, word]], "hist": [[line, reason]], "stale": [[line, name]],
#            "dup": [[first, last, nfiles, [other files]]]}, only the non-empty keys.
#   items   Rust items of the file, one row each (ITEM_FIELDS order): the same counts over the item's lexical span (its
#           `///` block through its closing brace or semicolon). A nested item is counted in its parent as well.
# A Rust file gets every field. Any other file the comment scanner reads (TS, CSS, shell, ...) gets only `todo` and
# `hist`: its code lines are not tokenized here, so a ratio over it would be meaningless.
#   M1-M5  comment ratio, doc share, pub doc coverage, history hits, TODO markers
#   N1     stale: a backticked snake_case or PascalCase identifier in a comment that appears in no code of the repo and
#          is not cited as another library's name (a candidate, not an error)
#   N2     jscode / js: code lines and comment lines of the JS or CSS held in Rust strings
#   N3     words: whitespace-separated words in comment lines
#   N4     dup: comment lines inside a run of 3+ lines whose normalised text also occurs in another file
#   N6     items
#   N5     line_kinds(): which lines are comment and which are code, for build_blame.py (ages come from git blame)
import bisect, importlib.util, os, re

CM_FIELDS = ("code", "com", "doc", "pubs", "doced", "fields", "fdoced", "todo", "hist", "js", "jscode", "words", "stale", "dup")
ITEM_KINDS = ("fn", "struct", "enum", "trait", "union", "type", "static", "mod", "const", "impl", "macro")
ITEM_FIELDS = ("kind", "name", "start", "end", "pub", "documented", "code", "com", "doc", "todo", "hist", "words")
_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("comment_scan", os.path.join(_HERE, "..", "comment-scan.py"))
cs = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cs)

_PUB_ITEM = re.compile(r"^\s*pub\s+(?:(?:async|unsafe|const)\s+)*(fn|struct|enum|trait|const|static|type|mod)\s+(\w+)")
_FIELD = re.compile(r"^\s*pub\s+(\w+)\s*:")
_ANY_ITEM = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:(?:default|async|unsafe|const|extern(?:\s+\"[^\"]*\")?)\s+)*"
    r"(?:(fn|struct|enum|trait|union|type|static|mod|const)\s+([A-Za-z_]\w*)|(impl)\b(.*)|macro_rules!\s*([A-Za-z_]\w*))")
_MARK = re.compile(r"\b(TODO|FIXME|HACK|XXX)\b")
_TEST_PATH = re.compile(r"(^|/)tests?/|(^|/)tests?\.rs$|_tests?\.rs$|\.spec\.ts$|/ts-test/")
_TICKED = re.compile(r"`([^`\n]{2,60})`")
_CODEISH = re.compile(r"^[A-Za-z_][\w:<>.()!&*,\[\]]*$")
_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_STALE_MIN = 3
# N1 candidates are this repo's own names. A backticked name is not a candidate when a rule says it is not one of them:
#   R1  not name-shaped: this repo's names are snake_case with an underscore, PascalCase with two humps or SCREAMING_SNAKE with
#       an underscore, so a lone word (`rustc`, `signum`, `TAB`) is prose or a keyword, and a lowerCamelCase name is JS
#   R2  cited source: the paragraph (consecutive non-blank comment lines) cites another source, so the names in it belong to
#       that source: a URL, "Forked/Adapted from", "<name> <version>" or "<name>-<version>", a library or framework the
#       comment names (_UPSTREAM), a ticked external crate of Cargo.lock (`dioxus-ssr`, `manganis-core::x`), a ticked file
#       the repo does not have (`scheduler.rs`, `stack.c`), or a compiler or lint tool (`clippy`, `rustc`)
#   R3  Web platform naming: PascalCase ending in Error, Exception, Event or Data (`InvalidStateError`, `SelectionData`)
#   R4  a path into std, core, alloc, a primitive type or a tool namespace (`f64::signum`, `clippy::approx_constant`)
#   R5  placeholders and patterns: `xxx`, a lone-capital prefix (`XProps`), a `*` wildcard (`MenuSub*`)
#   R6  a naming-conversion example: the same line holds the name's kebab or snake spelling (`scroll-lock` -> `ScrollLock`)
_UPSTREAM = re.compile(r"Base UI|Radix|radix-ui|Sonner|shadcn|React|react-[a-z-]+|upstream|MUI|Floating UI|Headless|Recharts|Vaul|cmdk|[Ee]mbla|"
                       r"@[a-z][\w-]*/|\.(?:mjs|tsx|jsx|ts|js)\b|\bwindow\.|\bdocument\.|\bMath\.|\bW3C\b|\bAPG\b|\bMDN\b|"
                       r"https?://|[Ff]ork(?:ed)?\b|[Aa]dapted\b|\bclippy\b|\brustc\b|\brustfmt\b|[\w-]+[ -]v?\d+\.\d+(?:\.\d+)?\b")
_CAMEL = re.compile(r"^[a-z][a-z0-9]*[A-Z]")
_PASCAL = re.compile(r"^(?:[A-Z][a-z0-9]+){2,}$")
_SNAKE = re.compile(r"^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$")
_SCREAM = re.compile(r"^[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)+$")
_WEB = re.compile(r"^[A-Z]\w*(?:Error|Exception|Event|Data)$")
_STD_HEAD = {"std", "core", "alloc", "clippy", "rustc", "f32", "f64", "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64",
             "isize", "str", "bool", "char", "String", "Vec", "Option", "Result"}
_TICK_FILE = re.compile(r"`(?:[\w./-]*/)?([\w-]+\.(?:rs|c|h|ts|js|mjs|css|toml|sh))`")
_TOKEN_EXTS = (".sh", ".py", ".toml", ".json", ".yml", ".yaml", ".cjs")
_ext_crates, _repo_files = set(), set()


def _lock_externals(repo):
    """Cargo.lock packages that come from a registry or git (the workspace's own crates have no `source`), with `-` as `_`."""
    out = set()
    try:
        with open(os.path.join(repo, "Cargo.lock"), encoding="utf-8") as fh:
            for block in fh.read().split("\n\n"):
                m = re.search(r'^name = "([^"]+)"', block, re.M)
                if m and re.search(r"^source = ", block, re.M):
                    out.add(m.group(1).replace("-", "_"))
    except OSError:
        pass
    return out


def _cites_source(para):
    """R2: does this paragraph cite a source other than this repo?"""
    if _UPSTREAM.search(para):
        return True
    for t in _TICKED.finditer(para):
        c = t.group(1).strip()
        head = re.split(r"::|'s$", c)[0].replace("-", "_")
        head = re.sub(r"_\d[\d.]*$", "", head)
        if head in _ext_crates:
            return True
    if any(w.replace("-", "_") in _ext_crates for w in re.findall(r"\b[a-z][a-z0-9]*(?:-[a-z0-9]+)+\b", para)):
        return True  # a hyphenated crate name from Cargo.lock, ticked or not (dioxus-core)
    return any(m.group(1) not in _repo_files for m in _TICK_FILE.finditer(para))


def _cited_lines(text):
    """The comment lines that sit in a paragraph (consecutive non-blank comment lines, across comment nodes) citing a source."""
    body = {ln: b for kind, _i, _j, p in cs.walk_rust(text) if kind == "comment" for ln, b in p}
    out, cur = set(), []
    for ln in sorted(body) + [-9]:
        if cur and (ln != cur[-1] + 1 or not body.get(ln, "").strip()):
            if _cites_source(" ".join(body[k] for k in cur)):
                out.update(cur)
            cur = []
        if ln > 0 and body[ln].strip():
            cur.append(ln)
    return out


def _repo_style(ident, body):
    """R1, R3 and R6 for one identifier of a ticked text found in the comment line `body`."""
    if not (_SNAKE.match(ident) or _SCREAM.match(ident) or _PASCAL.match(ident)) or _WEB.match(ident):
        return False
    spelled = re.sub(r"[^a-z]", "", ident.lower())
    # R6: the kebab spelling of the same name on the line (`scroll-lock` -> `ScrollLock`)
    return not any(w.replace("-", "") == spelled for w in re.findall(r"[a-z]+(?:-[a-z]+)+", body.lower()))


_DUP_ALPHA = 45


def is_test_path(path):
    return bool(_TEST_PATH.search(path))


def scannable_paths(repo):
    """The files the history gate reads (tracked, minus its skip lists); the metrics use the same universe."""
    return set(cs.tracked_files(repo))


def _doc_top(k, lines, code_lines, doc_lines):
    """The first line of the `///` block directly above the item on line k (1-based), attributes and `//` notes skipped,
    or 0 when there is none."""
    q, top = k - 1, 0
    while q >= 1:
        if q in doc_lines:
            top = q
            q -= 1
            continue
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
            break
    return top


def _norm(body):
    b = re.sub(r"`[^`]*`", "x", body.lower())
    return " ".join(re.sub(r"[^a-z0 ]+", " ", re.sub(r"\d+", "0", b)).split())


def _prefix(arr):
    """p[i] = sum of arr over lines 1..i (arr is indexed by line, slot 0 unused), so a span a..b is p[b] - p[a - 1]."""
    out, acc = [0], 0
    for v in arr[1:]:
        acc += v
        out.append(acc)
    return out


def _items(text_b, lines, code_lines, doc, pre):
    """Item rows of one Rust file. `text_b` is the source with comments and strings blanked (braces match there)."""
    starts = [0]
    for m in re.finditer("\n", text_b):
        starts.append(m.end())
    match, stack = {}, []
    for m in re.finditer(r"[{}]", text_b):
        if m.group() == "{":
            stack.append(m.start())
        elif stack:
            match[stack.pop()] = m.start()
    n = len(lines)
    rows = []
    for k, ln in enumerate(text_b.split("\n"), 1):
        m = _ANY_ITEM.match(ln)
        if not m:
            continue
        if m.group(1):
            kind, name, hdr_end = m.group(1), m.group(2), m.end()
        elif m.group(3):
            kind, hdr_end = "impl", m.end(3)
            name = re.split(r"\bwhere\b|\{", m.group(4))[0].strip()[:48] or "impl"
        else:
            kind, name, hdr_end = "macro", m.group(5), m.end()
        pos = starts[k - 1] + hdr_end
        depth, end_off = 0, None
        value_item = kind in ("const", "static", "type")
        for t in re.finditer(r"[{;()\[\]}]", text_b[pos:pos + 200000]):
            c, at = t.group(), pos + t.start()
            if c in "([":
                depth += 1
            elif c in ")]":
                depth -= 1
            elif c == "{":
                if value_item or depth > 0:
                    depth += 1
                else:
                    end_off = match.get(at, len(text_b) - 1)
                    break
            elif c == "}":
                depth -= 1
            elif c == ";" and depth <= 0:
                end_off = at
                break
        if end_off is None:
            continue
        end = min(n, bisect.bisect_right(starts, end_off))
        top = _doc_top(k, lines, code_lines, doc)
        a = top or k
        code, com, docl, todo, hist, words = (p[end] - p[a - 1] for p in pre)
        rows.append([ITEM_KINDS.index(kind), name, a, end, 1 if re.match(r"\s*pub\b", ln) else 0, 1 if top else 0,
                     code, com, docl, todo, hist, words])
    return rows


def _rust(text):
    lines = text.split("\n")
    n = len(lines)
    code_lines = cs.code_of(text, keep_strings=True).split("\n")
    com, doc, hist, js, jscode, words, todo = set(), set(), [], 0, 0, 0, []
    wline, tline, hline = [0] * (n + 2), [0] * (n + 2), [0] * (n + 2)
    cands, norm = [], []
    cited = _cited_lines(text)
    for kind, i, _j, p in cs.walk_rust(text):
        if kind == "comment":
            is_doc = text.startswith("///", i) or text.startswith("//!", i)
            for ln, body in p:
                com.add(ln)
                if is_doc:
                    doc.add(ln)
                w = len(body.split())
                words += w
                wline[ln] += w
                m = _MARK.search(body)
                if m:
                    todo.append([ln, m.group(1)])
                    tline[ln] += 1
                why = cs.history_reason(body)
                if why:
                    hist.append([ln, why])
                    hline[ln] += 1
                norm.append((ln, _norm(body)))
                if ln in cited:
                    continue
                for t in _TICKED.finditer(body):
                    c = t.group(1).strip()
                    if _CODEISH.match(c) and "*" not in c and "xxx" not in c and c.split("::")[0] not in _STD_HEAD:
                        cands.append((ln, c, body))
        elif len(p[3]) > 8:  # JS or CSS held in a Rust string: counted for the gate's history rule, apart from M1
            if cs.embedded_kind(p[3]):
                clines = set()
                for ln, body in cs.embedded_comments(p[3], p[4]):
                    js += 1
                    clines.add(ln)
                    why = cs.history_reason(body)
                    if why:
                        hist.append([ln, why])
                        hline[ln] += 1
                live = {p[4][k] for k, ch in enumerate(p[3]) if k < len(p[4]) and not ch.isspace()}
                jscode += len(live - clines)
    pubs = doced = fields = fdoced = 0
    und = []
    for k, ln in enumerate(code_lines, 1):
        m = _PUB_ITEM.match(ln)
        f = None if m else _FIELD.match(ln)
        if not (m or f):
            continue
        if m and m.group(1) == "mod" and ln.rstrip().endswith(";"):
            continue  # `pub mod x;` is documented by the `//!` header of x itself
        has = bool(_doc_top(k, lines, code_lines, doc))
        if m:
            pubs += 1
            doced += has
        else:
            fields += 1
            fdoced += has
        if not has:
            und.append([k, (m or f).group(2 if m else 1)])
    cline, mline, dline = [0] * (n + 2), [0] * (n + 2), [0] * (n + 2)
    for k, c in enumerate(code_lines, 1):
        if c.strip():
            cline[k] = 1
    for ln in com:
        if ln <= n:
            mline[ln] = 1
    for ln in doc:
        if ln <= n:
            dline[ln] = 1
    pre = tuple(_prefix(a[:n + 1]) for a in (cline, mline, dline, tline, hline, wline))
    items = _items(cs.code_of(text), lines, code_lines, doc, pre)
    counts = [sum(cline), len(com), len(doc), pubs, doced, fields, fdoced, len(todo), len(hist), js, jscode, words, 0, 0]
    detail = {k: v for k, v in (("und", und), ("todo", todo), ("hist", sorted(hist))) if v}
    return counts, detail, items, {"cands": cands, "norm": norm, "tokens": set(_IDENT.findall("\n".join(code_lines)))}


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
    counts = [0] * len(CM_FIELDS)
    counts[CM_FIELDS.index("todo")], counts[CM_FIELDS.index("hist")] = len(todo), len(hist)
    detail = {k: v for k, v in (("todo", todo), ("hist", hist)) if v}
    return counts, detail, [], {"tokens": set(_IDENT.findall(text))}


def line_kinds(path, text):
    """(comment line numbers, code line numbers) of one file, for the comment-age metric (N5)."""
    if path.endswith(".rs"):
        com = {ln for kind, _i, _j, p in cs.walk_rust(text) if kind == "comment" for ln, _b in p}
        code = {k for k, c in enumerate(cs.code_of(text, keep_strings=True).split("\n"), 1) if c.strip()}
        return com, code
    com = {ln for ln, _b in cs.EXTRACTORS[cs.EXT_KIND[os.path.splitext(path)[1]]](text)}
    code = {k for k, c in enumerate(text.split("\n"), 1) if c.strip() and k not in com}
    return com, code


NODE_EXTS = (".rs", ".ts", ".css", ".html", ".js", ".mjs", ".txt")


def node_paths(repo):
    """Every scannable file a graph node can be: the Rust, TS, CSS, HTML, JS and TXT files the comment scanner reads."""
    return sorted(p for p in scannable_paths(repo) if p.endswith(NODE_EXTS))


def _stale(cands, tokens):
    out = []
    for ln, c, body in cands:
        for ident in _IDENT.findall(c):
            if len(ident) >= _STALE_MIN and ident not in tokens and not ident.endswith("_") and _repo_style(ident, body):
                out.append([ln, ident])
                break
    return out


def _duplicates(norms):
    """N4: {path: [[first, last, nfiles, [others]]]} for runs of 3+ comment lines whose normalised text is in 2+ files."""
    seen = {}
    for path, rows in norms.items():
        run = []
        for ln, nb in rows + [(-9, "")]:
            if not nb:
                run = []
                continue
            run = run + [(ln, nb)] if run and ln == run[-1][0] + 1 else [(ln, nb)]
            if len(run) >= 3:
                w = run[-3:]
                if sum(len(x[1].replace(" ", "")) for x in w) >= _DUP_ALPHA:
                    seen.setdefault(" | ".join(x[1] for x in w), []).append((path, w[0][0]))
    out = {}
    for hits in seen.values():
        files = sorted({h[0] for h in hits})
        if len(files) < 2:
            continue
        for path, start in hits:
            out.setdefault(path, []).append([start, start + 2, len(files), [f for f in files if f != path][:3]])
    merged = {}
    for path, ranges in out.items():
        cur, res = None, []
        for r in sorted(ranges):
            if cur and r[0] <= cur[1] + 1:
                cur[1] = max(cur[1], r[1])
                cur[2] = max(cur[2], r[2])
                cur[3] = sorted(set(cur[3]) | set(r[3]))[:3]
            else:
                cur = list(r)
                res.append(cur)
        merged[path] = res
    return merged


def measure(repo, paths):
    global _ext_crates, _repo_files
    scan = scannable_paths(repo)
    _ext_crates = _lock_externals(repo)
    _repo_files = {os.path.basename(q) for q in scan}
    out, tokens, norms, cands = {}, set(), {}, {}
    for p in paths:
        if p not in scan:
            continue
        with open(os.path.join(repo, p), encoding="utf-8", errors="replace") as fh:
            text = fh.read()
        counts, detail, items, extra = _rust(text) if p.endswith(".rs") else _other(p, text)
        tokens |= extra["tokens"]
        if "norm" in extra:
            norms[p], cands[p] = extra["norm"], extra["cands"]
        out[p] = [counts, detail, items]
    for p in scan:  # a name defined in a script or a config file is not stale either
        if p.endswith(_TOKEN_EXTS) and p not in out:
            with open(os.path.join(repo, p), encoding="utf-8", errors="replace") as fh:
                tokens |= set(_IDENT.findall(fh.read()))
    dups = _duplicates(norms)
    si, di = CM_FIELDS.index("stale"), CM_FIELDS.index("dup")
    for p, (counts, detail, _items_) in out.items():
        if p in cands:
            stale = _stale(cands[p], tokens)
            if stale:
                detail["stale"] = stale
                counts[si] = len(stale)
        if p in dups:
            detail["dup"] = dups[p]
            covered = set()
            for a, b, _n, _o in dups[p]:
                covered.update(range(a, b + 1))
            counts[di] = len(covered)
    return {p: (tuple(c), d, i) for p, (c, d, i) in out.items()}
