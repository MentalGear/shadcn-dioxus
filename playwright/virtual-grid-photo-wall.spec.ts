import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { demoFrame, demoRole } from "./demo-frame";
import { gotoHydrated } from "./hydration";

// VirtualGrid page, `photo_wall` demo: a wall of 90,000 mixed-aspect photos in uniform cells of the two-axis VirtualGrid, with drag-to-pan, a minimap, a random jump and a Dialog preview.
// Controls are found by role, position and data attributes, never by demo copy (row 220).

const PAGE_URL = `${BASE_URL}/component/?name=virtual_grid&`;
const VARIANT = "photo_wall";
const MAX_TILES = 400;

const wall = (page: Page) => demoFrame(page, VARIANT).getByRole("grid");
const tiles = (grid: Locator) => grid.locator("[data-grid-cell]");
const counter = (page: Page) => demoFrame(page, VARIANT).locator("[data-photo-wall-count]");
const minimap = (page: Page) => demoFrame(page, VARIANT).locator("[data-photo-wall-minimap]");
const scrollOf = (grid: Locator) =>
  grid.evaluate((el) => ({ left: Math.abs(el.scrollLeft), top: el.scrollTop }));
const counted = async (page: Page) => Number((await counter(page).innerText()).match(/^(\d+)/)![1]);

async function open(page: Page) {
  await page.setViewportSize({ width: 1100, height: 900 });
  await gotoHydrated(page, PAGE_URL);
  const grid = wall(page);
  await grid.scrollIntoViewIfNeeded();
  await expect(tiles(grid).first()).toBeVisible();
  return grid;
}

test.describe("virtual_grid photo_wall", () => {
  test("mounts a small window of 90,000 cells and reports it", async ({ page }) => {
    const grid = await open(page);
    await expect(grid).toHaveAttribute("aria-rowcount", "300");
    await expect(grid).toHaveAttribute("aria-colcount", "300");
    const n = await tiles(grid).count();
    expect(n).toBeGreaterThan(10);
    expect(n).toBeLessThan(MAX_TILES);
    await expect(counter(page)).toContainText("90,000");
    await expect.poll(() => counted(page)).toBe(n);
  });

  test("cells keep identical boxes while the photos differ in aspect; the switch changes the fit", async ({ page }) => {
    const grid = await open(page);
    const boxes = await tiles(grid).evaluateAll((els) => [...new Set(els.map((e) => `${e.clientWidth}x${e.clientHeight}`))]);
    expect(boxes).toHaveLength(1);
    const ratios = await grid.locator("[data-photo-img]").evaluateAll((els) => [...new Set(els.map((e) => e.getAttribute("data-photo-img")))]);
    expect(ratios.length).toBeGreaterThanOrEqual(4);
    const img = grid.locator("[data-photo-img]").first();
    const fit = () => img.evaluate((el) => getComputedStyle(el).objectFit);
    expect(await fit()).toBe("contain");
    // photo_wall demo: the first switch crops photos to fill their cells
    await demoRole(page, "switch", 0, VARIANT).click();
    await expect.poll(fit).toBe("cover");
    const after = await tiles(grid).evaluateAll((els) => [...new Set(els.map((e) => `${e.clientWidth}x${e.clientHeight}`))]);
    expect(after).toEqual(boxes);
    await demoRole(page, "switch", 0, VARIANT).click();
    await expect.poll(fit).toBe("contain");
  });

  test("scrolling right and down mounts far cells on both axes", async ({ page }) => {
    const grid = await open(page);
    await grid.evaluate((el) => {
      el.scrollLeft = 128 * 200;
      el.scrollTop = 128 * 150;
    });
    await expect(grid.locator('[data-grid-cell="150:200"]')).toBeAttached();
    await expect(grid.locator('[data-grid-cell="0:0"]')).toHaveCount(0);
    expect(await tiles(grid).count()).toBeLessThan(MAX_TILES);
    await expect.poll(async () => (await counted(page)) === (await tiles(grid).count())).toBe(true);
    const box = demoFrame(page, VARIANT).locator("[data-photo-wall-viewport]");
    await expect.poll(() => box.evaluate((el) => parseFloat(getComputedStyle(el).top))).toBeGreaterThan(20);
  });

  test("dragging pans both axes; a drag is not a click, a still press is", async ({ page }) => {
    const grid = await open(page);
    const box = (await grid.boundingBox())!;
    const x = box.x + box.width / 2;
    const y = box.y + 200;
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x - 150, y - 90, { steps: 8 });
    await page.mouse.up();
    const after = await scrollOf(grid);
    expect(after.left).toBeGreaterThan(100);
    expect(after.top).toBeGreaterThan(60);
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await page.mouse.click(x, y);
    await expect(page.getByRole("dialog")).toBeVisible();
  });

  test("the minimap jumps to a spot of the wall", async ({ page }) => {
    const grid = await open(page);
    const m = (await minimap(page).boundingBox())!;
    await page.mouse.click(m.x + m.width * 0.75, m.y + m.height * 0.5);
    const at = await scrollOf(grid);
    const total = await grid.evaluate((el) => ({ w: el.scrollWidth, h: el.scrollHeight }));
    expect(at.left).toBeGreaterThan(total.w * 0.5);
    expect(at.top).toBeGreaterThan(total.h * 0.3);
    await expect(tiles(grid).first()).toBeAttached();
    expect(await tiles(grid).count()).toBeLessThan(MAX_TILES);
  });

  test("jump to random photo focuses a rendered cell", async ({ page }) => {
    const grid = await open(page);
    // photo_wall demo: the first button is "jump to random photo" (the switch has role switch)
    await demoRole(page, "button", 0, VARIANT).click();
    await expect
      .poll(() => page.evaluate(() => document.activeElement?.getAttribute("data-grid-cell") ?? null))
      .not.toBeNull();
    const key = await page.evaluate(() => document.activeElement!.getAttribute("data-grid-cell")!);
    await expect(grid.locator(`[data-grid-cell="${key}"]`)).toBeFocused();
    expect(await tiles(grid).count()).toBeLessThan(MAX_TILES);
  });

  test("Enter on a cell opens the photo at its real aspect ratio", async ({ page }) => {
    const grid = await open(page);
    await grid.locator('[data-grid-cell="0:0"]').focus();
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("Enter");
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    const preview = dialog.locator("[data-photo-preview]");
    await expect(preview).toBeVisible();
    const [w, h] = (await preview.getAttribute("data-photo-preview"))!.split(":").map(Number);
    await expect
      .poll(() => preview.evaluate((el) => el.clientWidth / el.clientHeight))
      .toBeCloseTo(w / h, 1);
    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
  });

  test("mobile width: the wall scrolls, the page does not", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 800 });
    await gotoHydrated(page, PAGE_URL);
    const grid = wall(page);
    await grid.scrollIntoViewIfNeeded();
    await expect(tiles(grid).first()).toBeVisible();
    // viewport-ok: measures page overflow, not an overlay's containing block
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    await expect(minimap(page)).toBeVisible();
  });

  test("mirrors in RTL: the inline-start scroll offset windows columns and ArrowLeft moves on", async ({ page }) => {
    const grid = await open(page);
    await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
    await grid.evaluate((el) => (el.scrollLeft = -128 * 100));
    await expect(grid.locator('[data-grid-cell="0:100"]')).toBeAttached();
    expect(await tiles(grid).count()).toBeLessThan(MAX_TILES);
    await grid.locator('[data-grid-cell="0:100"]').focus();
    await page.keyboard.press("ArrowLeft");
    await expect(grid.locator('[data-grid-cell="0:101"]')).toBeFocused();
  });
});
