#!/usr/bin/env python3
"""Every relative markdown link in dev-docs/**/*.md, CLAUDE.md and README.md must resolve to an existing path.

Called by scripts/check-backlog-index.sh (pure filesystem, no network). Ignores fenced code blocks, inline code
spans, http(s)/mailto links and pure #anchors; a `#fragment` is stripped, the file must exist. Exit 1 on a miss.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FILES = sorted((ROOT / "dev-docs").rglob("*.md")) + [ROOT / "CLAUDE.md", ROOT / "README.md"]
LINK = re.compile(r"\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
REFDEF = re.compile(r"(?m)^\s{0,3}\[[^\]]+\]:\s*<?(\S+?)>?\s*$")


def strip_code(text):
    text = re.sub(r"(?ms)^(```|~~~).*?^\1[^\n]*$", "", text)
    return re.sub(r"`[^`\n]*`", "", text)


bad = []
n = 0
for f in FILES:
    if not f.is_file():
        continue
    text = strip_code(f.read_text(encoding="utf-8"))
    for t in LINK.findall(text) + REFDEF.findall(text):
        if re.match(r"[a-z][a-z0-9+.-]*:", t, re.I) or t.startswith("#") or t.startswith("//"):
            continue
        path = re.split(r"[#?]", t, maxsplit=1)[0]
        if not path:
            continue
        n += 1
        dest = (ROOT / path.lstrip("/")) if path.startswith("/") else (f.parent / path)
        if not dest.exists():
            bad.append(f"{f.relative_to(ROOT)}: broken link ({t})")
if bad:
    print("check-doc-links: broken relative links")
    for b in bad:
        print("  - " + b)
    sys.exit(1)
print(f"check-doc-links: OK -- {n} relative links resolve")
