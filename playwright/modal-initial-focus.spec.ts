/**
 * Modal initial focus follows the modality that opened the modal (backlog row 206).
 *
 * The failure it prevents: tapping a demo button that opens a dialog/drawer must not focus its first field, which
 * pops the mobile keyboard. The construction is ONE place -- `use_dialog_open_driver`
 * (`primitives/src/lib.rs`), the driver every native-`<dialog>` modal goes through (Dialog,
 * AlertDialog, Sheet, Drawer, CommandDialog, the modal Popover). At open time it reads the
 * element that holds focus (the opener): opened by KEYBOARD it matches `:focus-visible` (the
 * browser's own focus-ring heuristic), so `showModal()`'s default stands and the first
 * focusable descendant is focused. Anything else -- a mouse or tap click, an unfocused opener
 * (Safari does not focus a clicked button), no opener at all (focus on <body>) -- focuses the
 * `<dialog>` panel itself (`tabindex="-1"`). An `autofocus` inside always wins (the Command
 * palette's input declares one: a search box is the point of a palette). No listener, no
 * window global, no install order: the result is the same on every page, which the modal
 * Popover case proves on a page with no Dialog/AlertDialog root.
 *
 * Row 226: the open-focus policy reads the interaction modality (`<html
 * data-dx-modality>`, written once by `primitives/src/modality.rs`), not the opener's `:focus-visible` (iOS
 * keeps focus on a field tapped earlier, which always matches `:focus-visible`), and the palette uses
 * `data-dx-autofocus`. The policy is Base UI's default open-focus policy: keyboard AND
 * mouse focus the first control (the palette's search box included), touch and pen focus the panel;

 * `touch-open-focus.spec.ts` is the touch half (every overlay, iOS-like emulation);
 * `mouse-open-focus.spec.ts` has the explicit mouse cases.
 *
 * Also proved here: Tab from the panel enters the dialog, Escape closes it and the
 * trigger gets focus back, and the latest modality wins on a second open.
 *
 * Touch is a `hasTouch` context with `locator.tap()`; mouse is a plain `click()`.
 */
