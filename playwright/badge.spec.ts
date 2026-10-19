import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL = `${BASE_URL}/component/badge/`;
const GOTO = { timeout: 20 * 60 * 1000 };

// Scope to a demo frame. The component page also renders each demo's own source
// in a syntax-highlighted code viewer, so every visible string appears twice.
const main = (page: Page) => page.locator("#component-preview-frame").first();
const variant = (page: Page, name: string) => page.locator(`#component-preview-frame-${name}`).first();
const TRANSPARENT = "rgba(0, 0, 0, 0)";

// The component's stylesheet is linked after mount: wait until it has applied
// before any test reads a computed style.
async function styled(page: Page) {
  await expect
    .poll(() => main(page).locator('.dx-badge[data-style="primary"]').first().evaluate((el) => getComputedStyle(el).backgroundColor), {
      timeout: 60_000,
    })
    .not.toBe(TRANSPARENT);
}

test("the main demo renders the seven variants, Ghost and Link included", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  await styled(page);
  const badges = main(page).locator("span.dx-badge");
  await expect(badges).toHaveCount(7);
  const styles = await badges.evaluateAll((els) => els.map((e) => e.getAttribute("data-style")));
  expect(styles).toEqual(["primary", "secondary", "destructive", "outline", "ghost", "link", "secondary"]);
  await expect(main(page).getByText("Ghost", { exact: true })).toBeVisible();
  await expect(main(page).getByText("Link", { exact: true })).toBeVisible();
});

test("Ghost is transparent at rest and fills with muted on hover", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  await styled(page);
  const ghost = main(page).locator('.dx-badge[data-style="ghost"]');
  await expect(ghost).toBeVisible();
  expect(await ghost.evaluate((el) => getComputedStyle(el).backgroundColor)).toBe(TRANSPARENT);
  await ghost.hover();
  await expect
    .poll(() => ghost.evaluate((el) => getComputedStyle(el).backgroundColor), { timeout: 10_000 })
    .not.toBe(TRANSPARENT);
});

test("Link is primary-coloured text and underlines only on hover", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  await styled(page);
  const link = main(page).locator('.dx-badge[data-style="link"]');
  await expect(link).toBeVisible();
  expect(await link.evaluate((el) => getComputedStyle(el).backgroundColor)).toBe(TRANSPARENT);
  expect(await link.evaluate((el) => getComputedStyle(el).textDecorationLine)).toBe("none");
  await link.hover();
  await expect
    .poll(() => link.evaluate((el) => getComputedStyle(el).textDecorationLine), { timeout: 10_000 })
    .toBe("underline");
});

test("as renders the badge as a real link that keeps the badge's look", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  await styled(page);
  const demo = variant(page, "as_link");
  const link = demo.getByRole("link", { name: "Open Link" });
  await expect(link).toHaveAttribute("href", "#link");
  await expect(demo.locator("span.dx-badge")).toHaveCount(0);
  // Its `display` is the flex item's blockified `flex`, so the look is checked by the badge's 20px height.
  const cls = await link.evaluate((el) => ({ tag: el.tagName, badge: el.classList.contains("dx-badge"), height: getComputedStyle(el).height }));
  expect(cls).toEqual({ tag: "A", badge: true, height: "20px" });
  // The icon sits after the text, inside the link.
  await expect(link.locator("svg")).toHaveCount(1);
});

test("as: the link is focusable and takes the theme's focus ring, not the UA outline", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  await styled(page);
  const link = variant(page, "as_link").getByRole("link", { name: "Open Link" });
  await link.focus();
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Tab");
  await expect(link).toBeFocused();
  await expect
    .poll(() => link.evaluate((el) => getComputedStyle(el).boxShadow), { timeout: 10_000 })
    .not.toBe("none");
  expect(await link.evaluate((el) => getComputedStyle(el).outlineStyle)).toBe("none");
});

test.describe("Axe automated scan", () => {
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, URL, GOTO);
    await styled(page);
    await expectNoAxeViolations(page, "badge: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
