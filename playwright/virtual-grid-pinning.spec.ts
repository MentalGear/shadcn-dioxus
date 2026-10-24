import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { demoFrame, demoRole } from "./demo-frame";
import { gotoHydrated } from "./hydration";

// VirtualGrid pinning (backlog row 388), on the `pinning` demo: 5,000 sites x 29 columns with `pinned_columns_start: 2`
// (columns 0 and 1), `pinned_columns_end: 1` (column 28, a checkbox) and a header row that follows the demo's
// switch (`pin_header`). Column widths 112 / 96 / 104 (x26) / 80. Controls are located by role and position inside
// the demo frame (row 220).

const PAGE_URL = `${BASE_URL}/component/virtual_grid/`;
const VARIANT = "pinning";
const END_COLUMN = 28;

const grid = (page: Page) => demoFrame(page, VARIANT).getByRole("grid");
const cell = (g: Locator, row: number | "header", column: number) =>
  g.locator(`[data-grid-cell="${row}:${column}"]`);
const focusedCell = (page: Page) =>
  page.evaluate(() => document.activeElement?.closest("[data-grid-cell]")?.getAttribute("data-grid-cell") ?? null);

/** Scroll the container to `top` and `left` (a distance from the inline start, so negative in RTL). */
const scrollGrid = (g: Locator, top: number, left: number) =>
  g.evaluate(
    (el, [t, l]) => {
      el.scrollTop = t;
      el.scrollLeft = l;
    },
    [top, left] as const,
  );

/** The scrollport (inside the border and the scrollbars) and a cell's box, in viewport pixels. */
async function boxes(g: Locator, key: string) {
  return g.evaluate((el, key) => {
    const box = el.getBoundingClientRect();
    const r = el.querySelector(`[data-grid-cell="${key}"]`)!.getBoundingClientRect();
    return {
      gridLeft: box.left + el.clientLeft,
      gridRight: box.left + el.clientLeft + el.clientWidth,
      gridTop: box.top + el.clientTop,
      left: r.left,
      right: r.right,
      top: r.top,
      bottom: r.bottom,
    };
  }, key);
}

/**
 * Whether the cell is hidden behind a pinned region: its box overlaps a pinned cell of its own row, or the pinned
 * header (when the header row is pinned), by more than a pixel.
 */
async function covered(g: Locator, row: number, column: number) {
  return g.evaluate(
    (el, [row, column]) => {
      const target = el.querySelector(`[data-grid-cell="${row}:${column}"]`)!.getBoundingClientRect();
      const overlap = (a: DOMRect, b: DOMRect) =>
        Math.min(a.right, b.right) - Math.max(a.left, b.left) > 1 &&
        Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top) > 1;
      const pinned = [...el.querySelectorAll(`[data-grid-cell^="${row}:"][data-grid-pinned]`)]
        .filter((n) => n.getAttribute("data-grid-cell") !== `${row}:${column}`)
        .map((n) => n.getBoundingClientRect());
      const header = el.querySelector("[data-grid-header]")!.getBoundingClientRect();
      const headerPinned = getComputedStyle(el.querySelector("[data-grid-header]")!).position === "sticky";
      const grid = el.getBoundingClientRect();
      const outside =
        target.top < grid.top + el.clientTop - 1 ||
        target.bottom > grid.top + el.clientTop + el.clientHeight + 1;
      return outside || (headerPinned && overlap(target, header)) || pinned.some((r) => overlap(target, r));
    },
    [row, column] as const,
  );
}

/** The computed `property` of the first element matching `selector`. */
const styleOf = (g: Locator, selector: string, property: string) =>
  g.locator(selector).first().evaluate((el, p) => getComputedStyle(el).getPropertyValue(p), property);

/** The alpha channel of a computed colour (`rgb()`, `rgba()`, `oklab(... / a)`, `color(srgb ... / a)`). */
function alphaOf(color: string): number {
  const slash = /\/\s*([\d.]+)(%?)\s*\)$/.exec(color);
  if (slash) return Number(slash[1]) / (slash[2] ? 100 : 1);
  const legacy = /^rgba\((?:[^,]+,){3}\s*([\d.]+)\s*\)$/.exec(color);
  return legacy ? Number(legacy[1]) : 1;
}

async function open(page: Page) {
  await gotoHydrated(page, PAGE_URL);
  const g = grid(page);
  await g.scrollIntoViewIfNeeded();
  await expect(cell(g, 0, 2)).toBeVisible();
  return g;
}

