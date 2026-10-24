#!/usr/bin/env bash
# The postcondition of an SSG build: "dx exited 0" is NOT evidence that the site is complete.
#
#   scripts/ssg-verify.sh <public_dir> <dx_log> <freshness_marker_file> [--catalog MOD_RS] [--base-path P]
#   scripts/ssg-verify.sh --self-test
#
# Run by scripts/build-ssg.sh (before and after the CSS bundle), so scripts/deploy-preview.sh gets the
# same guarantee through it. Prints `ssg-verify: OK -- N routes` on stdout and exits 0, or details on
# stderr ending in `ssg-verify: FAIL -- <reason>` and exits 1. If $SSG_VERIFY_REASON_FILE is set the
# one-line reason is also written there (build-ssg.sh quotes it in its own FAIL line).
#
# THE CLASS (backlog row 196): dx 0.7.9 logs a failed static-route render as
#     ERROR ssg error: reqwest::Error { kind: Request, url: "http://127.0.0.1:9999/component/table/?", ... IncompleteMessage ... }
# and then carries on: `SSG complete`, `Server build completed successfully!`, exit 0, with only some of the
# pages written. Nothing downstream looked at the log or at the route list, so the first symptom was a
# confusing failure two steps later (ssg-css-bundle: "no page links main.css") or no failure at all.
#
# What is checked, all derived from this build's own evidence (no hard-coded route count):
#  1. The dx log: the SSG pass ran (`Running SSG at`, `SSG complete`), and holds NO `ssg error:` line and no
#     `Failed to get/parse static routes` line.
#  2. The route list dx was asked to pre-render = every `Rendering <route> for SSG` line (dx fetched that list
#     from preview's `server_static_routes()` server fn; the log is the only place it is observable). Every
#     route must have `public/<route>/index.html` that is fresh (newer than the build-start marker: dx 0.7.9
#     keeps the previous build's HTML for routes it did not re-render), non-trivial (>= 4000 bytes) and
#     complete (`</html>` present). Every `index.html` in public/ must belong to a requested route (an extra is
#     a leftover from another build).
#  3. Independent of dx's own list (a truncated list would otherwise verify itself): every row of
#     `CATALOG` in preview/src/components/mod.rs (== every component, asserted by the
#     `catalog_classifies_every_demo` test) must be among the requested routes as `/component/<name>/`.
set -uo pipefail

MIN_BYTES=4000

strip_ansi() { sed -e $'s/\x1b\\[[0-9;]*[A-Za-z]//g'; }

