"""Which gates of scripts/run-gates.sh read Rust source, and how.

Method: take every `name|scripts/check-*.sh` entry of the gate list; read the script plus any
scripts/*.py or scripts/lib/*.py it names (one level). A gate READS RUST when that text names a
`.rs` path or glob, `--type rust`/`-t rust`/`-trust`, `type=rust`, or `*.rs`. Its METHOD is
`syn` when it runs the attr-spread-collision-checker member or the public-api snapshot, else
`text` (regex: rg, grep, python re). Python-in-heredoc is `<<` plus `python3` in the .sh.
Usage: python3 -I gates_rust.py REPO
"""
import os
import re
import sys

repo = sys.argv[1]
gl = open(os.path.join(repo, "scripts/run-gates.sh")).read()
entries = re.findall(r'"([\w-]+)\|(scripts/check-[\w-]+\.sh)"', gl)
RUST = re.compile(r"\.rs\b|--type[ =]rust|-t ?rust|type=rust|\*\.rs|'rs'|\"rs\"")
SYN = re.compile(r"attr-spread-collision-checker|public-api-snapshot|attr_spread_collision_checker")
HEREDOC_PY = re.compile(r"python3[^\n]*<<|<<-?\s*'?(PY|EOF|PYTHON)'?[^\n]*\n")
read_rust, syn_based, text_based, heredoc = [], [], [], []
for name, path in entries:
    full = os.path.join(repo, path)
    if not os.path.isfile(full):
        continue
    txt = open(full, encoding="utf-8", errors="replace").read()
    body = txt
    for helper in set(re.findall(r"scripts/(?:lib/)?[\w-]+\.py", txt)):
        hp = os.path.join(repo, helper)
        if os.path.isfile(hp):
            body += open(hp, encoding="utf-8", errors="replace").read()
    if "python3" in txt and "<<" in txt:
        heredoc.append(name)
    if RUST.search(body):
        read_rust.append(name)
        (syn_based if SYN.search(body) else text_based).append(name)
print("gate entries that are check scripts:", len(entries))
print("of them embed python in a heredoc:", len(heredoc))
print("read Rust source:", len(read_rust))
print("  through the syn member:", len(syn_based), syn_based)
print("  as text patterns:", len(text_based))
for n in text_based:
    print("    ", n)

# Second, tighter cut: of the text-pattern gates, those whose text holds a regular expression or
# match string over Rust syntax (a hook or rsx form, `fn`, an attribute, a signal method, a closure).
SHAPE = re.compile(r"rsx|use_effect|use_memo|use_hook|use_\\w|\\bfn\\b|fn\\s|#\\\[|#\[(?:cfg|component|test)|"
                   r"Signal|move\s*\|\||\.set\(|\.read\(|onkey|addEventListener|document::eval|spawn\(")
shape = []
for name, path in entries:
    if name not in text_based:
        continue
    full = os.path.join(repo, path)
    body = open(full, encoding="utf-8", errors="replace").read()
    for helper in set(re.findall(r"scripts/(?:lib/)?[\w-]+\.py", body)):
        hp = os.path.join(repo, helper)
        if os.path.isfile(hp):
            body += open(hp, encoding="utf-8", errors="replace").read()
    if SHAPE.search(body):
        shape.append(name)
print("text-pattern gates whose patterns name Rust syntax (Rust-shape):", len(shape))
print("   ", " ".join(shape))
print("text-pattern gates reading .rs for other facts (paths, strings, CSS classes, docs):", len(text_based) - len(shape))
print("   ", " ".join(n for n in text_based if n not in shape))
