/**
 * Row 188 leftovers: `InputGroupTextarea` (textarea footer) and the group's custom control, and
 * `BreadcrumbLink { r#as }` (router link). Tabs and ToggleGroup vertical already have their specs
 * (`tabs.spec.ts`, `toggle_group.spec.ts`). Controls are located by role inside the demo frame; axe runs
 * in light and dark.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const frame = (page: Page, variant: string) => page.locator(`#component-preview-frame-${variant}`).first();

async function open(page: Page, name: string, variant: string, mode: "light" | "dark") {
  await gotoHydrated(page, `${BASE_URL}/component/${name}/`, GOTO_OPTS);
  await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
  const root = frame(page, variant);
  await root.scrollIntoViewIfNeeded();
  return root;
}

const css = (el: Locator, prop: string) => el.evaluate((e, p) => getComputedStyle(e).getPropertyValue(p), prop);

for (const mode of ["light", "dark"] as const) {
  test.describe(`row 188 leftovers, ${mode}`, () => {
    test.use({ colorScheme: mode });

    test(`input_group/textarea: bare control, footer below it, live count, group ring`, async ({ page }) => {
      const root = await open(page, "input_group", "textarea", mode);
      const group = root.locator(".dx-input-group");
      const area = root.getByRole("textbox", { name: "Message" });
      const send = root.getByRole("button", { name: "Send message" });
      // The control paints no chrome of its own and cannot resize; the group draws the one edge.
      expect(await css(area, "resize")).toBe("none");
      expect(await css(area, "box-shadow")).toBe("none");
      expect(await css(area, "border-radius")).toBe("0px");
      expect(await css(group, "box-shadow")).not.toBe("none");
      // The footer sits under the textarea, inside the group, with the button at the inline end.
      const a = (await area.boundingBox())!;
      const s = (await send.boundingBox())!;
      const g = (await group.boundingBox())!;
      expect(s.y).toBeGreaterThanOrEqual(a.y + a.height - 1);
      expect(s.y + s.height).toBeLessThanOrEqual(g.y + g.height + 1);
      expect(a.height).toBeGreaterThanOrEqual(63);
      await expect(root.locator("#input-group-textarea-count")).toHaveText("0 / 120");
      await area.fill("hello");
      await expect(root.locator("#input-group-textarea-count")).toHaveText("5 / 120");
      // Keyboard focus rings the group, not the textarea.
      await page.keyboard.press("Tab");
      await area.focus();
      await expect.poll(() => css(group, "box-shadow")).not.toBe("none");
      expect(await css(area, "box-shadow")).toBe("none");
      await expectNoAxeViolations(page, `input_group/textarea ${mode}`);
    });

    test(`input_group/custom_control: any element wearing the class takes the group's states`, async ({ page }) => {
      const root = await open(page, "input_group", "custom_control", mode);
      const groups = root.locator(".dx-input-group");
      const code = root.getByRole("textbox", { name: "Code" });
      await code.focus();
      const before = await css(groups.nth(0), "box-shadow");
      expect(before).not.toBe("none");
      // The second group's control is aria-invalid: the group draws the invalid ring.
      const invalid = await css(groups.nth(1), "box-shadow");
      expect(invalid).not.toBe("none");
      expect(invalid).not.toBe(before);
      // The addon is a block-start row above the control.
      const addon = (await groups.nth(0).locator(".dx-input-group-addon").boundingBox())!;
      const c = (await code.boundingBox())!;
      expect(addon.y + addon.height).toBeLessThanOrEqual(c.y + 1);
      await expectNoAxeViolations(page, `input_group/custom_control ${mode}`);
    });

    test(`breadcrumb/router_link: as renders the router link with the crumb's look`, async ({ page }) => {
      const root = await open(page, "breadcrumb", "router_link", mode);
      const link = root.getByRole("link").first();
      await expect(link).toHaveClass(/dx-breadcrumb-link/);
      await expect(link).toHaveAttribute("href", /\/component\/card\/?/);
      expect(await css(link, "text-decoration-line")).toBe("none");
      await expect(root.getByRole("link")).toHaveCount(2);
      await expect(root.locator(".dx-breadcrumb-page")).toHaveAttribute("aria-current", "page");
      await expectNoAxeViolations(page, `breadcrumb/router_link ${mode}`);
    });
  });
}