import type { Browser, Locator, Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { CONSUMERS } from "./modal-consumers";

const GOTO = { timeout: 20 * 60 * 1000 };

/** Consumers whose search input is the focus target of a keyboard or mouse open (and of nothing else). */
const OWN_AUTOFOCUS = new Set(["command"]);

type Where = {
  isDialog: boolean;
  inDialog: boolean;
  tag: string;
  textEntry: boolean;
};

/** Where `document.activeElement` is relative to the (single) open dialog matching `selector`. */
async function whereIsFocus(page: Page, selector: string): Promise<Where> {
  return page.evaluate((sel) => {
    const dialog = document.querySelector(sel) as HTMLElement | null;
    const active = document.activeElement as HTMLElement | null;
    const tag = active ? active.tagName : "none";
    return {
      isDialog: !!dialog && active === dialog,
      inDialog: !!dialog && !!active && dialog.contains(active),
      tag,
      textEntry: tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || !!active?.isContentEditable,
    };
  }, selector);
}

async function open(page: Page, component: string): Promise<void> {
  await gotoHydrated(page, `${BASE_URL}/component/${component}/`, GOTO);
}

async function openedBy(page: Page, trigger: Locator, how: "click" | "tap" | "enter" | "space") {
  await trigger.scrollIntoViewIfNeeded();
  if (how === "click") await trigger.click();
  else if (how === "tap") await trigger.tap();
  else {
    await trigger.focus();
    await page.keyboard.press(how === "enter" ? "Enter" : "Space");
  }
}

async function touchPage(browser: Browser) {
  const context = await browser.newContext({ hasTouch: true });
  return { context, page: await context.newPage() };
}

for (const consumer of CONSUMERS) {
  const variant = consumer.variants[0];
  const ownAutofocus = OWN_AUTOFOCUS.has(consumer.component);

  test.describe(`${consumer.component}: initial focus by opening modality`, () => {
    const trigger = (page: Page) => page.getByRole("button", { name: variant.trigger, exact: true }).first();
    const dialog = (page: Page) => page.locator(`${consumer.dialogSelector}[open]`);

    test("opened by mouse click", async ({ page }) => {
      await open(page, consumer.component);
      await openedBy(page, trigger(page), "click");
      await expect(dialog(page)).toBeVisible();
      // Base UI's default: a mouse open is a keyboard open (the first control, the palette's search
      // box), the Drawer included.
      await expect.poll(() => whereIsFocus(page, consumer.dialogSelector)).toMatchObject(
        ownAutofocus ? { inDialog: true, isDialog: false, tag: "INPUT" } : { inDialog: true, isDialog: false },
      );
    });

    test("opened by touch tap", async ({ browser }) => {
      const { context, page } = await touchPage(browser);
      try {
        await open(page, consumer.component);
        await openedBy(page, trigger(page), "tap");
        await expect(dialog(page)).toBeVisible();
        // The palette's search box is kept for a keyboard or a mouse, never for a tap (row 226): a
        // tap leaves the panel focused and the user taps the field to type.
        await expect.poll(() => whereIsFocus(page, consumer.dialogSelector)).toMatchObject({
          isDialog: true,
          textEntry: false,
        });
      } finally {
        await context.close();
      }
    });

    for (const key of ["enter", "space"] as const) {
      test(`opened by keyboard (${key}) focuses a control inside, not the panel`, async ({ page }) => {
        await open(page, consumer.component);
        await openedBy(page, trigger(page), key);
        await expect(dialog(page)).toBeVisible();
        await expect.poll(() => whereIsFocus(page, consumer.dialogSelector)).toMatchObject({
          inDialog: true,
          isDialog: false,
        });
        if (ownAutofocus) expect((await whereIsFocus(page, consumer.dialogSelector)).tag).toBe("INPUT");
      });
    }

    test("a touch-focused panel paints no focus ring; Tab enters it; Escape returns focus to the trigger", async ({
      browser,
    }) => {
      test.skip(ownAutofocus, "the palette focuses its own input, there is no focused panel to inspect");
      const { context, page } = await touchPage(browser);
      try {
        await ringAndTrap(page);
      } finally {
        await context.close();
      }
    });

    async function ringAndTrap(page: Page) {
      await open(page, consumer.component);
      await openedBy(page, trigger(page), "tap");
      await expect(dialog(page)).toBeVisible();
      await expect.poll(() => whereIsFocus(page, consumer.dialogSelector)).toMatchObject({ isDialog: true });

      const ring = await page.evaluate((sel) => {
        const el = document.querySelector(sel) as HTMLElement;
        const cs = getComputedStyle(el);
        return {
          focusVisible: el.matches(":focus-visible"),
          outlineStyle: cs.outlineStyle,
          outlineWidth: cs.outlineWidth,
          tabindex: el.getAttribute("tabindex"),
        };
      }, consumer.dialogSelector);
      expect(ring.tabindex, "the panel is programmatically focusable only").toBe("-1");
      expect(ring.focusVisible, "a pointer-opened panel is not :focus-visible").toBe(false);
      expect(ring.outlineStyle === "none" || ring.outlineWidth === "0px", "no UA outline on the panel").toBe(true);

      // The trap still works from the panel: Tab lands on the first control inside.
      await page.keyboard.press("Tab");
      await expect.poll(() => whereIsFocus(page, consumer.dialogSelector)).toMatchObject({
        inDialog: true,
        isDialog: false,
      });

      await page.keyboard.press("Escape");
      await expect(page.locator(consumer.dialogSelector)).toHaveCount(0);
      await expect(trigger(page)).toBeFocused();
    }

    test("a mouse open reaches the same control as a keyboard open, and a reopen does too", async ({ page }) => {
      await open(page, consumer.component);
      await openedBy(page, trigger(page), "enter");
      await expect(dialog(page)).toBeVisible();
      await expect.poll(() => whereIsFocus(page, consumer.dialogSelector)).toMatchObject({
        inDialog: true,
        isDialog: false,
      });
      const keyboardTarget = await page.evaluate(() => document.activeElement?.tagName);
      await page.keyboard.press("Escape");
      await expect(page.locator(consumer.dialogSelector)).toHaveCount(0);
      await page.mouse.click(1, 1);
      await openedBy(page, trigger(page), "click");
      await expect(dialog(page)).toBeVisible();
      await expect.poll(() => whereIsFocus(page, consumer.dialogSelector)).toMatchObject({
        inDialog: true,
        isDialog: false,
      });
      expect(await page.evaluate(() => document.activeElement?.tagName)).toBe(keyboardTarget);
    });
  });
}

test.describe("sheet text field: a tap leaves it alone, a mouse or keyboard open focuses the first control", () => {
  // The Sheet demo is the one with text inputs ("Edit profile"): the first control a keyboard
  // open reaches, and the field a tap must leave alone.
  test("Sheet: tap opens on the panel with the keyboard closed; keyboard open reaches a control", async ({
    browser,
    page,
  }) => {
    const { context, page: touch } = await touchPage(browser);
    try {
      await open(touch, "sheet");
      await touch.getByRole("button", { name: "Right", exact: true }).tap();
      await expect(touch.locator("dialog.dx-sheet[open]")).toBeVisible();
      const where = await whereIsFocus(touch, "dialog.dx-sheet");
      expect(where.textEntry, `a tap must not focus a text field (focused ${where.tag})`).toBe(false);
      expect(where.isDialog).toBe(true);
      await expect(touch.locator("#sheet-demo-name")).not.toBeFocused();
    } finally {
      await context.close();
    }

    await open(page, "sheet");
    await openedBy(page, page.getByRole("button", { name: "Right", exact: true }), "enter");
    await expect(page.locator("dialog.dx-sheet[open]")).toBeVisible();
    await expect.poll(() => whereIsFocus(page, "dialog.dx-sheet")).toMatchObject({ inDialog: true, isDialog: false });
    const keyboardId = await page.evaluate(() => document.activeElement?.id);
    await page.keyboard.press("Escape");
    await expect(page.locator("dialog.dx-sheet")).toHaveCount(0);

    // A mouse open lands on the same first control (Base UI's default).
    await page.mouse.click(1, 1);
    await openedBy(page, page.getByRole("button", { name: "Right", exact: true }), "click");
    await expect(page.locator("dialog.dx-sheet[open]")).toBeVisible();
    await expect.poll(() => whereIsFocus(page, "dialog.dx-sheet")).toMatchObject({ inDialog: true, isDialog: false });
    expect(await page.evaluate(() => document.activeElement?.id)).toBe(keyboardId);
  });
});

test.describe("modal Popover: the same result on a page with no Dialog or AlertDialog root", () => {
  const url = `${BASE_URL}/component/popover/`;
  const trigger = (page: Page) => page.getByRole("button", { name: "Open modal popover", exact: true });
  const panel = "dialog[open]";

  test("the page has no Dialog or AlertDialog", async ({ page }) => {
    await gotoHydrated(page, url, GOTO);
    expect(await page.locator("dialog.dx-dialog, dialog.dx-alert-dialog").count()).toBe(0);
  });

  test("mouse click and keyboard both focus a field", async ({ page }) => {
    await gotoHydrated(page, url, GOTO);
    await openedBy(page, trigger(page), "click");
    await expect(page.locator(panel)).toBeVisible();
    await expect.poll(() => whereIsFocus(page, panel)).toMatchObject({ inDialog: true, isDialog: false, textEntry: true });
    await page.keyboard.press("Escape");
    await expect(page.locator(panel)).toHaveCount(0);

    await openedBy(page, trigger(page), "enter");
    await expect(page.locator(panel)).toBeVisible();
    await expect.poll(() => whereIsFocus(page, panel)).toMatchObject({ inDialog: true, isDialog: false, textEntry: true });
  });

  test("touch tap focuses the panel", async ({ browser }) => {
    const { context, page } = await touchPage(browser);
    try {
      await gotoHydrated(page, url, GOTO);
      await openedBy(page, trigger(page), "tap");
      await expect(page.locator(panel)).toBeVisible();
      await expect.poll(() => whereIsFocus(page, panel)).toMatchObject({ isDialog: true, textEntry: false });
    } finally {
      await context.close();
    }
  });

  test("an open with no opener (focus on <body>) on a desktop reads as a mouse open: a field", async ({ page }) => {
    await gotoHydrated(page, url, GOTO);
    // A synthetic click never focuses the button: activeElement stays <body>, like Safari's click.
    // With no event seen the modality falls back to the pointer media query: `mouse` on a desktop.
    await trigger(page).dispatchEvent("click");
    await expect(page.locator(panel)).toBeVisible();
    await expect.poll(() => whereIsFocus(page, panel)).toMatchObject({ inDialog: true, isDialog: false, textEntry: true });
  });
});
