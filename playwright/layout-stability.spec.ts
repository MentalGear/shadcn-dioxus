/**
 * LAYOUT STABILITY (dev-docs/backlog.md row 204): after a page's first paint, nothing in its top group may
 * move or resize by itself.
 *
 * The owner saw the top demo group's elements "flicker (re-layout)" at iPad-landscape widths. The cause was
 * never one rule at one width: it was a CLASS -- the page paints, and then something changes a box without
 * any input. Two members were found by recording every rAF frame from the first one (sweep 320..1440, both
 * colour schemes, touch emulation on and off):
 *
 *   1. SSR != hydrated. `Avatar` is prerendered with `data-state="empty"`, i.e. the image AND the fallback
 *      initials side by side in a centred flex row, so the picture sits ~8px left of the circle's centre with
 *      the initials peeking in on the right; when the wasm hydrates, the fallback is removed and the image
 *      jumps back to the centre. Twelve avatars in the home page's widget masonry, at every width.
 *   2. Estimated != real size. The masonry is CSS multi-column, which balances columns by card height, and the
 *      cards were `content-visibility: auto`: a not-yet-rendered card is laid out at its ESTIMATE, then
 *      unskipped over the next frames (the unskip of one card moves the next into range), and every unskip
 *      re-balances the columns -- three to six re-layouts in the first ~300 ms, some after the first paint.
 *
 * The construction: a spec that records the boxes of the top group on EVERY animation frame from the first one
 * (an init script, so the SSR frame is included) and fails when any box changes after the first paint, until
 * hydration has finished and 1.5 s more have passed with no input. A second change of the same box in that
 * window is an oscillation and is reported as such. It does not know which rule did it, so a new member of
 * the class fails here at the width where it shows, whatever its cause.
 *
 * Hydration is held back until the prerendered page has been painted (the wasm request waits for the first paint
 * plus 200 ms), so the prerendered-vs-hydrated differences are on screen at every width, not only on a slow one.
 *
 * "After the first paint": a frame whose rAF callback runs after `first-contentful-paint` AND after
 * DOMContentLoaded (+ one frame of grace). Frames before the first paint are never shown, so what the page
 * settles into before the user sees it is free; frames between the first paint and the end of the parse are the
 * browser painting a half-parsed document (see "Known gap" below).
 * The top group is every element of the navbar, the desktop sidebar and the first 700 elements of
 * `#main-content` that start within the first 3000 page px (the hero, the section header and the whole widget
 * masonry on `/`; the first demo card on a component page). A box counts as changed when x, width or height differs by more than 0.5 px, or the page
 * y by more than 0.5 px (an element pushed down by something above it is a visible jump too). A pure height
 * change of a box that paints nothing of its own (no text, background, border or shadow: the page, the shell,
 * the inset, which grow when a card further down is unskipped) is not counted, nor is one whose bottom edge lies
 * below the first 3000 px (a section that grows because of what is below it); its children are judged alone.
 *
 * Widths: the default sweep is 320..1440 step 32 plus every CSS breakpoint of the preview and its +-1px
 * neighbours plus the iPad widths (768, 820, 834, 1024, 1180, 1194, 1366). `LS_STEP=8` is the full 8px sweep.
 * Dark runs and the secondary routes use the breakpoint/iPad widths only. Measured runtime on the 4-core
 * sandbox against a debug SSG build is in the commit message of the change that added this file.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8108 npx playwright test --config=ssg.local.config.ts layout-stability.spec.ts --workers=3
 *
 * Known gap: progressive rendering. The prerendered home page is ~580 KB and is parsed in chunks; a browser that
 * paints between chunks shows the masonry with only the cards parsed so far, and every card that arrives
 * re-balances the columns (cards jump between columns). Chrome did it in 5 of 8 cold loads at 1280px, with the
 * first paint between 330 and 730 ms and the parse ending ~300 ms later. This spec starts its clock when the parse
 * has finished, so it does not see it; closing it needs a masonry whose column assignment does not depend on how
 * much of it has arrived (backlog row 204, open part).
 *
 * Not covered, on purpose: `/charts/`. Its charts are prerendered at a default width and re-measured when the wasm
 * hydrates (the plot, its axes and the header's select all re-flow ~100-1000ms after the first paint at every
 * width, 120-630 boxes), which is a separate defect in the chart primitive, not in the layout; it is its own
 * backlog item, and adding the route here belongs with that fix.
 *
 * `LS_DUMP=1` prints every late box event of a failing width. `LS_WIDTHS=480,1280` runs just those widths.
 *
 * Chromium only. Debug builds are fine: this counts box changes, not milliseconds.
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";

type Box = { x: number; y: number; w: number; h: number };
type BoxEvent = { t: number; f: number; id: number; cls: string; txt: string; from: Box; to: Box };

/** Installed before any page script, so the very first frame is recorded. Reads boxes only. */
const RECORDER = `
(() => {
  const rec = { events: [], frames: 0, hydratedAt: -1, parsedAt: -1 };
  window.__layoutRec = rec;
  const SEL = '.dx-preview-navbar *, .dx-sidebar-desktop *, #main-content *';
  const TOP_PX = 3000;
  const prev = new Map();
  const paints = (e) => {
    if (/^(IMG|SVG|CANVAS|VIDEO|INPUT|TEXTAREA|SELECT|BUTTON)$/i.test(e.tagName)) return true;
    for (const c of e.childNodes) if (c.nodeType === 3 && c.textContent.trim()) return true;
    const cs = getComputedStyle(e);
    const colour = (v) => v !== 'rgba(0, 0, 0, 0)' && v !== 'transparent';
    return colour(cs.backgroundColor) || cs.backgroundImage !== 'none' || cs.boxShadow !== 'none' ||
      ['Top', 'Right', 'Bottom', 'Left'].some((d) => parseFloat(cs['border' + d + 'Width']) > 0 && cs['border' + d + 'Style'] !== 'none');
  };
  const ids = new Map();
  const r = (n) => Math.round(n * 10) / 10;
  function tick(now) {
    rec.frames++;
    const els = document.querySelectorAll(SEL);
    let main = 0, n = 0;
    for (const e of els) {
      if (e.closest('#main-content') && ++main > 700) continue;
      if (++n > 1000) break;
      const b = e.getBoundingClientRect();
      const cur = { x: r(b.x), y: r(b.y + window.scrollY), w: r(b.width), h: r(b.height) };
      const p = prev.get(e);
      const moved = p && (Math.abs(p.x - cur.x) > 0.5 || Math.abs(p.y - cur.y) > 0.5 || Math.abs(p.w - cur.w) > 0.5);
      // A pure height change of a box that paints nothing itself (the page, the shell, the inset: they grow when
      // a card further down is unskipped) moves nothing that can be seen, nor does one whose bottom edge lies below the top
      // group; the children are judged on their own.
      const grew = p && !moved && Math.min(p.y + p.h, cur.y + cur.h) >= TOP_PX;
      const silent = p && !moved && (grew || !paints(e));
      if (p && (moved || Math.abs(p.h - cur.h) > 0.5) && !silent && Math.min(p.y, cur.y) < TOP_PX) {
        if (!ids.has(e)) ids.set(e, ids.size);
        rec.events.push({ t: Math.round(now), f: rec.frames, id: ids.get(e), cls: String(e.getAttribute('class') || e.tagName).slice(0, 40), txt: (e.textContent || '').trim().slice(0, 24), from: p, to: cur });
      }
      prev.set(e, cur);
    }
    requestAnimationFrame(tick);
  }
  requestAnimationFrame(tick);
  document.addEventListener('DOMContentLoaded', () => { rec.parsedAt = Math.round(performance.now()); });
  new MutationObserver(() => {
    if (rec.hydratedAt < 0 && document.documentElement.dataset.hydrated === 'true') rec.hydratedAt = Math.round(performance.now());
  }).observe(document, { attributes: true, subtree: true, attributeFilter: ['data-hydrated'] });
})();
`;

