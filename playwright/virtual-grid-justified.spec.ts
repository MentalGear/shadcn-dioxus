import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { demoFrame, demoRole } from "./demo-frame";
import { gotoHydrated } from "./hydration";

// VirtualGrid page, `justified` demo: VirtualJustifiedGrid, 20,000 generated photos of mixed aspect ratios packed
// into justified rows (every tile its photo's aspect, rows filling the width edge to edge), rows windowed, and in
// wall mode (the demo's switch) a canvas six times as wide whose rows are also windowed by x. Controls are found by
// role, position and data attributes inside the demo frame (row 220). Functional spec: no timing thresholds.

const PAGE_URL = `${BASE_URL}/component/virtual_grid/`;
const VARIANT = "justified";
const PHOTOS = 20_000;
const MAX_TILES = 300;

const grid = (page: Page) => demoFrame(page, VARIANT).getByRole("grid");
const tiles = (g: Locator) => g.locator('[role="gridcell"]');
const tile = (g: Locator, index: number) => g.locator(`[data-grid-index="${index}"]`);
const focusedIndex = (page: Page) =>
  page.evaluate(() => {
    const v = document.activeElement?.getAttribute("data-grid-index");
    return v == null ? null : Number(v);
  });

async function open(page: Page) {
  await gotoHydrated(page, PAGE_URL);
  const g = grid(page);
  await g.scrollIntoViewIfNeeded();
  await expect(tile(g, 0)).toBeVisible();
  return g;
}

/**
 * For every mounted row that meets the grid's visible box (an off-screen row may hold only the pinned active tile):
 * the gap between its first tile and the content box's inline-start edge, between its last tile and the inline-end
 * edge, whether it is the last row, and each tile's width/height against the photo's aspect (from its `aria-label`, which
 * ends in "w:h").
 */
function rowGeometry(g: Locator) {
  return g.evaluate((el) => {
    const box = el.getBoundingClientRect();
    const left = box.left + el.clientLeft;
    const right = left + el.clientWidth;
    const top = box.top + el.clientTop;
    const bottom = top + el.clientHeight;
    const rtl = getComputedStyle(el).direction === "rtl";
    const rowCount = Number(el.getAttribute("aria-rowcount"));
    const rows = [];
    for (const row of el.querySelectorAll('[role="row"]')) {
      const r = row.getBoundingClientRect();
      if (r.bottom <= top || r.top >= bottom) continue;
      const cells = [...row.querySelectorAll('[role="gridcell"]')];
      const rects = cells.map((c) => c.getBoundingClientRect());
      const first = rects[0];
      const last = rects[rects.length - 1];
      rows.push({
        index: Number(row.getAttribute("aria-rowindex")),
        last: Number(row.getAttribute("aria-rowindex")) === rowCount,
        firstColumn: Number(cells[0].getAttribute("aria-colindex")),
        startGap: rtl ? right - first.right : first.left - left,
        endGap: rtl ? last.left - left : right - last.right,
        aspects: cells.map((c, i) => {
          const m = c.querySelector("img")!.getAttribute("aria-label")!.match(/(\d+):(\d+)$/)!;
          return { want: Number(m[1]) / Number(m[2]), got: rects[i].width / rects[i].height };
        }),
      });
    }
    return rows;
  });
}

async function expectRowsFill(g: Locator) {
  const rows = await rowGeometry(g);
  expect(rows.length).toBeGreaterThan(1);
  for (const row of rows) {
    expect(row.firstColumn, `row ${row.index} starts at its first tile`).toBe(1);
    expect(Math.abs(row.startGap), `row ${row.index} starts at the edge`).toBeLessThanOrEqual(1);
    if (!row.last) expect(Math.abs(row.endGap), `row ${row.index} ends at the edge`).toBeLessThanOrEqual(1);
    for (const a of row.aspects) expect(Math.abs(a.got / a.want - 1)).toBeLessThan(0.01);
  }
}

