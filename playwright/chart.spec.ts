import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/chart/`);
  const frame = page.locator("#component-preview-frame").first();

  // the chart renders as a single accessible image. An attribute selector,
  // not getByRole("img") -- this page's Select trigger also renders a bare
  // decorative chevron <svg> with no explicit role, which Chromium's own
  // accessibility tree still computes an implicit "img" role for, so
  // getByRole("img").first() picks up that unrelated icon instead (it
  // resolves to `svg.dx-select-expand-icon`, not the chart).
  const svg = frame.locator('svg[role="img"]').first();
  await expect(svg).toBeVisible();
  await expect(svg).toHaveAccessibleName(/.+/);

  // hovering a data point's hit-band shows the tooltip
  const tooltip = frame.locator('[data-slot="chart-tooltip"]');
  await frame.locator('[data-slot="chart-hit-band"]').first().hover();
  await expect(tooltip).toHaveAttribute("data-state", "open");
  await expect(tooltip).toBeVisible();

  // moving the mouse off the chart hides it again
  await page.mouse.move(0, 0);
  await expect(tooltip).toHaveAttribute("data-state", "closed");
});

test.describe("Axe automated scan", () => {
  test("tooltip open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/chart/`);
    const frame = page.locator("#component-preview-frame").first();
    await frame.locator('[data-slot="chart-hit-band"]').first().hover();
    await expect(frame.locator('[data-slot="chart-tooltip"]')).toHaveAttribute("data-state", "open");
    await expectNoAxeViolations(page, "chart: tooltip open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// The tooltip must not cover the legend (the demo page's labels over
// the legend). `ChartTooltip` positions itself with `left`/`top` PERCENTAGES of
// `Chart`'s own logical width/height -- i.e. of the svg's own box -- so the
// tooltip's containing block must be that box: `.dx-chart` (a flex column
// holding the svg wrapper, the tooltip, AND the legend as siblings) must not be
// it, or those percentages resolve against the COMBINED svg+legend box. With
// `ChartLegend`'s `vertical_align: Top` (the `bar` demo variant, the ONLY one
// affected in practice -- the default/bottom alignment's tooltip only ever
// extends UPWARD, away from a legend below it, so it is never at risk), that
// places the tooltip's anchor -- and often its entire body -- inside the
// legend's own row, e.g. hovering the `bar` demo's 2nd bar renders the tooltip
// directly over the legend, fully obscuring "Desktop".
//
// Two things are asserted below, independently:
//  1. The STATIC x-axis label group and the plot svg's own box never overlap
//     the legend, at any viewport tested (measured real rendered boxes, not
//     the CSS). A real invariant worth locking in.
//  2. The DYNAMIC open tooltip never overlaps the legend, for the `bar`
//     variant specifically (the failure is a bar chart's hit-band
//     tooltip overlapping the legend).
test.describe("Legend never overlaps the chart's axis labels, svg box, or open tooltip", () => {
  function intersects(
    a: { x: number; y: number; width: number; height: number } | null,
    b: { x: number; y: number; width: number; height: number } | null,
  ): boolean {
    if (!a || !b) return false;
    return a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y;
  }

  // Every variant that has a legend, and how many independent `.dx-chart`
  // roots its frame renders. `stacked` renders two (area, then bar),
  // sharing one legend each; every other variant renders one. `line` (a
  // single series) has no `ChartLegend` at all -- nothing to check there.
  const VARIANTS: { frame: string; chartCount: number }[] = [
    { frame: "#component-preview-frame", chartCount: 1 },
    { frame: "#component-preview-frame-bar", chartCount: 1 },
    { frame: "#component-preview-frame-stacked", chartCount: 2 },
  ];

  for (const { frame: frameSel, chartCount } of VARIANTS) {
    test(`${frameSel}: x-axis labels and the svg box don't overlap the legend`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/chart/`);
      const charts = page.locator(frameSel).first().locator(".dx-chart");
      await expect(charts).toHaveCount(chartCount);
      for (let i = 0; i < chartCount; i++) {
        const chart = charts.nth(i);
        const legendBox = await chart.locator('[data-slot="chart-legend"]').first().boundingBox();
        const axisBox = await chart
          .locator('[data-slot="chart-axis"][data-axis="x"]')
          .first()
          .boundingBox();
        const svgBox = await chart.locator('svg[role="img"]').first().boundingBox();
        expect(intersects(axisBox, legendBox), `chart ${i}: x-axis labels vs legend`).toBe(false);
        expect(intersects(svgBox, legendBox), `chart ${i}: svg box vs legend`).toBe(false);
      }
    });

    test(`${frameSel}: the open tooltip doesn't overlap the legend, for every data point`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/chart/`);
      const charts = page.locator(frameSel).first().locator(".dx-chart");
      for (let i = 0; i < chartCount; i++) {
        const chart = charts.nth(i);
        const legend = chart.locator('[data-slot="chart-legend"]').first();
        const tooltip = chart.locator('[data-slot="chart-tooltip"]').first();
        const hitBands = chart.locator('[data-slot="chart-hit-band"]');
        const count = await hitBands.count();
        // Every hit-band for the small, fixed demos (<=6 here); a stride
        // for `main`'s 90-point dataset keeps this fast while still
        // covering its full value range (always including index 0).
        const step = Math.max(1, Math.floor(count / 10));
        for (let b = 0; b < count; b += step) {
          await hitBands.nth(b).hover();
          await expect(tooltip).toHaveAttribute("data-state", "open");
          // `legendBox` is re-measured on EVERY iteration, right alongside
          // `tooltipBox` -- not hoisted above the loop. `boundingBox()`
          // returns VIEWPORT-relative coordinates, and `.hover()` force-
          // scrolls its target into view as part of Playwright's own
          // actionability protocol while a bare `boundingBox()` does not;
          // hoisting `legendBox` above the loop would measure it against the
          // page's initial (pre-scroll) position while `tooltipBox` is
          // measured after `.hover()` has already scrolled the page --
          // two boxes in two different scroll states are not comparable,
          // and that gives a false pass in this exact test, even against
          // CSS that lets the tooltip overlap the legend.
          const legendBox = await legend.boundingBox();
          const tooltipBox = await tooltip.boundingBox();
          expect(intersects(tooltipBox, legendBox), `chart ${i}, hit-band ${b}: tooltip vs legend`).toBe(
            false,
          );
        }
        await page.mouse.move(0, 0);
      }
    });
  }
});

