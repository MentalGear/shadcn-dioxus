/**
 * Enter, Escape, Enter reopens a modal at once (backlog row 223).
 *
 * A modal is closed only after its exit animation (row 141), and for that whole window it is
 * still a real modal: the page behind it, the trigger included, is blocked and cannot be focused (an
 * `inert` closing panel does not unblock it either). A second Enter would therefore be
 * typed into the closing panel and the reopen lost. `use_dialog_open_driver`
 * (`primitives/src/lib.rs`) treats an Enter or Space aimed at a dialog that is closing -- or that
 * has just been cancelled and whose state has not flipped yet -- as the opener's: it ends the exit on
 * the spot (the real `close()`, which returns focus) and activates the opener.
 *
 * Consumers are DERIVED from `modal-consumers.ts` (Dialog, AlertDialog, Sheet, Drawer, Command), plus
 * the modal Popover. Each is driven with the keyboard only: focus the trigger, Enter, Escape, Enter at
 * several delays after the Escape.
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { CONSUMERS } from "./modal-consumers";

const GOTO = { timeout: 20 * 60 * 1000 };

const TARGETS = [
  ...CONSUMERS.map((c) => ({
    name: c.component,
    url: `${BASE_URL}/component/${c.component}/`,
    trigger: c.variants[0].trigger,
    panel: `${c.dialogSelector}[open]`,
  })),
  {
    name: "popover (modal)",
    url: `${BASE_URL}/component/popover/`,
    trigger: "Open modal popover",
    panel: "dialog[open]",
  },
];

/** 0 is "as fast as Playwright can send the next key"; the rest straddle the exit animation. */
const DELAYS = [0, 0, 0, 40, 120, 400];

async function isOpen(page: Page, panel: string) {
  return page.evaluate((sel) => !!document.querySelector(sel), panel);
}

for (const t of TARGETS) {
  test(`${t.name}: Enter, Escape, Enter reopens at every delay`, async ({ page }) => {
    await gotoHydrated(page, t.url, GOTO);
    const trigger = page.getByRole("button", { name: t.trigger, exact: true }).first();
    await trigger.scrollIntoViewIfNeeded();
    for (const delay of DELAYS) {
      await trigger.focus();
      await page.keyboard.press("Enter");
      await expect(page.locator(t.panel)).toBeVisible();
      await page.waitForTimeout(450); // let the enter animation finish: the reopen is what is timed
      await page.keyboard.press("Escape");
      if (delay) await page.waitForTimeout(delay);
      await page.keyboard.press("Enter");
      await expect
        .poll(() => isOpen(page, t.panel), { message: `${t.name}: reopened at ${delay} ms`, timeout: 4000 })
        .toBe(true);
      // The reopen is a real one: the closing state ended, focus is inside, Escape closes it again.
      await page.waitForTimeout(450);
      expect(await page.evaluate((sel) => document.querySelectorAll(sel).length, t.panel)).toBe(1);
      await page.keyboard.press("Escape");
      await expect(page.locator(t.panel)).toHaveCount(0);
      await expect(trigger).toBeFocused();
    }
  });

  test(`${t.name}: Space reopens too, and an unrelated key is not swallowed`, async ({ page }) => {
    await gotoHydrated(page, t.url, GOTO);
    const trigger = page.getByRole("button", { name: t.trigger, exact: true }).first();
    await trigger.scrollIntoViewIfNeeded();
    await trigger.focus();
    await page.keyboard.press("Enter");
    await expect(page.locator(t.panel)).toBeVisible();
    await page.waitForTimeout(450);
    await page.keyboard.press("Escape");
    await page.keyboard.press("Space");
    await expect.poll(() => isOpen(page, t.panel), { timeout: 4000 }).toBe(true);
    await page.waitForTimeout(450);
    await page.keyboard.press("Escape");
    await expect(page.locator(t.panel)).toHaveCount(0);
    // After a full close nothing is forwarded: Enter on the focused trigger is an ordinary Enter.
    await expect(trigger).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.locator(t.panel)).toBeVisible();
  });
}
