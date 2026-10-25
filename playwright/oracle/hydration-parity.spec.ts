/**
 * ORACLE: hydration/deployment parity — fullstack SSG prerender vs. wasm
 * client hydration.
 *
 * Source: the WHATWG HTML Living Standard's hydration-relevant contract and
 * Dioxus's own fullstack hydration model:
 *   - The DOM the server sends and the DOM the client's first render
 *     produces must describe the *same* tree for hydration (attaching
 *     listeners to existing nodes rather than replacing them) to work at
 *     all -- Dioxus's hydration walks the server-rendered DOM by the
 *     `data-node-hydration` ids embedded in the markup below and expects
 *     the client's own render pass to produce a structurally identical
 *     tree at each id. A structural mismatch (a different element, a
 *     missing/extra attribute the client's event delegation depends on)
 *     does not "just" cause a one-time console warning the way React's
 *     reconciler-level hydration mismatch does -- Dioxus's event
 *     delegation is wired by walking the *hydrated* tree from those ids, so
 *     a mismatch there can silently leave event listeners unattached
 *     page-wide, not merely on the mismatched subtree.
 *   - The `popover` attribute and the top-layer promotion it implies:
 *     https://html.spec.whatwg.org/multipage/popover.html#the-popover-attribute
 *
 * ## The production failure this guards against (backlog row 22)
 *
 * The deployed site (https://mentalgear.github.io/shadcn-dioxus/) is
 * built as fullstack SSG by `scripts/deploy-preview.sh` (`scripts/build-ssg.sh`:
 * `dx build --ssg --features fullstack`). The SSG prerender runs the *server* binary -- a
 * host (non-wasm) build, not the wasm client. An overlay component that
 * splits its rendered markup on `#[cfg(target_family = "wasm")]` renders
 * the *native* arm there (plain `div`s, no `popover` attributes, no
 * `<dialog>` elements) while the wasm client then hydrates against that
 * structurally different markup (`dev-docs/reference/platform-rationale.md`
 * Caveat 1: the arm is chosen on the `web` feature axis instead). The
 * always-mounted `ToastProvider` region is the diagnostic marker: it must
 * render as `<div role="region" class="dx-toast-container-..." popover=
 * "manual">` in the served HTML, as the wasm web arm's own render does.
 * The user-visible symptom of a mismatch is "Dropdown Menu -> Open Menu:
 * nothing happens" on a hard-loaded page of the deployed site -- and, more
 * broadly, broken events page-wide, because Dioxus's event delegation is
 * wired from the hydration walk (see above), so one popover-shaped
 * mismatch can take down listener attachment for the whole page, not just
 * the mismatched element.
 *
 * Fixture: no dedicated fixture -- this spec exercises the real home page
 * (`/`, `preview/src/site/routes.rs`'s `Route::Home`) exactly as a real visitor's
 * browser would hard-load it from the deployed site, using the SSG static
 * build's own served HTML (see `playwright/ssg.local.config.ts`'s header
 * for how to build and serve it). Rule 3's Dropdown Menu demo is
 * `preview/src/components/dropdown_menu/variants/main/mod.rs`, embedded
 * directly on the home page's component gallery.
 *
 * Calibration: N/A (tier: hydration/deployment parity, not an HTML/APG/
 * Radix behavioural tier -- there is no meaningful "native reference"
 * for "did the server and client agree on markup").
 *
 * Rules:
 *   1. (served-markup invariant) The raw HTTP-served HTML of `/` --
 *      fetched via `request.get`, before any JS runs -- contains the
 *      always-mounted `ToastProvider` region carrying a `popover`
 *      attribute. This is the canary for "the SSG server rendered the web
 *      arm": the toast region mounts on every page regardless of which
 *      demo is visible, so its presence/attributes in the raw response are
 *      a direct read of what the *server* rendered, with zero client JS
 *      involved.
 *   2. (zero hydration errors) `page.goto("/")`, collecting console
 *      messages and `pageerror`s through load and a 2s settle. None may
 *      match /hydrat/i, and there must be no `pageerror`s. A structural
 *      server/client mismatch is exactly the class of defect Dioxus (like
 *      every hydrating framework) surfaces as a console warning/error
 *      naming "hydration" when it has to recover from one, or as an
 *      uncaught panic/exception when it can't.
 *   3. (post-hydration interactivity on a hard load) On the hard-loaded
 *      home page -- no client-side navigation, matching a real
 *      visitor's hard load -- clicking the Dropdown Menu
 *      demo's "Open Menu" trigger opens its menu. This is the literal
 *      symptom a split arm causes ("Dropdown Menu -> Open Menu: nothing is
 *      happening") on the deployed site's main page.
 *   4. (no duplicated attribute names in served markup) The raw HTTP-served
 *      HTML of `/` contains NO start tag with the same attribute name
 *      written twice. `name` lives in the URL PATH on the canonical
 *      component route (`ComponentDemoPath`, `/component/:name/`), which
 *      `dx build --ssg` enumerates one static file per `components::DEMOS`
 *      entry for (see `server_static_routes` in `preview/src/main.rs`), so
 *      `/component/button/` and `/component/top_layer/` serve genuinely
 *      different, real markup. The "hydration parity — per-component
 *      pages" describe block below is Rule 4's extension to every one of
 *      those pages; Rule 4 here is the narrower `/`-only guard for the
 *      five primitives that carry the hazard. The legacy query-string route
 *      (`Route::ComponentDemo`) still serves the same name-agnostic shell
 *      regardless of `X` by construction (see that route's own doc comment
 *      in `preview/src/site/routes.rs`) -- it is what the "legacy query-URL
 *      redirect" describe block below exercises. `/` is where the fixtures
 *      live pre-JS: the whole component gallery is embedded directly on
 *      the home page (Rule 3's Dropdown Menu is the same pattern). The
 *      `top_layer` fixture itself is excluded from the `/` gallery grid
 *      (`preview/src/site/gallery.rs`'s `ComponentGallery` -- it is an oracle
 *      fixture, not an installable component; it has its own
 *      `/component/top_layer/` page). That does not weaken Rule 4's
 *      coverage, since every duplicate-attribute defect it guards lives in
 *      a *primitive* (`toast.rs`, `progress.rs`, `context_menu.rs`,
 *      `popover.rs`, `select/components/trigger.rs`) that other components
 *      embedded on `/` exercise (the `progress` demo, the
 *      `context_menu`/`popover`/`select` component cards themselves); it
 *      is why Rule 4b's subject is the avatar card (see its own doc
 *      below).
 *      Rule source: the WHATWG HTML parsing
 *      spec's tokenizer step for start tags --
 *      https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
 *      -- "When the user agent leaves the attribute name state ... the
 *      complete attribute's name must be compared to the other attributes
 *      on the same token; if there is already an attribute on the token
 *      with the exact same name, then this is a duplicate-attribute parse
 *      error and the new attribute must be removed from the token." I.e.
 *      on a real duplicate, the browser's HTML parser silently keeps the
 *      *first* occurrence and drops every later one. This makes any
 *      component that SSR-serializes an explicit attribute and then also
 *      serializes a caller override of the same name (rather than merging
 *      them into one value before rendering) a hydration hazard: Dioxus's
 *      web (CSR/post-hydration) DOM path applies attributes sequentially,
 *      so there the *last*-applied value wins -- the exact opposite of
 *      what the parser above does with the *served* HTML. Server and
 *      client then structurally agree on "one aria-label attribute" but
 *      disagree on its *value*, which is invisible to Rules 1-3 (they
 *      check tree shape and interactivity, not computed accessible names)
 *      but fails axe's `landmark-unique` on the SSG lane wherever two
 *      landmarks are meant to be told apart by a caller-supplied name
 *      (`preview.spec.ts`, the top-layer fixture's toast region vs. the
 *      app shell's). The motivating case is `ToastRegionRendered`
 *      (`primitives/src/toast.rs`): its `aria_label: "{length}
 *      notifications"` followed by `..attributes` would serialize BOTH the
 *      primitive's default and the top-layer fixture's
 *      `aria_label: "Top-layer fixture notifications"` override into the
 *      same `<div>` start tag. Also asserts directly that the fixture
 *      region's *effective* (first-wins, per the parse rule quoted above)
 *      served `aria-label` is the caller's override, not the primitive's
 *      default -- the concrete, human-readable form of "no duplicate" for
 *      this exact case.
 *
 * Failure shapes: Rule 1 fails when the server-rendered toast region
 * carries no `popover` attribute at all (the native arm). Rules 2 and 3
 * fail as direct consequences of rule 1's markup mismatch: hydration
 * cannot reconcile the wasm client's web-arm render against the server's
 * native-arm markup, so hydration errors surface and click handlers
 * page-wide (including the Dropdown Menu trigger) are never attached.
 *
 * Rule 4 rejects any duplicate-attribute start tag on `/`: a caller
 * override (the `preview/src/components/top_layer/component.rs` oracle
 * fixture, or a dashboard `Progress` bar) colliding with the primitive's
 * own explicit attribute of the same name. The shapes it rejects, one
 * representative per component (the fixture's `id="clip-*"`/`id="scroll-*"`/
 * `id="edge-*"`/`id="popover-modal-*"` triggers repeat the same
 * popover/context-menu/select shape):
 *
 *   duplicate "style" in a dashboard Progress bar:
 *     <div role="progressbar" ... style="--progress-value: 68%"
 *          aria-label="Toward Q2 target" style="width: 100%;" ...>
 *   duplicate "style" in ContextMenuTrigger (`context_menu.rs`):
 *     <div id="dxc-82" ... style="-webkit-touch-callout: none; ..."
 *          style="padding:20px;...;cursor:context-menu;..." ...>
 *   duplicate "id" in PopoverTrigger (`popover.rs`):
 *     <button id="dxc-238" type="button" style="anchor-name: --dxa-dxc-240;"
 *             id="clip-popover-trigger" ...>
 *   duplicate "id" in ContextMenuRoot (`context_menu.rs`):
 *     <div id="dxc-232" tabindex=0 data-state="closed" data-disabled=false
 *          id="clip-context-menu-root" ...>
 *   duplicate "id" in ContextMenuTrigger (`context_menu.rs`):
 *     <div id="dxc-233" tabindex="-1" role="button" aria-haspopup="menu"
 *          aria-expanded=false style="-webkit-touch-callout: none; ..."
 *          id="clip-context-menu-trigger" ...>
 *   duplicate "id" in SelectTrigger (`select/components/trigger.rs`):
 *     <button id="dxc-226" style="anchor-name: --dxa-dxc-226;" type="button"
 *             aria-haspopup="listbox" aria-expanded=false
 *             id="clip-select-trigger" ...>
 *   duplicate "style"+"aria-label" in ToastRegionRendered (`toast.rs`):
 *     <div id="dxc-216" role="region" aria-label="0 notifications"
 *          tabindex="-1" popover="manual" style="--toast-count: 0"
 *          style="position: fixed; top: auto; right: 0; bottom: 0; left:
 *          auto; margin: 0;" aria-label="Top-layer fixture notifications"
 *          data-node-hydration="2062">
 *
 * Held by construction in all five files (`toast.rs`'s `ToastRegionRendered`
 * both arms, `progress.rs`'s `Progress`, `context_menu.rs`'s
 * `ContextMenuRoot` and `ContextMenuTrigger`, `popover.rs`'s
 * `PopoverTrigger`, `select/components/trigger.rs`'s `SelectTrigger`):
 * `merge_attributes` combines each component's own default attributes
 * with the caller's `attributes` into a single deduped list (caller wins)
 * before the `rsx!` call, so every tag is only ever built with one value
 * per attribute name -- neither the server nor the client path can emit a
 * duplicate, because there is only ever one attribute *to* emit.
 * `PopoverTrigger`'s `id` is the one exception carried unmerged, on its
 * own single explicit binding (its `use_effect`-based caller-id-to-
 * `ctx.labelledby` sync already makes it end up holding the caller's
 * value) -- every other `id` above is caller-wins like the rest, matching
 * each element's client (CSR, post-hydration) behavior (spread order makes
 * the caller's id win there), not protected against override the way
 * `toast.rs`'s own `id` is (see that function's doc): nothing else here
 * does a fixed-expectation `document.getElementById` lookup against its
 * *own* id winning -- `use_outside_dismiss`/`use_refocus_on_close_unless`'s
 * lookups silently no-op whenever a caller's override id makes them miss,
 * on both paths (a separate behavior gap once a caller overrides one of
 * these ids -- out of scope here, which is only about the served markup
 * agreeing with the DOM).
 *
 * ## Per-component pages (backlog rows 46 and 22): every component page, not just `/`
 *
 * Rows 46 and 22 (dev-docs/backlog.md): a `/component/?name=X&` page can
 * only prerender as the "Component not found" shell for every `X` (a
 * query string can never become a distinct static file -- see
 * `preview/src/site/routes.rs`'s `Route::ComponentDemo` doc comment), so `name`
 * lives in the URL PATH on the canonical `ComponentDemoPath` route
 * (`/component/:name/`), and `dx build
 * --ssg` prerenders one real, distinct static file per
 * `components::DEMOS` entry. Rules 1-4b's underlying method (does the
 * server-rendered markup agree with what the client hydrates against, with
 * zero hydration errors and no duplicated attributes) extends to every one
 * of them. Two `describe` blocks below do that extension:
 *
 *   - "hydration parity — per-component pages": Rule 5 (no prerendered
 *     component page served the not-found shell -- row 46 as a black-box
 *     assertion) and Rule 2/Rule 4's own method
 *     applied to every discovered page (zero hydration errors, no
 *     duplicated attributes), enumerated from the BUILT OUTPUT itself
 *     (`SSG_SITE_DIR`'s `component/*` subdirectories that have their own
 *     `index.html`) rather than a hand-maintained name list that could
 *     drift from `components::DEMOS`.
 *   - "hydration parity — legacy query-URL redirect on a static host": Rule
 *     6, the specific black-box case row 22 asks for -- a plain static
 *     file host (no `dx serve`, no server-side redirect logic at all) can
 *     still only ever serve ONE physical file for every `/component/?name=
 *     X&` request regardless of `X` (the query string is invisible to
 *     file-path resolution), so the legacy `ComponentDemo` route's client-side
 *     redirect (`preview/src/site/demos.rs`) is what has to resolve the real
 *     component after hydration, not the server. Both blocks `test.skip`
 *     (not fail) when `SSG_SITE_DIR` is unset, so rules 1-4b still run
 *     against an SSG build that was not invoked through that env var.
 *
 * ## Rule 4c: synthesized collisions (backlog row 93)
 *
 * Rules 4/4b/4-extended above all share one limitation: they can only ever
 * find a duplicate-attribute collision that some `preview` demo page
 * *happens* to produce (a caller override that happens to collide with a
 * component's own literal attribute of the same name). Manual sweeps of
 * demo pages therefore each leave different survivors: a sweep fixes only
 * what the demo content incidentally exercises. A component that carries the
 * same literal-attribute-beside-a-raw-spread hazard shape, but that no demo
 * page currently happens to override, is invisible to all of the rules
 * above -- not fixed, just not yet triggered.
 *
 * "hydration parity — synthesized attribute collisions (Rule 4c)" below
 * closes that gap by manufacturing the collision directly, per component,
 * rather than waiting for one. It builds and runs a small, standalone Rust
 * binary crate, `playwright/oracle/attr-synth/` (its own doc comment,
 * `attr-synth/src/main.rs`, has the full account: why it exists as a
 * separate crate, why it renders `dioxus-primitives` directly rather than
 * through `preview`, exactly how each case is constructed, and what it
 * still cannot reach). For each covered component, that crate pushes a
 * caller-supplied, distinctive marker value for an attribute the component
 * ALSO sets literally directly into that component's own `attributes`
 * catch-all field, renders it via `dioxus-ssr` (the same
 * `VirtualDom`/`rebuild_in_place`/`render_immediate` technique this repo's
 * own `primitives/src/dropdown_menu.rs`/`menubar.rs`/`tooltip.rs`
 * `#[cfg(test)]` modules already use for exactly this proof, on a handful of
 * sites), and prints one JSON line per case (`case`, `attr`, `expected`,
 * `source`, `html`). Rule 4c re-uses this file's OWN `extractStartTags`
 * tokenizer against each case's `html` -- the identical WHATWG-tokenizer
 * method Rules 1-4b already use against real served markup, just pointed at
 * a synthesized fragment instead -- and asserts the target start tag
 * carries `attr` exactly once, with its effective (first-wins) value
 * containing the caller's marker.
 *
 * This does not depend on `SSG_SITE_DIR` or a running server at `BASE`: it
 * needs only `cargo` (already a hard requirement everywhere else in this
 * repo's toolchain). It runs unconditionally, so it keeps exercising
 * `dioxus-primitives`'s attribute-merge construction even in a plain
 * `npx playwright test` run with no SSG build present at all -- broader
 * standing coverage than the SSG-gated rules above, not narrower.
 *
 * Coverage: 27 real component+attribute+element sites across
 * 13 primitives (`progress`, `toast`, `context_menu`, `dropdown_menu`,
 * `menubar`, `popover`, `navbar`, `navigation_menu`, `collapsible`,
 * `tooltip`, `select`, `hover_card`, `combobox`), plus 2 self-test sites
 * that prove the detection pipeline itself (not a real component) can tell
 * a live collision apart from a fixed one -- see attr-synth/src/main.rs's
 * own doc for the exhaustive list and for what it cannot reach
 * (four menu-family `*Content` components gated behind an effect-driven
 * render signal that does not settle synchronously, plus `Drawer`).
 * The demo-page-driven rules above cannot catch a bare
 * `style: anchor_name_style(..)` literal beside a raw `..attributes`
 * spread (the `navigation_menu:trigger:style` shape); this block can.
 * Nine anchored-trigger cases (`navigation_menu`, `hover_card`, `menubar`,
 * `navbar`, `context_menu:sub_trigger`, `dropdown_menu:trigger`,
 * `dropdown_menu:sub_trigger`, `popover:trigger`, `tooltip:trigger`, all
 * on `style`) share one
 * construction, `top_layer::anchored_trigger_attributes` (backlog row 99),
 * and each is asserted here the same way as every other case in this
 * block, so none of them can regress to a bare literal, or to a plain
 * (non-folding) merge, without this file failing.
 */

