/**
 * `CheckboxState::Indeterminate` must not render the Check tick in an UNFILLED box (an indicator that draws
 * its icon for any non-unchecked state, with a sheet that only styles `data-state="checked"`, does). By
 * construction the indicator holds both glyphs and CSS shows the one matching `data-state`, and the filled
 * surface covers checked and indeterminate alike.
 */
import { test, expect } from "./fixtures";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole } from "./demo-frame";

const URL_ = `${BASE_URL}/component/checkbox/`;
const GOTO = { timeout: 20 * 60 * 1000 };

const glyphs = (box: ReturnType<typeof demoRole>) =>
  box.evaluate((el) => {
    const shown = (sel: string) => {
      const g = el.querySelector(sel);
      return !!g && getComputedStyle(g).display !== "none";
    };
    return {
      check: shown(".dx-checkbox-glyph-check"),
      minus: shown(".dx-checkbox-glyph-minus"),
      bg: getComputedStyle(el.firstElementChild!).backgroundColor,
    };
  });

test("indeterminate shows a minus in a filled box, checked a tick, unchecked neither", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  const box = demoRole(page, "checkbox", 0, "indeterminate");
  await box.scrollIntoViewIfNeeded();
  await expect(box).toHaveAttribute("data-state", "indeterminate");
  await expect(box).toHaveAttribute("aria-checked", "mixed");
  const mixed = await glyphs(box);
  expect(mixed.minus).toBe(true);
  expect(mixed.check).toBe(false);
  expect(mixed.bg).not.toBe("rgba(0, 0, 0, 0)");

  // Clicking an indeterminate box unchecks it (primitive contract); the next click checks it.
  await box.click();
  await expect(box).toHaveAttribute("data-state", "unchecked");
  const off = await glyphs(box);
  expect(off.check).toBe(false);
  expect(off.minus).toBe(false);
  await box.click();
  await expect(box).toHaveAttribute("data-state", "checked");
  const on = await glyphs(box);
  expect(on.check).toBe(true);
  expect(on.minus).toBe(false);
  expect(on.bg).toBe(mixed.bg);
});

test("table header goes indeterminate when some rows are selected, and a click selects all", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  const header = demoRole(page, "checkbox", 0, "table");
  await header.scrollIntoViewIfNeeded();
  await expect(header).toHaveAttribute("data-state", "indeterminate"); // demo starts with one row selected
  expect((await glyphs(header)).minus).toBe(true);
  await header.click();
  await expect(header).toHaveAttribute("data-state", "checked");
  await header.click();
  await expect(header).toHaveAttribute("data-state", "unchecked");
});

for (const mode of ["light", "dark"] as const) {
  test(`indeterminate demo has no axe violations (${mode})`, async ({ page }) => {
    await gotoHydrated(page, URL_, GOTO);
    await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
    await expectNoAxeViolations(page, `checkbox indeterminate ${mode}`, {
      include: "#component-preview-frame-indeterminate",
    });
  });
}
