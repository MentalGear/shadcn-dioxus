import { devices, test } from "./fixtures";
import type { Page } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { BASE_URL } from "./base-url";
import { PLAIN_TRIGGER_PAGES, emptyCoverage, gotoSettled, hoverPass, openPass } from "./computed-style-states";
import type { StateCoverage } from "./computed-style-states";

/**
 * Not a conformance test — a TOOL, run on demand to capture a computed-style
 * snapshot of every themed component page (and a few chrome routes), in BOTH
 * colour schemes.
 *
 * Purpose (docs/backlog.md row 31b, row 111): a literal/ramp -> token
 * migration is meant to be value-PRESERVING. The honest check is not "does it
 * still look plausible" but "did any computed value change at all". Capture
 * before, capture after, diff: an empty diff is the proof.
 *
 * USAGE (needs a server on PLAYWRIGHT_BASE_URL, default http://127.0.0.1:8080;
 * run from `playwright/`; see dev-docs/dev-loop.md "Per-lane servers"):
 *
 *   # 1. capture "before" (any isolated build of the parent commit)
 *   DX_SNAPSHOT=1 CSS_SNAPSHOT_OUT=/abs/before.json \
 *     npx playwright test computed-style-snapshot --project=chromium -g capture
 *   # 2. capture "after" (build of the change)
 *   DX_SNAPSHOT=1 CSS_SNAPSHOT_OUT=/abs/after.json \
 *     npx playwright test computed-style-snapshot --project=chromium -g capture
 *   # 3. diff. Either plain text (one `key: value` per line, stable order):
 *   diff /abs/before.json /abs/after.json          # empty output == proof
 *   #    or the built-in compare, which names mode/page/element/property and
 *   #    FAILS on any difference (full report: <after>.diff.txt):
 *   CSS_SNAPSHOT_A=/abs/before.json CSS_SNAPSHOT_B=/abs/after.json \
 *     npx playwright test computed-style-snapshot --project=chromium -g compare
 *
 * Skipped unless DX_SNAPSHOT is set (capture) or CSS_SNAPSHOT_A + _B are set
 * (compare), so it never runs as part of the suite.
 *
 * Env:
 *   CSS_SNAPSHOT_OUT   output path (alias: DX_SNAPSHOT_OUT; default /tmp/dx-computed.json)
 *   DX_SNAPSHOT_CHROME=0   skip the non-component chrome routes (home, docs, ...)
 *   DX_SNAPSHOT_FOCUS=0    skip the `:focus-visible` pass
 *   DX_SNAPSHOT_HOVER=0    skip the HOVER pass
 *   DX_SNAPSHOT_OPEN=0     skip the OPEN-STATE pass
 *   DX_SNAPSHOT_ONLY=a,b   only these pages: component names, or `route:/path` for a chrome route (debugging aid)
 *   DX_SNAPSHOT_MODES=light  only this colour scheme (debugging aid; the files are then not comparable
 *                            with a two-scheme capture)
 *   DX_SNAPSHOT_PARALLEL=n pages captured concurrently, one browser context each (default 3)
 *   DX_SNAPSHOT_VERBOSE=1  log every page and every overlay that had to be closed by reloading
 *
 * Beside the snapshot, `<OUT>.coverage.json` says what the HOVER and OPEN-STATE passes reached per page and
 * scheme: triggers tried / opened (and by which input), triggers that did nothing WITH THE REASON, items
 * hovered inside opened overlays, hover targets captured / skipped (and why), reloads. It is deliberately
 * not part of the snapshot, so the byte-identical gate never depends on it; read it to see what a green
 * diff actually covers. The last lines of the run print the same totals.
 *
 * OUTPUT FORMAT (version 3, row 135; NOT compatible with version 2: it adds 14 properties and two sections,
 * and `compare` refuses to mix files with different property sets):
 *
 *   { "meta": { format, modes, properties, focusProperties, sections, ... },
 *     "light": { "<page>": { "base": [..], "pseudo": [..], "focus": [..], "hover": [..], "open": [..] }, ... },
 *     "dark":  { same } }
 *
 * `<page>` is a component name, `route:<path>` for chrome routes, with
 * `#frame<k>` appended for the k-th child iframe of that page. Pages are sorted,
 * property keys are sorted, and there is no timestamp, so two captures of an
 * unchanged build are byte-identical (animations/transitions are settled, see
 * `settleAndCapture`).
 *
 * Colour schemes: the site switches on `html[data-theme]` (cookie `dx_theme`,
 * applied by theme.rs `theme_seed`) and, absent that, on `prefers-color-scheme`
 * (dx-components-theme.css). Each mode sets BOTH — the cookie and
 * `page.emulateMedia({ colorScheme })` — and asserts both took effect.
 *
 * THE TWO STATE PASSES (row 135; implementation and the reasoning in
 * `computed-style-states.ts`). They put the page in a state with REAL input and record the DELTA against a
 * settled rest-state baseline: every in-scope element that is new or whose computed style changed.
 *
 *  - `hover`: every distinct interactive control of the Demo frames (links, buttons, role=tab/menuitem/
 *    option/..., inputs, table rows, ...; distinct = same tag, class, role, data-* and aria-* state, position,
 *    digits masked; at most 60 per page) gets the pointer moved onto it (`page.mouse`, so `:hover`
 *    genuinely applies, verified with `matches(":hover")`). Ancestors' and descendants' hover rules show up
 *    because it is a delta, not just the hovered node. A control that opens something on hover (tooltip,
 *    hover card) records the opened content too.
 *  - `open`: for each trigger (DERIVED: `aria-haspopup`, `aria-expanded`, `popovertarget`, `summary`,
 *    tooltip/hover-card `aria-describedby`, `*-trigger` classes, the first focusable inside a
 *    `data-state=closed` root; plus EVERY button on `PLAIN_TRIGGER_PAGES`, whose demos open overlays from
 *    plain `onclick` buttons) try click, right-click (context menus), hover, focus until the page changes;
 *    record the delta (content, its pseudo-elements, `::backdrop` of a top-layer `<dialog>`/`[popover]`);
 *    then hover each distinct item inside it (menu items, options, links) against the open-state
 *    baseline; then close it (pointer away, Escape, re-click) and VERIFY it closed before the next trigger,
 *    reloading the page as the last resort (counted in the coverage file).
 *
 * Records of both sections carry `state` (`hover:<k>`, `open:<k>`, `open:<k>/hover:<j>`; k = index in
 * document order), `via` (the input that opened it) and `target` (a readable description). `compare` matches
 * records by tag/class/pseudo/state, so a restyle that stops an overlay from opening, or changes any
 * recorded value, is reported.
 *
 * Determinism (A/A; every item was found by an A/A failure): every read is preceded by `settle()` (finite
 * animations/transitions finished, infinite ones parked on frame 0) and by a DOM-quiescence wait; the pointer is
 * parked on a fixed inert spot between targets and right after a click (an overlay opens UNDER the pointer, and
 * which node is `:hover` is then a hit-test race); the page is only captured once the legacy-route "Loading
 * component..." stub is gone and every `<img>` settled (`gotoSettled`); `content-visibility: auto` cards are laid
 * out for real (`--dx-cv: visible`), because rendering them as they scroll into view changes document heights by
 * an amount that depends on load; hover-intent delays (submenus, 200 ms) are waited out, not raced; Google Fonts
 * requests are aborted (whether they beat `font-display: optional` changed every text width); the document is
 * reported hidden, which stops every `use_motion`-gated timer (the home music player) and keeps toasts from
 * auto-dismissing; `setInterval` tickers that call `dioxus.send` are frozen.
 *
 * NOT covered: `:active`, `::selection`, `::-webkit-scrollbar*`; checked/pressed states reached by clicking
 * ordinary controls (only overlays/disclosures are clicked); state inside child iframes (the block demos:
 * `sidebar`, ...) for the two new passes; what is listed as "not opened" in the coverage file.
 */

