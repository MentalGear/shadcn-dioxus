/**
 * ORACLE: one generated sweep over EVERY page of the preview, in light and dark.
 *
 * The class this closes: a11y and layout defects that only ever showed up on a page nobody had a spec
 * for. A sweep of every route in both themes finds in minutes what a hand audit finds slowly: icon-only
 * buttons with no name, an `ItemGroup` of non-`listitem`s, a badge legible in only one theme, demos wider
 * than a phone, no skip link, and no keyboard focus indicator at all in Windows high-contrast
 * (`forced-colors`) mode.
 *
 * The route list is READ from `CATALOG` in `preview/src/components/mod.rs` (the `("name", Origin::..),`
 * rows, exactly as `component-catalog.spec.ts` does), so a new component is covered the day it is added
 * to the catalog -- there is no second list here to forget. (`scripts/check-component-catalog.sh` keeps
 * CATALOG complete against the component directories; this file keeps every CATALOG row covered.) The
 * four site pages (`/`, `/docs`, `/demos`, `/charts/`) ride along.
 *
 * One NAVIGATION per route, for both schemes: the three probe families below share one load, because a
 * navigation-plus-settle is ~2 s on a debug build (rows 327, 358). One test per route loads it once, in light, and
 * runs every probe in both schemes on that load, switching the scheme in place (`ensureScheme`: the theme is attributes and CSS,
 * `__dxApplyMode` is its one writer, so the DOM does not depend on the scheme). The ORDER, and the fresh-load
 * comparison every route was checked against, are in the comment above the sweep loop. A failure in one probe does
 * not hide the others: every probe runs, and the test's error carries each failing probe's own message under its name.
 *
 * For every route, in `light` and in `dark`:
 *  1. AXE. `expectNoAxeViolations` with the repo's tag set, closed state. Known, tracked findings are
 *     listed in `KNOWN_AXE` below, one entry per route and theme, each scoped to the exact selector that
 *     is still wrong and naming its backlog row -- never a blanket rule or route disable. An entry also
 *     asserts that its finding still reproduces, so fixing the defect fails the sweep until the entry is
 *     deleted: the list cannot go stale.
 *  2. NO HORIZONTAL OVERFLOW AT 390px. `html`/`body` are `overflow-x: clip`, so a page never grows a
 *     scrollbar and `scrollWidth` proves nothing; a demo wider than the phone is clipped silently. The
 *     check therefore walks the content for any box whose right edge passes the viewport and that no
 *     ancestor scrolls or clips (a `pre` or a scroll area legitimately does).
 *  3. KEYBOARD, FORCED COLORS. Under `forcedColors: "active"` the browser discards `box-shadow` (which is
 *     how almost every component draws its focus ring) and any `outline: none` stays none, so a control
 *     with no `outline` rule of its own has NO focus indicator. The theme's one `@media (forced-colors:
 *     active)` rule gives every `:focus-visible` element an inset outline; this check proves it on the real
 *     pages: the first Tab stop is the "Skip to content" link, Enter on it moves focus into `<main>`, and
 *     every distinct control signature inside `<main>`, once keyboard-focused, has a visible outline
 *     (computed `outline-style` is not `none`, width > 0). The computed style is faithful here: in forced
 *     colors Chromium reports the discarded `box-shadow` as `none`. A control whose own box is invisible
 *     (the OTP input is an `opacity: 0` overlay) must show focus on its `[data-active]` proxy instead.
 *
 * Settling before a scan. Every scan first disables transitions and animations (and asks for
 * `prefers-reduced-motion: reduce`): the homepage's `content-visibility: auto` cards never finish their
 * 100 ms colour transition while off-screen, and axe then reads the first value of the in-flight
 * transition (the UA's `#0000ee`/`#000`/`#efefef`), reporting contrast failures that are not real.
 *
 * Hermetic. Every request that leaves the dev server is aborted (the Google fonts, the avatar demo's
 * deliberately-pending `httpbin.org` image), so nothing waits on a request that never settles on a machine
 * with a real network, and text metrics are the same everywhere (the system-ui fallback). Hydration is
 * awaited with `gotoHydrated`, never a bare `goto` (see `openRoute` for what else a scan waits for).
 *
 * Filtering while iterating: `npx playwright test --config=baseline.local.config.ts all-routes -g "button"`
 * (test titles start with the route id; they list the probes that route runs).
 *
 * Run against a debug dev server or a release SSG build alike: nothing here is a timing assertion.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { expectNoAxeViolations, type AxeRegionExclusion } from "./axe";
import { readBlockDemos, readCatalog } from "./catalog";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { fixedViewportBox } from "./viewport-box";

type Scheme = "light" | "dark";
const SCHEMES: Scheme[] = ["light", "dark"];
const NAV_TIMEOUT = 5 * 60 * 1000; // the first visit under `dx serve` can wait on a rebuild

// --- the route list, read from Rust --------------------------------------------------------
const CATALOG = readCatalog();
const COMPONENTS = CATALOG.map((r) => r.name);

interface Route {
  id: string;
  path: string;
  /** `block`: a block demo's own page (`/component/block/<name>/<variant>/`), no site chrome around it. */
  kind: "page" | "block";
}
/** Block demos (`variants/demo.css` components) render at `/component/block/<name>/<variant>/`; read from the directories. */
const BLOCK_ROUTES: Route[] = readBlockDemos().map((b) => ({
  id: `block-${b.name}-${b.variant}`,
  path: b.path,
  kind: "block" as const,
}));
const ROUTES: Route[] = [
  ...COMPONENTS.map((name) => ({ id: name, path: `/component/${name}/`, kind: "page" as const })),
  { id: "site-home", path: "/", kind: "page" },
  { id: "site-docs", path: "/docs", kind: "page" },
  { id: "site-demos", path: "/demos", kind: "page" },
  { id: "site-charts", path: "/charts/", kind: "page" },
  ...BLOCK_ROUTES,
];
const PAGE_ROUTES = ROUTES.filter((r) => r.kind === "page");

