/**
 * Controlled vs uncontrolled guidance (backlog rows 392 and 387): the explanation is written once, in the
 * `/docs#controlled` section, and every component page whose props hold a controlled pair carries a
 * "Controlled state" block that `preview/build.rs` generates from the page's own props and that links there.
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const BLOCKS: Record<string, string[]> = {
  dialog: ["open", "on_open_change", "default_open"],
  alert_dialog: ["open", "on_open_change", "default_open"],
  sheet: ["open", "on_open_change", "default_open"],
  popover: ["open", "on_open_change", "default_open"],
  toggle: ["pressed", "on_pressed_change", "default_pressed"],
  input_otp: ["value", "on_value_change", "default_value"],
  tag_group: ["value", "on_value_change", "default_value"],
  date_picker: ["selected_date", "on_value_change"],
  color_picker: ["color", "on_color_change"],
};

for (const [name, tokens] of Object.entries(BLOCKS)) {
  test(`${name}: the Controlled state block lists its pairs and links the explanation`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/${name}/`);
    const item = page.locator(".dx-docs-accordion > .dx-accordion-item", { has: page.locator('[id="docs-controlled-state"]') });
    await expect(item).toHaveCount(1);
    await item.locator("> h3 > button").click();
    const body = item.locator("> .dx-accordion-content");
    for (const token of tokens) {
      await expect(body.locator("code", { hasText: new RegExp(`^${token}$`) }).first()).toBeVisible();
    }
    await expect(body.getByRole("link", { name: "Controlled and uncontrolled state" })).toHaveAttribute("href", /\/docs#controlled$/);
  });
}

test("a page with no controlled pair has no block", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/badge/`);
  await expect(page.locator('[id="docs-controlled-state"]')).toHaveCount(0);
});

test("the docs page explains controlled, uncontrolled, forms and the frozen literal at #controlled", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/docs#controlled`);
  const section = page.locator("section.dx-docs-section", { has: page.locator("h2#controlled") });
  await expect(section).toBeVisible();
  for (const word of ["Controlled", "Uncontrolled", "Forms"]) {
    await expect(section.locator("strong", { hasText: word }).first()).toBeVisible();
  }
  await expect(section.locator("pre").first()).toContainText("Frozen");
  await expect(page.locator("h2#controlled")).toBeInViewport();
});
