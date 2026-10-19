/**
 * Row 240 (Base UI focus model, the lifted items), on the ONE open-focus policy (`dxOpenFocus` /
 * `dxCloseFocus`, `primitives/src/modality.rs`):
 *
 *   1. function-form `initial_focus` / `final_focus` props (Dialog, AlertDialog, Popover) receive the
 *      open modality; asserted for keyboard, mouse, touch and pen;
 *   2. a programmatic open (focus on some element X, a script `click()` on the trigger) returns focus
 *      to X on close, for the modal Dialog and the non-modal Popover closed by state;
 *   3. a Select opens on its selected option for a pointer open too; a Menu highlights nothing for a
 *      pointer open and its first item for a keyboard open;
 *   4. `click.detail === 0` is the keyboard fallback of the modality writer;
 *   5. a text field focused by a mouse open still matches `:focus-visible` (so script focus must not
 *      pass `focusVisible: false`; the policy deliberately does not use that option).
 */
import type { Browser, Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };
const url = (q: string) => `${BASE_URL}/component/${q}`;

const activeId = (page: Page) => page.evaluate(() => (document.activeElement as HTMLElement | null)?.id ?? "");
const panelFocused = (page: Page, sel: string) =>
  page.evaluate((s) => document.activeElement === document.querySelector(s), sel);
const modality = (page: Page) => page.evaluate(() => document.documentElement.getAttribute("data-dx-modality"));

/** A pen press on `el` followed by a click with a real `detail`, as a pen produces. */
async function penClick(page: Page, el: ReturnType<Page["getByRole"]>) {
  await el.evaluate((node) => {
    node.dispatchEvent(new PointerEvent("pointerdown", { pointerType: "pen", bubbles: true, composed: true }));
    node.dispatchEvent(new MouseEvent("click", { detail: 1, bubbles: true, composed: true }));
  });
}

async function touchPage(browser: Browser, q: string) {
  const context = await browser.newContext({ hasTouch: true });
  const page = await context.newPage();
  await gotoHydrated(page, url(q), GOTO);
  return { context, page };
}

const DIALOG = { q: "dialog/", trigger: "Open Focus Dialog", panel: "dialog.dx-dialog[open]" };
const POPOVER = { q: "popover/", trigger: "Open Focus Popover", panel: "dialog[popover]:popover-open" };
const ALERT = { q: "alert_dialog/", trigger: "Show Focus Alert", panel: "dialog.dx-alert-dialog[open]" };

const trig = (page: Page, name: string) => page.getByRole("button", { name, exact: true }).first();

async function ready(page: Page, panel: string) {
  await expect(page.locator(panel)).toBeVisible();
  await page.waitForTimeout(450);
}