/**
 * A component page embeds each block demo (Sidebar's) in an iframe that is itself a full page: its own `<main>`,
 * its own sr-only `h1`, its own `banner`. Axe flattens frames, so scanning the host page with them in
 * pulls those landmarks and headings into the host's uniqueness and heading-order checks (`landmark-unique`
 * x3, and `heading-order` for the `h3` that follows an iframe's `h1`), which are not defects of either
 * document. Each block page is scanned on its own (the `block-*` routes), so the framed pages are excluded
 * from the host scan. What axe would check on the `<iframe>` element itself (`frame-title`) is asserted
 * directly instead (`everyFrameHasATitle`), and a calibration test proves that assertion bites.
 */
const EXCLUDE_BLOCK_DEMO_FRAMES: AxeRegionExclusion = {
  selector: "iframe.dx-block-demo-iframe",
  reason: "the framed block demo is a whole page, scanned on its own as `block-<name>-<variant>`; flattened into the host it fakes landmark-unique and heading-order findings",
};

/** The `frame-title` check, for the iframes the exclusion above hides from axe. */
async function everyFrameHasATitle(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    [...document.querySelectorAll("iframe")].filter((f) => !(f.getAttribute("title") ?? "").trim()).map((f) => f.outerHTML.slice(0, 120)),
  );
}

test("the route list parses from CATALOG", () => {
  expect(COMPONENTS.length).toBeGreaterThan(60);
  expect(BLOCK_ROUTES.map((r) => r.path)).toEqual(expect.arrayContaining(["/component/block/sidebar/main/", "/component/block/sidebar/inset/"]));
  expect(COMPONENTS).toEqual(expect.arrayContaining(["button", "item", "badge", "resizable", "scroll_area", "top_layer"]));
  expect(new Set(COMPONENTS).size).toBe(COMPONENTS.length);
  // The Extra pages (`Origin::Extra("reason")` rows) are swept too: `readCatalog` throws when a row does not parse.
  const extras = CATALOG.filter((r) => r.origin === "Extra").map((r) => r.name);
  expect(extras.length).toBeGreaterThan(0);
  expect(COMPONENTS).toEqual(expect.arrayContaining(extras));
});

// --- known, tracked axe findings -----------------------------------------------------------
/**
 * A finding that is real, out of this sweep's scope, and tracked in `dev-docs/backlog.md`. One entry per
 * route and theme; `selector` is the exact subtree that is still wrong (those nodes are excluded from the
 * main scan, everything else on the page is still scanned), `row` the backlog row that owns the fix. The
 * test then asserts the subtree STILL violates, so deleting the defect without deleting the entry fails.
 */
interface KnownAxe {
  selector: string;
  row: string;
  why: string;
}
// Empty: the chart toggle label finding (row 111, dark `--dx-accent` 4.13:1) went away when the active toggle moved to
// `--dx-accent-selected` (phase C). Add an entry here only for a finding that is real and tracked.
const KNOWN_AXE: Record<string, Partial<Record<Scheme, KnownAxe[]>>> = {};