/** Every width at which the preview's own stylesheets change something (+-1px), and the iPad widths. */
const KEY_WIDTHS = [
  319, 320, 321, 359, 360, 361, 479, 480, 481, 639, 640, 641, 759, 760, 761, 767, 768, 769, 819, 820, 833, 834,
  1023, 1024, 1025, 1099, 1100, 1101, 1180, 1194, 1279, 1280, 1281, 1366,
];
function sweep(step: number): number[] {
  const out: number[] = [];
  for (let w = 320; w <= 1440; w += step) out.push(w);
  return out;
}
const FULL = !!process.env.LS_STEP;
const STEP = Number(process.env.LS_STEP ?? 32);
const ALL_WIDTHS = [...new Set([...sweep(STEP), ...KEY_WIDTHS])].sort((a, b) => a - b);
const HEIGHT = 800;
const ONLY = (process.env.LS_WIDTHS ?? "").split(",").filter(Boolean).map(Number);

const GROUPS: { route: string; scheme: "light" | "dark"; widths: number[] }[] = [
  { route: "/", scheme: "light", widths: ALL_WIDTHS },
  { route: "/", scheme: "dark", widths: FULL ? ALL_WIDTHS : KEY_WIDTHS },
  ...["/docs/", "/demos/", "/component/avatar/", "/component/button/"].map((route) => ({
    route,
    scheme: "light" as const,
    widths: FULL ? ALL_WIDTHS : KEY_WIDTHS,
  })),
  { route: "/component/avatar/", scheme: "dark", widths: FULL ? ALL_WIDTHS : [768, 1024, 1180, 1194, 1366] },
];

