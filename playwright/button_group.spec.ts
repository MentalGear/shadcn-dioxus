import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
  // Scope to the demo frame: the component page renders its own source in a
  // code viewer and carries sitewide chrome, either of which can contribute
  // another role="group" and make an unscoped locator ambiguous.
  const groups = page.locator("#component-preview-frame").first().getByRole("group");
  await expect(groups).toHaveCount(2);
  await expect(groups.first().getByRole("button", { name: "Today" })).toBeVisible();
  await expect(groups.last().getByRole("button", { name: "More" })).toBeVisible();
});

test.describe("Axe automated scan", () => {
  // Button Group has no overlay/expand/select interaction -- one state to scan.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
    await expectNoAxeViolations(page, "button_group: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// Backlog row 188 (g): the popover demo's chevron trigger wears the Button recipe (shadcn's
// `<Button variant="outline" size="icon">`), so it is the icon Button's 32px square with a 16px icon,
// as tall as the "Assistant" button it is joined to.
test("popover trigger is an outline icon Button beside its group sibling", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
  const frame = page.locator("#component-preview-frame-popover").first();
  const trigger = frame.getByRole("button", { name: "Assistant options" });
  await expect(trigger).toBeVisible({ timeout: 60_000 });
  const box = await trigger.evaluate((e) => {
    const r = e.getBoundingClientRect();
    const icon = e.querySelector("svg")!.getBoundingClientRect();
    return { w: r.width, h: r.height, iconW: icon.width, iconH: icon.height };
  });
  const siblingH = await frame
    .getByRole("button", { name: "Assistant", exact: true })
    .evaluate((e) => e.getBoundingClientRect().height);
  expect(box.w).toBeCloseTo(32, 0);
  expect(box.h).toBeCloseTo(32, 0);
  expect(box.iconW).toBeCloseTo(16, 0);
  expect(box.iconH).toBeCloseTo(16, 0);
  expect(box.h).toBeCloseTo(siblingH, 0);
});