// --- helpers -------------------------------------------------------------------------------
/**
 * Open `route` hermetically, in `scheme`, with motion off, and wait until it is hydrated and settled
 * (`settle`). This is the ONE navigation a (route, scheme) pair gets: every probe below runs on it.
 *
 * "Settled" is checked by what a scan needs, not by `networkidle`: Chromium never reports `networkidle` for
 * a page that embeds the block-demo iframes (`/component/sidebar/`) even when every frame has loaded and
 * hydrated (measured: no request in flight for 30 s, all three iframes `complete` and `data-hydrated`).
 * So: `load`, then the app's own hydration signal (`gotoHydrated`), then `settle`.
 */
async function openRoute(page: Page, route: Route, scheme: Scheme, viewport?: { width: number; height: number }) {
  const origin = new URL(BASE_URL).origin;
  await page.route((url) => url.origin !== origin, (r) => r.abort());
  await page.context().addCookies([{ name: "dx_theme", value: scheme, url: BASE_URL }]);
  await page.emulateMedia({ colorScheme: scheme, reducedMotion: "reduce" });
  if (viewport) await page.setViewportSize(viewport);
  await gotoHydrated(page, BASE_URL + route.path, { waitUntil: "load", timeout: NAV_TIMEOUT });
  await expect(page.locator("html")).toHaveAttribute("data-theme", scheme);
  await settle(page);
}

/**
 * Wait until the page has nothing left to do before a scan: every block-demo iframe hydrated, then every
 * stylesheet loaded and the set of stylesheets unchanged for 400 ms (a `document::Link` is inserted when its
 * component first renders, so a scan must not run before the late ones are in), then motion off in every frame
 * and two animation frames so styles are applied, in-flight transitions cancelled and layout flushed.
 *
 * Also the re-settle after a probe changes the viewport: a component that first renders at the new width (a
 * mobile-only sheet) inserts its stylesheet then, exactly as it does on a page that was loaded at that width.
 * `watchNewSheets: false` (the restore to the desktop viewport) skips the 400 ms quiet window: going back to the
 * width the page was loaded at can only re-render components whose stylesheets are already in the head, so it
 * checks that every stylesheet is loaded and flushes layout, nothing more.
 */
async function settle(page: Page, { watchNewSheets = true }: { watchNewSheets?: boolean } = {}) {
  await page.waitForFunction(
    () =>
      [...document.querySelectorAll("iframe")].every((f) => f.contentDocument?.documentElement.dataset.hydrated === "true"),
    null,
    { timeout: 120_000 },
  );
  const sheets = () =>
    page.evaluate(() => {
      const links = [...document.querySelectorAll('link[rel="stylesheet"]')] as HTMLLinkElement[];
      return { count: links.length, loaded: links.every((l) => l.sheet !== null) };
    });
  await expect
    .poll(
      async () => {
        const before = await sheets();
        if (!watchNewSheets) return before.loaded;
        await page.waitForTimeout(400);
        const after = await sheets();
        return before.loaded && after.loaded && before.count === after.count;
      },
      { timeout: 120_000, intervals: [100] },
    )
    .toBe(true);
  // Motion off in every frame (the block demos are iframes with their own document). Idempotent: a re-settle
  // after a resize must not stack a second tag.
  for (const frame of page.frames()) {
    await frame.evaluate(() => {
      if (document.getElementById("all-routes-no-motion")) return;
      const s = document.createElement("style");
      s.id = "all-routes-no-motion";
      s.textContent =
        "*,*::before,*::after{transition:none !important;animation-duration:0s !important;" +
        "animation-delay:0s !important;animation-iteration-count:1 !important;scroll-behavior:auto !important}";
      document.head.appendChild(s);
    });
  }
  // Two frames: styles applied, in-flight transitions cancelled, layout flushed.
  await page.evaluate(() => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))));
}

/**
 * Put the loaded page in `scheme` in place, as a fresh load in that scheme would have it (row 358): the cookie (so a
 * page opened from here, the fresh phone loads, reads the same), the media query, and the one mode writer
 * `__dxApplyMode` (`preview/src/theme.rs`'s PREPAINT_JS) in every frame, the block-demo iframes included; then the
 * usual settle. A no-op when the page is already in `scheme`. Every probe starts with it, so a probe that failed
 * half-way cannot leave the next one judging the wrong scheme.
 */
