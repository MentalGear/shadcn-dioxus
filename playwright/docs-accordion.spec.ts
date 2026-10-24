/**
 * The component page's docs accordion is the library's own `Accordion` (owner rule: the docs are built from the
 * library's own components; a native `<details>` draws its chevron past the divider and never animates).
 * `scripts/check-docs-own-components.sh` keeps native disclosures out of the docs chrome, and this spec is the
 * browser half:
 *
 *  1. an item animates open: the content's height passes through intermediate values over time;
 *  2. every item's chevron stays inside the item's border box (nothing is cut off at the side);
 *  3. a `#fragment` naming a heading INSIDE a closed item (a `###` or a props struct, both given ids by
 *     `build.rs`) opens that item and lands on the heading, on a fresh load and on `hashchange`.
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

const URL = `${BASE_URL}/component/accordion/`;
const items = (page: import("@playwright/test").Page) => page.locator(".dx-docs-accordion > .dx-accordion-item");
const itemOf = (page: import("@playwright/test").Page, id: string) =>
  page.locator(".dx-docs-accordion > .dx-accordion-item", { has: page.locator(`[id="${id}"]`) });

test("a docs accordion item animates open", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await gotoHydrated(page, URL);
  const item = itemOf(page, "docs-props-reference");
  await expect(item).toHaveAttribute("data-open", "false");
  const content = item.locator("> .dx-accordion-content");
  // Sample the content's height every frame from the click until it has been stable for a while.
  const sampling = content.evaluate(
    (el) =>
      new Promise<number[]>((resolve) => {
        const heights: number[] = [];
        const t0 = performance.now();
        const tick = () => {
          heights.push(el.getBoundingClientRect().height); // rest-ok: sampling the animation itself
          if (performance.now() - t0 < 1500) requestAnimationFrame(tick);
          else resolve(heights);
        };
        requestAnimationFrame(tick);
      }),
  );
  await item.locator("> h2 > button").click();
  const heights = await sampling;
  const final = heights[heights.length - 1];
  expect(final, "the opened item has content").toBeGreaterThan(40);
  const intermediate = new Set(heights.filter((h) => h > 1 && h < final - 1).map((h) => Math.round(h)));
  expect(intermediate.size, `heights: ${heights.map((h) => Math.round(h)).join(",")}`).toBeGreaterThanOrEqual(3);
  await expect(item).toHaveAttribute("data-open", "true");
});

test("every docs accordion chevron stays inside its section's border box", async ({ page }) => {
  for (const width of [1280, 390]) {
    await page.setViewportSize({ width, height: 900 });
    await gotoHydrated(page, URL);
    const count = await items(page).count();
    expect(count).toBeGreaterThan(2);
    for (let i = 0; i < count; i++) {
      const item = items(page).nth(i);
      const fit = await readAtRest(item, (el) => {
        const box = el.getBoundingClientRect();
        const icon = el.querySelector(":scope > h2 .dx-accordion-expand-icon")!.getBoundingClientRect();
        return {
          id: el.querySelector(":scope > h2")!.id,
          inside:
            icon.left >= box.left - 0.5 &&
            icon.right <= box.right + 0.5 &&
            icon.top >= box.top - 0.5 &&
            icon.bottom <= box.bottom + 0.5,
          icon: [icon.left, icon.right],
          box: [box.left, box.right],
        };
      });
      expect(fit.inside, `${width}px ${fit.id}: chevron ${fit.icon} outside ${fit.box}`).toBe(true);
    }
  }
});

test("a fragment to a heading inside a closed item opens it, on load and on hashchange", async ({ page, request }) => {
  const html = await (await request.get(URL)).text();
  // A docs.md `###` (accordion's "Browser support") and a props struct heading, both inside closed items.
  const sub = "docs-searchable-closed-content-browser-support";
  const props = html.match(/id="(docs-props-reference-[a-z0-9-]+)"/)?.[1];
  expect(html, `the page has no #${sub}`).toContain(`id="${sub}"`);
  expect(props, "no props struct heading id").toBeTruthy();

  await gotoHydrated(page, `${URL}#${sub}`);
  const subItem = itemOf(page, sub);
  await expect(subItem).toHaveAttribute("data-open", "true");
  await expect(page.locator(`[id="${sub}"]`)).toBeVisible();
  await expect(page.locator(`[id="${sub}"]`)).toBeInViewport();

  const propsItem = itemOf(page, props!);
  await expect(propsItem).toHaveAttribute("data-open", "false");
  await page.evaluate((id) => (location.hash = id), props!);
  await expect(propsItem).toHaveAttribute("data-open", "true");
  await expect(page.locator(`[id="${props}"]`)).toBeVisible();
  await expect(page.locator(`[id="${props}"]`)).toBeInViewport();
});
