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
 * "After the first paint": a frame whose rAF callback runs after `first-contentful-paint` (+ 20 ms of grace). Frames
 * before the first paint are never shown, so what the page settles into before the user sees it is free; every frame
 * after it is on screen, including those of a half-parsed document (see "Progressive rendering" below).
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
 * Progressive rendering (backlog row 214). The prerendered home page is ~580 KB and is parsed in chunks; a browser
 * paints between chunks. The clock of the sweep therefore starts at the FIRST PAINT, not at DOMContentLoaded: a frame
 * painted from a half-parsed document counts like any other. The one thing a half-parsed document may do is grow --
 * an element whose end tag has not arrived is still being filled, so while `document.readyState` is "loading" a pure
 * height change (same x, y and width) is not an event; anything that MOVES or RESIZES sideways is. The row 204 sweep
 * delays only the wasm, so on a fast machine its parse ends before the first paint and this clause is idle there; the
 * second group of tests ("progressive paint") is what makes the parse slow: Chrome's own network and CPU throttling
 * (CDP `Network.emulateNetworkConditions` + `Emulation.setCPUThrottlingRate`), so the document really arrives in
 * chunks and is painted between them, and it records every widget card's position from the first frame in which the
 * card exists. A card must stay in its column and at its place: the multi-column masonry it replaced re-assigned
 * cards to columns on every parsed card (6 to 8 cards changed column in one frame at 1280px, 3 at 768px); the
 * construction that made it pass is that a card's column is its DOM position (`MasonrySlot` in `preview/src/main.rs`).
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
import http from "node:http";

type Box = { x: number; y: number; w: number; h: number };
type BoxEvent = { t: number; f: number; id: number; cls: string; txt: string; from: Box; to: Box };

/** Installed before any page script, so the very first frame is recorded. Reads boxes only. */
const RECORDER = `
(() => {
  const rec = { events: [], frames: 0, hydratedAt: -1 };
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
      const growing = p && !moved && document.readyState === 'loading'; // still being filled by the parser
      const silent = p && !moved && (grew || growing || !paints(e));
      if (p && (moved || Math.abs(p.h - cur.h) > 0.5) && !silent && Math.min(p.y, cur.y) < TOP_PX) {
        if (!ids.has(e)) ids.set(e, ids.size);
        rec.events.push({ t: Math.round(now), f: rec.frames, id: ids.get(e), cls: String(e.getAttribute('class') || e.tagName).slice(0, 40), txt: (e.textContent || '').trim().slice(0, 24), from: p, to: cur });
      }
      prev.set(e, cur);
    }
    requestAnimationFrame(tick);
  }
  requestAnimationFrame(tick);
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
      // goto-ok: measures layout from the first committed byte through hydration; waiting for hydration first would hide the shift
      await page.goto(`${BASE_URL}${route}`, { waitUntil: "commit" });
      await page.waitForSelector('html[data-hydrated="true"]', { state: "attached", timeout: 120_000 });
      await page.evaluate(() => document.fonts.ready.then(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))));
      await page.waitForTimeout(1500); // no input: whatever still moves by itself shows up in this window
      const { events, paint } = await page.evaluate(() => ({
        events: (window as any).__layoutRec.events as BoxEvent[],
        paint: performance.getEntriesByType("paint").map((p) => ({ name: p.name, t: Math.round(p.startTime) })),
      }));
      const firstPaint = (paint.find((p) => p.name === "first-contentful-paint") ?? paint[0])?.t;
      expect(firstPaint, `${route} ${width}px: no paint entry`).toBeDefined();
      // The clock starts at the first paint: a frame painted from a half-parsed document counts (see the header).
      const settledAt = firstPaint!;
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

/**
 * PROGRESSIVE PAINT (backlog row 214): the home page's widget cards, recorded from the first frame in which each card
 * exists, while the document really arrives in chunks. A card may grow downward as its own content is parsed; it may
 * never change column or width, and never move vertically. The wasm is refused so the run is about the parse alone
 * (hydration is covered by the sweep above).
 *
 * Making the parse progressive deterministically: a small local server (`slowDocServer`) sits in front of the SSG
 * site. Every asset (the render-blocking stylesheets, fonts, images) passes straight through, but the home document
 * itself is written in `LS_PP_CHUNK` byte pieces (default 4000) `LS_PP_DELAY` ms apart (default 40) after the head
 * and a `LS_PP_HEAD_WAIT` (default 400 ms) for its stylesheets, so the browser paints, and the hero and the widget
 * masonry then keep arriving under a painting page. (Throttling the whole connection with CDP `Network.emulateNetworkConditions` also makes the parse progressive
 * but delays the stylesheets as much as the body, so the first paint often comes after the document has arrived.) The
 * CPU is throttled as well (`Emulation.setCPUThrottlingRate`, `LS_PP_CPU`, default 4) so a frame can be late the way
 * it is on an iPad. The run first checks that the parse WAS progressive -- cards observed after the first paint in at
 * least three different counts -- otherwise there was nothing to judge and it fails rather than passes.
 *
 * The widths are the iPad portrait and landscape sizes (768, 820, 834, 1024, 1180, 1194, 1366) and the desktop ones
 * the masonry has columns at (1101, 1280, 1440), plus the 2-column / 1-column neighbours 641 and 640.
 * `LS_PP_WIDTHS=768,1280` runs just those widths.
 */
/**
 * A static front for the SSG site that streams `/` slowly and passes every other request through. The document's
 * head (stylesheets, preloads) is sent at once and the server then waits `headWait` ms, so the render-blocking
 * stylesheets have arrived and the first paint can happen before the body does; the body up to the "All components"
 * section (the hero and the whole widget masonry, ~130 KB) is then written `chunk` bytes at a time, `delay` ms
 * apart, and the rest of the page goes out at full speed.
 */
async function slowDocServer(chunk: number, delay: number, headWait: number): Promise<{ url: string; close: () => Promise<void> }> {
  const server = http.createServer(async (req, res) => {
    try {
      const upstream = await fetch(`${BASE_URL}${req.url}`);
      const body = Buffer.from(await upstream.arrayBuffer());
      const headers: Record<string, string> = {};
      upstream.headers.forEach((v, k) => { if (!/^(content-length|content-encoding|transfer-encoding|connection)$/i.test(k)) headers[k] = v; });
      res.writeHead(upstream.status, headers);
      const slowEnd = body.indexOf("dx-catalog-section");
      if (req.url === "/" && /text\/html/.test(headers["content-type"] ?? "") && slowEnd > 0) {
        const bodyStart = body.indexOf("<body");
        res.write(body.subarray(0, bodyStart));
        await new Promise((r) => setTimeout(r, headWait));
        for (let i = bodyStart; i < slowEnd; i += chunk) {
          res.write(body.subarray(i, Math.min(i + chunk, slowEnd)));
          await new Promise((r) => setTimeout(r, delay));
        }
        res.end(body.subarray(slowEnd));
      } else res.end(body);
    } catch {
      res.destroy();
    }
  });
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  const port = (server.address() as { port: number }).port;
  return { url: `http://127.0.0.1:${port}`, close: () => new Promise<void>((r) => { server.closeAllConnections(); server.close(() => r()); }) };
}

