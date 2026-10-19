import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL_ = `${BASE_URL}/component/?name=trace&`;
const GOTO = { timeout: 20 * 60 * 1000 };
const frame = (page: Page, name?: string) =>
  page.locator(name ? `#component-preview-frame-${name}` : "#component-preview-frame").first();
const styled = (page: Page) =>
  expect
    .poll(() => frame(page).locator('[data-slot="trace-line"]').first().evaluate((el) => getComputedStyle(el).display), {
      timeout: 60_000,
    })
    .toBe("flex");

test("the trigger toggles aria-expanded and the detail with click, Enter and Space", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page);
  const trigger = demo.getByRole("button", { name: /Ran cargo check/ });
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
  await expect(demo.getByText("Finished dev profile")).toHaveCount(0);
  await trigger.click();
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  await expect(demo.getByText("Finished dev profile")).toBeVisible();
  await trigger.focus();
  await page.keyboard.press("Enter");
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
  await page.keyboard.press("Space");
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  // aria-controls points at the revealed panel.
  const id = await trigger.getAttribute("aria-controls");
  await expect(page.locator(`[id="${id}"]`)).toBeVisible();
});

test("focus shows the ring and the chevron turns when open", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const trigger = frame(page).getByRole("button", { name: /Ran cargo check/ });
  await trigger.focus();
  expect(await trigger.evaluate((el) => getComputedStyle(el).boxShadow)).not.toBe("none");
  const chevron = trigger.locator(".dx-trace-chevron");
  expect(await chevron.evaluate((el) => getComputedStyle(el).rotate)).toBe("none");
  await trigger.press("Enter");
  await expect.poll(() => chevron.evaluate((el) => getComputedStyle(el).rotate)).toContain("90deg");
});

test("icons are decorative and statuses are marked", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page, "status");
  for (const icon of await demo.locator('[data-slot="trace-icon"]').all())
    await expect(icon).toHaveAttribute("aria-hidden", "true");
  await expect(demo.locator('[data-slot="trace-line"][data-status="running"] .dx-shimmer')).toHaveCount(1);
  await expect(demo.locator('[data-slot="trace-line"][data-status="error"]')).toContainText("Failed to read");
  expect(
    await demo.locator('[data-status="pending"]').evaluate((el) => getComputedStyle(el).opacity),
  ).toBe("0.6");
});

test("controlled variant: open state follows the signal and the reader can toggle", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page, "controlled");
  const trigger = demo.getByRole("button", { name: /Thinking/ });
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  await trigger.click();
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
  await trigger.click();
  await demo.getByRole("button", { name: "Finish thinking" }).click();
  const done = demo.getByRole("button", { name: /Thought/ });
  await expect(done).toHaveAttribute("aria-expanded", "false");
});

for (const mode of ["light", "dark"] as const) {
  test(`trace demos have no axe violations (${mode})`, async ({ page }) => {
    await gotoHydrated(page, URL_, GOTO);
    await styled(page);
    await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
    for (const name of ["main", "status", "reasoning"]) {
      await expectNoAxeViolations(page, `trace ${name} ${mode}`, {
        include: name === "main" ? "#component-preview-frame" : `#component-preview-frame-${name}`,
      });
    }
  });
}