test.describe("function-form initial_focus receives the open modality", () => {
  test("Dialog: keyboard -> second field, mouse -> Cancel, pen -> fallback (panel)", async ({ page }) => {
    await gotoHydrated(page, url(DIALOG.q), GOTO);
    const t = trig(page, DIALOG.trigger);
    await t.focus();
    await page.keyboard.press("Enter");
    await ready(page, DIALOG.panel);
    expect(await modality(page)).toBe("keyboard");
    expect(await activeId(page)).toBe("dialog-focus-second");
    await page.keyboard.press("Escape");
    await expect(page.locator(DIALOG.panel)).toHaveCount(0);

    await t.click();
    await ready(page, DIALOG.panel);
    expect(await modality(page)).toBe("mouse");
    expect(await activeId(page)).toBe("dialog-focus-cancel");
    await page.keyboard.press("Escape");
    await expect(page.locator(DIALOG.panel)).toHaveCount(0);

    await penClick(page, t);
    await ready(page, DIALOG.panel);
    expect(await modality(page)).toBe("pen");
    // The selector matches nothing: the built-in policy decides (pen follows touch -> the panel).
    expect(await panelFocused(page, DIALOG.panel)).toBe(true);
  });

  test("Dialog: touch -> the panel, never a field", async ({ browser }) => {
    const { context, page } = await touchPage(browser, DIALOG.q);
    try {
      await trig(page, DIALOG.trigger).tap();
      await ready(page, DIALOG.panel);
      expect(await modality(page)).toBe("touch");
      expect(await panelFocused(page, DIALOG.panel)).toBe(true);
    } finally {
      await context.close();
    }
  });

  test("AlertDialog: keyboard -> Cancel, mouse -> the panel", async ({ page }) => {
    await gotoHydrated(page, url(ALERT.q), GOTO);
    const t = trig(page, ALERT.trigger);
    await t.focus();
    await page.keyboard.press("Enter");
    await ready(page, ALERT.panel);
    expect(await activeId(page)).toBe("alert-focus-cancel");
    await page.keyboard.press("Escape");
    await expect(page.locator(ALERT.panel)).toHaveCount(0);
    await t.click();
    await ready(page, ALERT.panel);
    expect(await panelFocused(page, ALERT.panel)).toBe(true);
  });

  test("AlertDialog: touch -> the panel", async ({ browser }) => {
    const { context, page } = await touchPage(browser, ALERT.q);
    try {
      await trig(page, ALERT.trigger).tap();
      await ready(page, ALERT.panel);
      expect(await panelFocused(page, ALERT.panel)).toBe(true);
    } finally {
      await context.close();
    }
  });

  test("Popover: keyboard -> second field, mouse -> first field, pen -> the panel", async ({ page }) => {
    await gotoHydrated(page, url(POPOVER.q), GOTO);
    const t = trig(page, POPOVER.trigger);
    await t.focus();
    await page.keyboard.press("Enter");
    await ready(page, POPOVER.panel);
    expect(await activeId(page)).toBe("popover-focus-second");
    await page.keyboard.press("Escape");
    await expect(page.locator(POPOVER.panel)).toHaveCount(0);

    await t.click();
    await ready(page, POPOVER.panel);
    expect(await activeId(page)).toBe("popover-focus-first");
    await page.keyboard.press("Escape");
    await expect(page.locator(POPOVER.panel)).toHaveCount(0);

    await penClick(page, t);
    await ready(page, POPOVER.panel);
    expect(await modality(page)).toBe("pen");
    expect(await panelFocused(page, POPOVER.panel)).toBe(true);
  });

  test("Popover: touch -> the panel", async ({ browser }) => {
    const { context, page } = await touchPage(browser, POPOVER.q);
    try {
      await trig(page, POPOVER.trigger).tap();
      await ready(page, POPOVER.panel);
      expect(await panelFocused(page, POPOVER.panel)).toBe(true);
    } finally {
      await context.close();
    }
  });
});

test.describe("function-form final_focus", () => {
  test("Dialog: a keyboard close goes to the named target, a mouse close to the trigger", async ({ page }) => {
    await gotoHydrated(page, url(DIALOG.q), GOTO);
    const t = trig(page, DIALOG.trigger);
    await t.focus();
    await page.keyboard.press("Enter");
    await ready(page, DIALOG.panel);
    await page.keyboard.press("Escape");
    await expect(page.locator(DIALOG.panel)).toHaveCount(0);
    expect(await activeId(page)).toBe("dialog-focus-return");

    await t.click();
    await ready(page, DIALOG.panel);
    await page.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(page.locator(DIALOG.panel)).toHaveCount(0);
    expect(await modality(page)).toBe("mouse");
    await expect(t).toBeFocused();
  });

  test("Popover: a keyboard close goes to the named target", async ({ page }) => {
    await gotoHydrated(page, url(POPOVER.q), GOTO);
    const t = trig(page, POPOVER.trigger);
    await t.focus();
    await page.keyboard.press("Enter");
    await ready(page, POPOVER.panel);
    await page.keyboard.press("Escape");
    await expect(page.locator(POPOVER.panel)).toHaveCount(0);
    expect(await activeId(page)).toBe("popover-focus-return");
  });
});

