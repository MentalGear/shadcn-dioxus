/**
 * CARD FIT (backlog row 280): no home-page card may be narrower than its content, at any width.
 *
 * The owner saw the "Reply to roadmap thread" showcase card cut its footer off: B, I, U, Save draft and Post sat
 * in one `display: flex` row with no wrap, wider than the card, and `.dx-widget-card`'s `overflow: hidden`
 * clipped Post in half. A width sweep of the deployed site found it was a CLASS, not one card: nine cards clipped
 * or spilled their content somewhere between 320 and 1920 px --
 *
 *   - showcase cards whose content does not wrap (the composer's footer; the Schedule card, whose calendar demo
 *     carried its own 20px padding inside the card's);
 *   - component cards whose preview column was too narrow for the demo: the gallery switched to two columns on a
 *     VIEWPORT breakpoint, so with the docs sidebar open the preview column was 286 px at 780 px wide (Field, Input
 *     Group, Pagination, Skeleton, Chart Tooltip, Resizable, 780-1160 px);
 *   - demos with a fixed width wider than a phone's card (Skeleton's shadcn 250 px lines, Resizable's 450 px group
 *     capped by too small a viewport margin, an Input whose UA intrinsic width did not shrink in a flex row).
 *
 * The construction is this spec: it loads `/`, and at every width of the sweep requires each showcase card
 * (`.dx-widget-card`) and each component card's preview frame (`.dx-component-card-preview`) to be at least as
 * wide as what it lays out (`scrollWidth <= clientWidth`, 1px of rounding allowed). It does not know which rule
 * did it, so a new member of the class -- a new block, a new demo, a changed breakpoint -- fails here at the width
 * where it shows, whatever its cause. A box that scrolls on purpose is its own scroll container and does not
 * count toward its card's `scrollWidth`; a popup in the top layer does not either.
 *
 * Widths: 320 (WCAG reflow) to 1920 in steps of 16, plus every breakpoint of the home page and its +-1 px
 * neighbours. `html[data-cv-settle]` forces every `content-visibility: auto` card to render, because a skipped
 * card has no layout and would pass by measuring nothing.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8090 npx playwright test --config=ssg.local.config.ts card-fit.spec.ts
 */
import { test, expect } from "./fixtures";
import { baseUrlOr } from "./base-url";
import { gotoHydrated } from "./hydration";

const BASE = baseUrlOr("http://127.0.0.1:8080");
const BREAKPOINTS = [640, 768, 1024, 1100, 1180, 1280];

function widths(): number[] {
  const set = new Set<number>();
  for (let w = 320; w <= 1920; w += 16) set.add(w);
  for (const b of BREAKPOINTS) for (const d of [-1, 0, 1]) set.add(b + d);
  return [...set].sort((a, b) => a - b);
}

test("every home-page card is as wide as its content at every width", async ({ page }) => {
  test.setTimeout(180_000);
  await page.setViewportSize({ width: 1280, height: 900 });
  await gotoHydrated(page, `${BASE}/`);

  const failures: string[] = [];
  for (const width of widths()) {
    await page.setViewportSize({ width, height: 900 });
    const overflows = await page.evaluate(async () => {
      document.documentElement.setAttribute("data-cv-settle", "");
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      const out: string[] = [];
      const cards = document.querySelectorAll<HTMLElement>(".dx-widget-card, .dx-component-card-preview");
      for (const card of cards) {
        const over = card.scrollWidth - card.clientWidth;
        if (over <= 1) continue;
        const owner = card.closest(".dx-component-card") ?? card;
        const name = (owner.querySelector(".dx-component-card-title, h3")?.textContent ?? "").trim();
        out.push(`${card.classList[0]} "${name}" content is ${over}px wider than the card`);
      }
      return out;
    });
    for (const o of overflows) failures.push(`${width}px: ${o}`);
  }
  expect(failures, failures.join("\n")).toEqual([]);
});