test.describe("pinned columns (windowed data grid)", () => {
  test("the start and end columns keep their place while the grid scrolls sideways", async ({ page }) => {
    const g = await open(page);
    await expect(g).toHaveAttribute("data-virtualize", "windowed");
    for (const left of [0, 700, 1e9]) {
      await scrollGrid(g, 0, left);
      await expect(cell(g, 0, 28)).toBeVisible();
      for (const row of [0, "header"] as const) {
        const first = await boxes(g, `${row}:0`);
        const second = await boxes(g, `${row}:1`);
        const last = await boxes(g, `${row}:${END_COLUMN}`);
        expect(Math.abs(first.left - first.gridLeft)).toBeLessThan(1.5);
        expect(Math.abs(second.left - (first.gridLeft + 112))).toBeLessThan(1.5);
        expect(Math.abs(last.right - last.gridRight)).toBeLessThan(1.5);
      }
    }
    // Scrolled to the end, the last scrolling column (27) sits just before the end column, not under it.
    const before = await boxes(g, `0:${END_COLUMN - 1}`);
    const last = await boxes(g, `0:${END_COLUMN}`);
    expect(before.right).toBeLessThanOrEqual(last.left + 1);
  });

  test("pinned columns are mounted once, outside the column window, and carry the pin attributes", async ({
    page,
  }) => {
    const g = await open(page);
    await scrollGrid(g, 2000 * 36, 1500);
    await expect(cell(g, 2000, 20)).toBeVisible();
    const report = await g.evaluate((el) => {
      const keys = [...el.querySelectorAll("[data-grid-cell]")].map((n) => n.getAttribute("data-grid-cell")!);
      const pinned = (key: string) => el.querySelector(`[data-grid-cell="${key}"]`)?.getAttribute("data-grid-pinned") ?? "none";
      const edge = (key: string) => el.querySelector(`[data-grid-cell="${key}"]`)?.getAttribute("data-grid-pin-edge") ?? "none";
      const row = el.querySelector('[role="row"][aria-rowindex="2002"]')!;
      return {
        duplicates: keys.length - new Set(keys).size,
        total: keys.length,
        start: [pinned("2000:0"), pinned("2000:1"), pinned("2000:2")],
        end: pinned(`2000:${28}`),
        edges: [edge("2000:0"), edge("2000:1"), edge(`2000:${28}`), edge("2000:2")],
        pinnedPerRow: row.querySelectorAll("[data-grid-pinned]").length,
      };
    });
    expect(report.duplicates).toBe(0);
    expect(report.total).toBeLessThan(600);
    expect(report.start).toEqual(["start", "start", "none"]);
    expect(report.end).toBe("end");
    // The edge marks the pinned column next to the scrolling ones, once per side.
    expect(report.edges).toEqual(["none", "true", "true", "none"]);
    expect(report.pinnedPerRow).toBe(3);
  });

  test("only the first column is a row header, and the aria counts are the whole grid's", async ({ page }) => {
    const g = await open(page);
    await expect(g).toHaveAttribute("aria-colcount", "29");
    await expect(cell(g, 0, 0)).toHaveAttribute("role", "rowheader");
    await expect(cell(g, 0, 1)).toHaveAttribute("role", "gridcell");
    await expect(cell(g, 0, END_COLUMN)).toHaveAttribute("role", "gridcell");
    await expect(cell(g, "header", 0)).toHaveAttribute("role", "columnheader");
    await expect(cell(g, 0, END_COLUMN)).toHaveAttribute("aria-colindex", "29");
  });

  test("a pinned cell is opaque and gets the separating edge", async ({ page }) => {
    const g = await open(page);
    await page.mouse.move(0, 0);
    for (const key of ["0:0", "0:1", `0:${END_COLUMN}`, "header:0", `header:${END_COLUMN}`]) {
      expect(alphaOf(await styleOf(g, `[data-grid-cell="${key}"]`, "background-color")), key).toBe(1);
    }
    expect(await styleOf(g, '[data-grid-cell="0:1"]', "border-inline-end-width")).toBe("1px");
    expect(await styleOf(g, '[data-grid-cell="0:0"]', "border-inline-end-width")).toBe("0px");
    expect(await styleOf(g, `[data-grid-cell="0:${END_COLUMN}"]`, "border-inline-start-width")).toBe("1px");
    expect(await styleOf(g, '[data-grid-cell="0:2"]', "border-inline-start-width")).toBe("0px");
  });
});