# verify <public_dir> <dx_log> <marker> <catalog_file|""> <base_path|"">
verify() {
  local public_dir="$1" log="$2" marker="$3" catalog="$4" base="$5"
  local -a problems=()
  local reason=""
  local clean
  clean="$(mktemp)"
  strip_ansi < "$log" > "$clean"

  # 1. dx log
  if ! grep -q 'Running SSG at' "$clean"; then problems+=("dx log has no 'Running SSG at' line: the SSG pass never ran"); fi
  if ! grep -q 'SSG complete' "$clean"; then problems+=("dx log has no 'SSG complete' line: the SSG pass did not finish"); fi
  local errs
  errs="$(grep -E 'ssg error:|Failed to (get|parse) static routes' "$clean" || true)"
  if [ -n "$errs" ]; then
    problems+=("dx reported $(printf '%s\n' "$errs" | wc -l | tr -d ' ') failed render(s) (it still exits 0):")
    while IFS= read -r line; do problems+=("    ${line:0:260}"); done < <(printf '%s\n' "$errs" | head -n 12)
  fi

  # 2. requested routes -> files
  local -a routes=()
  local r
  while IFS= read -r r; do routes+=("$r"); done < <(
    sed -nE 's/.*Rendering ([^[:space:]]+) for SSG.*/\1/p' "$clean" |
      sed -E 's/\?.*$//' |
      awk -v base="$base" '{
        if (base != "" && index($0, "/" base "/") == 1) $0 = substr($0, length(base) + 2)
        if ($0 !~ /\/$/) $0 = $0 "/"
        if (!seen[$0]++) print
      }'
  )
  local n=${#routes[@]}
  if [ "$n" -eq 0 ]; then
    problems+=("dx log lists no 'Rendering <route> for SSG' line: no route was requested")
  fi
  local missing=() stale=() tiny=() incomplete=()
  declare -A want=()
  for r in "${routes[@]}"; do
    want["$r"]=1
    local f="$public_dir${r}index.html"
    if [ ! -f "$f" ]; then missing+=("$r"); continue; fi
    if [ ! "$f" -nt "$marker" ]; then stale+=("$r"); fi
    if [ "$(wc -c < "$f")" -lt "$MIN_BYTES" ]; then tiny+=("$r"); fi
    if ! grep -q '</html>' "$f"; then incomplete+=("$r"); fi
  done
  if [ "${#missing[@]}" -gt 0 ]; then problems+=("${#missing[@]} of $n requested route(s) have no index.html (render failed or was dropped): ${missing[*]:0:12}"); fi
  if [ "${#stale[@]}" -gt 0 ]; then problems+=("STALE_HTML=${#stale[@]} of $n route(s) were not written by this build: ${stale[*]:0:12}"); fi
  if [ "${#tiny[@]}" -gt 0 ]; then problems+=("${#tiny[@]} route(s) are under $MIN_BYTES bytes: ${tiny[*]:0:12}"); fi
  if [ "${#incomplete[@]}" -gt 0 ]; then problems+=("${#incomplete[@]} route(s) are truncated (no </html>): ${incomplete[*]:0:12}"); fi

  # every index.html in public/ must be a requested route
  local extra=() f2 rel
  while IFS= read -r -d '' f2; do
    rel="${f2#"$public_dir"}"; rel="${rel%index.html}"
    [ "${want[$rel]:-}" = 1 ] || extra+=("${rel:-/}")
  done < <(find "$public_dir" -name index.html -print0 2>/dev/null)
  if [ "${#extra[@]}" -gt 0 ]; then problems+=("${#extra[@]} index.html file(s) in public/ were not requested by this build's route list: ${extra[*]:0:12}"); fi

  # 3. the component catalog is covered
  if [ -n "$catalog" ]; then
    if [ ! -f "$catalog" ]; then
      problems+=("catalog file $catalog not found")
    else
      local names
      names="$(awk '/^pub const CATALOG/{f=1;next} f&&/^\];/{exit} f' "$catalog" | sed -nE 's/^[[:space:]]*\("([a-z0-9_]+)",[[:space:]]*Origin::.*/\1/p')"
      if [ -z "$names" ]; then
        problems+=("could not read any CATALOG row from $catalog")
      else
        local nm absent=() total=0
        for nm in $names; do
          total=$((total + 1))
          [ "${want[/component/$nm/]:-}" = 1 ] || absent+=("$nm")
        done
        if [ "${#absent[@]}" -gt 0 ]; then
          problems+=("${#absent[@]} of $total CATALOG component(s) were never requested from dx (truncated /api/static_routes?): ${absent[*]:0:12}")
        fi
      fi
    fi
  fi

  rm -f "$clean"
  if [ "${#problems[@]}" -gt 0 ]; then
    local nerr=0
    [ -n "$errs" ] && nerr="$(printf '%s\n' "$errs" | wc -l | tr -d ' ')"
    if [ "${#missing[@]}" -gt 0 ] || [ "$nerr" -gt 0 ]; then
      reason="SSG output incomplete: ${#missing[@]} of $n route(s) missing, ${#stale[@]} stale, $nerr dx render error(s)"
    else
      reason="SSG output failed its postcondition: ${problems[0]}"
    fi
    printf 'ssg-verify: problem: %s\n' "${problems[@]}" >&2
    echo "ssg-verify: FAIL -- $reason" >&2
    [ -n "${SSG_VERIFY_REASON_FILE:-}" ] && printf '%s\n' "$reason" > "$SSG_VERIFY_REASON_FILE"
    return 1
  fi
  echo "ssg-verify: OK -- $n routes"
  return 0
}

