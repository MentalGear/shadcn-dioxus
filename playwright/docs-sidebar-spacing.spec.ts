import { test, expect } from "./fixtures";
import type { Locator } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

// The docs site's sidebar IS the Sidebar component (`DocsLayout`, `preview/src/docs_nav.rs`), so its menu items
// must be spaced exactly like the Sidebar demo's: the site applies no spacing of its own over the component's.
// A `.dx-docs-shell*` / `.dx-nav-*` rule that adds or drops a gap, margin, height or padding on the menu or its
// buttons makes the two disagree, and this fails. Run against the SSG lane.

// The demo's rows carry a menu action, which widens `padding-inline-end` by design; the block axis and the start
// edge are what the two lists must share.
type Spacing = { menuGap: string; margin: string; padding: string; height: number; pitch: number };

/** The spacing of the first menu that lists at least two plain default-size buttons under `scope`. */
async function spacing(scope: Locator): Promise<Spacing> {
  return scope.locator(".dx-sidebar-menu").evaluateAll((menus) => {
    for (const menu of menus) {
      const items = [...menu.children].filter((el) => el.matches(".dx-sidebar-menu-item"));
      const buttons = items.map((el) => el.querySelector(":scope > .dx-sidebar-menu-button[data-size='default']"));
      if (items.length < 2 || !buttons[0] || !buttons[1]) continue;
      const style = getComputedStyle(buttons[0]);
      const [a, b] = items.slice(0, 2).map((el) => el.getBoundingClientRect());
      return {
        menuGap: getComputedStyle(menu).rowGap,
        margin: style.margin,
        padding: `${style.paddingTop} ${style.paddingBottom} ${style.paddingLeft}`,
        height: buttons[0].getBoundingClientRect().height,
        pitch: b.top - a.top,
      };
    }
    throw new Error("no menu with two default-size buttons");
  });
}

test("docs sidebar spaces its menu items like the Sidebar demo", async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  await gotoHydrated(page, `${BASE_URL}/component/sidebar/`);
  const demoFrame = page.frameLocator("#component-preview-frame iframe, iframe#component-preview-frame").first();
  await expect(demoFrame.locator(".dx-sidebar-menu-button").first()).toBeVisible();
  const demo = await spacing(demoFrame.locator("body"));
  const docs = await spacing(page.locator(".dx-docs-shell"));
  expect(docs).toEqual(demo);
});
