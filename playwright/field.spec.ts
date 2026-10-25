import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/field/`, { timeout: 20 * 60 * 1000 });
  // Label/control association: clicking the label focuses its input.
  const username = page.getByLabel("Username");
  await expect(username).toBeVisible();
  await username.click();
  await expect(username).toBeFocused();

  // The invalid field's error message is present and associated by role.
  await expect(page.getByRole("alert")).toHaveText("Enter a valid email address.");

  // The horizontal field: checkbox beside its label/description.
  await expect(page.getByRole("checkbox", { name: "Marketing emails" })).toBeVisible();
});

// `FieldLabel { required }` (an addition to shadcn): the marker renders only with the prop, is decorative, and
// takes the theme's destructive TEXT role.
for (const mode of ["light", "dark"] as const) {
  test.describe(`FieldLabel required marker, ${mode}`, () => {
    test.use({ colorScheme: mode });

    test(`field/required: the marker is drawn only with required, aria-hidden, in --dx-destructive-text`, async ({
      page,
    }) => {
      await gotoHydrated(page, `${BASE_URL}/component/field/`, { timeout: 20 * 60 * 1000 });
      await page.evaluate(
        (m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m),
        mode,
      );
      const frame = page.locator("#component-preview-frame-required").first();
      await frame.scrollIntoViewIfNeeded();
      const labels = frame.locator(".dx-field-label");
      await expect(labels).toHaveCount(2);

      // Only the label with `required` has a marker.
      await expect(labels.nth(0).locator(".dx-field-label-required")).toHaveCount(1);
      await expect(labels.nth(1).locator(".dx-field-label-required")).toHaveCount(0);

      const marker = labels.nth(0).locator(".dx-field-label-required");
      await expect(marker).toHaveText("*");
      await expect(marker).toHaveAttribute("aria-hidden", "true");

      // Decorative: the accessible name of the control is the label text alone.
      await expect(frame.getByRole("textbox").nth(0)).toHaveAccessibleName("Email");

      // Colour: exactly the theme's destructive text role, resolved on the same element.
      const [color, role] = await marker.evaluate((el) => {
        const probe = document.createElement("span");
        probe.style.color = "var(--dx-destructive-text)";
        el.appendChild(probe);
        const resolved = getComputedStyle(probe).color;
        probe.remove();
        return [getComputedStyle(el).color, resolved];
      });
      expect(color).toBe(role);
      expect(color).not.toBe("");

      // The prop only draws: the input's own `required` is what makes it required.
      await expect(frame.getByRole("textbox").nth(0)).toBeRequired();
      await expect(frame.getByRole("textbox").nth(1)).not.toBeRequired();
    });

    test(`input/required: the demo's asterisk is FieldLabel's marker`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/input/`, { timeout: 20 * 60 * 1000 });
      await page.evaluate(
        (m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m),
        mode,
      );
      const frame = page.locator("#component-preview-frame-required").first();
      await frame.scrollIntoViewIfNeeded();
      const marker = frame.locator(".dx-field-label .dx-field-label-required");
      await expect(marker).toHaveCount(1);
      await expect(marker).toHaveAttribute("aria-hidden", "true");
      await expect(frame.getByRole("textbox")).toBeRequired();
    });
  });
}
