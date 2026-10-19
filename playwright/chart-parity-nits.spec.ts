/**
 * Chart demo parity nits (dev-docs/backlog row 125), measured on the gallery pages like the other demo-level checks:
 *   (a) the radar legend overhangs the 250px square by 10px (shadcn `mt-8` + Recharts `margin.bottom: -10`) while
 *       the container keeps the full square;
 *   (b) the pie-interactive select trigger leads with the selected slice's color swatch, and it follows the choice;
 *   (c) the area `icons` legend glyphs are foreground, not muted.
 */
import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const card = (page: Page, title: string) =>
  page.locator('[data-slot="card"]').filter({ has: page.locator('[data-slot="card-title"]', { hasText: title }) });

for (const title of ["Radar Chart - Legend", "Radar Chart - Icons"]) {
  test(`${title}: the legend overhangs the square by 10px`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/radar/`);
    const c = card(page, title);
    await c.scrollIntoViewIfNeeded();
    await page.waitForSelector('svg[data-slot="chart-svg"][data-measured="true"]');
    const m = await c.evaluate((el) => {
      const r = (e: Element) => e.getBoundingClientRect();
      const box = r(el.querySelector(".dx-chart")!), svg = r(el.querySelector('svg[data-slot="chart-svg"]')!);
      const legend = r(el.querySelector(".dx-chart-legend")!);
      return { box: box.height, svg: svg.height, over: legend.bottom - box.bottom, svgOver: legend.bottom - svg.bottom };
    });
    expect(m.box, "the container keeps the full square").toBeCloseTo(250, 0);
    expect(m.svg).toBeCloseTo(250, 0);
    expect(m.over).toBeCloseTo(10, 0);
  });
}

test("pie interactive: the select trigger shows the selected slice's swatch", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/charts/pie/`);
  const c = card(page, "Pie Chart - Interactive");
  await c.scrollIntoViewIfNeeded();
  const trigger = c.getByRole("combobox");
  const swatch = trigger.locator('[data-slot="chart-swatch"]');
  const bg = () => swatch.evaluate((e) => getComputedStyle(e).backgroundColor);
  await expect(swatch).toBeVisible();
  const box = await swatch.boundingBox();
  expect(box!.width).toBeCloseTo(12, 0);
  expect(box!.height).toBeCloseTo(12, 0);
  const january = await bg();
  expect(january).not.toBe("rgba(0, 0, 0, 0)");
  await trigger.click();
  await page.getByRole("option", { name: "February" }).click();
  await expect(trigger).toContainText("February");
  expect(await bg()).not.toBe(january);
});

test("area icons: the legend glyphs are foreground, not muted", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/charts/`);
  const c = card(page, "Area Chart - Icons");
  await c.scrollIntoViewIfNeeded();
  const glyph = c.locator('.dx-chart-legend [data-slot="chart-icon"]').first();
  const color = await glyph.evaluate((e) => getComputedStyle(e).color);
  const fg = await c.locator('[data-slot="card-title"]').evaluate((e) => getComputedStyle(e).color);
  const muted = await c.locator('[data-slot="card-description"]').evaluate((e) => getComputedStyle(e).color);
  expect(color).toBe(fg);
  expect(color).not.toBe(muted);
});
