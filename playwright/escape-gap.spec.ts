/**
 * Escape in the gap before a dismiss layer is on the page (backlog row 487, the dismiss core's "gap").
 *
 * An overlay's layer registers on the page only after its panel has mounted, a pointer open runs one task after
 * the press, and a hover surface opens after a delay. An Escape that arrives anywhere in that window reaches no
 * layer. The core covers it for every overlay by one rule (`dismiss::escape_in_gap`, `dismiss::OpenGap`, the hover
 * surfaces' layer on the root): the overlay is closed, or its pending open cancelled, and it stays closed.
 *
 * Each case sends Escape in the same task as the open gesture (or between the press and the open task, or during
 * a hover delay) and asserts the overlay is gone and stays gone. Select's own cases live in
 * `select-escape-race.spec.ts`.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoFrame, demoRole } from "./demo-frame";

const goto = (page: Page, component: string) =>
  gotoHydrated(page, `${BASE_URL}/component/${component}/`, { timeout: 20 * 60 * 1000 });

const ROUNDS = 6;

/** Dispatch keydown events of the given keys on the focused element, all in one task. */
const keysInOneTask = (page: Page, keys: string[]) =>
  page.evaluate((keys) => {
    const t = document.activeElement as HTMLElement;
    for (const key of keys) {
      t.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
    }
  }, keys);

test.describe("Combobox: Escape right after the open gesture", () => {
  test("ArrowDown, then Escape in the same task, leaves the list closed", async ({ page }) => {
    await goto(page, "combobox");
    const input = demoRole(page, "combobox");
    for (let i = 0; i < ROUNDS; i++) {
      await input.focus();
      await keysInOneTask(page, ["ArrowDown", "Escape"]);
      await expect(page.getByRole("listbox"), `round ${i}`).toHaveCount(0);
      await page.waitForTimeout(100);
      await expect(page.getByRole("listbox"), `round ${i} stays closed`).toHaveCount(0);
    }
  });

  test("a click, then Escape with no wait, leaves the list closed", async ({ page }) => {
    await goto(page, "combobox");
    const input = demoRole(page, "combobox");
    for (let i = 0; i < ROUNDS; i++) {
      await input.click();
      await page.keyboard.press("Escape");
      await expect(page.getByRole("listbox"), `round ${i}`).toHaveCount(0);
    }
  });
});

// The delay case needs a trigger with a pointer delay: HoverCard's (600 ms) and the Tooltip "delays" variant's
// "Rests (600)" button; the Tooltip demo's own default is 0 and opens on contact.
for (const [component, trigger, delayed] of [
  ["tooltip", (page: Page) => demoFrame(page).locator("[aria-describedby]").first(), (page: Page) => page.getByRole("button", { name: "Rests (600)" })],
  ["hover_card", (page: Page) => demoRole(page, "button"), (page: Page) => demoRole(page, "button")],
] as const) {
  const surface = (page: Page) => page.getByRole("tooltip");
  test.describe(`${component}: Escape before the surface has mounted`, () => {
    test("focus, then Escape in the same task, leaves it closed", async ({ page }) => {
      await goto(page, component);
      const el = trigger(page);
      for (let i = 0; i < ROUNDS; i++) {
        await el.evaluate((node) => {
          (node as HTMLElement).focus();
          node.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
        });
        await expect(surface(page), `round ${i}`).toHaveCount(0);
        await page.waitForTimeout(100);
        await expect(surface(page), `round ${i} stays closed`).toHaveCount(0);
        await el.evaluate((node) => (node as HTMLElement).blur());
      }
    });

    test("Escape during the pointer's open delay cancels the open", async ({ page }) => {
      await goto(page, component);
      const el = delayed(page);
      await page.mouse.move(0, 0);
      await el.hover();
      // Past the layer's registration, well inside the delay (600 ms by default).
      await page.waitForTimeout(150);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(900);
      await expect(surface(page)).toHaveCount(0);
    });
  });
}

test.describe("DropdownMenu: Escape right after the open gesture", () => {
  test("Enter, then Escape in the same task, leaves the menu closed", async ({ page }) => {
    await goto(page, "dropdown_menu");
    const trigger = demoRole(page, "button");
    for (let i = 0; i < ROUNDS; i++) {
      await trigger.focus();
      await keysInOneTask(page, ["Enter", "Escape"]);
      await expect(page.getByRole("menu"), `round ${i}`).toHaveCount(0);
      await page.waitForTimeout(100);
      await expect(page.getByRole("menu"), `round ${i} stays closed`).toHaveCount(0);
    }
  });
});
