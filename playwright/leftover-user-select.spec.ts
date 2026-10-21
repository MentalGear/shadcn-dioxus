/**
 * Backlog row 294 leftovers: `kbd`, `.dx-label` and the input-group addon are not text (shadcn new-york-v4 gives
 * all three `select-none`). None is a button or widget role, so the theme's shared control rule does not reach
 * them; each sheet declares it with a reasoned `USER_SELECT_ALLOW` entry in `scripts/check-theme-baseline.sh`.
 *
 * The other side stays true: a text field inside an input group keeps its selectable value.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8150 npx playwright test --config=ssg.local.config.ts leftover-user-select.spec.ts
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { baseUrlOr } from "./base-url";
import { gotoHydrated } from "./hydration";

const BASE = baseUrlOr("http://127.0.0.1:8080");

async function open(page: Page, path: string) {
  await page.route(
    (url) => !["127.0.0.1", "localhost"].includes(url.hostname),
    (route) => route.abort(),
  );
  await gotoHydrated(page, `${BASE}${path}`);
}

const CASES: { page: string; selector: string; what: string }[] = [
  { page: "kbd", selector: ".dx-kbd", what: "kbd" },
  { page: "label", selector: ".dx-label", what: "label" },
  { page: "field", selector: ".dx-label", what: "field label" },
  { page: "input_group", selector: ".dx-input-group-addon", what: "input-group addon" },
];

for (const c of CASES) {
  test(`every ${c.what} on /component/${c.page}/ computes user-select: none`, async ({ page }) => {
    await open(page, `/component/${c.page}/`);
    const styles = await page.evaluate((sel) => {
      return [...document.querySelectorAll<HTMLElement>(sel)].map((el) => {
        const cs = getComputedStyle(el);
        return { select: cs.userSelect, webkit: cs.getPropertyValue("-webkit-user-select") || cs.userSelect };
      });
    }, c.selector);
    expect(styles.length, `the page renders a ${c.what}`).toBeGreaterThan(0);
    for (const s of styles) {
      expect(s.select).toBe("none");
      expect(s.webkit).toBe("none");
    }
  });
}

test("a double-click on a label leaves an empty selection", async ({ page }) => {
  await open(page, "/component/label/");
  const label = page.locator(".dx-label").first();
  await label.dblclick();
  expect(await page.evaluate(() => String(getSelection()))).toBe("");
});

test("the field beside an addon stays selectable", async ({ page }) => {
  await open(page, "/component/input_group/");
  const control = page.locator(".dx-input-group-control").first();
  expect(await control.evaluate((el) => getComputedStyle(el).userSelect)).not.toBe("none");
  await control.fill("selectable value");
  await control.selectText();
  expect(await control.evaluate((el: HTMLInputElement) => el.value.slice(el.selectionStart ?? 0, el.selectionEnd ?? 0))).toBe(
    "selectable value",
  );
});
