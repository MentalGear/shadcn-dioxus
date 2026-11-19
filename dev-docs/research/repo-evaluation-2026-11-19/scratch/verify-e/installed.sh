cd /home/user/shadcn-dioxus
G=(-g 'preview/src/components/*/**' -g '!preview/src/components/*/variants/**' -g '!preview/src/components/*/component.json' -g '!preview/src/components/*/docs.md' -g '!preview/src/components/top_layer/**' -g '!preview/src/components/mod.rs')
echo "installed files:"; rg --no-config --files "${G[@]}" . | wc -l
echo "installed bytes:"; rg --no-config --files "${G[@]}" . | xargs cat | wc -c
echo "installed lines:"; rg --no-config --files "${G[@]}" . | xargs cat | wc -l
echo "components with files:"; rg --no-config --files "${G[@]}" . | awk -F/ '{print $5}' | sort -u | wc -l
echo "file kinds:"; rg --no-config --files "${G[@]}" . | sed 's/.*\.//' | sort | uniq -c
pat() { echo "== $1"; echo -n "lines: "; rg --no-config --no-messages --count-matches "${G[@]}" -e "^.*$1.*\$" . | awk -F: '{s+=$2} END{print s+0}'; echo -n "components: "; rg --no-config --no-messages --files-with-matches "${G[@]}" -e "$1" . | awk -F/ '{print $5}' | sort -u | wc -l; }
pat 'row [0-9]+'
pat 'dev-docs'
pat 'backlog'
pat 'scripts/'
pat 'dx_repo_lints'
echo "== comment lines (Rust //, CSS /* ... */ lines starting with * or /*)"
echo -n "rs // lines: "; rg --no-config --count-matches "${G[@]}" -t rust -e '^\s*//' . | awk -F: '{s+=$2} END{print s}'
echo -n "css comment-ish lines (^\s*(/\*|\*)): "; rg --no-config --count-matches "${G[@]}" -t css -e '^\s*(/\*|\*)' . | awk -F: '{s+=$2} END{print s}'
echo "== python line count (splitlines) over the same set"
rg --no-config --files "${G[@]}" . | python3 -I -c '
import sys
n=0;c=0;rs=0
for f in sys.stdin.read().split("\n"):
    if not f:continue
    L=open(f,encoding="utf-8",errors="replace").read().splitlines();n+=len(L)
print("lines",n)'
echo "== other jargon"
pat 'playwright|\.spec\.ts'
pat 'primitives/src'
pat 'preview/src'