/** Index of the tile in row `row` (0-based) whose horizontal centre is closest to `x`. */
function nearestInRow(g: Locator, row: number, x: number) {
  return g.evaluate(
    (el, [row, x]) => {
      let best = -1;
      let dist = Infinity;
      for (const c of el.querySelectorAll(`[role="row"][aria-rowindex="${row + 1}"] [role="gridcell"]`)) {
        const r = c.getBoundingClientRect();
        const d = Math.abs((r.left + r.right) / 2 - x);
        if (d < dist - 0.01) {
          dist = d;
          best = Number(c.getAttribute("data-grid-index"));
        }
      }
      return best;
    },
    [row, x] as const,
  );
}

const centreX = (l: Locator) => l.evaluate((el) => {
  const r = el.getBoundingClientRect();
  return (r.left + r.right) / 2;
});
const rowOf = (l: Locator) => l.evaluate((el) => Number(el.getAttribute("data-grid-cell")!.split(":")[0]));

/** Whether the tile is fully inside the grid's scrollport (the grid may itself be partly off the page viewport). */
const inGridView = (g: Locator, index: number) =>
  g.evaluate((el, index) => {
    const t = el.querySelector(`[data-grid-index="${index}"]`);
    if (!t) return false;
    const r = t.getBoundingClientRect();
    const b = el.getBoundingClientRect();
    const left = b.left + el.clientLeft;
    const top = b.top + el.clientTop;
    return r.left >= left - 1 && r.right <= left + el.clientWidth + 1 && r.top >= top - 1 && r.bottom <= top + el.clientHeight + 1;
  }, index);