test.describe("keyboard with pinned regions", () => {
  test("Home and End reach the pinned cells of the row, and the arrows reveal a cell clear of every pinned region", async ({
    page,
  }) => {
    const g = await open(page);
    await cell(g, 3, 5).click();
    await page.keyboard.press("End");
    await expect.poll(() => focusedCell(page)).toBe(`3:${END_COLUMN}`);
    await expect.poll(() => covered(g, 3, END_COLUMN)).toBe(false);
    await page.keyboard.press("Home");
    await expect.poll(() => focusedCell(page)).toBe("3:0");

    // Walking right from the pinned columns: each scrolling cell is revealed beside the start columns first
    // and, near the end, beside the end column.
    for (let column = 1; column <= END_COLUMN; column++) {
      await page.keyboard.press("ArrowRight");
      await expect.poll(() => focusedCell(page)).toBe(`3:${column}`);
      await expect.poll(() => covered(g, 3, column), `column ${column}`).toBe(false);
    }
    // And back left, the cells scrolled under the end column and then the start columns come clear.
    for (let column = END_COLUMN - 1; column >= 0; column--) {
      await page.keyboard.press("ArrowLeft");
      await expect.poll(() => focusedCell(page)).toBe(`3:${column}`);
      await expect.poll(() => covered(g, 3, column), `column ${column}`).toBe(false);
    }
  });

  test("a vertical move reveals the row below the pinned header", async ({ page }) => {
    const g = await open(page);
    await scrollGrid(g, 100 * 36, 0);
    await expect(cell(g, 101, 3)).toBeVisible();
    await cell(g, 101, 3).click();
    for (let i = 0; i < 4; i++) await page.keyboard.press("ArrowUp");
    await expect.poll(() => focusedCell(page)).toBe("97:3");
    await expect.poll(() => covered(g, 97, 3)).toBe(false);
    await page.keyboard.press("PageDown");
    await expect.poll(() => focusedCell(page)).not.toBe("97:3");
    const now = (await focusedCell(page))!.split(":").map(Number);
    await expect.poll(() => covered(g, now[0], now[1])).toBe(false);
  });
});

test.describe("pin_header", () => {
  test("the header stays on top while it is pinned and scrolls away with the body when it is not", async ({ page }) => {
    const g = await open(page);
    await scrollGrid(g, 500 * 36, 0);
    await expect(cell(g, 500, 3)).toBeVisible();
    const pinned = await boxes(g, "header:3");
    expect(Math.abs(pinned.top - pinned.gridTop)).toBeLessThan(1.5);

    // The switch is the demo's only one: turn the pin off.
    const toggle = demoRole(page, "switch", 0, VARIANT);
    await expect(toggle).toBeChecked();
    await toggle.click();
    await expect(toggle).not.toBeChecked();
    await expect.poll(async () => (await boxes(g, "header:3")).bottom).toBeLessThan(pinned.gridTop);
    // It is still the first row of the grid: back at the top it is visible again, and its pinned cells still
    // hold the columns' edges while the grid scrolls sideways.
    await scrollGrid(g, 0, 700);
    const top = await boxes(g, "header:10");
    expect(Math.abs(top.top - top.gridTop)).toBeLessThan(1.5);
    const first = await boxes(g, "header:0");
    expect(Math.abs(first.left - first.gridLeft)).toBeLessThan(1.5);
    expect(await g.locator("[data-grid-header]").evaluate((el) => getComputedStyle(el).position)).toBe("relative");

    // A keyboard move still reveals the row, now against the top edge of the grid.
    await scrollGrid(g, 30 * 36, 0);
    await expect(cell(g, 30, 3)).toBeVisible();
    await cell(g, 30, 3).click();
    for (let i = 0; i < 3; i++) await page.keyboard.press("ArrowUp");
    await expect.poll(() => focusedCell(page)).toBe("27:3");
    await expect.poll(() => covered(g, 27, 3)).toBe(false);

    await toggle.click();
    await expect(toggle).toBeChecked();
    expect(await g.locator("[data-grid-header]").evaluate((el) => getComputedStyle(el).position)).toBe("sticky");
  });
});

