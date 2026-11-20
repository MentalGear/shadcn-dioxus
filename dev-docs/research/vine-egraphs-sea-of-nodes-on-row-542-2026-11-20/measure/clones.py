"""Token-level clone detection over non-test Rust (the PMD-CPD method, stdlib only).

Method: per file, blank comments; tokenize into identifiers, literals, punctuation. Mode
`exact` keeps identifiers (type-1 clones); mode `renamed` maps every identifier that is not a
Rust keyword to `I` and every literal to `L` (type-2 clones). Test code is cut: files named
tests.rs or *_tests.rs or under a tests/ directory are skipped, and a file is cut at its first
`#[cfg(test)]`. A window of W tokens is hashed at every position; windows that occur at two or
more places (different files, or the same file at least W tokens apart) are clone seeds; seeds
are merged into maximal runs per pair of places. A pair is reported once per maximal run of at
least MIN tokens, with both line ranges. Usage:
  python3 -I clones.py REPO MODE MIN [ROOT ...]
"""
import collections
import os
import re
import sys
import time

KW = set("""as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod
move mut pub ref return self Self static struct super trait true type unsafe use where while""".split())
TOK = re.compile(r"""r#*"|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])'|b'(?:\\.|[^'\\])'|[A-Za-z_][A-Za-z0-9_]*!?|\d[\d_]*(?:\.\d+)?\w*|::|->|=>|==|!=|<=|>=|&&|\|\||[^\sA-Za-z0-9_]""")


def strip_comments(src):
    out, i, n = [], 0, len(src)
    while i < n:
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
        elif src.startswith("/*", i):
            j = src.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(re.sub(r"[^\n]", " ", src[i:j]))
            i = j
        elif src[i] == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            out.append(src[i:j + 1])
            i = j + 1
        else:
            out.append(src[i])
            i += 1
    return "".join(out)


def tokens(path, mode):
    src = open(path, encoding="utf-8", errors="replace").read()
    cut = re.search(r"#\[cfg\(test\)\]", src)
    if cut:
        src = src[:cut.start()]
    src = strip_comments(src)
    toks = []
    for m in TOK.finditer(src):
        t = m.group(0)
        line = src.count("\n", 0, m.start()) + 1
        if mode == "renamed":
            if t[0] in "\"'" or t.startswith("b'") or t[0].isdigit():
                t = "L"
            elif (t[0].isalpha() or t[0] == "_") and not t.endswith("!") and t not in KW:
                t = "I"
        toks.append((t, line))
    return toks


def main():
    repo, mode, minimum = sys.argv[1], sys.argv[2], int(sys.argv[3])
    roots = sys.argv[4:]
    W = 40
    t0 = time.time()
    files = {}
    for root in roots:
        for d, _, fs in os.walk(os.path.join(repo, root)):
            if "/tests" in d or "/target" in d:
                continue
            for f in fs:
                if f.endswith(".rs") and f != "tests.rs" and not f.endswith("_tests.rs"):
                    p = os.path.join(d, f)
                    files[os.path.relpath(p, repo)] = tokens(p, mode)
    index = collections.defaultdict(list)
    for f, toks in files.items():
        seq = [t for t, _ in toks]
        for i in range(len(seq) - W + 1):
            index[hash(tuple(seq[i:i + W]))].append((f, i))
    pairs = collections.defaultdict(set)
    for occ in index.values():
        if len(occ) < 2 or len(occ) > 20:
            continue
        for a in range(len(occ)):
            for b in range(a + 1, len(occ)):
                (fa, ia), (fb, ib) = occ[a], occ[b]
                if fa == fb and abs(ia - ib) < W:
                    continue
                key = (fa, fb, ia - ib) if (fa, ia) < (fb, ib) else (fb, fa, ib - ia)
                pairs[key].add(ia if (fa, ia) < (fb, ib) else ib)
    runs = []
    for (fa, fb, off), starts in pairs.items():
        s = sorted(starts)
        run_start = prev = s[0]
        for x in s[1:] + [None]:
            if x is not None and x == prev + 1:
                prev = x
                continue
            length = prev - run_start + W
            if length >= minimum:
                ta, tb = files[fa], files[fb]
                ia, ib = run_start, run_start - off
                if 0 <= ib and ib + length <= len(tb):
                    runs.append((length, fa, ta[ia][1], ta[ia + length - 1][1], fb, tb[ib][1], tb[ib + length - 1][1]))
            if x is not None:
                run_start = prev = x
    runs.sort(reverse=True)
    nfiles = len(files)
    ntok = sum(len(t) for t in files.values())
    print(f"mode={mode} window={W} min_tokens={minimum} files={nfiles} tokens={ntok} seconds={time.time()-t0:.1f}")
    print(f"clone pairs (maximal runs >= {minimum} tokens): {len(runs)}")
    cross = sum(1 for r in runs if r[1] != r[4])
    print(f"  across two files: {cross}; inside one file: {len(runs)-cross}")
    lines = set()
    for _, fa, a0, a1, fb, b0, b1 in runs:
        lines.update((fa, x) for x in range(a0, a1 + 1))
        lines.update((fb, x) for x in range(b0, b1 + 1))
    print(f"  distinct source lines covered: {len(lines)}")
    for r in runs[:60]:
        print("  %4d tok  %s:%d-%d  ~  %s:%d-%d" % r)


main()