import { test, expect } from "../fixtures";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { gotoHydrated } from "../hydration";
import { baseUrlOr } from "../base-url";

const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app
// Default :8090 (the static SSG server -- see ssg.local.config.ts); override with
// PLAYWRIGHT_BASE_URL (playwright/base-url.ts) to point at any server or base path.
const BASE = baseUrlOr("http://127.0.0.1:8090");
const BASE_PATH = new URL(BASE).pathname.replace(/\/$/, "");

/**
 * Every component page this build actually prerendered, discovered from the
 * SSG output directory itself (row 46) rather than a hand-maintained name
 * list that could drift from `components::DEMOS`
 * (`preview/src/components/mod.rs`). Pass the site directory (the same one
 * served on :8090) via `SSG_SITE_DIR` -- e.g.
 * `SSG_SITE_DIR=$(pwd)/../target/dx/preview/release/web/public`. Excludes
 * `block` (a route family -- `/component/block/<name>/<variant>/` -- not a
 * component name itself) and any directory without its own `index.html`
 * (would not be a real prerendered page). Returns `[]` when `SSG_SITE_DIR`
 * is unset or does not exist, which every rule below treats as "skip this
 * extension," not "fail."
 */
function discoverComponentNames(): string[] {
  const siteDir = process.env.SSG_SITE_DIR;
  if (!siteDir) return [];
  const componentDir = path.join(siteDir, "component");
  if (!fs.existsSync(componentDir)) return [];
  return fs
    .readdirSync(componentDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name !== "block")
    .filter((entry) => fs.existsSync(path.join(componentDir, entry.name, "index.html")))
    .map((entry) => entry.name)
    .sort();
}

