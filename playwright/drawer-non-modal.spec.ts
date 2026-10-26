/**
 * Non-modal drawer (backlog row 156; Base UI `modal: false`). Demo: `variants/non_modal`.
 *
 * Contract: no scrim, no inert page, no scroll lock, no focus trap -- the page behind the panel stays
 * interactive and scrollable. Escape and the close button close it, and so does a press outside the
 * panel (Base UI's default light dismissal; the primitive composes the non-modal Dialog arm).
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole } from "./demo-frame";
import { awaitAnimationsSettled, measureAtRest } from "./animations";
import { pageMetrics } from "./viewport-box";

const URL = `${BASE_URL}/component/drawer/`;
const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
test.use({ viewport: { width: 390, height: 844 } });

const VARIANT = "non_modal";
const CONTENT = '[data-slot="drawer-content"]';
const ROOT = '[data-slot="drawer-root"]';

async function open(page: Page) {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await demoRole(page, "button", 0, VARIANT).click();
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
  await awaitAnimationsSettled(page.locator(CONTENT));
}

test("a non-modal drawer is no modal: not in the top layer, no scrim, page not inert", async ({ page }) => {
  await open(page);
  const info = await page.evaluate((sel) => {
    const el = document.querySelector(sel) as HTMLElement;
    const root = document.querySelector('[data-slot="drawer-root"]') as HTMLElement;
    const rootCs = getComputedStyle(root);
    return {
      tag: el.tagName,
      modal: el.matches(":modal"),
      dialogs: document.querySelectorAll("dialog[open]").length,
      scrim: rootCs.backgroundColor,
      blur: rootCs.backdropFilter,
      rootPointer: rootCs.pointerEvents,
      panelPointer: getComputedStyle(el).pointerEvents,
      locked: (window as unknown as { __dxScrollLocked?: boolean }).__dxScrollLocked === true,
    };
  }, CONTENT);
  expect(info.modal).toBe(false);
  expect(info.dialogs).toBe(0);
  expect(info.scrim).toBe("rgba(0, 0, 0, 0)");
  expect(info.blur).toBe("none");
  expect(info.rootPointer).toBe("none");
  expect(info.panelPointer).toBe("auto");
  expect(info.locked).toBe(false);
});

test("the page behind stays interactive: its button takes the press", async ({ page }) => {
  await open(page);
  const pageButton = demoRole(page, "button", 1, VARIANT);
  await expect(pageButton).toContainText("Page button: 0");
  // The press reaches the page control (a modal's inert page would swallow it) ...
  await pageButton.click();
  await expect(pageButton).toContainText("Page button: 1");
  // ... and, as Base UI's default light dismissal, a press outside the panel closes the drawer.
  await expect(page.locator(ROOT)).toHaveCount(0);
});

test("the page scrolls while a non-modal drawer is open, and the drawer stays", async ({ page }) => {
  await open(page);
  const { scrollExtent: scrollable } = await pageMetrics(page);
  test.skip(scrollable <= 40, "the demo page is not taller than the viewport");
  const before = await page.evaluate(() => window.scrollY);
  await page.mouse.move(195, 200);
  await page.mouse.wheel(0, 300);
  await expect.poll(() => page.evaluate(() => window.scrollY)).toBeGreaterThan(before);
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
});

test("no focus trap: Tab leaves the panel", async ({ page }) => {
  await open(page);
  await page.locator(CONTENT).getByRole("button", { name: "Close" }).focus();
  await page.keyboard.press("Tab");
  const inside = await page.evaluate((sel) => document.querySelector(sel)!.contains(document.activeElement), CONTENT);
  // Focus moving outside is a light dismissal for the non-modal arm; either way it is not trapped.
  expect(inside).toBe(false);
});

test("Escape and the close button close it", async ({ page }) => {
  await open(page);
  await page.keyboard.press("Escape");
  await expect(page.locator(ROOT)).toHaveCount(0);
  await open(page);
  await page.locator(CONTENT).getByRole("button", { name: "Close" }).click();
  await expect(page.locator(ROOT)).toHaveCount(0);
});

test("a non-modal drawer drags to dismiss", async ({ page }) => {
  await open(page);
  const box = await measureAtRest(page.locator(CONTENT));
  const x = box.x + box.width / 2;
  const y = box.y + 12;
  await page.mouse.move(x, y);
  await page.mouse.down();
  for (let i = 1; i <= 6; i++) {
    await page.mouse.move(x, y + (box.height * 0.6 * i) / 6, { steps: 2 });
    await page.waitForTimeout(50);
  }
  await page.waitForTimeout(200);
  await page.mouse.up();
  await expect(page.locator(ROOT)).toHaveCount(0);
});
