/**
 * Row 188 demo coverage, menus, overlays and floating: the demos of the research doc's section 5 (A-C):
 * tooltip/sides, drawer/scrollable_content, command/scrollable. Each is opened in its demo frame, its key
 * interaction is checked, and axe runs in light and dark. Controls are located by role inside the
 * frame, never by demo copy elsewhere on the page.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
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

for (const mode of ["light", "dark"] as const) {
  test.describe(`demos batch 2 A, ${mode}`, () => {
    test.use({ colorScheme: mode });

    test(`tooltip/sides: each trigger opens its tooltip on its side`, async ({ page }) => {
      const root = await open(page, "tooltip", "sides", mode);
      const expected: [string, string][] = [
        ["Tip above", "Opens above the trigger"],
        ["Tip right", "Opens to the right of the trigger"],
        ["Tip below", "Opens below the trigger"],
        ["Tip left", "Opens to the left of the trigger"],
      ];
      for (const [name, text] of expected) {
        const trigger = root.getByRole("button", { name });
        await expect(trigger).toBeVisible({ timeout: 60_000 });
        await trigger.focus();
        const tip = page.getByRole("tooltip").filter({ hasText: text });
        await expect(tip).toBeVisible();
        const t = await trigger.boundingBox();
        const c = await tip.boundingBox();
        if (!t || !c) throw new Error("no boxes");
        if (name === "Tip above") expect(c.y + c.height).toBeLessThanOrEqual(t.y + 1);
        if (name === "Tip below") expect(c.y).toBeGreaterThanOrEqual(t.y + t.height - 1);
        if (name === "Tip left") expect(c.x + c.width).toBeLessThanOrEqual(t.x + 1);
        if (name === "Tip right") expect(c.x).toBeGreaterThanOrEqual(t.x + t.width - 1);
        await expectNoAxeViolations(page, `tooltip/sides ${name} ${mode}`);
        await page.keyboard.press("Escape");
        await expect(tip).toBeHidden();
      }
    });

    test(`drawer/scrollable_content: the body scrolls, the footer stays visible`, async ({ page }) => {
      const root = await open(page, "drawer", "scrollable_content", mode);
      const trigger = root.getByRole("button", { name: "Open scrollable drawer" });
      await expect(trigger).toBeVisible({ timeout: 60_000 });
      await trigger.click();
      const dialog = page.getByRole("dialog", { name: "Release notes" });
      await expect(dialog).toBeVisible();
      const body = dialog.getByRole("region", { name: "Release notes list" });
      const m = await body.evaluate((el) => ({ sh: el.scrollHeight, ch: el.clientHeight }));
      expect(m.sh).toBeGreaterThan(m.ch);
      await body.evaluate((el) => (el.scrollTop = el.scrollHeight));
      await expect.poll(() => body.evaluate((el) => el.scrollTop)).toBeGreaterThan(0);
      await expect(dialog.getByRole("button", { name: "Done reading" })).toBeInViewport();
      await expectNoAxeViolations(page, `drawer/scrollable_content ${mode}`);
      await dialog.getByRole("button", { name: "Done reading" }).click();
      await expect(dialog).toBeHidden();
    });

    test(`command/scrollable: the list scrolls and the filter narrows it`, async ({ page }) => {
      const root = await open(page, "command", "scrollable", mode);
      const input = root.getByRole("combobox", { name: "Filter time zones" });
      await expect(input).toBeVisible({ timeout: 60_000 });
      const list = root.getByRole("listbox", { name: "Time zones" });
      const m = await list.evaluate((el) => ({ sh: el.scrollHeight, ch: el.clientHeight }));
      expect(m.sh).toBeGreaterThan(m.ch);
      expect(await root.getByRole("option").count()).toBe(24);
      await expectNoAxeViolations(page, `command/scrollable ${mode}`);
      await input.fill("Pacific");
      await expect(root.getByRole("option")).toHaveCount(4);
      await input.fill("zzzz");
      await expect(root.getByText("No time zone found.")).toBeVisible();
    });
  });
}