const COMPONENT_NAMES = discoverComponentNames();

/**
 * Rule 4 support: a minimal HTML start-tag tokenizer, just enough to answer
 * "does this tag have the same attribute name twice, and if so what value
 * does WHATWG's first-wins duplicate-attribute rule leave in effect."
 *
 * Deliberately not a full HTML parser -- text-node content in this app's
 * served markup is always entity-escaped by Dioxus's SSR renderer (rsx text
 * children are never written as raw `<`/`>`), so a naive "look for the next
 * `<`" scan never mistakes text for a tag boundary here. It does track
 * quote state so a `>` inside a quoted attribute value (e.g. an inline
 * `style` string) never truncates a tag early.
 */
type StartTag = {
  raw: string;
  name: string;
  /** Index of this tag's leading `<` in the source HTML -- lets a caller correlate a start
   * tag with a nearby sibling/descendant tag found elsewhere in the same document (Rule 4b
   * below uses this to find the `<span role="img">` ancestor of a specific `<img alt="...">`). */
  start: number;
  /** Attribute names in document order, lower-cased, one entry per occurrence. */
  attrNames: string[];
  /** name -> value of the FIRST occurrence only (WHATWG duplicate-attribute rule: later ones are dropped). */
  effectiveValues: Map<string, string | null>;
};

