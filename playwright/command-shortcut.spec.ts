import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

/** `CommandShortcut` (backlog row 188): the hint at the end of a command row. */
const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

async function open(page: Page, mode: "light" | "dark") {
  await gotoHydrated(page, `${BASE_URL}/component/command/`, GOTO_OPTS);
  await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
  const root = page.locator("#component-preview-frame-shortcuts").first();
  await root.scrollIntoViewIfNeeded();
  await expect(root.getByRole("combobox", { name: "Search settings" })).toBeVisible({ timeout: 60_000 });
  return root;
}

test("CommandShortcut sits at the inline end, muted, and takes the foreground when the row is highlighted", async ({ page }) => {
  const root = await open(page, "light");
  const option = root.getByRole("option", { name: /Billing/ });
  await expect(option).toHaveCount(1);
  await expect(option.locator("span", { hasText: "⌘B" })).toHaveCount(1);

  const read = () =>
    readAtRest(option, (el) => {
      const h = [...el.querySelectorAll("span")].find((s) => s.textContent === "⌘B") as HTMLElement;
      const hs = getComputedStyle(h);
      const s = getComputedStyle(el);
      return {
        endGap: el.getBoundingClientRect().right - h.getBoundingClientRect().right,
        paddingEnd: parseFloat(s.paddingRight),
        fontSize: hs.fontSize,
        letterSpacing: hs.letterSpacing,
        color: hs.color,
        itemColor: s.color,
      };
    });

  const rest = await read();
  expect(Math.abs(rest.endGap - rest.paddingEnd), "pushed to the inline end").toBeLessThanOrEqual(1.5);
  expect(rest.fontSize).toBe("12px");
  expect(rest.letterSpacing).toBe("1.2px");

  await root.getByRole("combobox").focus();
  await page.keyboard.press("ArrowDown");
  const highlighted = root.locator('.dx-command-item[data-highlighted="true"]');
  await expect(highlighted).toHaveCount(1);
  const ink = await highlighted.locator(".dx-command-shortcut").evaluate((h) => ({
    color: getComputedStyle(h).color,
    item: getComputedStyle(h.parentElement as HTMLElement).color,
  }));
  expect(ink.color, "takes the highlighted row's ink").toBe(ink.item);
  if (!(await option.evaluate((el) => el.getAttribute("data-highlighted") === "true"))) {
    expect(rest.color, "muted at rest").not.toBe(rest.itemColor);
  }
});

for (const mode of ["light", "dark"] as const) {
  test(`command/shortcuts has no axe violations (${mode})`, async ({ page }) => {
    await open(page, mode);
    await expectNoAxeViolations(page, `command/shortcuts ${mode}`);
  });
}
