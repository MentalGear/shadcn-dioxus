import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

const URL = `${BASE_URL}/component/navigation_menu/`;
const GOTO = { timeout: 20 * 60 * 1000 };

// Backlog row 249: shadcn's NavigationMenuIndicator. The arrow is centred under the open trigger,
// follows it when another item opens, and is hidden while nothing is open.
test("the indicator tracks the open trigger at rest", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  const nav = page.getByRole("navigation", { name: "Indicator navigation menu" });
  const indicator = nav.locator(".dx-navigation-menu-indicator");
  await expect(indicator).toHaveAttribute("data-state", "hidden");
  await expect(indicator).toHaveCSS("opacity", "0");

  const centreOf = async (name: string) => {
    const trigger = nav.getByRole("button", { name });
    const t = await readAtRest(trigger, (el) => {
      const r = el.getBoundingClientRect();
      return r.left + r.width / 2;
    });
    const a = await readAtRest(indicator.locator(".dx-navigation-menu-indicator-arrow"), (el) => {
      const r = el.getBoundingClientRect();
      return { x: r.left + r.width / 2, y: r.top };
    });
    const tb = await readAtRest(trigger, (el) => el.getBoundingClientRect().bottom);
    return { t, a, tb };
  };

  await nav.getByRole("button", { name: "Learn" }).click();
  await expect(indicator).toHaveAttribute("data-state", "visible");
  await expect(indicator).toHaveCSS("opacity", "1");
  let m = await centreOf("Learn");
  expect(Math.abs(m.a.x - m.t)).toBeLessThan(1.5);
  expect(m.a.y).toBeGreaterThanOrEqual(m.tb - 1);
  const firstX = m.a.x;

  await nav.getByRole("button", { name: "Build with the library" }).click();
  await expect(indicator).toHaveAttribute("data-state", "visible");
  m = await centreOf("Build with the library");
  expect(Math.abs(m.a.x - m.t)).toBeLessThan(1.5);
  expect(m.a.x).toBeGreaterThan(firstX + 20);

  await page.keyboard.press("Escape");
  await expect(indicator).toHaveAttribute("data-state", "hidden");
});
