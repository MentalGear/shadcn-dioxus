import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

/**
 * Backlog row 211 / next-round plan item 9: charts flickered on load. The SSG paints the final
 * chart; hydration then set `data-animate="true"` on the already painted chart, which restarted
 * the bars / lines / arcs from zero (full -> gone -> animate).
 *
 * The decision (`primitives/src/chart/components/chart.rs`, `plays_entrance`):
 *   - a chart on screen in the FIRST PAINT never animates;
 *   - a chart below the fold animates ONCE, when it is first seen entering the view
 *     (`activity.rs`'s `use_entered_view_when`); scrolling it out and back does not replay;
 *   - `prefers-reduced-motion` and no JS: the final drawing, always.
 *
 * The recorder below runs from before the page's own scripts, samples every `requestAnimationFrame`
 * and, for every chart on screen, compares a signature of the computed state of its marks (bar
 * transform, series clip-path, radar polygon transform, pie sweep dash, arc-label opacity) with the
 * state of the very first frame it was seen in (the server-rendered, final one), and counts the
 * CSS animations it runs. A collapsed frame (scale 0, clipped, dash reset, transparent labels) or
 * any running animation on an above-the-fold chart is a failure.
 *
 * Functional, not timing: it asserts what is painted on EVERY frame, so a debug build is fine.
 */

const NAV = { timeout: 20 * 60 * 1000 };

declare global {
  interface Window {
    __chartRec: {
      frames: number;
      first: Record<number, string>;
      bad: { t: number; i: number; why: string; sig?: string }[];
      anims: Record<number, number>;
      lastAnimAt: Record<number, number>;
      names: Record<number, string[]>;
      measured: Record<number, string | null>;
      animateAttr: Record<number, string | null>;
    };
  }
}

/** Installed with `addInitScript`: runs before any page script, in every document. */
function installRecorder() {
  const rec: Window["__chartRec"] = {
    frames: 0,
    first: {},
    bad: [],
    anims: {},
    lastAnimAt: {},
    names: {},
    measured: {},
    animateAttr: {},
  };
  window.__chartRec = rec;
  const seen = new WeakSet<object>();
  const t0 = performance.now();
  const onScreen = (el: Element) => {
    const r = el.getBoundingClientRect();
    // viewport-ok: vertical extent of the visible page; the scrollbar gutter is horizontal only
    return r.width > 0 && r.height > 0 && r.bottom > 0 && r.top < innerHeight;
  };
  const sig = (svg: Element) => {
    const css = (sel: string, prop: string) => {
      const el = svg.querySelector(sel);
      return el ? (getComputedStyle(el) as unknown as Record<string, string>)[prop] : "-";
    };
    return [
      css('[data-slot="chart-bar"]', "transform"),
      css('[data-slot="chart-series"]', "clipPath"),
      css('[data-kind="radar"] [data-series] > *', "transform"),
      css('[data-slot="chart-sweep-mask"] path', "strokeDasharray"),
      css('[data-slot="chart-arc-labels"]', "opacity"),
    ].join("|");
  };
  const tick = () => {
    const svgs = Array.from(document.querySelectorAll('[data-slot="chart-svg"]'));
    const t = Math.round(performance.now() - t0);
    rec.frames++;
    svgs.forEach((svg, i) => {
      rec.measured[i] = svg.getAttribute("data-measured");
      rec.animateAttr[i] = svg.getAttribute("data-animate") ?? rec.animateAttr[i] ?? null;
      const fresh = (svg.getAnimations({ subtree: true }) as Animation[]).filter(
        (a) => typeof CSSAnimation !== "undefined" && a instanceof CSSAnimation && !seen.has(a),
      );
      for (const a of fresh) {
        seen.add(a);
        const tgt = (a.effect as KeyframeEffect | null)?.target as Element | null;
        (rec.names[i] ??= []).push(`${(a as CSSAnimation).animationName}@${tgt?.getAttribute("data-slot") ?? tgt?.tagName}`);
      }
      if (fresh.length) {
        rec.anims[i] = (rec.anims[i] ?? 0) + fresh.length;
        rec.lastAnimAt[i] = t;
      }
      if (!onScreen(svg)) return;
      const s = sig(svg);
      if (!(i in rec.first)) rec.first[i] = s;
      else if (s !== rec.first[i]) rec.bad.push({ t, i, why: "signature changed", sig: s });
    });
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}

type Rec = Window["__chartRec"];
const rec = (page: Page): Promise<Rec> => page.evaluate(() => JSON.parse(JSON.stringify(window.__chartRec)));

/** Indexes (document order) of the charts on screen right now. */
const onScreenIndexes = (page: Page): Promise<number[]> =>
  page.evaluate(() =>
    Array.from(document.querySelectorAll('[data-slot="chart-svg"]'))
      .map((svg, i) => ({ r: svg.getBoundingClientRect(), i }))
      // viewport-ok: vertical extent of the visible page; the scrollbar gutter is horizontal only
      .filter(({ r }) => r.width > 0 && r.height > 0 && r.bottom > 0 && r.top < innerHeight)
      .map(({ i }) => i),
  );

// One tab / component page per chart kind. The gallery's first card and a component page's first
// demo both sit in the first screen at this viewport.
const KINDS = ["area", "bar", "line", "pie", "radar", "radial"] as const;
const PAGES = [
  ...KINDS.map((k) => ({ name: `/charts/${k}/`, url: `${BASE_URL}/charts/${k}/` })),
  ...KINDS.map((k) => ({ name: `${k}_chart component page`, url: `${BASE_URL}/component/${k}_chart/` })),
];

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.addInitScript(installRecorder);
});

