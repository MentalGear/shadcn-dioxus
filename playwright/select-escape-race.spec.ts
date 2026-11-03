/**
 * Escape right after the open gesture (backlog row 487): an overlay opens in a later task, then mounts, then its
 * dismiss layer registers on the page. A key that arrives anywhere in that window must still close it, so the list
 * is never left open by an Escape the user pressed.
 *
 * Each case presses Escape with no wait after the gesture that opens the Select, in a loop, and asserts the
 * listbox is gone and focus is on the trigger.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole } from "./demo-frame";

const goto = (page: Page) =>
  gotoHydrated(page, `${BASE_URL}/component/select/`, { timeout: 20 * 60 * 1000 });

const ROUNDS = 8;

test.describe("Select: Escape right after the open gesture", () => {
  test("a mouse press, then Escape with no wait, closes the list", async ({ page }) => {
    await goto(page);
    const trigger = demoRole(page, "combobox");
    for (let i = 0; i < ROUNDS; i++) {
      await trigger.click();
      await page.keyboard.press("Escape");
      await expect(page.getByRole("listbox"), `round ${i}`).toHaveCount(0);
      await expect(trigger, `round ${i}`).toBeFocused();
    }
  });

  test("a mouse press, ArrowDown, Escape with no wait, closes the list", async ({ page }) => {
    await goto(page);
    const trigger = demoRole(page, "combobox");
    for (let i = 0; i < ROUNDS; i++) {
      await trigger.click();
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("Escape");
      await expect(page.getByRole("listbox"), `round ${i}`).toHaveCount(0);
      await expect(trigger, `round ${i}`).toBeFocused();
    }
  });

  test("Enter, then Escape in the same task, closes the list", async ({ page }) => {
    await goto(page);
    const trigger = demoRole(page, "combobox");
    for (let i = 0; i < ROUNDS; i++) {
      await trigger.focus();
      await page.evaluate(() => {
        const t = document.activeElement as HTMLElement;
        for (const key of ["Enter", "Escape"]) {
          t.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
        }
      });
      await expect(page.getByRole("listbox"), `round ${i}`).toHaveCount(0);
    }
  });

  test("an Escape before the press's open task has run cancels the open", async ({ page }) => {
    await goto(page);
    const trigger = demoRole(page, "combobox");
    for (let i = 0; i < ROUNDS; i++) {
      await trigger.evaluate((el) => {
        el.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true, pointerType: "mouse", button: 0, isPrimary: true }));
        el.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
      });
      await page.waitForTimeout(300);
      await expect(page.getByRole("listbox"), `round ${i}`).toHaveCount(0);
    }
  });
});