function extractStartTags(html: string): StartTag[] {
  const tags: StartTag[] = [];
  let i = 0;
  const n = html.length;
  while (i < n) {
    if (html[i] !== "<") {
      i++;
      continue;
    }
    if (html.startsWith("<!--", i)) {
      const end = html.indexOf("-->", i + 4);
      i = end === -1 ? n : end + 3;
      continue;
    }
    if (html[i + 1] === "/" || html[i + 1] === "!" || html[i + 1] === "?") {
      const end = html.indexOf(">", i);
      i = end === -1 ? n : end + 1;
      continue;
    }
    if (!/[a-zA-Z]/.test(html[i + 1] ?? "")) {
      // Not actually a tag start (stray '<'); move on one char.
      i++;
      continue;
    }
    let j = i + 1;
    let inSingle = false;
    let inDouble = false;
    while (j < n) {
      const c = html[j];
      if (inSingle) {
        if (c === "'") inSingle = false;
      } else if (inDouble) {
        if (c === '"') inDouble = false;
      } else if (c === "'") {
        inSingle = true;
      } else if (c === '"') {
        inDouble = true;
      } else if (c === ">") {
        break;
      }
      j++;
    }
    const raw = html.slice(i, j + 1);
    const nameMatch = raw.match(/^<([a-zA-Z][a-zA-Z0-9-]*)/);
    if (nameMatch) {
      const name = nameMatch[1].toLowerCase();
      let body = raw.slice(1 + nameMatch[1].length, raw.length - 1);
      if (body.endsWith("/")) body = body.slice(0, -1);
      const attrNames: string[] = [];
      const effectiveValues = new Map<string, string | null>();
      const attrRe =
        /([^\s"'=<>`/]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;
      let m: RegExpExecArray | null;
      while ((m = attrRe.exec(body)) !== null) {
        const attrName = m[1].toLowerCase();
        const value = m[2] ?? m[3] ?? m[4] ?? null;
        attrNames.push(attrName);
        if (!effectiveValues.has(attrName)) {
          effectiveValues.set(attrName, value); // first occurrence wins, per WHATWG
        }
      }
      tags.push({ raw, name, start: i, attrNames, effectiveValues });
    }
    i = j + 1;
  }
  return tags;
}

function duplicateAttrTags(html: string): { raw: string; dup: string }[] {
  const out: { raw: string; dup: string }[] = [];
  for (const tag of extractStartTags(html)) {
    const seen = new Set<string>();
    for (const a of tag.attrNames) {
      if (seen.has(a)) {
        out.push({ raw: tag.raw, dup: a });
        break;
      }
      seen.add(a);
    }
  }
  return out;
}

/**
 * Rule 4c support: runs the BUILT `playwright/oracle/attr-synth/` (built by
 * `tools/ensure-attr-synth.sh`, through the build lock, never here -- row 338) (see
 * that crate's own `src/main.rs` doc comment for the full rationale) and
 * parses its one-JSON-object-per-line stdout. Run once, synchronously, at
 * module load -- same timing as `COMPONENT_NAMES` above -- so the resulting
 * cases can be turned into statically-declared, individually named `test()`
 * calls below (one per case, for per-case pass/fail visibility) rather than
 * a single opaque test.
 *
 * Deliberately never throws: a build/run failure is captured as `error` and
 * surfaced through an always-present test below ("Rule 4c: attr-synth built
 * and ran successfully") instead, so it fails loudly with the captured
 * output rather than silently producing zero cases -- the same concern the
 * "legacy query-URL redirect" describe block below documents for its own
 * three always-listed tests.
 */
type AttrSynthCase = {
  case: string;
  attr: string;
  expected: string;
  source: string;
  html: string;
};

function runAttrSynth(): { cases: AttrSynthCase[]; error: string | null } {
  const crateDir = path.join(__dirname, "attr-synth");
  const binPath = path.join(crateDir, "target", "release", "attr-synth");
  if (!fs.existsSync(binPath)) {
    // Never build here (row 338): module load runs for `--list` too, outside the build lock.
    return {
      cases: [],
      error:
        `attr-synth is not built (${binPath}). Run \`playwright/tools/ensure-attr-synth.sh\` (builds it through the ` +
        "build lock; `npm run test:changed` and tools/run-suite.sh do it for you), then rerun.",
    };
  }
  const run = spawnSync(binPath, [], { cwd: crateDir, timeout: NAV_TIMEOUT, encoding: "utf-8" });
  if (run.error) {
    return { cases: [], error: `failed to spawn the attr-synth binary at ${binPath}: ${run.error}` };
  }
  if (run.status !== 0) {
    return {
      cases: [],
      error: `attr-synth exited ${run.status}:\n--- stderr ---\n${run.stderr}\n--- stdout ---\n${run.stdout}`,
    };
  }

  try {
    const cases = run.stdout
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0)
      .map((line) => JSON.parse(line) as AttrSynthCase);
    if (cases.length === 0) {
      return {
        cases: [],
        error: "attr-synth produced zero cases (empty stdout) -- see attr-synth/src/main.rs's own `cases()`.",
      };
    }
    return { cases, error: null };
  } catch (e) {
    return { cases: [], error: `failed to parse attr-synth's stdout as JSON lines: ${e}\nstdout:\n${run.stdout}` };
  }
}

const ATTR_SYNTH = runAttrSynth();

test.describe("hydration parity — synthesized attribute collisions (Rule 4c)", () => {
  test("Rule 4c: attr-synth built and ran successfully", () => {
    expect(ATTR_SYNTH.error, ATTR_SYNTH.error ?? "").toBeNull();
  });

  // `selftest:unfixed:id` is a deliberate, permanent meta-proof that this file's own detection logic can tell a
  // real collision apart from a fixed one (attr-synth/src/main.rs, `self_test_unfixed_collision`): the synthesized
  // fragment carries the collision on purpose, so its assertions MUST fail. It is declared `test.fail`, so
  // Playwright reports the failure as an expected outcome (`stats.expected`, which `deploy-preview.sh`'s verdict
  // counts as a pass) and the case turns RED the day its assertions pass, i.e. the day the detection stops seeing
  // the collision. Every other case is asserted as an ordinary test (backlog row 377).
  const EXPECTED_FAILING = new Set(["selftest:unfixed:id"]);

  for (const c of ATTR_SYNTH.cases) {
    const expectedFailing = EXPECTED_FAILING.has(c.case);
    const label = expectedFailing ? " (expected failure: the detector's meta-proof)" : "";
    const caseTest = expectedFailing ? test.fail : test;
    caseTest(`Rule 4c: ${c.case} -- caller override of a literal attribute is served exactly once${label}`, () => {
      const tags = extractStartTags(c.html);
      // Finds the right tag by substring on the RAW text, not by the
      // tokenizer's first-wins effective value: when the collision is
      // live, the effective value is the component's own internal
      // default, not this case's marker, but the raw tag text still
      // contains BOTH values as substrings pre-parse (that is the whole
      // point of the defect) -- so this lookup succeeds identically
      // whether the case is RED or GREEN, and only the assertions below
      // tell those two states apart.
      const target = tags.find((t) => t.raw.includes(c.expected));
      expect(
        target,
        `attr-synth case "${c.case}" (${c.source}): no start tag in the synthesized fragment contains the ` +
          `marker "${c.expected}" anywhere in its raw text -- the target element may not have rendered at all ` +
          `(see attr-synth/src/main.rs's own doc, "What this still cannot reach"). Rendered fragment:\n${c.html}`,
      ).toBeDefined();

      const occurrences = target!.attrNames.filter((n) => n === c.attr).length;
      expect(
        occurrences,
        `attr-synth case "${c.case}" (${c.source}): expected exactly one "${c.attr}" attribute on the ` +
          `synthesized element, found ${occurrences}. Per WHATWG HTML's attribute-name parsing state, a ` +
          `duplicate is a parse error and the browser keeps only the FIRST occurrence -- but Dioxus's web ` +
          `(CSR/hydrated) DOM path applies attributes sequentially, so there the LAST-applied value wins. ` +
          `Server and client then disagree about which value is in effect. Raw tag: ${target!.raw}`,
      ).toBe(1);

      // `.toContain`, not `.toBe`: two families of these deliberately FOLD
      // the caller's style together with an internal anchor-positioning
      // binding, rather than letting either one flatly replace the other --
      // the anchored-CONTENT family (`popover:content:style`/
      // `tooltip:content:style`/`select:list:style`/`hover_card:content:
      // style`/`combobox:list:style`) via `top_layer::
      // anchored_content_attributes`, and the anchored-TRIGGER family
      // (`context_menu:sub_trigger:style`/`dropdown_menu:trigger:style`/
      // `dropdown_menu:sub_trigger:style`/`hover_card:trigger:style`/
      // `menubar:trigger:style`/`navbar:trigger:style`/`navigation_menu:
      // trigger:style`/`popover:trigger:style`/`tooltip:trigger:style`) via
      // `top_layer::anchored_trigger_attributes` -- documented at this
      // file's own header ("Rule 4c" section) and in each function's own
      // doc. Both exist for the same reason: plain `merge_attributes` only
      // ever folds `class` (`primitives/src/lib.rs`'s own
      // `later_list_overwrites`/`style_attribute_is_overwritten_not_folded`
      // tests), so without folding first, a caller's own `style` would
      // either collide with a bare literal (the defect this rule
      // exists to catch) or, once merged the naive way, silently REPLACE --
      // not combine with -- the anchor/position-anchor binding, which is a
      // quieter, worse failure (CSS Anchor Positioning breaks with no
      // duplicate attribute left for anything to catch). The effective
      // value for every one of these folding cases is real CSS text
      // containing the marker, not equal to it verbatim; for every other
      // (plain-merge, or no merge at all) case the two checks are
      // equivalent, since nothing else ever gets folded in.
      expect(
        target!.effectiveValues.get(c.attr),
        `attr-synth case "${c.case}" (${c.source}): the served, EFFECTIVE (first-wins) value of "${c.attr}" ` +
          `should contain the caller's override ("${c.expected}") -- it must not be silently dropped in favor ` +
          `of the component's own internal default. Raw tag: ${target!.raw}`,
      ).toContain(c.expected);
    });
  }
});

test.describe("hydration parity — SSG server markup vs. wasm client", () => {
  // Row 123 / row 100: a `--base-path shadcn-dioxus` build (the deploy build) served at
  // `/` makes every internal URL `/shadcn-dioxus/...`, which fails several rules below
  // for a test-setup reason that reads exactly like a hydration regression. Say so once, first.
  // Skipped when BASE itself carries a path (a base-path build served under its prefix).
  test("Rule 0: the served build matches the served base path (no base-path build served at root)", async ({
    request,
  }) => {
    test.skip(BASE_PATH !== "", `BASE has a path prefix (${BASE_PATH}); a prefixed build is expected`);
    const response = await request.get(`${BASE}/`, { timeout: NAV_TIMEOUT });
    expect(response.ok()).toBeTruthy();
    const html = await response.text();
    const prefixed = html.match(/["'(]\/shadcn-dioxus\//g) ?? [];
    expect(
      prefixed.length,
      `WRONG BUILD SERVED, not a hydration failure: ${BASE}/ contains ${prefixed.length} ` +
        `"/shadcn-dioxus/" URL(s) -- a --base-path build (scripts/deploy-preview.sh, backlog row 100) ` +
        `is being served at the root. Serve a no-base-path build at "/", or set ` +
        `PLAYWRIGHT_BASE_URL=http://127.0.0.1:8090/shadcn-dioxus to serve it under its prefix.`,
    ).toBe(0);
  });

  test("Rule 1: served HTML's ToastProvider region carries popover (web-arm markup canary)", async ({
    request,
  }) => {
    const response = await request.get(`${BASE}/`, { timeout: NAV_TIMEOUT });
    expect(response.ok()).toBeTruthy();
    const html = await response.text();

    // `\bdx-toast-container\b` (a whole class TOKEN anywhere in the
    // attribute value), not `dx-toast-container-[^"]*` (a literal trailing
    // hyphen then more characters): the class is the plain, unhashed,
    // multi-class `class="dx-toast-base-region dx-toast-container"` (row 32:
    // no `#[css_module]` hashing), so a pattern that requires a literal
    // "-" immediately after "container" can never match.
    const containerMatch = html.match(/<div[^>]*class="[^"]*\bdx-toast-container\b[^"]*"[^>]*>/);
    expect(
      containerMatch,
      "expected a dx-toast-container element in the raw server-rendered HTML",
    ).not.toBeNull();

    const containerTag = containerMatch![0];
    expect(
      containerTag,
      `server-rendered ToastProvider region is missing popover — this is the ` +
        `native (non-web) render arm; raw tag: ${containerTag}`,
    ).toMatch(/\bpopover="manual"/);
  });

  test("Rule 2: zero hydration errors on a hard load of the home page", async ({ page }) => {
    const consoleMessages: string[] = [];
    const pageErrors: string[] = [];
    page.on("console", (msg) => consoleMessages.push(msg.text()));
    page.on("pageerror", (err) => pageErrors.push(err.message));

    await gotoHydrated(page, `${BASE}/`, { timeout: NAV_TIMEOUT, waitUntil: "networkidle" });
    // Settle window: hydration mismatch recovery/errors can surface a tick
    // or two after the initial load event, not necessarily synchronously.
    await page.waitForTimeout(2000);

    const hydrationMessages = consoleMessages.filter((m) => /hydrat/i.test(m));
    expect(
      hydrationMessages,
      `console messages mentioning hydration:\n${hydrationMessages.join("\n")}`,
    ).toHaveLength(0);
    expect(
      pageErrors,
      `uncaught page errors during load:\n${pageErrors.join("\n")}`,
    ).toHaveLength(0);
  });

  test("Rule 3: Dropdown Menu 'Open Menu' opens its menu on a hard-loaded main page", async ({
    page,
  }) => {
    // dev-docs/backlog.md row 109: this test clicks the
    // trigger right after navigating, on the SSG lane -- `gotoHydrated` so
    // that click can't land before hydration attaches listeners. (Rule 2
    // above is a pure console/error check with no interaction, so it keeps
    // a plain `page.goto`.)
    await gotoHydrated(page, `${BASE}/`, { timeout: NAV_TIMEOUT });

    // The home page mounts every component's gallery card, so there is no single demo frame to scope to: the
    // dropdown menu's trigger and open content are selected by the component's own classes, inside ITS card --
    // the docs chrome's own "Jump to category" menu is a dropdown-menu trigger too and comes first.
    const trigger = page
      .locator('article.dx-component-card:has(a[href^="/component/dropdown_menu/"]) button.dx-dropdown-menu-trigger')
      .first();
    await expect(trigger).toBeVisible();
    await expect(trigger).toHaveAttribute("data-state", "closed");

    await trigger.click();

    await expect(
      trigger,
      "clicking 'Open Menu' on a hard-loaded page did not open the menu " +
        "(data-state never became 'open'): a hard-loaded page must be " +
        "interactive after hydration",
    ).toHaveAttribute("data-state", "open");
    // `menuitem`, not `option`: DropdownMenu's items carry the APG Menu
    // Button pattern role (dev-docs/backlog.md row 24, `menu-roles.spec.ts`),
    // not the listbox `option` role. Scoped through
    // `getByLabel("Open Menu")` (the popup's own accessible name, via
    // `aria-labelledby` back to its trigger) rather than a bare
    // page-wide `getByRole` -- with every component's gallery card mounted
    // on `/`, `menubar`'s own demo ALSO has an "Edit" menuitem, and a
    // page-wide locator hits a strict-mode "2 elements" violation across
    // the two components' items.
    await expect(
      page.locator('.dx-dropdown-menu-content[data-state="open"]').getByRole("menuitem").first(),
    ).toBeVisible();
  });

  // `/component/?name=top_layer&`/`/component/?name=dialog&` are not
  // included: `name` is a query param this app's SSG build does not
  // prerender per-value (see Rule 4's doc above) -- `/` already embeds
  // every component gallery card directly (`top_layer` itself is not among
  // them, see Rule 4's own doc note above, but every duplicate-attribute
  // defect Rule 4 guards lives in a primitive exercised by another gallery
  // card).
  const RULE4_URLS = [`${BASE}/`];

  test("Rule 4: no start tag in served HTML has a duplicated attribute name (WHATWG HTML duplicate-attribute parse error)", async ({
    request,
  }) => {
    for (const url of RULE4_URLS) {
      const response = await request.get(url, { timeout: NAV_TIMEOUT });
      expect(response.ok()).toBeTruthy();
      const html = await response.text();

      const dupes = duplicateAttrTags(html);
      expect(
        dupes,
        `${url}: found start tag(s) with a duplicated attribute name. Per WHATWG HTML's ` +
          `attribute-name parsing state, a duplicate is a parse error and the browser keeps ` +
          `only the FIRST occurrence -- but Dioxus's web (CSR/hydrated) DOM path applies ` +
          `attributes sequentially, so there the LAST-applied value wins. Server and client ` +
          `then disagree about which value is in effect:\n` +
          dupes.map((d) => `  duplicate "${d.dup}" in: ${d.raw}`).join("\n"),
      ).toHaveLength(0);
    }
  });

  // Rule 4b's subject is the `avatar` component's own gallery card, always
  // mounted on `/`. (The top-layer oracle fixture's `ToastProvider` region --
  // a caller `aria_label: "Top-layer fixture notifications"` override of
  // `ToastRegionRendered`'s own default (`toast.rs`) -- would be the more
  // direct subject, but `preview/src/site/gallery.rs`'s `ComponentGallery` does not
  // embed the `top_layer` fixture on the home page (it is an oracle probe
  // surface, not an installable component; see Rule 4's doc note above), and
  // `/component/?name=X&` cannot serve as a fixture page for the reason
  // Rule 4's doc gives: this app's SSG build does not prerender per
  // query-string value.)
  //
  // The avatar's "Error State" example
  // (`preview/src/components/avatar/variants/states/mod.rs`) renders
  // `ImageAvatar { alt: "Jordan Reyes", aria_label: "Error avatar", ... }`.
  // `ImageAvatar` (`preview/src/components/avatar/component.rs`) computes
  // its OWN default accessible name from `alt` (`aria_label: "{alt}"` --
  // row 34, axe `role-img-alt`) and merges it with the caller's
  // `attributes` via `merge_attributes` (caller-wins, deduped) before
  // rendering `Avatar`'s `role="img"` root -- structurally the same
  // "explicit default + caller override of the same attribute name" shape
  // Rule 4b guards, just one layer up (a preview-level
  // themed-wrapper default, not a primitive's own): if that merge order
  // were ever reversed (`merge_attributes(vec![props.attributes, base])`
  // instead of `vec![base, props.attributes]`), the alt-derived default
  // ("Jordan Reyes") would win over the caller's real, more specific
  // override ("Error avatar") in the served markup -- a real accessible-
  // name regression this rule catches directly, on the one page every
  // visitor and every gallery-wide oracle already loads.
  //
  // Correlating the two: `ImageAvatarProps`/`AvatarImage` put `alt` on the
  // inner `<img>`, not on the outer `role="img"` `<span>` this test cares
  // about, so the span is found by proximity to its own descendant `<img
  // alt="Jordan Reyes">` (`start` index, both from `extractStartTags`)
  // rather than by a marker on the span's own tag -- the alt text and the
  // aria-label override text are deliberately DIFFERENT strings (unlike
  // e.g. the "Large avatar" example, whose `alt` and `aria_label` happen to
  // be identical text and so could never distinguish "default won" from
  // "override won").
  test("Rule 4b: the avatar demo's caller-overridden accessible name is served, not the alt-derived default", async ({
    request,
  }) => {
    // The avatar page (not the `/` gallery card, which shows shadcn's avatar-demo): the "Error State" example lives in
    // the `states` variant (backlog row 235).
    const response = await request.get(`${BASE}/component/avatar/`, { timeout: NAV_TIMEOUT });
    expect(response.ok()).toBeTruthy();
    const html = await response.text();

    const tags = extractStartTags(html);
    // The example is found by its deliberately dead `src` (it is the only avatar that is meant to fail to load) plus
    // its alt text: the demo names an invented person ("Jordan Reyes", with no real photo hot-linked), so the alt is
    // the alt-derived default that must LOSE to the override.
    const markerImg = tags.find(
      (t) =>
        t.name === "img" &&
        t.effectiveValues.get("alt") === "Jordan Reyes" &&
        (t.effectiveValues.get("src") ?? "").includes("invalid-url.example"),
    );
    expect(
      markerImg,
      `expected the avatar demo's "Error State" example -- an <img alt="Jordan Reyes"> with an invalid-url.example src -- ` +
        `in served HTML; the avatar component's main-variant demo may have changed ` +
        `(preview/src/components/avatar/variants/states/mod.rs)`,
    ).toBeDefined();

    // The nearest preceding `<span role="img">` is this `<img>`'s own
    // `Avatar` root -- `ImageAvatar` always renders its `AvatarImage` as a
    // direct-ish descendant of that span, and no OTHER `role="img"` span in
    // this demo sits between them (each avatar item is a fully separate
    // `ImageAvatar`/`Avatar` subtree).
    const roleImgSpans = tags.filter(
      (t) => t.name === "span" && t.effectiveValues.get("role") === "img" && t.start < markerImg!.start,
    );
    const avatarSpan = roleImgSpans.at(-1);
    expect(
      avatarSpan,
      `expected a preceding <span role="img"> ancestor for the "Jordan Reyes" <img> in served HTML`,
    ).toBeDefined();

    const effective = avatarSpan!.effectiveValues.get("aria-label");
    expect(
      effective,
      `the avatar demo's "Error State" example's EFFECTIVE served aria-label should be the ` +
        `caller's override ("Error avatar", from preview/src/components/avatar/variants/states/` +
        `mod.rs), not ImageAvatar's own alt-derived default ("Jordan Reyes") -- raw tag: ` +
        `${avatarSpan!.raw}`,
    ).toBe("Error avatar");
  });
});

// Per-component pages (row 46) -- see this file's header ("Per-component
// pages") for the full rationale. Both describe blocks below skip (not
// fail) when SSG_SITE_DIR is unset, so this file stays runnable against an
// SSG build invoked without that env var.
test.describe("hydration parity — per-component pages (row 46)", () => {
  test.skip(
    COMPONENT_NAMES.length === 0,
    "SSG_SITE_DIR not set -- export it to the SSG build's public/ dir (or run via the CI job that builds the SSG) to enumerate every prerendered component page; see this file's header.",
  );

  test("Rule 5: no prerendered component page served the not-found shell", async ({ request }) => {
    const offenders: string[] = [];
    for (const name of COMPONENT_NAMES) {
      const response = await request.get(`${BASE}/component/${name}/`, { timeout: NAV_TIMEOUT });
      expect(response.ok(), `${name}: expected the prerendered page to respond 200`).toBeTruthy();
      const html = await response.text();
      if (html.includes("dx-component-demo-not-found")) {
        offenders.push(name);
      }
    }
    expect(
      offenders,
      `component page(s) served the "Component not found" shell instead of their real content ` +
        `-- every prerendered page must carry its own content (row 46): ${offenders.join(", ")}`,
    ).toHaveLength(0);
  });

  test("Rule 2 (extended): zero hydration errors on every prerendered component page", async ({
    page,
  }) => {
    // One test, 62+ sequential navigations -- well past ssg.local.config.ts's
    // 90s default test timeout even when every page is fast.
    test.setTimeout(5 * 60 * 1000);

    const failures: string[] = [];
    for (const name of COMPONENT_NAMES) {
      const consoleMessages: string[] = [];
      const pageErrors: string[] = [];
      const onConsole = (msg: { text(): string }) => consoleMessages.push(msg.text());
      const onError = (err: Error) => pageErrors.push(err.message);
      page.on("console", onConsole);
      page.on("pageerror", onError);

      // "load", not "networkidle" (Rule 2's own home-page-only test uses
      // "networkidle" and that page settles fine): `top_layer`'s oracle
      // fixture page is "a large probe surface" with its own ongoing
      // timers/listeners by design (playwright/oracle/tier2-html/
      // top-layer.spec.ts), so it may never reach network-idle at all.
      // A hydration mismatch surfaces during the
      // synchronous hydration walk right after `load`, not from later
      // background network activity, so "load" is both sufficient for what
      // this rule checks and immune to a fixture page's own idle timers.
      await gotoHydrated(page, `${BASE}/component/${name}/`, { timeout: NAV_TIMEOUT, waitUntil: "load" });
      // Shorter settle window than Rule 2's own 2s: this loop already pays
      // one full navigation per component, and a hydration-mismatch
      // recovery (the defect class this guards) surfaces within a tick or
      // two of load, not seconds later.
      await page.waitForTimeout(500);

      page.off("console", onConsole);
      page.off("pageerror", onError);

      const hydrationMessages = consoleMessages.filter((m) => /hydrat/i.test(m));
      if (hydrationMessages.length > 0 || pageErrors.length > 0) {
        failures.push(
          `${name}: hydration console messages=${JSON.stringify(hydrationMessages)} ` +
            `page errors=${JSON.stringify(pageErrors)}`,
        );
      }
    }
    expect(failures, failures.join("\n")).toHaveLength(0);
  });

  test("Rule 4 (extended): no duplicated attribute names on any prerendered component page", async ({
    request,
  }) => {
    const failures: string[] = [];
    for (const name of COMPONENT_NAMES) {
      const response = await request.get(`${BASE}/component/${name}/`, { timeout: NAV_TIMEOUT });
      const html = await response.text();
      const dupes = duplicateAttrTags(html);
      if (dupes.length > 0) {
        failures.push(`${name}: ${dupes.map((d) => `duplicate "${d.dup}" in ${d.raw}`).join("; ")}`);
      }
    }
    expect(failures, failures.join("\n")).toHaveLength(0);
  });
});

// A plain static file server (`python3 -m http.server`, and
// GitHub Pages in production) resolves a URL to a file by PATH alone -- the
// query string plays no part, so it can only ever serve ONE physical file
// for every `/component/?name=X&` request regardless of `X`
// (`preview/src/site/routes.rs`'s `Route::ComponentDemo` doc comment). There is no
// server-side redirect available at all in this deployment shape, so the
// ONLY thing that can ever land a query-string deep link on the right
// component is the client: `ComponentDemo`'s `use_effect` reading the real
// query string once hydrated and calling `navigator().replace`. Sampled at
// three fixed POSITIONS (first/middle/last of the discovered, sorted list)
// rather than one, for broader but still cheap coverage; deterministic
// across runs of the same build. Three always-listed tests (not a
// dynamic-per-name loop over `COMPONENT_NAMES`, which would generate ZERO
// tests -- silently, not even a visible "skipped" -- when `SSG_SITE_DIR` is
// unset): each resolves its own sample name at RUNTIME and skips itself,
// with a reason, when there is none.
test.describe("hydration parity — legacy query-URL redirect on a static host (row 46/22)", () => {
  const SAMPLE_POSITIONS = [
    ["first", (names: string[]) => names[0]],
    ["middle", (names: string[]) => names[Math.floor(names.length / 2)]],
    ["last", (names: string[]) => names[names.length - 1]],
  ] as const;

  for (const [label, pick] of SAMPLE_POSITIONS) {
    test(`Rule 6 (${label} component): a query-string deep link resolves client-side after hydration`, async ({
      page,
    }) => {
      test.skip(
        COMPONENT_NAMES.length === 0,
        "SSG_SITE_DIR not set -- see this file's header.",
      );
      const name = pick(COMPONENT_NAMES);

      await gotoHydrated(page, `${BASE}/component/?name=${name}&`, { // legacy-route-ok: this test IS the legacy redirect
        timeout: NAV_TIMEOUT,
        waitUntil: "load",
      });

      await expect(page).toHaveURL(new RegExp(`/component/${name}/`));

      const displayName = name.replace(/_/g, " ");
      await expect(
        page.getByRole("heading", { name: displayName, exact: false }).first(),
      ).toBeVisible({ timeout: 15_000 });
      expect(await page.content()).not.toContain("dx-component-demo-not-found");
    });
  }
});