const PP_WIDTHS = (process.env.LS_PP_WIDTHS ?? "640,641,768,820,834,1024,1101,1180,1194,1280,1366,1440").split(",").map(Number);

const CARD_RECORDER = `
(() => {
  const rec = { frames: [] };
  window.__cardRec = rec;
  function tick(now) {
    const cards = document.querySelectorAll('.dx-widget-card');
    const boxes = [];
    for (const c of cards) { const b = c.getBoundingClientRect(); boxes.push([Math.round(b.x * 10) / 10, Math.round((b.y + scrollY) * 10) / 10, Math.round(b.width * 10) / 10]); }
    rec.frames.push({ t: Math.round(now), boxes });
    requestAnimationFrame(tick);
  }
  requestAnimationFrame(tick);
})();
`;

test.describe("progressive paint (home widget masonry)", () => {
  for (const width of PP_WIDTHS) {
    test(`cards never change column while the page is parsed (${width}px)`, async ({ browser }) => {
      test.skip(browser.browserType().name() !== "chromium", "CDP throttling is Chromium only");
      test.setTimeout(5 * 60 * 1000);
      const front = await slowDocServer(Number(process.env.LS_PP_CHUNK ?? 4000), Number(process.env.LS_PP_DELAY ?? 40), Number(process.env.LS_PP_HEAD_WAIT ?? 400));
      const context = await browser.newContext({ viewport: { width, height: HEIGHT } });
      await context.addCookies([{ name: "dx_theme", value: "light", url: front.url }]);
      const page = await context.newPage();
      await page.addInitScript(CARD_RECORDER);
      await page.route("**/*.wasm", (route) => route.abort());
      const cdp = await context.newCDPSession(page);
      await cdp.send("Emulation.setCPUThrottlingRate", { rate: Number(process.env.LS_PP_CPU ?? 4) });
      // goto-ok: measures layout from the first committed byte through hydration; waiting for hydration first would hide the shift
      await page.goto(`${front.url}/`, { waitUntil: "commit" });
      await page.waitForLoadState("load", { timeout: 4 * 60 * 1000 });
      await page.waitForTimeout(1000);
      const { frames, fcp } = await page.evaluate(() => ({
        frames: (window as any).__cardRec.frames as { t: number; boxes: number[][] }[],
        fcp: performance.getEntriesByType("paint").find((p) => p.name === "first-contentful-paint")?.startTime ?? -1,
      }));
      await context.close();
      await front.close();
      expect(fcp, `${width}px: no first-contentful-paint entry`).toBeGreaterThan(0);

      const shown = frames.filter((f) => f.t >= fcp);
      const counts = new Set(shown.map((f) => f.boxes.length));
      const total = Math.max(...frames.map((f) => f.boxes.length));
      expect(total, `${width}px: the home page has 14 widget cards`).toBe(14);
      expect(counts.size, `${width}px: cards were seen in only ${counts.size} distinct counts after the first paint -- the parse was not progressive; raise LS_PP_DELAY or lower LS_PP_CHUNK`).toBeGreaterThanOrEqual(3);

      // Each card is judged against the first box it was ever seen with (after the first paint).
      const first = new Map<number, number[]>();
      const problems: string[] = [];
      const reported = new Set<number>();
      for (const f of shown) {
        f.boxes.forEach((b, k) => {
          const a = first.get(k);
          if (!a) { first.set(k, b); return; }
          const what = Math.abs(a[0] - b[0]) > 0.5 ? "column (x)" : Math.abs(a[2] - b[2]) > 0.5 ? "width" : Math.abs(a[1] - b[1]) > 0.5 ? "y" : "";
          if (what && !reported.has(k)) {
            reported.add(k);
            problems.push(`card ${k} ${what}: ${a[0]},${a[1]} w${a[2]} -> ${b[0]},${b[1]} w${b[2]} at ${f.t}ms (${f.boxes.length} cards present)`);
          }
        });
      }
      expect(problems, `${width}px: cards moved while the page was being parsed`).toEqual([]);
    });
  }
});
