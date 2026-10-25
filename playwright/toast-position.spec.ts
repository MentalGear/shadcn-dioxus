import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";
import { gotoHydrated } from "./hydration";
import { fixedViewportBox } from "./viewport-box";

// Toast `position` (backlog row 398): the toaster sits at the side and inline placement the prop
// names, written as `data-toaster-side` / `data-toaster-align` on the region.
const REGION = "Positioned toast notifications";

const CASES = [
  { button: "Top start", side: "top", align: "start" },
  { button: "Top center", side: "top", align: "center" },
  { button: "Top end", side: "top", align: "end" },
  { button: "Bottom start", side: "bottom", align: "start" },
  { button: "Bottom center", side: "bottom", align: "center" },
  { button: "Bottom end", side: "bottom", align: "end" },
] as const;

test.describe("toast: position", () => {
  for (const { button, side, align } of CASES) {
    test(`${button} puts the toaster at the ${side} ${align}`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/toast/`);
      await page.getByRole("button", { name: button, exact: true }).click();

      const region = page.getByRole("region", { name: REGION });
      await expect(region).toHaveAttribute("data-toaster-side", side);
      await expect(region).toHaveAttribute("data-toaster-align", align);

      const toast = region.getByRole("alertdialog");
      await expect(toast).toHaveCount(1);
      const box = await measureAtRest(toast);
      const viewport = await fixedViewportBox(page);
      const centreX = box.x + box.width / 2;
      const centreY = box.y + box.height / 2;

      if (side === "top") expect(centreY).toBeLessThan(viewport.y + viewport.height / 2);
      else expect(centreY).toBeGreaterThan(viewport.y + viewport.height / 2);

      if (align === "center") expect(Math.abs(centreX - (viewport.x + viewport.width / 2))).toBeLessThan(2);
      else if (align === "start") expect(centreX).toBeLessThan(viewport.x + viewport.width / 2);
      else expect(centreX).toBeGreaterThan(viewport.x + viewport.width / 2);
    });
  }
});
