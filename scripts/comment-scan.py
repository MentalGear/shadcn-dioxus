#!/usr/bin/env python3
"""comment-scan.py -- the ONE comment extractor behind two gates (backlog row 266 (a)).

    scripts/check-history-comments.sh   per-file ratchet on comment lines that retell history
    scripts/check-comment-refs.sh       every `row N` and every repo path cited in a comment resolves

Both are thin bash wrappers over this file (`python3 -I scripts/comment-scan.py <history|refs> ...`), so what
counts as "a comment" is decided in exactly one place: Rust (`//`, `///`, `//!`, nested `/* */`), CSS (`/* */`),
TS/JS (`//`, `/* */`) and shell (`#`, not in a string, not `$#`/`${#x}`, heredoc bodies read per line). A comment
marker inside a string literal is not a comment. Not covered (known gap, ratchet-neutral): CSS or JS that lives
inside a Rust string literal, and Python files.

Scanned: `git ls-files` with the extensions above, minus SKIP_PREFIXES (dev-docs/ is where history belongs;
docs/ is the deployed site; the APG reference tree is vendored; generated primitives JS), the two gates'
own sources, lock files.

Exit status: 0 clean, 1 violation, 2 bad usage.
"""
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

EXT_KIND = {
    ".rs": "rust",
    ".css": "css",
    ".ts": "js", ".tsx": "js", ".js": "js", ".mjs": "js", ".cjs": "js",
    ".sh": "shell",
}
SKIP_PREFIXES = (
    "dev-docs/",
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


_HEREDOC = re.compile(r"<<-?\s*(['\"]?)([A-Za-z_][A-Za-z_0-9]*)\1")


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


def extract_shell(text):
    in_quote, heredoc = None, None
    for lineno, ln in enumerate(text.split("\n"), 1):
        if heredoc:
            if ln.strip() == heredoc:
                heredoc = None
                continue
            body, _ = _shell_line_comment(ln, None)  # a heredoc body is some other language: per-line, no state
        else:
            if lineno == 1 and ln.startswith("#!"):
                continue
            body, in_quote = _shell_line_comment(ln, in_quote)
            m = _HEREDOC.search(ln if body is None else ln[:ln.rfind("#" + body)])
            if m and not in_quote:
                heredoc = m.group(2)
        if body is not None:
            yield lineno, body


EXTRACTORS = {"rust": extract_rust, "css": extract_css, "js": extract_js, "shell": extract_shell}


def comments_of(root, path):
    with open(os.path.join(root, path), encoding="utf-8", errors="replace") as fh:
        text = fh.read()
    return list(EXTRACTORS[EXT_KIND[os.path.splitext(path)[1]]](text))


# ----------------------------------------------------------------------------- history detection
# The repo's first commit is 2026-08-29, so only a date from 2026-08 on is a retelling; a 2024 or 2026-05 date in a
# comment is test data (a calendar fixture), not history.
DATE = re.compile(r"(?<![\w.-])(?:2026-(?:0[89]|1[0-2])|20(?:2[7-9]|[3-9]\d)-(?:0[1-9]|1[0-2]))-(?:0[1-9]|[12]\d|3[01])(?![\w-])")
HEX = re.compile(r"(?<![\w#.\-/$@])(?:[0-9a-f]{7,40})(?![\w\-/])")
PHRASES = [
    (re.compile(r"\bbefore the (?:fix|change|rewrite|refactor|migration)\b", re.I), "before the fix"),
    (re.compile(r"\bformerly\b", re.I), "formerly"),
    (re.compile(r"\bold (?:behaviou?r|implementation|approach|name)\b", re.I), "old behaviour"),
    (re.compile(r"\b(?:was|were|got|has been|have been) (?:changed|renamed|rewritten)\b", re.I), "was changed"),
    (re.compile(r"\bwe (?:now|used to|no longer|previously)\b", re.I), "we now"),
    (re.compile(r"\bnow (?:uses|reads|writes|goes|lives|comes|calls|returns|lets|owns|runs|derives)\b", re.I), "now uses"),
    (re.compile(r"\bno longer (?:uses|reads|writes|carries|ships|sets|renders|needs|does|do|owns|"
                r"holds|lives|calls|emits|paints|draws|returns|takes)", re.I), "no longer"),
    (re.compile(r"\bsince (?:been )?(?:removed|replaced|renamed|dropped|fixed)\b", re.I), "was changed"),
]
USED_TO = re.compile(r"(?:\b(\w+)\s+)?\bused to\b", re.I)
# "is used to X", "can be used to X", "used to" after these is the passive/instrument sense, not history.
PASSIVE_PREV = {"is", "are", "be", "been", "being", "was", "were", "get", "gets", "got", "it's", "also", "only",
                "then", "often", "just", "can", "could", "may", "might", "must", "should", "will", "would",
                "not", "if", "when", "that", "which", "and", "or", "both", "either", "so", "thus", "than"}
PREVIOUSLY = re.compile(r"\bpreviously\b(?!-)(?:\s+(\w+))?", re.I)
# Upstream revisions the repo pins on purpose (provenance, not history): a comment naming one stays unflagged.
PINNED_REVS = ("7e4034b262bc0d25332e330d8a582aaf34113829",  # w3c/aria-practices, playwright/oracle/reference/
               "295a1f114a138f23b5dfee0e0c6812394dfeb90c")  # shadcn-ui/ui, the style-nova reference
# "the previously focused element" is a state word, not history; history reads "previously, X" / "previously was".
PREVIOUSLY_STATE_OK = re.compile(r"(?:ed|en|n|t|d)$|^(?:set|read|run|put|shown|sent|hidden|open|active|visible)$")


def history_reason(body):
    """Why a comment line retells history, or None. One reason per line is enough: the unit is the line."""
    if "history-ok:" in body:
        return None
    if DATE.search(body):
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
            return why
    for m in USED_TO.finditer(body):
        if (m.group(1) or "").lower() not in PASSIVE_PREV:
            return "used to"
    for m in PREVIOUSLY.finditer(body):
        if re.search(r"\b(?:whatever|what|that|which)\s+(?:was|were|is)\s+$", body[:m.start()], re.I):
            continue
        nxt = (m.group(1) or "").lower()
        if not nxt or not PREVIOUSLY_STATE_OK.search(nxt):
            return "previously"
        # "previously set/..ed" read as a state word unless the sentence is clearly a story
        if re.search(r"\b(?:was|were|had|did) previously\b|previously,", body, re.I):
            return "previously"
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
PLACEHOLDER = re.compile(r"[<>{}*$|]|NNN|XXX|\.\.\.|…|<name>|\bN\b")
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
        ("skipped dir is ignored", {"dev-docs/x.rs": "// 2026-10-19 used to\n", "docs/y.js": "// 2026-10-19\n"}, [], 0),
    ]
    fails = 0
    for name, files, bl, want in cases:
        rc, out = _run_in_fixture("history", files, bl)
        ok = rc == want
        fails += not ok
        print(f"  {'ok  ' if ok else 'FAIL'} {name} (rc {rc}, want {want})")
        if not ok:
            print("    " + out.replace("\n", "\n    "))
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