const OUT =
  process.env.CSS_SNAPSHOT_OUT ?? process.env.DX_SNAPSHOT_OUT ?? "/tmp/dx-computed.json";
const CMP_A = process.env.CSS_SNAPSHOT_A;
const CMP_B = process.env.CSS_SNAPSHOT_B;
const DO_CHROME = process.env.DX_SNAPSHOT_CHROME !== "0";
const DO_FOCUS = process.env.DX_SNAPSHOT_FOCUS !== "0";
const DO_HOVER = process.env.DX_SNAPSHOT_HOVER !== "0";
const DO_OPEN = process.env.DX_SNAPSHOT_OPEN !== "0";
const VERBOSE = !!process.env.DX_SNAPSHOT_VERBOSE;
/** Pages captured concurrently, each in its own browser context (default 3). */
const PARALLEL = Number(process.env.DX_SNAPSHOT_PARALLEL) > 0 ? Number(process.env.DX_SNAPSHOT_PARALLEL) : 3;
/** Where the interaction targets of a component page live: the DEMO tab's preview frames (`id` is
 *  `component-preview-frame` or `component-preview-frame-<variant>`, see preview/src/main.rs). The shell around
 *  them (sidebar, theme picker) is identical on every page and is captured by the `route:` pages instead. */
const COMPONENT_SCOPE = '[id^="component-preview-frame"]';
/** Comma list of component names to capture (debugging aid; default all). */
const ONLY = process.env.DX_SNAPSHOT_ONLY?.split(",").filter(Boolean);