# ---------------------------------------------------------------------------------------------
self_test() {
  local tmp ok=1
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  local pub="$tmp/public" log="$tmp/dx.log" marker="$tmp/marker" cat="$tmp/mod.rs"
  local pad; pad="$(head -c 5000 /dev/zero | tr '\0' 'x')"
  touch -d '2 minutes ago' "$marker"

  mkpage() { mkdir -p "$pub$1"; printf '<html><body>%s</body></html>\n' "$pad" > "$pub${1}index.html"; }
  reset() {
    rm -rf "$pub"; mkdir -p "$pub"
    {
      echo '1.0s  INFO Running SSG at http://127.0.0.1:9999 for "/x/server"'
      for r in '/?' '/docs?' '/component/?name=&' '/component/alpha/?' '/component/beta/?'; do echo "1.1s  INFO Rendering $r for SSG"; done
      echo '2.0s  INFO SSG complete'
    } > "$log"
    for p in / /docs/ /component/ /component/alpha/ /component/beta/; do mkpage "$p"; done
    cat > "$cat" <<'EOF'
pub const CATALOG: &[(&str, Origin)] = &[
    ("alpha", Origin::Shadcn),
    ("beta", Origin::Extra("because")),
];
EOF
  }
  expect() { # <name> <0|1> -- run verify, compare exit code
    local name="$1" want="$2" rc
    verify "$pub" "$log" "$marker" "$cat" "" >/dev/null 2>"$tmp/err"; rc=$?
    [ "$rc" -ne 0 ] && rc=1
    if [ "$rc" = "$want" ]; then echo "  ok   $name"; else echo "  FAIL $name (exit $rc, wanted $want)"; sed 's/^/       /' "$tmp/err"; ok=0; fi
  }

  reset; expect "clean tree passes" 0
  reset; rm "$pub/component/beta/index.html"; expect "missing route page fails" 1
  reset; touch -d '1 hour ago' "$pub/docs/index.html"; expect "stale page fails" 1
  reset; printf '<html></html>' > "$pub/docs/index.html"; expect "trivial page fails" 1
  reset; head -c 6000 /dev/zero | tr '\0' 'x' > "$pub/docs/index.html"; expect "truncated page (no </html>) fails" 1
  reset; sed -i 's|^2.0s  INFO SSG complete|\x1b[31m2.0s ERROR ssg error: reqwest::Error { kind: Request, url: "http://127.0.0.1:9999/component/beta/?", source: IncompleteMessage }\x1b[0m\n2.0s  INFO SSG complete|' "$log"; expect "dx 'ssg error:' line fails even when every page exists" 1
  reset; sed -i '/SSG complete/d' "$log"; expect "missing 'SSG complete' fails" 1
  reset; sed -i '/Running SSG/d' "$log"; expect "missing 'Running SSG' fails" 1
  reset; : > "$log"; expect "empty dx log fails" 1
  reset; sed -i '/alpha/d' "$log"; expect "catalog component never requested fails (truncated route list)" 1
  reset; mkpage /leftover/; expect "unrequested leftover index.html fails" 1
  reset; sed -i 's|^1.1s  INFO Rendering /docs? for SSG|&\n1.1s  INFO Rendering /docs? for SSG|' "$log"; expect "duplicate Rendering line counts once" 0
  reset
  # base path: dx may log routes with the base prefix; files are not nested under it
  sed -i 's|Rendering /docs?|Rendering /shadcn-dioxus/docs?|' "$log"
  verify "$pub" "$log" "$marker" "$cat" "shadcn-dioxus" >/dev/null 2>"$tmp/err" && echo "  ok   base-path-prefixed route resolves" || { echo "  FAIL base-path-prefixed route"; ok=0; }
  reset; SSG_VERIFY_REASON_FILE="$tmp/reason" ; export SSG_VERIFY_REASON_FILE
  rm "$pub/docs/index.html"; verify "$pub" "$log" "$marker" "$cat" "" >/dev/null 2>&1
  if [ -s "$tmp/reason" ]; then echo "  ok   reason file written: $(cat "$tmp/reason")"; else echo "  FAIL reason file empty"; ok=0; fi
  unset SSG_VERIFY_REASON_FILE
  [ "$ok" = 1 ] && { echo "ssg-verify self-test: OK"; return 0; }
  echo "ssg-verify self-test: FAIL" >&2; return 1
}

# ---------------------------------------------------------------------------------------------
if [ "${1:-}" = "--self-test" ]; then self_test; exit $?; fi

catalog="" base=""
pos=()
while [ $# -gt 0 ]; do
  case "$1" in
    --catalog) catalog="${2:?--catalog needs a path}"; shift 2 ;;
    --base-path) base="${2:?--base-path needs a value}"; shift 2 ;;
    *) pos+=("$1"); shift ;;
  esac
done
if [ "${#pos[@]}" -ne 3 ]; then
  echo "usage: $0 <public_dir> <dx_log> <freshness_marker_file> [--catalog MOD_RS] [--base-path P] | --self-test" >&2
  exit 2
fi
public_dir="${pos[0]%/}"
[ -d "$public_dir" ] || { echo "ssg-verify: FAIL -- $public_dir is not a directory" >&2; exit 1; }
[ -f "${pos[1]}" ] || { echo "ssg-verify: FAIL -- dx log ${pos[1]} not found" >&2; exit 1; }
verify "$public_dir" "${pos[1]}" "${pos[2]}" "$catalog" "$base"
