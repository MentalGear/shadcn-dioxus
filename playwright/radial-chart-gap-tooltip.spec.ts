import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// shadcn's Recharts RadialBarChart + ChartTooltip only activates the tooltip
// while the pointer is over a bar sector: the empty remainder of a ring and
// the gap between two rings are outside every sector, so the tooltip hides
// there. This spec pins that behaviour (hover resolves from coordinates in
// `primitives/src/chart/components/pointer.rs`, sectors from `series/radial.rs`).
// The `main` variant has no background track, so `elementFromPoint` tells a
// bar from a gap.

const URL = `${BASE_URL}/component/?name=radial_chart&`;
let FRAME = "#component-preview-frame";

type Pt = { x: number; y: number };

/** Along a ray from the chart centre, the radial runs covered by a bar (px from the centre). */
async function ringRuns(page: Page): Promise<{ bar: Pt[]; gap: Pt[] }> {
  return page.locator(`${FRAME} [data-slot="chart-arc"]`).first().evaluate((first) => {
    const svg = first.closest("svg")!;
    const r = svg.getBoundingClientRect();
    const cx = r.left + r.width / 2;
    const cy = r.top + r.height / 2;
    const reach = Math.min(r.width, r.height) / 2;
    const arcs = Array.from(svg.querySelectorAll('[data-slot="chart-arc"]'));
    for (let deg = 3; deg < 360; deg += 7) {
      const a = (deg * Math.PI) / 180;
      const at = (d: number) => ({ x: cx + d * Math.sin(a), y: cy - d * Math.cos(a) });
      const runs: [number, number][] = [];
      let start = -1;
      for (let d = 0; d <= reach; d += 0.5) {
        const p = at(d);
        const el = document.elementFromPoint(p.x, p.y);
        const on = !!el && arcs.includes(el);
        if (on && start < 0) start = d;
        if (!on && start >= 0) {
          runs.push([start, d - 0.5]);
          start = -1;
        }
      }
      if (runs.length === arcs.length) {
        const bar = runs.map(([s, e]) => at((s + e) / 2));
        const gap = runs.slice(1).map(([s], i) => at((runs[i][1] + s) / 2));
        return { bar, gap };
      }
    }
    throw new Error("no ray crosses every ring");
  });
}

for (const variant of ["", "-label", "-grid"]) {
  test(`radial${variant || "-main"}: tooltip shows over a bar and hides over the gap between rings`, async ({ page }) => {
    FRAME = `#component-preview-frame${variant}`;
  await page.emulateMedia({ reducedMotion: "reduce" });
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const tooltip = page.locator(`${FRAME} [data-slot="chart-tooltip"]`).first();
  await page.locator(`${FRAME} [data-slot="chart-arc"]`).first().scrollIntoViewIfNeeded();
  const { bar, gap } = await ringRuns(page);
  expect(gap.length).toBeGreaterThan(2);

  for (let i = 0; i < gap.length; i++) {
    await page.mouse.move(bar[i].x, bar[i].y);
    await expect(tooltip, `bar ${i}`).toHaveAttribute("data-state", "open");
    await page.mouse.move(gap[i].x, gap[i].y, { steps: 6 });
    await expect(tooltip, `gap after ring ${i}`).toHaveAttribute("data-state", "closed");
    // Back on the next ring's bar it reopens.
    await page.mouse.move(bar[i + 1].x, bar[i + 1].y);
    await expect(tooltip, `bar ${i + 1}`).toHaveAttribute("data-state", "open");
  }
});
}