const MODES = (["light", "dark"] as const).filter(
  (m) => !process.env.DX_SNAPSHOT_MODES || process.env.DX_SNAPSHOT_MODES.split(",").includes(m),
);
type Mode = (typeof MODES)[number];

/** Non-component routes whose chrome (`main.css`, `hero.css`, dashboard) the
 *  `dx-` selector alone misses. Captured with every classed element. */
const CHROME_ROUTES = ["/", "/docs", "/demos", "/charts/", "/dashboard/email-client"];

/** Colour-bearing computed properties (also the set re-read in `:focus-visible`
 *  and for pseudo-elements). Sorted. */
const COLOR_PROPERTIES = [
  "-webkit-tap-highlight-color",
  "-webkit-text-fill-color",
  "-webkit-text-stroke-color",
  "accent-color",
  "background-color",
  "background-image",
  "backdrop-filter",
  "border-bottom-color",
  "border-color",
  "border-image-source",
  "border-left-color",
  "border-right-color",
  "border-top-color",
  "box-shadow",
  "caret-color",
  "color",
  "color-scheme",
  "column-rule-color",
  "fill",
  "fill-opacity",
  "filter",
  "flood-color",
  "lighting-color",
  "opacity",
  "outline-color",
  "outline-offset",
  "outline-style",
  "outline-width",
  "scrollbar-color",
  "stop-color",
  "stroke",
  "stroke-opacity",
  "stroke-width",
  "text-decoration-color",
  "text-decoration-line",
  "text-emphasis-color",
  "text-shadow",
];

/** Geometry / motion properties (the original row 31b set). */
const LAYOUT_PROPERTIES = [
  // Added with format 3 (row 135): what an open overlay / hovered control changes is often not a colour at
  // all -- it is `display`, where it sits (`position`/`top`/`left`/`transform`/`translate`/`scale`) or whether
  // it takes the pointer. Without them a delta against a closed baseline could not even see content appear.
  "bottom", "cursor", "display", "left", "overflow-x", "overflow-y", "pointer-events", "position", "right",
  "scale", "top", "transform", "translate", "visibility",
  "border-radius", "border-width",
  "font-size", "font-weight", "gap", "height", "line-height",
  "margin-bottom", "margin-left", "margin-right", "margin-top",
  "padding-bottom", "padding-left", "padding-right", "padding-top",
  "transition-duration", "transition-property", "transition-timing-function",
  "width", "z-index",
];

const PROPERTIES = Array.from(new Set([...COLOR_PROPERTIES, ...LAYOUT_PROPERTIES])).sort();
const FOCUS_PROPERTIES = COLOR_PROPERTIES;