test.describe("row fill on pinned cells", () => {
  test("a hovered body row tints its pinned cells over their opaque base, and the header stays untouched", async ({
    page,
  }) => {
    const g = await open(page);
    await scrollGrid(g, 0, 400);
    await expect(cell(g, 4, 6)).toBeVisible();
    await page.mouse.move(0, 0);
    const rest = {
      start: await styleOf(g, '[data-grid-cell="4:0"]', "background-color"),
      end: await styleOf(g, `[data-grid-cell="4:${END_COLUMN}"]`, "background-color"),
      header: await styleOf(g, '[data-grid-cell="header:0"]', "background-image"),
    };
    expect(await styleOf(g, '[data-grid-cell="4:0"]', "background-image")).toBe("none");

    await cell(g, 4, 6).hover();
    const row = g.locator('[role="row"][aria-rowindex="6"]');
    await expect(row).toHaveCSS("background-color", /\//); // the translucent hover fill
    const fill = await row.evaluate((el) => getComputedStyle(el).backgroundColor);
    for (const [key, base] of [
      ["4:0", rest.start],
      ["4:1", rest.start],
      [`4:${END_COLUMN}`, rest.end],
    ] as const) {
      const sel = `[data-grid-cell="${key}"]`;
      // The base stays opaque; the row's fill is the image layer over it.
      expect(alphaOf(await styleOf(g, sel, "background-color")), key).toBe(1);
      expect(await styleOf(g, sel, "background-image"), key).toContain("linear-gradient");
      expect(await styleOf(g, sel, "background-image"), key).toContain(fill);
      expect(base).toBeTruthy();
    }
    // Plain cells carry the fill on the row, with no layer of their own.
    expect(await styleOf(g, '[data-grid-cell="4:6"]', "background-image")).toBe("none");

    // The header row keeps its opaque cover, hovered or not.
    await cell(g, "header", 0).hover();
    expect(await styleOf(g, '[data-grid-cell="header:0"]', "background-image")).toBe(rest.header);
    expect(alphaOf(await styleOf(g, '[data-grid-cell="header:0"]', "background-color"))).toBe(1);
    // A row that is not hovered is back to its plain cell.
    expect(await styleOf(g, '[data-grid-cell="4:0"]', "background-image")).toBe("none");
  });

  test("a selected row tints its pinned cells with the selected fill", async ({ page }) => {
    const g = await open(page);
    await page.mouse.move(0, 0);
    const checkbox = cell(g, 2, END_COLUMN).getByRole("checkbox");
    await checkbox.click();
    const row = g.locator('[role="row"][aria-rowindex="4"]');
    await expect(row).toHaveAttribute("data-state", "selected");
    await page.mouse.move(0, 0);
    const selected = await row.evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(alphaOf(selected)).toBe(1);
    for (const key of ["2:0", "2:1", `2:${END_COLUMN}`]) {
      const sel = `[data-grid-cell="${key}"]`;
      expect(alphaOf(await styleOf(g, sel, "background-color")), key).toBe(1);
      expect(await styleOf(g, sel, "background-image"), key).toContain(selected);
    }
    // A row that is not selected has no layer.
    expect(await styleOf(g, '[data-grid-cell="3:0"]', "background-image")).toBe("none");
  });
});

test.describe("right-to-left", () => {
  test("the start columns sit at the right, the end column at the left, and the keys still clear every pinned region", async ({
    page,
  }) => {
    const g = await open(page);
    await page.locator("#dx-direction-toggle").click();
    await expect(demoFrame(page, VARIANT)).toHaveAttribute("dir", "rtl");
    await expect(cell(g, 0, 2)).toBeVisible();
    // scrollLeft is negative in RTL: a distance from the inline start.
    for (const left of [0, -700, -1e9]) {
      await scrollGrid(g, 0, left);
      await expect(cell(g, 0, END_COLUMN)).toBeVisible();
      for (const row of [0, "header"] as const) {
        const first = await boxes(g, `${row}:0`);
        const second = await boxes(g, `${row}:1`);
        const last = await boxes(g, `${row}:${END_COLUMN}`);
        expect(Math.abs(first.right - first.gridRight)).toBeLessThan(1.5);
        expect(Math.abs(second.right - (first.gridRight - 112))).toBeLessThan(1.5);
        expect(Math.abs(last.left - last.gridLeft)).toBeLessThan(1.5);
      }
    }
    await scrollGrid(g, 0, 0);
    await cell(g, 3, 5).click();
    for (let column = 6; column <= END_COLUMN; column++) {
      await page.keyboard.press("ArrowLeft"); // inline end in RTL: the next column
      await expect.poll(() => focusedCell(page)).toBe(`3:${column}`);
      await expect.poll(() => covered(g, 3, column), `column ${column}`).toBe(false);
    }
    await page.keyboard.press("Home");
    await expect.poll(() => focusedCell(page)).toBe("3:0");
    await page.keyboard.press("ArrowRight"); // inline start in RTL: already at the first column
    await expect.poll(() => focusedCell(page)).toBe("3:0");
    await page.keyboard.press("ArrowLeft");
    await page.keyboard.press("ArrowLeft");
    await expect.poll(() => focusedCell(page)).toBe("3:2");
    await expect.poll(() => covered(g, 3, 2)).toBe(false);
  });
});
