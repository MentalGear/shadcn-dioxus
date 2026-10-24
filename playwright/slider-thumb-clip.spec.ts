import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// dev-docs/backlog.md row 293 -- the primitive renders `SliderThumb` INSIDE `SliderTrack`, so a 4px track with
// `overflow: hidden` (Nova's, where the thumb is a sibling outside the track) would clip the 12px thumb to a
// sliver, with its ring and its `::after` hit area. By construction no ancestor of a thumb below the
// slider root may clip it: the clipping is on `.dx-slider-range`.

const URL = `${BASE_URL}/component/slider/`;
const RING = 3; // the 3px focus halo (`--dx-ring-focus`)

/** Every `[role=slider]` thumb on the page: its box, the smallest box any clipping ancestor leaves it, and what a hit test at its extremes finds. */
async function inspectThumbs(page: Page) {
  return page.evaluate((ring) => {
    return Array.from(document.querySelectorAll<HTMLElement>(".dx-slider-thumb")).map((thumb) => {
      thumb.scrollIntoView({ block: "center", inline: "center" });
      const r = thumb.getBoundingClientRect();
      const clippers: { cls: string; fits: boolean }[] = [];
      for (let el = thumb.parentElement; el && el !== document.body; el = el.parentElement) {
        const cs = getComputedStyle(el);
        if (cs.overflowX === "visible" && cs.overflowY === "visible") continue;
        const b = el.getBoundingClientRect();
        // the thumb plus its ring must lie inside anything that clips
        const fits = b.left <= r.left - ring && b.right >= r.right + ring && b.top <= r.top - ring && b.bottom >= r.bottom + ring;
        clippers.push({ cls: el.className.toString(), fits });
        if (el.classList.contains("dx-slider")) break;
      }
      // Hit tests just inside the thumb's four extremes: a clipped thumb answers with something else.
      const inset = 1;
      const points = [
        [r.left + r.width / 2, r.top + inset],
        [r.left + r.width / 2, r.bottom - inset],
        [r.left + inset + 2, r.top + r.height / 2],
        [r.right - inset - 2, r.top + r.height / 2],
      ];
      const hits = points.map(([x, y]) => {
        const hit = document.elementFromPoint(x, y);
        return !!hit && (hit === thumb || thumb.contains(hit));
      });
      return { width: r.width, height: r.height, clippers, hits, orientation: thumb.closest(".dx-slider")?.getAttribute("data-orientation") };
    });
  }, RING);
}

test("no ancestor clips a slider thumb (horizontal, vertical, range), and its extremes are hittable", async ({ page }) => {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const thumbs = await inspectThumbs(page);
  expect(thumbs.length).toBeGreaterThanOrEqual(4); // single, range x2, vertical ...
  expect(new Set(thumbs.map((t) => t.orientation))).toEqual(new Set(["horizontal", "vertical"]));
  for (const t of thumbs) {
    expect(t.width).toBeGreaterThanOrEqual(12 - 0.5);
    expect(t.height).toBeGreaterThanOrEqual(12 - 0.5);
    for (const c of t.clippers) expect(c.fits, `thumb (${t.orientation}) is clipped by .${c.cls}`).toBe(true);
    expect(t.hits, `thumb (${t.orientation}) hit tests at top, bottom, left, right`).toEqual([true, true, true, true]);
  }
});

test("the focused thumb's ring is not cut off", async ({ page }) => {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const thumb = page.getByRole("slider", { name: "Demo Slider" });
  await thumb.focus();
  await page.keyboard.press("ArrowRight"); // keyboard modality: :focus-visible
  await expect(thumb).toHaveCSS("box-shadow", / 0px 0px 0px 3px$/);
  const clipped = await thumb.evaluate((el, ring) => {
    const r = el.getBoundingClientRect();
    for (let a = el.parentElement; a && a !== document.body; a = a.parentElement) {
      const cs = getComputedStyle(a);
      if (cs.overflowX === "visible" && cs.overflowY === "visible") continue;
      const b = a.getBoundingClientRect();
      if (b.top > r.top - ring || b.bottom < r.bottom + ring) return a.className.toString();
    }
    return null;
  }, RING);
  expect(clipped).toBeNull();
});

test.describe("coarse pointer", () => {
  test.use({ hasTouch: true, isMobile: true, viewport: { width: 390, height: 844 }, deviceScaleFactor: 3 });

  test("the thumb is 20px with a hit area of at least 44px", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const sizes = await page.evaluate(() => {
      const thumb = document.querySelector<HTMLElement>(".dx-slider-thumb")!;
      const after = getComputedStyle(thumb, "::after");
      const r = thumb.getBoundingClientRect();
      const inset = Math.abs(parseFloat(after.top));
      return { thumb: r.width, target: r.width + 2 * inset, media: matchMedia("(pointer: coarse)").matches };
    });
    expect(sizes.media).toBe(true);
    expect(sizes.thumb).toBeCloseTo(20, 0);
    expect(sizes.target).toBeGreaterThanOrEqual(44);
  });
});