type CaptureArgs = {
  selector: string;
  props: string[];
  focusProps: string[];
  mode: string;
  doFocus: boolean;
};

type Rec = Record<string, string | number>;
type Section = { base: Rec[]; pseudo: Rec[]; focus: Rec[]; hover?: Rec[]; open?: Rec[] };
/** Sections compared, in order. `hover`/`open` exist from format 3. */
const SECTIONS = ["base", "pseudo", "focus", "hover", "open"] as const;

/**
 * Runs IN the page (serialised by Playwright: keep it self-contained, no
 * closure over module scope, no TS-emitted helpers such as async/await).
 * Everything after the fonts promise is synchronous, so nothing re-renders
 * between reading the base state and the focus pass.
 */
function settleAndCapture(args: CaptureArgs) {
  return document.fonts.ready.then(() => {
    const w = window as any;
    const settle = () => {
      // Make animated/transitioning values deterministic: finish transitions
      // and finite animations at their end state; park infinite animations on
      // their first frame (e.g. dx-progress-indicator re-sampled 34/54/32/54px
      // on one unchanged build before this).
      for (const a of document.getAnimations()) {
        try {
          const isTransition = w.CSSTransition && a instanceof w.CSSTransition;
          const timing = a.effect ? a.effect.getComputedTiming() : null;
          if (!isTransition && timing && timing.iterations === Infinity) {
            a.pause();
            a.currentTime = 0;
          } else {
            a.finish();
          }
        } catch (_) {
          /* ignore: nothing else to do for an animation we cannot settle */
        }
      }
    };

    const root = document.documentElement;
    // Lay every `content-visibility: auto` card out for real (`--dx-cv`, main.css): with `auto` a card near the
    // viewport is rendered (and its height remembered) at a moment that depends on machine load, and the
    // document height differed by 12 px between two captures of one build taken under load.
    root.style.setProperty("--dx-cv", "visible");
    const themeAttr = root.getAttribute("data-theme");
    const mediaDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    let forced = false;
    if (themeAttr !== args.mode) {
      root.setAttribute("data-theme", args.mode);
      forced = true;
    }
    settle();

    const sorted = (cs: CSSStyleDeclaration, el: Element, props: string[], extra: Rec) => {
      const rec: Rec = { ...extra, tag: el.tagName.toLowerCase(), cls: Array.from(el.classList).sort().join(" ") };
      for (const p of props) rec[p] = cs.getPropertyValue(p);
      return rec;
    };

    const els = Array.from(document.querySelectorAll(args.selector)).filter(
      (e) => !["SCRIPT", "STYLE", "LINK", "META", "TITLE", "HEAD", "NOSCRIPT"].includes(e.tagName.toUpperCase()),
    );
    const base: Rec[] = [];
    const pseudo: Rec[] = [];
    const focus: Rec[] = [];

    els.forEach((el, n) => {
      const cs = getComputedStyle(el);
      // `n` = document-order index; tag + class list are the stable identity.
      base.push(sorted(cs, el, args.props, { n }));

      for (const ps of ["::before", "::after"]) {
        const pcs = getComputedStyle(el, ps);
        const content = pcs.content;
        if (content && content !== "none" && content !== "normal") {
          pseudo.push(sorted(pcs, el, args.props, { n, pseudo: ps }));
        }
      }
      const tag = el.tagName.toLowerCase();
      if (tag === "input" || tag === "textarea") {
        pseudo.push(sorted(getComputedStyle(el, "::placeholder"), el, args.focusProps, { n, pseudo: "::placeholder" }));
      }
      if (cs.display === "list-item") {
        pseudo.push(sorted(getComputedStyle(el, "::marker"), el, args.focusProps, { n, pseudo: "::marker" }));
      }
    });

    const finishTransitions = () => {
      for (const a of document.getAnimations()) {
        try {
          if (w.CSSTransition && a instanceof w.CSSTransition) a.finish();
        } catch (_) {
          /* ignore */
        }
      }
    };
    const result = { themeAttr, mediaDark, forced, section: { base, pseudo, focus } as Section };
    if (!args.doFocus) return result;

    // The focus pass YIELDS to the event loop between elements. Run as one
    // synchronous loop it hung the renderer at 100% CPU on `calendar` (focus
    // handlers of ~200 grid cells in a row; reproduced with a plain
    // focus()/blur() loop, no snapshot code involved, and never when each
    // focus() is its own task). The base/pseudo sections above are already
    // read, so yielding here cannot change them.
    const focusable = 'a[href], button, input, textarea, select, summary, [tabindex], [contenteditable="true"]';
    const targets: [Element, number][] = [];
    els.forEach((el, n) => {
      if (el.matches(focusable) && !el.matches(":disabled")) targets.push([el, n]);
    });
    const step = (k: number): Promise<typeof result> => {
      if (k >= targets.length) return Promise.resolve(result);
      const [el, n] = targets[k];
      if (!el.isConnected) return step(k + 1);
      const h = el as HTMLElement;
      h.focus({ preventScroll: true, focusVisible: true } as FocusOptions);
      if (document.activeElement === h) {
        finishTransitions();
        focus.push(sorted(getComputedStyle(el), el, args.focusProps, { n, state: h.matches(":focus-visible") ? "focus-visible" : "focus" }));
        h.blur();
        finishTransitions();
      }
      return new Promise<void>((r) => setTimeout(r, 0)).then(() => step(k + 1));
    };
    return step(0);
  });
}

