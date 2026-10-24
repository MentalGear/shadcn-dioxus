import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL = `${BASE_URL}/component/button/`;
const GOTO = { timeout: 20 * 60 * 1000 };

const variant = (page: Page, name: string) => page.locator(`#component-preview-frame-${name}`).first();

// The component's stylesheet is linked after mount: wait until it has applied
// before any test reads a computed style.
async function styled(page: Page) {
  await expect
    .poll(() => variant(page, "as_link").getByRole("link", { name: "Login" }).evaluate((el) => getComputedStyle(el).height), {
      timeout: 60_000,
    })
    .toBe("28px");
}

test("as renders the button as an anchor that keeps the button's attributes", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  await styled(page);
  const link = variant(page, "as_link").getByRole("link", { name: "Login" });
  await expect(link).toHaveAttribute("href", "#");
  await expect(link).toHaveClass(/dx-button/);
  await expect(link).toHaveAttribute("data-style", "secondary");
  await expect(link).toHaveAttribute("data-size", "sm");
  // The anchor is not underlined: the theme's anchor reset covers dx- links.
  expect(await link.evaluate((el) => getComputedStyle(el).textDecorationLine)).toBe("none");
});

test("the onclick handler reaches the rendered anchor", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  await styled(page);
  const demo = variant(page, "as_link");
  await expect(demo.getByText("Clicked 0 time(s)")).toBeVisible();
  await demo.getByRole("link", { name: "Login" }).click();
  await expect(demo.getByText("Clicked 1 time(s)")).toBeVisible();
});