async function ensureScheme(page: Page, scheme: Scheme) {
  if ((await page.locator("html").getAttribute("data-theme")) === scheme) return;
  await page.context().addCookies([{ name: "dx_theme", value: scheme, url: BASE_URL }]);
  await page.emulateMedia({ colorScheme: scheme }); // the other emulated features (reduced motion, forced colors) stay
  for (const frame of page.frames()) {
    await frame.evaluate((m) => (window as unknown as { __dxApplyMode?: (m: string) => void }).__dxApplyMode?.(m), scheme);
  }
  await expect(page.locator("html")).toHaveAttribute("data-theme", scheme);
  await settle(page);
}

/** Does `selector` still violate any rule? (For the "known finding is still real" assertion.) */
async function stillViolates(page: Page, selector: string): Promise<boolean> {
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"])
    .options({ resultTypes: ["violations"] })
    .include(selector)
    .analyze();
  return results.violations.length > 0;
}

// --- the probes ----------------------------------------------------------------------------
// Each probe asserts what a stand-alone test for it would assert, with the same messages. A (route, scheme) pair
// is navigated ONCE and the probes run on that load, in the order of `PROBE ORDER` below.

/** 1. axe, closed state, at the default viewport, with the page as it was loaded. */
async function probeAxe(page: Page, route: Route, scheme: Scheme) {
  await ensureScheme(page, scheme);
  const known = KNOWN_AXE[route.id]?.[scheme] ?? [];
  await expectNoAxeViolations(page, `all-routes: ${route.id} (${scheme})`, {
    excludeRegions: [EXCLUDE_BLOCK_DEMO_FRAMES, ...known.map((k) => ({ selector: k.selector, reason: `${k.row}: ${k.why}` }))],
  });
  expect(await everyFrameHasATitle(page), "every <iframe> has a title (axe frame-title)").toEqual([]);
  for (const k of known) {
    expect(
      await stillViolates(page, k.selector),
      `${route.id} (${scheme}): known finding \`${k.selector}\` (${k.row}) no longer reproduces -- delete its KNOWN_AXE entry`,
    ).toBe(true);
  }
}

/**
 * Routes whose DOM depends on the viewport they have HAD, not only the one they have: measured against a
 * fresh-load-at-390px reading, a viewport round trip leaves a different set of boxes on these three
 * (`virtual_grid` renders 2,955 elements after the round trip against 2,784 on a fresh phone load, `site-home` 4,799
 * against 4,763 from `content-visibility: auto` sections, `message_scroller` moves 10 boxes), while on every other
 * route the round trip is box-for-box identical. Their phone-width reading therefore comes from a second, fresh
 * page loaded at 390px. `collapsible` and `tabs` are in the set too (row 358): the in-place scheme switch
 * is a second re-layout, and after one their closed find-in-page panels (`hidden="until-found"`) lay out 1-2 boxes
 * that no ancestor clips (a second viewport round trip on a fresh page does the same), so their dark reading would
 * not be a fresh dark page's. Every other route's switched reading has exactly the fresh load's unclipped boxes.
 */
const FRESH_PHONE_LOAD = new Set(["virtual_grid", "site-home", "message_scroller", "collapsible", "tabs"]);
const PHONE = { width: 390, height: 844 };

/** The 390px reading itself: which boxes pass the viewport and are neither scrollable nor clipped by an ancestor. */
async function readOverflow(page: Page) {
  const bound = (await fixedViewportBox(page)).width;
  return page.evaluate((vw) => {
    const de = document.documentElement;
    const clipped = (e: Element) => {
      for (let p = e.parentElement; p && p !== document.body; p = p.parentElement) {
        if (/(auto|scroll|hidden|clip)/.test(getComputedStyle(p).overflowX)) return true;
      }
      return false;
    };
    const offenders: string[] = [];
    for (const e of document.querySelectorAll(document.querySelector("main") ? "main *" : "body *")) {
      const r = e.getBoundingClientRect();
      if (r.width === 0 || r.height === 0) continue;
      if (r.right <= vw + 1 || clipped(e) || getComputedStyle(e).position === "fixed") continue;
      const cls = (e.getAttribute("class") ?? "").split(/\s+/).slice(0, 2).join(".");
      offenders.push(`${e.tagName.toLowerCase()}${cls ? "." + cls : ""} (right ${Math.round(r.right)}, width ${Math.round(r.width)})`);
    }
    // viewport-ok: page-level sideways scroll is `scrollWidth` vs the document's own `clientWidth`, not an overlay bound
    return { vw, pageScroll: de.scrollWidth - de.clientWidth, offenders };
  }, bound);
}