const fmt = (b: Box) => `${b.x},${b.y} ${b.w}x${b.h}`;

for (const { route, scheme, widths } of GROUPS) {
  test(`layout stability ${route} ${scheme} (${widths.length} widths)`, async ({ browser }) => {
    test.skip(browser.browserType().name() !== "chromium", "frame-by-frame box recording is measured in Chromium");
    test.setTimeout(45 * 60 * 1000);
    const context = await browser.newContext({ viewport: { width: widths[0], height: HEIGHT }, colorScheme: scheme });
    await context.addCookies([{ name: "dx_theme", value: scheme, url: BASE_URL }]);
    const page = await context.newPage();
    await page.addInitScript(RECORDER);
    // Hydration waits until the prerendered page has been painted (+200 ms): the slow-wasm case, the worst one for
    // a prerendered page, and the only one in which every prerendered-vs-hydrated difference is on screen. Without
    // it a fast or a loaded machine hydrates before its first paint and the same defect passes at that width.
    await page.route("**/*.wasm", async (route) => {
      for (let i = 0; i < 400 && !(await page.evaluate(() => performance.getEntriesByType("paint").length).catch(() => 0)); i++) await page.waitForTimeout(50);
      await page.waitForTimeout(200);
      await route.continue().catch(() => {});
    });
    const failures: string[] = [];

    for (const width of ONLY.length ? ONLY : widths) {
      await page.setViewportSize({ width, height: HEIGHT });
      await page.goto(`${BASE_URL}${route}`, { waitUntil: "commit" });
      await page.waitForSelector('html[data-hydrated="true"]', { state: "attached", timeout: 120_000 });
      await page.evaluate(() => document.fonts.ready.then(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))));
      await page.waitForTimeout(1500); // no input: whatever still moves by itself shows up in this window
      const { events, paint, parsedAt } = await page.evaluate(() => ({
        events: (window as any).__layoutRec.events as BoxEvent[],
        parsedAt: (window as any).__layoutRec.parsedAt as number,
        paint: performance.getEntriesByType("paint").map((p) => ({ name: p.name, t: Math.round(p.startTime) })),
      }));
      const firstPaint = (paint.find((p) => p.name === "first-contentful-paint") ?? paint[0])?.t;
      expect(firstPaint, `${route} ${width}px: no paint entry`).toBeDefined();
      // The 580 KB prerendered document is parsed in chunks and a browser may paint between them; every card parsed
      // later re-balances the multi-column masonry. That is progressive rendering, a separate (and open) matter
      // from a page that changes after it is complete, so the clock starts when the parse has finished too.
      const settledAt = Math.max(firstPaint!, parsedAt);
      const late = events.filter((e) => e.t > settledAt + 20);
      if (late.length === 0) continue;
      if (process.env.LS_DUMP) console.log(`${route} ${scheme} ${width}px`, JSON.stringify(late.slice(0, 40).map((e) => [e.t, e.f, e.cls, e.txt, fmt(e.from), fmt(e.to)])));
      const perBox = new Map<number, BoxEvent[]>();
      for (const e of late) perBox.set(e.id, [...(perBox.get(e.id) ?? []), e]);
      const oscillating = [...perBox.values()].filter((v) => v.length > 1).length;
      const classes = [...new Set(late.map((e) => e.cls))].slice(0, 4).join(" | ");
      const first = late[0];
      failures.push(
        `${width}px: ${perBox.size} box(es) changed after first paint (${oscillating} more than once), ` +
          `frames ${[...new Set(late.map((e) => e.f))].slice(0, 6).join(",")}, e.g. "${first.cls}" ${fmt(first.from)} -> ${fmt(first.to)}; classes: ${classes}`,
      );
    }
    await context.close();
    expect(failures, `${route} (${scheme}): the top group changed by itself after its first paint at ${failures.length} of ${widths.length} widths`).toEqual([]);
  });
}
