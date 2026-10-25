/**
 * `FieldLabel { required }` (row 188): a decorative `*` in the destructive ink after the label text. The
 * accessible name stays the label text, and the control carries `required` itself.
 */
import { test, expect } from "./fixtures";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

for (const mode of ["light", "dark"] as const) {
  test(`input/required: the required marker is drawn and hidden from the accessible name, ${mode}`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/input/`, GOTO_OPTS);
    await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
    const root = page.locator("#component-preview-frame-required").first();
    await root.scrollIntoViewIfNeeded();
    const marker = root.locator(".dx-field-label .dx-field-required-marker");
    await expect(marker).toHaveText("*");
    await expect(marker).toHaveAttribute("aria-hidden", "true");
    const ink = await marker.evaluate((e) => getComputedStyle(e).color);
    const label = await root.locator(".dx-field-label").evaluate((e) => getComputedStyle(e).color);
    expect(ink).not.toBe(label);
    const input = root.getByRole("textbox", { name: "Handle", exact: true });
    expect(await input.evaluate((e) => (e as HTMLInputElement).required)).toBe(true);
    await expectNoAxeViolations(page, `input/required ${mode}`);
  });
}