/**
 * 2. no horizontal overflow at 390px, in every scheme of `schemes`. Runs FIRST on its load: it resizes the page to a
 * phone ONCE, re-settles, reads each scheme (switched in place at the phone width), and restores the viewport, so it
 * must see the page before anything else has touched it (axe makes closed accordions lay out their content; the
 * forced-colors sweep focuses, scrolls and writes the URL). Three routes read a fresh phone-width page per scheme
 * instead (`FRESH_PHONE_LOAD`).
 *
 * The reading after the switch is not box-for-box the fresh dark load's (row 358): any later re-layout of the page --
 * the switch, or simply a second viewport round trip on a fresh page -- changes the boxes of CLOSED accordion content
 * (its `overflow: hidden` wrapper drops to zero size), and a second round trip on a fresh dark page gives exactly the
 * switched reading. Those boxes sit under an `overflow: hidden` ancestor, which `readOverflow` never counts, so the
 * set of boxes that can offend is the same (the two routes where it is not are in `FRESH_PHONE_LOAD`): checked on every route (`dev-docs/archive/research/row-358-one-navigation-both-schemes.md`).
 */
async function probeOverflow(page: Page, route: Route, schemes: Scheme[]) {
  const results: Partial<Record<Scheme, Awaited<ReturnType<typeof readOverflow>>>> = {};
  if (FRESH_PHONE_LOAD.has(route.id)) {
    for (const scheme of schemes) {
      const phone = await page.context().newPage();
      try {
        await openRoute(phone, route, scheme, PHONE);
        results[scheme] = await readOverflow(phone);
      } finally {
        await phone.close();
      }
    }
    // The cookie now names the last scheme; the shared page keeps its own (ensureScheme reads `data-theme`).
  } else {
    // viewport-ok: remembers the size to restore with `setViewportSize`, not a bound for any box
    const desktop = page.viewportSize()!;
    await page.setViewportSize(PHONE);
    await settle(page);
    for (const scheme of schemes) {
      await ensureScheme(page, scheme);
      results[scheme] = await readOverflow(page);
    }
    // Back to the desktop viewport for the probes after this one (axe and the forced-colors sweep judged a desktop page).
    await page.setViewportSize(desktop);
    await settle(page, { watchNewSheets: false });
  }
  const verdict = (r: Awaited<ReturnType<typeof readOverflow>>) => ({ pageScroll: Math.max(r.pageScroll, 0), offenders: r.offenders.slice(0, 6) });
  const vw = Object.values(results)[0]!.vw;
  expect(
    Object.fromEntries(schemes.map((s) => [s, verdict(results[s]!)])),
    `page-level horizontal scroll, and content wider than the ${vw}px viewport and not scrollable or clipped by an ancestor, per scheme`,
  ).toEqual(Object.fromEntries(schemes.map((s) => [s, { pageScroll: 0, offenders: [] }])));
}

/**
 * 3. keyboard focus in forced colors, in every scheme of `schemes`, on ONE keyboard walk. Turns `forcedColors: "active"`
 * on for the rest of the load and moves focus, scrolls and writes `#main-content` into the URL, so it comes after axe
 * and overflow, which judge the page's normal rendering.
 *
 * Why one walk for both schemes (row 358): the skip-link check presses Tab from the document's focus start point, and
 * a first walk moves that point into `<main>`; no reset tried was exact (blur, strip the hash, scroll to 0, a throw-away
 * button prepended and removed: 81 of 83 controls on the home page). A second sweep is not the same either: the first
 * leaves a roving-tabindex container (Menubar) handing focus to its item, so a second pick or a second `focus()` of it
 * judges something else. So every element is focused ONCE and its outline read in each scheme while it holds focus,
 * the scheme flipped in the page (`installFlipScheme`), which needs no style settle: `getComputedStyle` recalculates.
 * Every route was compared with a fresh load in each scheme (`dev-docs/archive/research/row-358-one-navigation-both-schemes.md`).
 */