/** Sort a record's keys: identity first (n, pseudo, state, tag, cls), then properties A-Z. */
function stable(rec: Rec): Rec {
  const head = ["n", "pseudo", "state", "via", "target", "tag", "cls"];
  const out: Rec = {};
  for (const k of head) if (k in rec) out[k] = rec[k];
  for (const k of Object.keys(rec).sort()) if (!(k in out)) out[k] = rec[k];
  return out;
}

async function capturePage(
  page: Page,
  url: string,
  key: string,
  mode: Mode,
  selector: string,
  into: Record<string, Section>,
  scope: string,
  plain: boolean,
  coverage: Record<string, StateCoverage>,
) {
  const t0 = Date.now();
  if (VERBOSE) console.log(`[snapshot] ${mode} ${key}`);
  // Wait for the wasm client to render (past the legacy-route redirect stub), its stylesheets to attach and
  // theme_seed to run.
  await gotoSettled(page, url);
  const args: CaptureArgs = {
    selector,
    props: PROPERTIES,
    focusProps: FOCUS_PROPERTIES,
    mode,
    doFocus: DO_FOCUS,
  };
  const frames = [page.mainFrame(), ...page.mainFrame().childFrames()];
  for (let k = 0; k < frames.length; k++) {
    const frame = frames[k];
    if (k > 0 && (frame.url() === "about:blank" || frame.url() === "")) continue;
    const r = await frame.evaluate(settleAndCapture, args);
    if (r.mediaDark !== (mode === "dark")) {
      throw new Error(`${key}: prefers-color-scheme did not follow emulateMedia (${mode})`);
    }
    if (r.forced) {
      console.warn(`[snapshot] ${mode} ${key}${k ? `#frame${k}` : ""}: data-theme was ${r.themeAttr}, forced`);
    }
    const sec = r.section;
    into[k === 0 ? key : `${key}#frame${k}`] = {
      base: sec.base.map(stable),
      pseudo: sec.pseudo.map(stable),
      focus: sec.focus.map(stable),
    };
  }

  // HOVER and OPEN-STATE passes (row 135): main frame only, after the rest-state sections above are read.
  // (Both reload / re-park freely, so they run last.)
  const sec = into[key];
  const stateArgs = { sel: selector, scope, props: PROPERTIES, colorProps: COLOR_PROPERTIES, mode };
  const cov = emptyCoverage();
  const t1 = Date.now();
  if (DO_HOVER) sec.hover = (await hoverPass(page, url, key, stateArgs, cov, plain, VERBOSE)).map((r) => stable(r as Rec));
  const t2 = Date.now();
  if (DO_OPEN) sec.open = (await openPass(page, url, key, stateArgs, cov, plain, VERBOSE)).map((r) => stable(r as Rec));
  coverage[key] = cov;
  if (VERBOSE) console.log(`[snapshot]   ${key}: base ${t1 - t0}ms, hover ${t2 - t1}ms (${cov.hoverCaptured}), open ${Date.now() - t2}ms (${cov.opened}/${cov.candidates} triggers, ${cov.itemHovers} items)`);
}

test.describe("computed style snapshot", () => {
  test("capture computed styles for every themed component page", async ({ browser }) => {
    test.skip(!process.env.DX_SNAPSHOT, "snapshot tool; set DX_SNAPSHOT=1 to run");
    test.setTimeout(90 * 60 * 1000);

    const base = path.join(__dirname, "../preview/src/components");
    const names = fs
      .readdirSync(base)
      .filter((n) => fs.existsSync(path.join(base, n, "style.css")))
      .filter((n) => !ONLY || ONLY.includes(n))
      .sort();
    const chrome = (DO_CHROME ? [...CHROME_ROUTES].sort() : []).filter((r) => !ONLY || ONLY.includes(`route:${r}`) || ONLY.length === 0);

    const all: Record<string, unknown> = {
      meta: {
        format: 3,
        modes: [...MODES],
        pages: [...names, ...chrome.map((r) => `route:${r}`)],
        componentSelector: '[class*="dx-"], [class*="dx-"] svg, [class*="dx-"] svg *',
        chromeSelector: "body [class], body svg, body svg *",
        properties: PROPERTIES,
        focusProperties: DO_FOCUS ? FOCUS_PROPERTIES : [],
        sections: SECTIONS.filter((x) => (x === "hover" ? DO_HOVER : x === "open" ? DO_OPEN : true)),
        componentScope: COMPONENT_SCOPE,
        plainTriggerPages: PLAIN_TRIGGER_PAGES,
      },
    };

    // Two demos advance their own state from a `setInterval(() => dioxus.send(...))`
    // ticker (progress: `Math.random() * 30` every 1s; the home music player:
    // elapsed wall-clock every 100ms), so `.dx-progress-indicator` (0/18/26px) and
    // the player's `.dx-slider-range` (116.891 vs 116.922px) differed between two
    // captures of an unchanged build. Their value depends on how many ticks ran
    // before the read, which no wait can make equal. Freeze every such ticker (the
    // demos stay at their initial value); no other preview code uses setInterval.
    const freezeTickers = () => {
      // Gated motion (use_motion / use_interval_while / use_timeout_while, primitives/src/activity.rs) runs only
      // while the document is visible and the element on screen -- the home page's music player ticks once a
      // second while scrolled into view, which a hover capture scrolls it into. Report the document as hidden
      // from the start: the gated timers then never start (a toast never auto-dismisses either), and nothing
      // else in the snapshot depends on visibility.
      try {
        Object.defineProperty(Document.prototype, "visibilityState", { get: () => "hidden", configurable: true });
        Object.defineProperty(Document.prototype, "hidden", { get: () => true, configurable: true });
      } catch (_) {
        /* leave visibility alone */
      }
      const realSetInterval = window.setInterval.bind(window);
      (window as any).setInterval = (handler: any, timeout?: number, ...rest: any[]) =>
        typeof handler === "function" && String(handler).includes("dioxus.send(")
          ? 0
          : realSetInterval(handler, timeout, ...rest);
    };

    const host = new URL(BASE_URL).hostname;
    let total = 0;
    const coverageAll: Record<string, Record<string, StateCoverage>> = {};
    const t0 = Date.now();
    for (const mode of MODES) {
      const pages: Record<string, Section> = {};
      const modeCov: Record<string, StateCoverage> = {};
      coverageAll[mode] = modeCov;

      type Job = { url: string; key: string; selector: string; scope: string; plain: boolean };
      const jobs: Job[] = [
        ...names.map((name) => ({
          url: `${BASE_URL}/component/?name=${name}&`,
          key: name,
          selector: '[class*="dx-"], [class*="dx-"] svg, [class*="dx-"] svg *',
          scope: COMPONENT_SCOPE,
          plain: PLAIN_TRIGGER_PAGES.includes(name),
        })),
        ...chrome.map((route) => ({
          url: `${BASE_URL}${route}`,
          key: `route:${route}`,
          selector: "body [class], body svg, body svg *",
          scope: "body",
          plain: false,
        })),
      ];

      // PARALLEL contexts (row 135: the two state passes made a serial run ~1 h). Each worker has its own
      // browser context -- cookie, colour-scheme emulation and ticker freeze set per context -- and pulls the
      // next page off a shared queue. The result is keyed by page and sorted at the end, so the order the
      // pages finish in cannot reach the output.
      const queue = [...jobs];
      const worker = async () => {
        // The theme is cookie-driven (theme.rs); the cookie path must be `/` whatever path prefix BASE_URL carries.
        const ctx = await browser.newContext({ ...devices["Desktop Chrome"] });
        try {
          await ctx.addCookies([{ name: "dx_theme", value: mode, domain: host, path: "/" }]);
          await ctx.addInitScript(freezeTickers);
          // Web fonts (Google Fonts, the SSG's font preload) are fetched from the network, and whether that
          // beats `font-display: optional`'s block period decides which font the whole page view uses: two
          // loads of one page differed in every text width (navbar brand 132.3 vs 136.7 px). Fail them at once,
          // always: the snapshot is of the fallback fonts, the same on every machine and every run.
          await ctx.route(/^https:\/\/fonts\.(googleapis|gstatic)\.com\//, (route) => route.abort());
          const page = await ctx.newPage();
          await page.emulateMedia({ colorScheme: mode });
          for (let job = queue.shift(); job; job = queue.shift()) {
            await capturePage(page, job.url, job.key, mode, job.selector, pages, job.scope, job.plain, modeCov);
          }
        } finally {
          await ctx.close();
        }
      };
      await Promise.all(Array.from({ length: Math.max(1, Math.min(PARALLEL, jobs.length)) }, worker));

      // Stable page order regardless of insertion order.
      const sortedPages: Record<string, Section> = {};
      for (const k of Object.keys(pages).sort()) sortedPages[k] = pages[k];
      all[mode] = sortedPages;
      const sortedCov: Record<string, StateCoverage> = {};
      for (const k of Object.keys(modeCov).sort()) sortedCov[k] = modeCov[k];
      coverageAll[mode] = sortedCov;
      total += Object.values(sortedPages).reduce((n, s) => n + s.base.length, 0);
    }

    fs.writeFileSync(OUT, JSON.stringify(all, null, 1));
    // Coverage is NOT part of the snapshot (timing-derived reasons must not enter the byte-identical gate);
    // it is the answer to "what did the hover and open-state passes actually reach?".
    const covOut = `${OUT}.coverage.json`;
    fs.writeFileSync(covOut, JSON.stringify(coverageAll, null, 1));
    for (const mode of MODES) {
      const cs = Object.values(coverageAll[mode] ?? {});
      const sum = (f: (c: StateCoverage) => number) => cs.reduce((n, c) => n + f(c), 0);
      console.log(
        `[coverage ${mode}] triggers tried ${sum((c) => c.candidates)}, opened ${sum((c) => c.opened)}, ` +
          `not opened ${sum((c) => c.unopened.length)}, item hovers ${sum((c) => c.itemHovers)} (skipped ${sum((c) => c.itemHoverSkipped)}), ` +
          `hover targets ${sum((c) => c.hoverTargets)} captured ${sum((c) => c.hoverCaptured)}, reloads ${sum((c) => c.reloads)}, ` +
          `records: hover ${sum((c) => c.hoverRecords)} open ${sum((c) => c.openRecords)}`,
      );
    }
    console.log(`elapsed ${Math.round((Date.now() - t0) / 1000)}s; coverage detail: ${covOut}`);
    console.log(`wrote ${OUT} (${names.length} component pages + ${chrome.length} routes, ${MODES.length} modes, ${total} base records)`);
  });

  test("compare two snapshots (empty diff == value-preserving)", async () => {
    test.skip(!(CMP_A && CMP_B), "set CSS_SNAPSHOT_A and CSS_SNAPSHOT_B to compare");
    const a = JSON.parse(fs.readFileSync(CMP_A!, "utf8"));
    const b = JSON.parse(fs.readFileSync(CMP_B!, "utf8"));
    const lines: string[] = [];

    if (JSON.stringify(a.meta?.properties) !== JSON.stringify(b.meta?.properties)) {
      lines.push("meta.properties differ: the two files captured different property sets");
    }

    // Identity of a record: tag|cls|pseudo|state + its occurrence count, so an
    // unrelated inserted element does not shift every later comparison.
    const index = (recs: Rec[]) => {
      const seen = new Map<string, number>();
      const m = new Map<string, Rec>();
      for (const r of recs) {
        const id = `${r.tag}.${stripHash(String(r.cls))}${r.pseudo ?? ""}${r.state ? `:${r.state}` : ""}`;
        const k = (seen.get(id) ?? 0) + 1;
        seen.set(id, k);
        m.set(`${id}#${k}`, r);
      }
      return m;
    };
    const IDENT = new Set(["n", "tag", "cls", "pseudo", "state", "target"]);
    // Build identity, not style: `#[css_module]` appends a hash of the source path
    // to every class (`dx-top-layer-hint-e446a291`, different per worktree), and
    // chart SVG ids carry a build-specific counter (`dxc-220-gradient-mobile`).
    // Two captures of the same sources can differ in both, so strip them.
    const stripHash = (cls: string) => cls.replace(/(\bdx-[a-z0-9-]*?)-[0-9a-f]{8}\b/g, "$1");
    const norm = (v: unknown) => String(v).replace(/\bdxc-\d+-/g, "dxc-#-");

    for (const mode of MODES) {
      const pa: Record<string, Section> = a[mode] ?? {};
      const pb: Record<string, Section> = b[mode] ?? {};
      for (const pg of Array.from(new Set([...Object.keys(pa), ...Object.keys(pb)])).sort()) {
        if (!pa[pg] || !pb[pg]) {
          lines.push(`${mode} ${pg}: page only in ${pa[pg] ? "A" : "B"}`);
          continue;
        }
        for (const sec of SECTIONS) {
          // hover/open exist from format 3; a file that has no such section on this page (older format, or
          // captured with DX_SNAPSHOT_HOVER=0 / DX_SNAPSHOT_OPEN=0) cannot be compared on it.
          if (pa[pg][sec] === undefined && pb[pg][sec] === undefined) continue;
          if (pa[pg][sec] === undefined || pb[pg][sec] === undefined) {
            lines.push(`${mode} ${pg} ${sec}: section only in ${pa[pg][sec] === undefined ? "B" : "A"}`);
            continue;
          }
          const ma = index(pa[pg][sec] ?? []);
          const mb = index(pb[pg][sec] ?? []);
          for (const id of Array.from(new Set([...ma.keys(), ...mb.keys()])).sort()) {
            const ra = ma.get(id);
            const rb = mb.get(id);
            if (!ra || !rb) {
              lines.push(`${mode} ${pg} ${sec} ${id}: only in ${ra ? "A" : "B"}`);
              continue;
            }
            for (const p of Object.keys(ra).filter((k) => !IDENT.has(k)).sort()) {
              if (norm(ra[p]) !== norm(rb[p])) lines.push(`${mode} ${pg} ${sec} ${id} ${p}: ${JSON.stringify(ra[p])} -> ${JSON.stringify(rb[p])}`);
            }
          }
        }
      }
    }

    const report = `${CMP_B}.diff.txt`;
    fs.writeFileSync(report, lines.join("\n") + (lines.length ? "\n" : ""));
    console.log(`${lines.length} differing value(s); full report: ${report}`);
    for (const l of lines.slice(0, 200)) console.log(l);
    if (lines.length) throw new Error(`${lines.length} computed-style difference(s) between A and B (see ${report})`);
  });
});