test.describe("virtual_grid justified", () => {
  test("mounts a small window of 20,000 photos; rows fill the width; tiles keep their aspect", async ({ page }) => {
    const g = await open(page);
    await expect(g).toHaveAttribute("data-justified-axes", "block");
    await expect(g).not.toHaveAttribute("aria-colcount", /.*/);
    const rowCount = Number(await g.getAttribute("aria-rowcount"));
    expect(rowCount).toBeGreaterThan(1000);
    const n = await tiles(g).count();
    expect(n).toBeGreaterThan(5);
    expect(n).toBeLessThan(MAX_TILES);
    await expect(g.locator('[data-grid-index][tabindex="0"]')).toHaveCount(1);
    await expectRowsFill(g);
    // The vertical-only wall never scrolls sideways.
    expect(await g.evaluate((el) => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(0);
  });

  test("scrolling to the end renders the last rows and keeps the DOM small", async ({ page }) => {
    const g = await open(page);
    await expect(async () => {
      await g.evaluate((el) => (el.scrollTop = el.scrollHeight));
      await expect(tile(g, PHOTOS - 1)).toBeAttached({ timeout: 1000 });
    }).toPass({ timeout: 15_000 });
    await expect(tile(g, PHOTOS - 1)).toBeInViewport();
    expect(await tiles(g).count()).toBeLessThan(MAX_TILES);
    // Tile 0 stays mounted as the tab stop; tile 1 does not.
    await expect(tile(g, 1)).toHaveCount(0);
    await expectRowsFill(g);
    // The middle too.
    await g.evaluate((el) => (el.scrollTop = el.scrollHeight / 2));
    await expect.poll(async () => {
      const idx = await tiles(g).evaluateAll((els) => els.map((e) => Number(e.getAttribute("data-grid-index"))));
      return Math.max(...idx);
    }).toBeGreaterThan(PHOTOS / 2 - 500);
    expect(await tiles(g).count()).toBeLessThan(MAX_TILES);
  });

  test("wall mode scrolls both axes and windows the tiles of each row by x", async ({ page }) => {
    const g = await open(page);
    // justified demo: the first switch is the wall toggle (the slider has role slider)
    await demoRole(page, "switch", 0, VARIANT).click();
    await expect(g).toHaveAttribute("data-justified-axes", "both");
    const size = await g.evaluate((el) => ({ sw: el.scrollWidth, cw: el.clientWidth, sh: el.scrollHeight, ch: el.clientHeight }));
    expect(size.sw).toBeGreaterThan(size.cw * 5);
    expect(size.sh).toBeGreaterThan(size.ch * 10);
    await g.evaluate((el) => {
      el.scrollLeft = el.scrollWidth / 2;
      el.scrollTop = el.scrollHeight / 2;
    });
    // Far tiles on both axes are mounted, nothing near the start of a row is.
    await expect.poll(async () => {
      const cols = await tiles(g).evaluateAll((els) =>
        els.filter((e) => e.getAttribute("data-grid-index") !== "0").map((e) => Number(e.getAttribute("aria-colindex"))),
      );
      return cols.length > 0 ? Math.min(...cols) : 0;
    }).toBeGreaterThan(3);
    const minIndex = await tiles(g).evaluateAll((els) =>
      Math.min(...els.filter((e) => e.getAttribute("data-grid-index") !== "0").map((e) => Number(e.getAttribute("data-grid-index")))),
    );
    expect(minIndex).toBeGreaterThan(PHOTOS / 3);
    expect(await tiles(g).count()).toBeLessThan(MAX_TILES);
    // Each mounted row spans well under half the canvas.
    const spans = await g.evaluate((el) =>
      [...el.querySelectorAll('[role="row"]')]
        .map((row) => [...row.querySelectorAll('[role="gridcell"]')].map((c) => c.getBoundingClientRect()))
        .filter((rects) => rects.length > 1)
        .map((rects) => Math.max(...rects.map((r) => r.right)) - Math.min(...rects.map((r) => r.left))),
    );
    expect(spans.length).toBeGreaterThan(1);
    for (const span of spans) expect(span).toBeLessThan(size.sw / 2);
    // What is mounted covers the viewport: the point at the grid's centre is a photo.
    const hit = await g.evaluate((el) => {
      const b = el.getBoundingClientRect();
      const at = document.elementFromPoint(b.left + el.clientWidth / 2, b.top + el.clientHeight / 2);
      return at?.closest('[role="gridcell"]') !== null;
    });
    expect(hit).toBe(true);
  });

  test("keyboard: Up/Down land on the nearest tile by centre; Left/Right continue across rows", async ({ page }) => {
    const g = await open(page);
    const start = g.locator('[data-grid-cell="1:1"]');
    await start.click();
    const from = Number(await start.getAttribute("data-grid-index"));
    expect(await focusedIndex(page)).toBe(from);

    const wantDown = await nearestInRow(g, 2, await centreX(start));
    await page.keyboard.press("ArrowDown");
    await expect.poll(() => focusedIndex(page)).toBe(wantDown);
    const down = tile(g, wantDown);
    const wantUp = await nearestInRow(g, 1, await centreX(down));
    await page.keyboard.press("ArrowUp");
    await expect.poll(() => focusedIndex(page)).toBe(wantUp);

    // End, then ArrowRight: the next row's first tile.
    await page.keyboard.press("End");
    const end = (await focusedIndex(page))!;
    expect(await rowOf(tile(g, end))).toBe(1);
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => focusedIndex(page)).toBe(end + 1);
    await expect(tile(g, end + 1)).toHaveAttribute("data-grid-cell", "2:0");
    await page.keyboard.press("ArrowLeft");
    await expect.poll(() => focusedIndex(page)).toBe(end);
    await page.keyboard.press("Home");
    await expect(tile(g, end)).not.toBeFocused();
    expect(await rowOf(tile(g, (await focusedIndex(page))!))).toBe(1);

    // PageDown moves a viewport of rows; Ctrl+End reaches the last photo, rendered, focused and in view.
    await page.keyboard.press("PageDown");
    await expect.poll(async () => rowOf(tile(g, (await focusedIndex(page))!))).toBeGreaterThan(2);
    await page.keyboard.press("Control+End");
    await expect.poll(() => focusedIndex(page)).toBe(PHOTOS - 1);
    await expect.poll(() => inGridView(g, PHOTOS - 1)).toBe(true);
    await page.keyboard.press("Control+Home");
    await expect.poll(() => focusedIndex(page)).toBe(0);
    await expect.poll(() => inGridView(g, 0)).toBe(true);
    await expect(g.locator('[data-grid-index][tabindex="0"]')).toHaveCount(1);
  });

  test("focus survives scrolling away and the keys continue from it", async ({ page }) => {
    const g = await open(page);
    await tile(g, 2).click();
    await g.evaluate((el) => (el.scrollTop = el.scrollHeight * 0.75));
    await expect(tile(g, 1)).toHaveCount(0);
    await expect(tile(g, 2)).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => focusedIndex(page)).toBe(3);
    await expect.poll(() => inGridView(g, 3)).toBe(true);
  });

  test("right-to-left: rows start at the right and ArrowLeft moves to the next photo", async ({ page }) => {
    const g = await open(page);
    await page.locator("#dx-direction-toggle").click();
    await expect(demoFrame(page, VARIANT)).toHaveAttribute("dir", "rtl");
    await expectRowsFill(g);
    const [a, b] = [await tile(g, 0).boundingBox(), await tile(g, 1).boundingBox()];
    expect(b!.x + b!.width).toBeLessThanOrEqual(a!.x + 1);
    await tile(g, 0).click();
    await page.keyboard.press("ArrowLeft");
    await expect.poll(() => focusedIndex(page)).toBe(1);
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => focusedIndex(page)).toBe(0);
    // Wall mode in RTL: scrolled toward the inline end (negative scrollLeft), far tiles render.
    await demoRole(page, "switch", 0, VARIANT).click();
    await expect(g).toHaveAttribute("data-justified-axes", "both");
    await g.evaluate((el) => (el.scrollLeft = -el.scrollWidth / 2));
    await expect.poll(async () => {
      const cols = await tiles(g).evaluateAll((els) =>
        els.filter((e) => e.getAttribute("data-grid-index") !== "0").map((e) => Number(e.getAttribute("aria-colindex"))),
      );
      return cols.length > 0 ? Math.min(...cols) : 0;
    }).toBeGreaterThan(3);
  });

  test("resizing re-lays the rows out for the new width", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    const g = await open(page);
    const before = await g.evaluate((el) => ({
      width: el.clientWidth,
      firstRow: el.querySelectorAll('[role="row"][aria-rowindex="1"] [role="gridcell"]').length,
    }));
    await expectRowsFill(g);
    await page.setViewportSize({ width: 640, height: 900 });
    await expect.poll(() => g.evaluate((el) => el.clientWidth)).toBeLessThan(before.width - 100);
    await expect.poll(() =>
      g.evaluate((el) => el.querySelectorAll('[role="row"][aria-rowindex="1"] [role="gridcell"]').length),
    ).toBeLessThan(before.firstRow);
    await expectRowsFill(g);
  });

  test("the target row height control re-packs the rows", async ({ page }) => {
    const g = await open(page);
    const meanRowHeight = () =>
      g.evaluate((el) => {
        const hs = [...el.querySelectorAll('[role="row"]')].map((r) => r.getBoundingClientRect().height);
        return hs.reduce((a, b) => a + b, 0) / hs.length;
      });
    const rowsBefore = Number(await g.getAttribute("aria-rowcount"));
    const heightBefore = await meanRowHeight();
    // justified demo: the one slider is the target row height (80..320, starts at 160)
    const slider = demoRole(page, "slider", 0, VARIANT);
    await slider.focus();
    await page.keyboard.press("End");
    // Taller rows hold fewer photos each, so the wall has more of them.
    await expect.poll(async () => Number(await g.getAttribute("aria-rowcount"))).toBeGreaterThan(rowsBefore * 1.3);
    expect(await meanRowHeight()).toBeGreaterThan(heightBefore * 1.3);
    await expectRowsFill(g);
  });

  test("Axe automated scan (wall mode, a tile focused)", async ({ page }) => {
    const g = await open(page);
    await demoRole(page, "switch", 0, VARIANT).click();
    await expect(g).toHaveAttribute("data-justified-axes", "both");
    await tile(g, 0).click();
    await expectNoAxeViolations(page, "virtual_grid justified: wall", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