async function probeForcedColors(page: Page, schemes: Scheme[]) {
  await page.emulateMedia({ forcedColors: "active" });
  expect(await page.evaluate(() => matchMedia("(forced-colors: active)").matches), "forced colors is emulated").toBe(true);
  await installFlipScheme(page);

  // The skip link: the very first Tab stop, with an indicator in every scheme, and it lands focus in <main>.
  await page.keyboard.press("Tab");
  const first = await page.evaluate((schemes) => {
    const flip = (window as unknown as FlipWindow).__allRoutesFlipScheme;
    const a = document.activeElement as HTMLElement | null;
    return {
      cls: a?.className ?? "",
      href: a?.getAttribute("href") ?? "",
      outline: a
        ? flip(schemes, () => {
            const cs = getComputedStyle(a);
            return cs.outlineStyle !== "none" && parseFloat(cs.outlineWidth) > 0 ? "visible" : `${cs.outlineStyle} ${cs.outlineWidth}`;
          })
        : {},
      target: document.getElementById("main-content")?.tagName ?? null,
    };
  }, schemes);
  expect(first.cls, "the first Tab stop is the skip link").toContain("dx-skip-link");
  expect(first.href).toBe("#main-content");
  expect(first.outline, "the focused skip link's outline (style and width > 0), per scheme").toEqual(
    Object.fromEntries(schemes.map((s) => [s, "visible"])),
  );
  expect(first.target, "the skip link's target exists and is a <main>").toBe("MAIN");
  await page.keyboard.press("Enter");
  expect(
    await page.evaluate(() => document.activeElement?.id),
    "Enter on the skip link moves focus to <main id=main-content>",
  ).toBe("main-content");
  await page.keyboard.press("Tab");
  const inMain = await page.evaluate(() => {
    const main = document.getElementById("main-content")!;
    const a = document.activeElement;
    const hasTabbable = !!main.querySelector(
      'a[href], button:not([disabled]), input:not([disabled]):not([type=hidden]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    );
    return { hasTabbable, inside: !!a && main.contains(a) && a !== main };
  });
  if (inMain.hasTabbable) {
    expect(inMain.inside, "the Tab after the skip link lands inside <main>, not back in the nav or sidebar").toBe(true);
  }

  // Every distinct control signature in <main>, focused by keyboard, has a visible indicator in every scheme.
  const failures = await page.evaluate((schemes) => {
    const flip = (window as unknown as FlipWindow).__allRoutesFlipScheme;
    const SELECTOR =
      'a[href], button:not([disabled]), input:not([disabled]):not([type=hidden]), select:not([disabled]), textarea:not([disabled]), summary, [tabindex]:not([tabindex="-1"]), [role=button], [role=tab], [role=switch], [role=checkbox], [role=radio], [role=slider], [role=menuitem], [role=option]';
    const mains = [...document.querySelectorAll("main")];
    const candidates = mains
      .flatMap((m) => [...m.querySelectorAll<HTMLElement>(SELECTOR)])
      .filter((e) => {
        const r = e.getBoundingClientRect();
        const cs = getComputedStyle(e);
        return (
          r.width > 0 &&
          r.height > 0 &&
          cs.visibility !== "hidden" &&
          !e.closest('[inert], [aria-hidden="true"], pre, .dx-preview-code-theme, [data-slot="sidebar"]')
        );
      });
    const seen = new Set<string>();
    const out: Record<string, string[]> = Object.fromEntries(schemes.map((s) => [s, []]));
    const visibleOutline = (e: Element) => {
      const cs = getComputedStyle(e);
      return cs.outlineStyle !== "none" && parseFloat(cs.outlineWidth) > 0;
    };
    let checked = 0;
    for (const e of candidates) {
      const dxClasses = (e.getAttribute("class") ?? "").split(/\s+/).filter((c) => c.startsWith("dx-")).slice(0, 2).join(".");
      const sig = [e.tagName, e.getAttribute("role") ?? "", dxClasses, e.getAttribute("type") ?? "", e.getAttribute("data-style") ?? ""].join("|");
      if (seen.has(sig)) continue;
      seen.add(sig);
      e.scrollIntoView({ block: "center", behavior: "instant" });
      e.focus();
      if (document.activeElement !== e) continue; // not focusable right now (hidden until hover, etc.)
      if (!e.matches(":focus-visible")) {
        for (const s of schemes) out[s].push(`${sig}: focused by script after a Tab but :focus-visible does not match (cannot be judged)`);
        continue;
      }
      checked++;
      const verdicts = flip(schemes, () => {
        let ok = visibleOutline(e);
        if (!ok && getComputedStyle(e).opacity === "0") {
          // An invisible overlay input (InputOtp): focus shows on the active proxy inside its group.
          const proxy = e.parentElement?.querySelector('[data-active="true"]');
          ok = !!proxy && visibleOutline(proxy);
        }
        if (ok) return "";
        const cs = getComputedStyle(e);
        return `${sig}: outline ${cs.outlineStyle} ${cs.outlineWidth}`;
      });
      for (const s of schemes) if (verdicts[s]) out[s].push(verdicts[s]);
      if (checked >= 120) break;
    }
    (document.activeElement as HTMLElement | null)?.blur();
    return out;
  }, schemes);
  expect(failures, "controls whose keyboard focus is invisible in forced-colors mode, per scheme").toEqual(
    Object.fromEntries(schemes.map((s) => [s, []])),
  );
}

type FlipWindow = { __allRoutesFlipScheme: <T>(schemes: string[], read: () => T) => Record<string, T> };
/**
 * Installs `window.__allRoutesFlipScheme(schemes, read)`: runs `read` once per scheme with the page flipped to it by
 * the one mode writer (`window.__dxApplyMode`, PREPAINT_JS), then restores the mode the page had. In-page and
 * synchronous, so a focused element keeps focus and no settle is needed. That is the whole scheme as CSS sees it:
 * `html[data-theme]` decides every theme token and outranks the `prefers-color-scheme` defaults on `:root`, so the
 * emulated media query (left as loaded) does not take part.
 */
async function installFlipScheme(page: Page) {
  await page.evaluate(() => {
    const w = window as unknown as FlipWindow & { __dxApplyMode: (m: string | null) => void };
    w.__allRoutesFlipScheme = (schemes, read) => {
      const before = document.documentElement.getAttribute("data-theme");
      const out: Record<string, ReturnType<typeof read>> = {};
      try {
        for (const s of schemes) {
          w.__dxApplyMode(s);
          out[s] = read();
        }
      } finally {
        w.__dxApplyMode(before);
      }
      return out;
    };
  });
}

// --- 3b. the active option of a listbox that never takes focus ------------------------------
// Combobox and Command keep focus in their input and mark the active option with `data-highlighted`
// (`aria-activedescendant`). Its highlight is a background change, which forced colors flattens, so the
// theme's forced-colors rule also outlines `[data-highlighted="true"]`. (Menus, Select and Menubar move real
// focus onto their items, which the focus sweep above already covers.)
/** How to put each list's input in focus (Command is a dialog opened from a button; its palette is outside `<main>`). */
const ACTIVEDESCENDANT_LISTS: Record<string, (page: Page) => Promise<void>> = {
  combobox: async (page) => {
    await page.locator('main input[role="combobox"]').first().focus();
  },
  command: async (page) => {
    await page.getByRole("button", { name: "Open Command Palette" }).focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog")).toBeVisible();
  },
};
/**
 * 3b runs LAST on its load: it types into the list and (Command) opens a dialog, so nothing may read the page
 * after it. Forced colors is already on from `probeForcedColors` (3b needs it).
 * The option is highlighted once and its outline read in every scheme, switched in place (row 358).
 */
async function probeActiveDescendant(page: Page, focusInput: (page: Page) => Promise<void>, schemes: Scheme[]) {
  await page.emulateMedia({ forcedColors: "active" });
  await page.keyboard.press("Tab"); // keyboard modality
  await focusInput(page);
  await page.keyboard.press("ArrowDown");
  const option = page.locator('[role="option"][data-highlighted="true"]').first();
  await expect(option).toBeVisible();
  const outlines: Partial<Record<Scheme, string>> = {};
  for (const scheme of schemes) {
    await ensureScheme(page, scheme);
    await expect(option).toBeVisible();
    outlines[scheme] = await option.evaluate((e) => {
      const cs = getComputedStyle(e);
      return cs.outlineStyle !== "none" && parseFloat(cs.outlineWidth) > 0 ? "visible" : `${cs.outlineStyle} ${cs.outlineWidth}`;
    });
  }
  expect(outlines, "the highlighted option's outline (style and width > 0), per scheme").toEqual(
    Object.fromEntries(schemes.map((s) => [s, "visible"])),
  );
}

/**
 * Run every probe even when an earlier one fails, so a red route reports ALL of its findings the way
 * one test per probe would. A single failure is rethrown as it was thrown (Playwright's own matcher output,
 * same message); several are joined, each under its probe's name. Each probe is also a `test.step`, so the
 * report shows which of them failed.
 */
async function runProbes(probes: Array<[title: string, run: () => Promise<void>]>) {
  const failed: Array<{ title: string; error: unknown }> = [];
  for (const [title, run] of probes) {
    try {
      await test.step(title, run);
    } catch (error) {
      failed.push({ title, error });
    }
  }
  if (failed.length === 1) throw failed[0].error;
  if (failed.length > 1) {
    throw new Error(
      failed.map((f) => `[${f.title}]\n${f.error instanceof Error ? f.error.message : String(f.error)}`).join("\n\n----\n\n"),
    );
  }
}

// --- the sweep: one navigation per route, both schemes on it ------------------------------------
// PROBE ORDER, and why it is this one. Row 327 measured each order against a fresh load per probe on every
// route (`dev-docs/archive/research/row-327-all-routes-and-goto-hydrated.md`): axe's full result and the 390px reading must
// come out as they did on a fresh load, and the forced-colors sweep may only gain controls, never lose or fail one.
// Row 358 checked the in-place scheme switch the same way, every route against a fresh load in each scheme
// (`dev-docs/archive/research/row-358-one-navigation-both-schemes.md`).
//   1. overflow   resizes to 390px once, re-settles, reads light, switches, reads dark, restores the viewport. First,
//                 because axe and the sweep both change what a later geometry read sees (axe: closed accordions lay
//                 out their content, `content-visibility` sections render; the sweep: focus, scroll, URL hash).
//   2. axe        dark (the page is in dark after 1), then light: the page at the default viewport with nothing
//                 focused and no forced colors. A viewport round trip and a scheme switch do not change its result.
//   3. forced colors  emulates forced colors, presses Tab/Enter ONCE, reads the skip link and sweeps the controls in
//                 light and in dark (see `probeForcedColors` for why one walk). After axe, which judges normal rendering.
//   3b. active option  combobox/command only: focuses an input and opens the palette, so the very last.
for (const route of ROUTES) {
  const focusInput = ACTIVEDESCENDANT_LISTS[route.id] as ((page: Page) => Promise<void>) | undefined;
  const title = [
    "axe",
    "no horizontal overflow at 390px",
    ...(route.kind === "page" ? ["skip link and visible focus in forced colors"] : []),
    ...(focusInput ? ["the active option shows an outline in forced colors"] : []),
  ].join(" + ");
  test(`${route.id}: ${title} (${SCHEMES.join(", ")})`, async ({ page }) => {
    test.setTimeout(NAV_TIMEOUT);
    await openRoute(page, route, SCHEMES[0]);
    const schemesLabel = SCHEMES.join(", ");
    await runProbes([
      [`no horizontal overflow at 390px (${schemesLabel})`, () => probeOverflow(page, route, SCHEMES)],
      ...[...SCHEMES].reverse().map((scheme) => [`axe (${scheme})`, () => probeAxe(page, route, scheme)] as [string, () => Promise<void>]),
      ...(route.kind === "page"
        ? [[`skip link and visible focus in forced colors (${schemesLabel})`, () => probeForcedColors(page, SCHEMES)] as [string, () => Promise<void>]]
        : []),
      ...(focusInput
        ? [[`the active option shows an outline in forced colors (${schemesLabel})`, () => probeActiveDescendant(page, focusInput, SCHEMES)] as [string, () => Promise<void>]]
        : []),
    ]);
  });
}

// --- 4. <html lang> is English, and there is no site-wide language switcher (row 354) ---------
// The site is English only: the header has no language control, `<html lang>` stays `en` on a route change
// (the `internationalized` Calendar/Date Picker demos mark their own subtree, see date-picker.spec.ts).
test("<html lang> is en, no header language switcher, and both survive navigating to another page", async ({ page }) => {
  test.setTimeout(NAV_TIMEOUT);
  await openRoute(page, { id: "button", path: "/component/button/", kind: "page" }, "light");
  const lang = page.locator("html");
  await expect(lang).toHaveAttribute("lang", "en");
  await expect(page.getByLabel("Language", { exact: true })).toHaveCount(0);
  await page.locator('a[data-sidebar="menu-button"][href*="/component/badge/"]:visible').click();
  await expect(page.locator(".dx-component-page-header h1")).toBeVisible();
  await expect(lang).toHaveAttribute("lang", "en");
  await expect(page.getByLabel("Language", { exact: true })).toHaveCount(0);
});

// --- 5. calibration: the frame-title assertion bites ----------------------------------------
// The host-page scan excludes the framed pages (EXCLUDE_BLOCK_DEMO_FRAMES), so `frame-title` is asserted by
// `everyFrameHasATitle`. Prove it: the real page passes, and the same page with a title removed does not.
test("calibration: an untitled block-demo iframe is caught on the host page", async ({ page }) => {
  test.setTimeout(NAV_TIMEOUT);
  await openRoute(page, { id: "sidebar", path: "/component/sidebar/", kind: "page" }, "light");
  await expect(page.locator("iframe.dx-block-demo-iframe")).not.toHaveCount(0);
  expect(await everyFrameHasATitle(page)).toEqual([]);
  await page.evaluate(() => document.querySelector("iframe.dx-block-demo-iframe")!.removeAttribute("title"));
  expect(await everyFrameHasATitle(page)).toHaveLength(1);
});
