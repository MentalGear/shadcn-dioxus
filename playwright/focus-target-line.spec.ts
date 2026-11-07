/**
 * The focus-target demos (Dialog, AlertDialog, Popover `variants/focus_target`) say what they do: a live line
 * inside the panel, "Opened with: <modality>, focus on: <target>", read from `<html data-dx-modality>`. Rings are
 * keyboard-only, so a mouse open that moves focus paints nothing; the line is what tells that user where focus
 * went. Each test opens the panel by keyboard and by mouse, and checks the line against the element that really
 * has focus, so the words cannot drift from the policy.
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };

const activeId = (page: Page) => page.evaluate(() => (document.activeElement as HTMLElement | null)?.id ?? "");

interface Case {
  name: string;
  path: string;
  trigger: string;
  panel: string;
  line: string;
  keyboard: { text: RegExp; focus: string };
  mouse: { text: RegExp; focus: string | "panel" };
}

const CASES: Case[] = [
  {
    name: "Dialog",
    path: "dialog/",
    trigger: "Open Focus Dialog",
    panel: "dialog.dx-dialog[open]",
    line: "#dialog-focus-line",
    keyboard: { text: /^Opened with: keyboard, focus on: the second field$/, focus: "dialog-focus-second" },
    mouse: { text: /^Opened with: mouse, focus on: Cancel/, focus: "dialog-focus-cancel" },
  },
  {
    name: "AlertDialog",
    path: "alert_dialog/",
    trigger: "Show Focus Alert",
    panel: "dialog.dx-alert-dialog[open]",
    line: "#alert-focus-line",
    keyboard: { text: /^Opened with: keyboard, focus on: Cancel/, focus: "alert-focus-cancel" },
    mouse: { text: /^Opened with: mouse, focus on: the panel/, focus: "panel" },
  },
  {
    name: "Popover",
    path: "popover/",
    trigger: "Open Focus Popover",
    panel: "dialog[popover]:popover-open",
    line: "#popover-focus-line",
    keyboard: { text: /^Opened with: keyboard, focus on: the second field$/, focus: "popover-focus-second" },
    mouse: { text: /^Opened with: mouse, focus on: the first field$/, focus: "popover-focus-first" },
  },
];

for (const c of CASES) {
  test(`${c.name} focus-target: the live line matches a keyboard open and a mouse open`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/${c.path}`, GOTO);
    const trigger = page.getByRole("button", { name: c.trigger, exact: true }).first();
    const line = page.locator(c.line);

    await trigger.focus();
    await page.keyboard.press("Enter");
    await expect(page.locator(c.panel)).toBeVisible();
    await expect(line).toHaveText(c.keyboard.text);
    await page.waitForTimeout(450);
    expect(await activeId(page)).toBe(c.keyboard.focus);
    await page.keyboard.press("Escape");
    await expect(page.locator(c.panel)).toHaveCount(0);

    await trigger.click();
    await expect(page.locator(c.panel)).toBeVisible();
    await expect(line).toHaveText(c.mouse.text);
    await page.waitForTimeout(450);
    if (c.mouse.focus === "panel") {
      expect(await page.evaluate((s) => document.activeElement === document.querySelector(s), c.panel)).toBe(true);
    } else {
      expect(await activeId(page)).toBe(c.mouse.focus);
    }
  });
}

test("a demo's title.txt replaces the title derived from its directory, and the frame id stays the directory", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/accordion/`, GOTO);
  await expect(page.locator("h3.dx-component-variant-title", { hasText: /^Find in Accordion$/ })).toHaveCount(1);
  await expect(page.locator("h3.dx-component-variant-title", { hasText: /Until Found/i })).toHaveCount(0);
  await expect(page.locator("#component-preview-frame-until_found")).toHaveCount(1);
});