test.describe("a programmatic open returns focus to the previously focused element", () => {
  /** A focusable element X outside the demo, focused by script. */
  async function addX(page: Page) {
    await page.evaluate(() => {
      const x = document.createElement("button");
      x.id = "open-focus-x";
      x.textContent = "X";
      document.body.prepend(x);
      x.focus();
    });
    expect(await activeId(page)).toBe("open-focus-x");
  }

  test("Dialog: Escape after a script open", async ({ page }) => {
    await gotoHydrated(page, url("dialog/"), GOTO);
    await addX(page);
    // A script click does not move focus: X still holds it when the dialog opens.
    await trig(page, "Open Dialog").evaluate((el) => (el as HTMLElement).click());
    await ready(page, "dialog.dx-dialog[open]");
    await page.keyboard.press("Escape");
    await expect(page.locator("dialog.dx-dialog[open]")).toHaveCount(0);
    expect(await activeId(page)).toBe("open-focus-x");
  });

  test("Popover closed by state (not by the browser): focus inside the panel goes back to X", async ({ page }) => {
    await gotoHydrated(page, url("popover/"), GOTO);
    await addX(page);
    const t = trig(page, "Show Popover");
    await t.evaluate((el) => (el as HTMLElement).click());
    await ready(page, "dialog[popover]:popover-open");
    // Focus is inside the panel (a field). Closing through the trigger's own state toggle does not
    // hide the popover natively, so only the shared close handoff can return focus.
    expect(await page.evaluate(() => !!document.activeElement?.closest("dialog[popover]"))).toBe(true);
    await t.evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("dialog[popover]:popover-open")).toHaveCount(0);
    expect(await activeId(page)).toBe("open-focus-x");
  });

  test("the opener is remembered by one mechanism (dxOpenFocus), for the modal Popover too", async ({ page }) => {
    await gotoHydrated(page, url("popover/"), GOTO);
    await addX(page);
    await trig(page, "Open modal popover").evaluate((el) => (el as HTMLElement).click());
    const panel = page.locator("dialog[open]");
    await ready(page, "dialog[open]");
    expect(await panel.evaluate((el: any) => el.__dxOpener?.id)).toBe("open-focus-x");
    await page.keyboard.press("Escape");
    await expect(panel).toHaveCount(0);
    expect(await activeId(page)).toBe("open-focus-x");
  });
});

test.describe("pointer open: Select highlights the selected option, Menu highlights nothing", () => {
  test("Select: a click open with nothing selected leaves focus on the listbox; once selected, a click open focuses it", async ({ page }) => {
    await gotoHydrated(page, url("select/"), GOTO);
    const trigger = page.getByRole("combobox").filter({ hasText: /Select an option|Apple|Banana/ });
    await trigger.click();
    const list = page.getByRole("listbox");
    await expect(list).toBeFocused();
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await expect(list).toHaveCount(0);
    await expect(trigger).toHaveText("Banana");

    await trigger.click();
    expect(await modality(page)).toBe("mouse");
    const selected = page.getByRole("listbox").getByRole("option", { name: "banana" });
    await expect(selected).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("listbox")).toHaveCount(0);

    await trigger.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("listbox").getByRole("option", { name: "banana" })).toBeFocused();
  });

  test("DropdownMenu: a click open highlights no item, a keyboard open the first", async ({ page }) => {
    await gotoHydrated(page, url("dropdown_menu/"), GOTO);
    const trigger = page.getByRole("button", { name: "Open Menu", exact: true });
    await trigger.click();
    const menu = page.getByRole("menu").first();
    await expect(menu).toBeVisible();
    expect(await page.evaluate(() => document.activeElement?.getAttribute("role"))).not.toBe("menuitem");
    await page.keyboard.press("Escape");
    await expect(page.getByRole("menu")).toHaveCount(0);

    await trigger.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("menu").first()).toBeVisible();
    await expect.poll(() => page.evaluate(() => document.activeElement?.getAttribute("role"))).toBe("menuitem");
  });
});

test.describe("modality writer", () => {
  test("a click with detail 0 is a keyboard activation; a real click is a mouse one", async ({ page }) => {
    await gotoHydrated(page, url("button/"), GOTO);
    const b = page.getByRole("button").first();
    await b.click();
    expect(await modality(page)).toBe("mouse");
    // No keydown, no pointerdown: a screen reader or voice-control activation (`click.detail === 0`).
    await b.evaluate((el) => (el as HTMLElement).click());
    expect(await modality(page)).toBe("keyboard");
    await b.click();
    expect(await modality(page)).toBe("mouse");
  });

  test("a field focused by a mouse open still matches :focus-visible (no focusVisible:false in the policy)", async ({ page }) => {
    await gotoHydrated(page, url("dialog/"), GOTO);
    await trig(page, "Open Dialog").click();
    await ready(page, "dialog.dx-dialog[open]");
    expect(await modality(page)).toBe("mouse");
    expect(await activeId(page)).toBe("dialog-edit-name");
    expect(await page.evaluate(() => (document.activeElement as HTMLElement).matches(":focus-visible"))).toBe(true);
  });
});
