import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest, readAtRest } from "./animations";

// The tooltip's arrow points at the trigger's centre for every side x align (the bug: `::after { left: 50% }`
// drew it at the bubble's centre, so a start-aligned bubble pointed at empty space). The arrow is a pseudo
// element, so its centre is read from its computed `left`/`top` (an absolutely positioned box, resolved to px)
// plus the bubble's own rect, at rest.
const CASES = [
  ["Top start", "top"],
  ["Top center", "top"],
  ["Top end", "top"],
  ["Bottom start", "bottom"],
  ["Bottom center", "bottom"],
  ["Bottom end", "bottom"],
] as const;

async function arrowCentreX(tip: import("@playwright/test").Locator): Promise<number> {
  return readAtRest(tip, (el) => {
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el, "::after");
    return r.left + parseFloat(cs.left) + parseFloat(cs.marginLeft) + parseFloat(cs.width) / 2;
  });
}

test.describe("tooltip arrow points at the trigger centre", () => {
  for (const [name, side] of CASES) {
    test(`${name}`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/?name=tooltip`);
      const trigger = page.getByRole("button", { name, exact: true });
      await trigger.hover();
      const tip = page.getByRole("tooltip");
      await expect(tip).toBeVisible();
      await expect(tip).toHaveAttribute("data-side", side);
      const tb = await measureAtRest(trigger);
      // Retry until the measured centre settles within tolerance (placement lands a frame after open).
      await expect
        .poll(async () => Math.abs((await arrowCentreX(tip)) - (tb.x + tb.width / 2)), { timeout: 5000 })
        .toBeLessThanOrEqual(2);
    });
  }

  test("the Extra badge on /component/markdown/ (bottom, start)", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/markdown/`);
    const badge = page.locator(".dx-component-page-header .dx-extra-badge");
    await badge.hover();
    const tip = page.getByRole("tooltip");
    await expect(tip).toBeVisible();
    const tb = await measureAtRest(badge);
    await expect
      .poll(async () => Math.abs((await arrowCentreX(tip)) - (tb.x + tb.width / 2)), { timeout: 5000 })
      .toBeLessThanOrEqual(2);
  });
});