test.describe("Chart entry animation: a chart in the first paint never restarts", () => {
  for (const { name, url } of PAGES) {
    test(`${name}: every frame from first paint shows the final drawing`, async ({ page }) => {
      await gotoHydrated(page, url, NAV);
      await page.waitForTimeout(1500); // the measured render and any (wrong) entrance would have started by now
      const visible = await onScreenIndexes(page);
      expect(visible.length, "a chart is on the first screen").toBeGreaterThan(0);
      const r = await rec(page);
      expect(r.frames, "the recorder sampled many frames").toBeGreaterThan(30);
      for (const i of visible) {
        expect(r.measured[i], `chart ${i} was measured (hydrated) inside the window`).toBe("true");
        expect(r.anims[i] ?? 0, `chart ${i} ran no CSS animation: ${r.names[i]}`).toBe(0);
        expect(r.animateAttr[i], `chart ${i} never got data-animate`).toBeNull();
      }
      const bad = r.bad.filter((b) => visible.includes(b.i));
      expect(bad, `frames whose marks differ from the first (final) frame: first=${JSON.stringify(r.first)}`).toEqual([]);
    });
  }
});

/** Scroll so the last chart of the page is centred; returns its index. */
async function scrollLastChartIn(page: Page): Promise<number> {
  return page.evaluate(() => {
    const svgs = Array.from(document.querySelectorAll('[data-slot="chart-svg"]'));
    const last = svgs.length - 1;
    const r = svgs[last].getBoundingClientRect();
    // viewport-ok: vertical extent of the visible page; the scrollbar gutter is horizontal only
    window.scrollTo(0, scrollY + r.top - (innerHeight - r.height) / 2);
    return last;
  });
}

test.describe("Chart entry animation: a chart below the fold animates once, when first seen", () => {
  for (const kind of KINDS) {
    test(`${kind}_chart: one entrance on first view, none on the way back`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/${kind}_chart/`, NAV);
      await page.waitForTimeout(1500);
      const count = await page.locator('[data-slot="chart-svg"]').count();
      expect(count, "the page stacks several charts").toBeGreaterThan(2);
      const last = count - 1;
      expect(await onScreenIndexes(page), "the last chart starts below the first screen").not.toContain(last);
      const before = await rec(page);
      expect(before.anims[last] ?? 0, "nothing animated before it was in view").toBe(0);
      expect(before.animateAttr[last]).toBeNull();

      expect(await scrollLastChartIn(page)).toBe(last);
      await expect
        .poll(async () => (await rec(page)).animateAttr[last], { timeout: 10_000, message: "entrance starts" })
        .toBe("true");
      await page.waitForTimeout(2500); // longest entrance is 1.5 s (+ the arc labels' fade)
      const played = await rec(page);
      const n = played.anims[last] ?? 0;
      expect(n, "the entrance ran CSS animations").toBeGreaterThan(0);

      // Out and back (twice): no second run, and the settled drawing is the final one.
      for (let k = 0; k < 2; k++) {
        await page.evaluate(() => window.scrollTo(0, 0));
        await page.waitForTimeout(400);
        await scrollLastChartIn(page);
        await page.waitForTimeout(1200);
      }
      const after = await rec(page);
      expect(after.anims[last], "scrolling out and back replayed nothing").toBe(n);
      expect(after.lastAnimAt[last], "no animation started after the first entrance").toBe(played.lastAnimAt[last]);
      // Charts above it were never animated by any of this.
      for (let i = 0; i < last; i++) {
        const wasOnScreenAtLoad = i in before.first;
        if (wasOnScreenAtLoad) expect(after.anims[i] ?? 0, `chart ${i} (in the first paint) never animated`).toBe(0);
      }
    });
  }
});

test.describe("Chart entry animation: a chart the client creates has nothing painted to protect", () => {
  test("the legacy /component/?name= redirect builds the chart in the browser, so it enters with an animation", async ({ // legacy-route-ok: this test IS the legacy redirect
    page,
  }) => {
    await gotoHydrated(page, `${BASE_URL}/component/?name=bar_chart&`, NAV); // legacy-route-ok: this test IS the legacy redirect
    await page.waitForTimeout(2000);
    const visible = await onScreenIndexes(page);
    expect(visible.length, "a chart is on the first screen").toBeGreaterThan(0);
    const r = await rec(page);
    // The recorder is installed per document; the redirect is a client-side route change, so it
    // spans both. The first chart on screen was created by the client: it plays its entrance.
    expect(r.animateAttr[visible[0]], "client-created chart in view animates").toBe("true");
    expect(r.anims[visible[0]] ?? 0).toBeGreaterThan(0);
  });
});

test.describe("Chart entry animation: reduced motion", () => {
  for (const kind of KINDS) {
    test(`${kind}_chart: no animation in the first paint or on entering the view`, async ({ page }) => {
      await page.emulateMedia({ reducedMotion: "reduce" });
      await gotoHydrated(page, `${BASE_URL}/component/${kind}_chart/`, NAV);
      await page.waitForTimeout(1000);
      const last = await scrollLastChartIn(page);
      await page.waitForTimeout(2500);
      const r = await rec(page);
      expect(r.frames).toBeGreaterThan(30);
      for (let i = 0; i <= last; i++) expect(r.anims[i] ?? 0, `chart ${i} ran no CSS animation: ${r.names[i]}`).toBe(0);
      expect(r.bad.filter((b) => b.why === "signature changed"), "marks never left their final state").toEqual([]);
    });
  }
});
