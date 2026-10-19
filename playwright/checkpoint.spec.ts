import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL_ = `${BASE_URL}/component/?name=checkpoint&`;
const GOTO = { timeout: 20 * 60 * 1000 };
const frame = (page: Page, name?: string) =>
  page.locator(name ? `#component-preview-frame-${name}` : "#component-preview-frame").first();
const styled = (page: Page) =>
  expect
    .poll(() => frame(page).locator('[data-slot="checkpoint"]').first().evaluate((el) => getComputedStyle(el).display), {
      timeout: 60_000,
    })
    .toBe("flex");

test("a checkpoint is a separator marker, not role=separator, with a uniquely named Restore button", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page);
  const rows = demo.locator('[data-slot="checkpoint"]');
  await expect(rows).toHaveCount(2);
  await expect(rows.first()).toHaveAttribute("data-variant", "separator");
  await expect(demo.getByRole("separator")).toHaveCount(0);
  const names = await demo.getByRole("button").evaluateAll((els) => els.map((e) => e.getAttribute("aria-label")));
  expect(names).toEqual(["Restore checkpoint: Before refactor", "Restore checkpoint: Tests passing"]);
  // Label in name: the visible word starts the accessible name.
  await expect(demo.getByRole("button", { name: "Restore checkpoint: Tests passing" })).toHaveText("Restore");
});

test("Restore reports the click (mouse and keyboard)", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page);
  await demo.getByRole("button", { name: /Before refactor/ }).click();
  await expect(demo.getByRole("status")).toHaveText("Restored: Before refactor");
  const second = demo.getByRole("button", { name: /Tests passing/ });
  await second.focus();
  await page.keyboard.press("Enter");
  await expect(demo.getByRole("status")).toHaveText("Restored: Tests passing");
});

test("thread: restore drops every later message and disables the current point", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page, "thread");
  await expect(demo.locator('[data-slot="checkpoint"]')).toHaveCount(3);
  await expect(demo.getByRole("button", { name: /After cleanup/ })).toBeDisabled();
  await demo.getByRole("button", { name: /After rename/ }).click();
  await expect(demo.locator('[data-slot="checkpoint"]')).toHaveCount(1);
  await expect(demo.getByText("Split into three files")).toHaveCount(0);
  await expect(demo.getByText("Done: config.rs is now settings.rs.")).toBeVisible();
  await expect(demo.getByRole("button", { name: /After rename/ })).toBeDisabled();
});

for (const mode of ["light", "dark"] as const) {
  test(`checkpoint demos have no axe violations (${mode})`, async ({ page }) => {
    await gotoHydrated(page, URL_, GOTO);
    await styled(page);
    await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
    await expectNoAxeViolations(page, `checkpoint main ${mode}`, { include: "#component-preview-frame" });
    await expectNoAxeViolations(page, `checkpoint thread ${mode}`, { include: "#component-preview-frame-thread" });
  });
}