// Every chart on the deployed SSG pages must resolve its series colors (a black/grey chart
// with nearly invisible lines is the failure). `ChartContainer` therefore emits its `--color-<slot>`
// declarations as an inline `style` attribute on the container (no `<style>` element at all): built as
// a `<style>` rule from a dynamic text node, SSR puts a `<!--node-id..-->` hydration marker inside that
// raw-text element, which CSS parses as part of the selector (`node-id444-- > [data-chart=...]`), so
// the rule never matches, `--series-color: var(--color-desktop)` is invalid, and marks fall back to
// `fill: black` / `stroke: none`.
test.describe("Series colors resolve (no dead --color-<slot> rule)", () => {
  test("the container carries its --color declarations inline, with no <style> child", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/chart/`);
    const container = page.locator("#component-preview-frame-bar").first().locator("[data-chart]").first();
    await expect(container).toHaveAttribute("style", /--color-desktop:\s*var\(--dx-chart-1\)/);
    await expect(container.locator("style")).toHaveCount(0);
  });

  test("a bar's computed fill is the resolved --dx-chart-1 color, not the black fallback", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/chart/`);
    const frame = page.locator("#component-preview-frame-bar").first();
    const bar = frame.locator('[data-slot="chart-series"][data-series="desktop"] [data-slot="chart-bar"]').first();
    await expect(bar).toBeVisible();

    const { fill, seriesColor, expected } = await bar.evaluate((el) => {
      const g = el.closest('[data-slot="chart-series"]') as Element;
      // Resolve `--dx-chart-1` to a concrete color by letting the browser
      // paint it on a throwaway probe inside the same subtree, so any
      // theme-scoped custom properties apply exactly as they do to the bar.
      const probe = document.createElement("span");
      probe.style.color = "var(--dx-chart-1)";
      g.ownerDocument.body.appendChild(probe);
      const expected = getComputedStyle(probe).color;
      probe.remove();
      return {
        fill: getComputedStyle(el).fill,
        seriesColor: getComputedStyle(g).getPropertyValue("--series-color").trim(),
        expected,
      };
    });

    expect(seriesColor).not.toBe("");
    expect(fill).not.toBe("rgb(0, 0, 0)");
    expect(fill).toBe(expected);
  });
});
