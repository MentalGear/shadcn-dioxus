/**
 * A touch (or pen) open never focuses a field and never draws a focus ring (backlog row 226).
 *
 * Owner report (iOS, deployed 83758d5): opening a modal, alert, sheet, drawer, popover or command bar
 * by touch drew a blue ring around the panel or the trigger, and a field inside took focus as soon as
 * the panel opened, so the software keyboard rose while the panel was still being placed.
 *
 * The construction under test is ONE modality writer (`primitives/src/modality.rs`, `<html
 * data-dx-modality>`), ONE open-focus policy (`dxOpenFocus`, shared by the dialog driver, the
 * non-modal Popover's show script and Command) and ONE ring rule in the theme. The keyboard half
 * (`modal-initial-focus.spec.ts`, `focus-ring.spec.ts`) is unchanged and re-proved at the bottom.
 * Since the owner decision of 2026-10-18 (Base UI's default open-focus policy) a MOUSE open behaves
 * like the keyboard one -- the first control, a text field included -- and only touch/pen is kept on
 * the panel; the Drawer included. Each overlay has a mouse
 * counterpart below.
 *
 * The emulation is what reproduced the report in plain Chromium: an iPhone 13 context (`hasTouch`,
 * `isMobile`) plus an "iOS-like" capture `mousedown` `preventDefault` on every target that is not a
 * text field. Safari does not focus a tapped button, so `document.activeElement` stays `<body>`
 * after a tap, and THAT is what made the script-focused panel match `:focus-visible`; a plain
 * Chromium tap focuses the button first and hides the whole class. What no Chromium run can prove
 * is Safari's own `:focus-visible` heuristic and the real software keyboard: see the manual iPhone
 * checklist in the lane report.
 *
 * Overlays: the modal ones are DERIVED from `modal-consumers.ts` (dialog, alert dialog, sheet,
 * drawer, command), so a new modal cannot ship without this proof; the popup family is listed here.
 * Combobox is exempt: its text input IS the trigger, so the user's own tap focuses it.
 */
import type { Browser, BrowserContext, Page } from "@playwright/test";
import { test, expect, devices } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { CONSUMERS } from "./modal-consumers";

const GOTO = { timeout: 20 * 60 * 1000 };

interface Overlay {
  name: string;
  /** The `/component/<name>/` path of the page that holds the demo. */
  path: string;
  /** Accessible name of the trigger, or a CSS selector. */
  trigger: { role: "button" | "combobox" | "menuitem"; name: string | RegExp } | { css: string };
  /** `true` when the first control in the panel is a text field (a keyboard open focuses it). */
  keyboardFocusesField?: boolean;
  /** What a keyboard open must do: land inside the panel, or only open it. */
  keyboard: "inside" | "opens";
  /**
   * A menu opened by touch moves no focus at all (its items are roving, the trigger is not focused
   * after a tap), so a field that already held focus keeps it, exactly as for any tapped button on
   * iOS. What it must not do is focus a field itself.
   */
  leavesFocusAlone?: boolean;
}

const MODALS: Overlay[] = CONSUMERS.map((c) => ({
  name: c.component,
  path: `${c.component}/`,
  trigger: { role: "button", name: c.variants[0].trigger },
  keyboard: "inside",
  keyboardFocusesField: c.component === "command",
}));

const POPUPS: Overlay[] = [
  {
    name: "popover (non-modal)",
    path: "popover/",
    trigger: { role: "button", name: "Show Popover" },
    keyboard: "inside",
    keyboardFocusesField: true,
  },
  {
    name: "popover (modal)",
    path: "popover/",
    trigger: { role: "button", name: "Open modal popover" },
    keyboard: "inside",
    keyboardFocusesField: true,
  },
  { name: "select", path: "select/", trigger: { css: "button[role='combobox'][aria-haspopup='listbox']:visible" }, keyboard: "opens" },
  { name: "date_picker", path: "date_picker/", trigger: { css: "button[aria-haspopup='dialog']:visible" }, keyboard: "inside" },
  {
    name: "dropdown_menu",
    path: "dropdown_menu/",
    trigger: { role: "button", name: "Open Menu" },
    keyboard: "opens",
    leavesFocusAlone: true,
  },
  { name: "menubar", path: "menubar/", trigger: { role: "menuitem", name: "File" }, keyboard: "opens" },
];

const OVERLAYS = [...MODALS, ...POPUPS];

/** iPhone 13 emulation plus Safari's "a tapped button takes no focus" (the setup that reproduced the report). */
async function iosLikeContext(browser: Browser): Promise<BrowserContext> {
  const { defaultBrowserType: _unused, ...iphone } = devices["iPhone 13"];
  const context = await browser.newContext({ ...iphone });
  await context.addInitScript(() => {
    addEventListener(
      "mousedown",
      (e) => {
        const t = e.target;
        if (t instanceof Element && !t.closest("input,textarea,select,[contenteditable]")) e.preventDefault();
      },
      true,
    );
    // The software keyboard cannot be observed here; a focused text-entry element is what raises it.
    (window as unknown as { __kbd: number }).__kbd = 0;
    addEventListener(
      "focusin",
      (e) => {
        const t = e.target;
        if (t instanceof HTMLElement && (t.matches("input:not([type=checkbox]):not([type=radio]):not([type=button]),textarea") || t.isContentEditable))
          (window as unknown as { __kbd: number }).__kbd++;
      },
      true,
    );
  });
  return context;
}

function triggerOf(page: Page, o: Overlay) {
  if ("css" in o.trigger) return page.locator(o.trigger.css).first();
  return page.getByRole(o.trigger.role, { name: o.trigger.name, exact: typeof o.trigger.name === "string" }).first();
}

const OPEN_PANEL = "dialog[open], :popover-open, [role='listbox']:popover-open";

async function openPanelCount(page: Page): Promise<number> {
  return page.evaluate((sel) => document.querySelectorAll(sel).length, OPEN_PANEL);
}

interface Report {
  modality: string | null;
  active: string;
  editable: boolean;
  keyboardRises: number;
  outlines: string[];
  rings: string[];
  focusInPanel: boolean;
}

/** Where focus is, what the focused element and every open panel paint, and whether a field ever took focus. */
async function report(page: Page): Promise<Report> {
  return page.evaluate((panelSel) => {
    const root = document.documentElement;
    const active = document.activeElement as HTMLElement | null;
    const tag = active ? active.tagName : "none";
    const editable =
      !!active &&
      ((tag === "INPUT" && !/^(button|checkbox|radio|submit|reset|image|file|range|color|hidden)$/i.test((active as HTMLInputElement).type)) ||
        tag === "TEXTAREA" ||
        tag === "SELECT" ||
        active.isContentEditable);
    // The ring this page would draw: the two shapes a non-field ring takes, resolved by the browser.
    const probe = document.createElement("div");
    probe.style.cssText =
      "position:fixed;left:-9999px;box-shadow:0 0 0 3px color-mix(in oklab, var(--dx-ring-color) 50%, transparent), inset 0 0 0 3px color-mix(in oklab, var(--dx-ring-color) 50%, transparent)";
    document.body.appendChild(probe);
    const parts = getComputedStyle(probe).boxShadow.split(/,(?![^(]*\))/).map((p) => p.trim());
    probe.remove();
    // The seeded stale field (the "iOS kept focus on a tapped input" fixture) is the test's own, and a
    // focused text field legitimately has its field ring: it is not what the overlay drew.
    const subjects = [active, ...Array.from(document.querySelectorAll(panelSel))].filter(
      (e): e is HTMLElement => !!e && (e as HTMLElement).id !== "stale-field",
    );
    const outlines: string[] = [];
    const rings: string[] = [];
    for (const el of subjects) {
      const cs = getComputedStyle(el);
      const label = `${el.tagName.toLowerCase()}${el.id ? "#" + el.id : ""}.${(el.getAttribute("class") || "").split(" ")[0]}`;
      if (cs.outlineStyle !== "none" && cs.outlineWidth !== "0px") outlines.push(`${label}: ${cs.outlineStyle} ${cs.outlineWidth}`);
      for (const part of parts) if (part && cs.boxShadow.includes(part)) rings.push(`${label}: ${part}`);
    }
    const panel = document.querySelector(panelSel);
    return {
      modality: root.getAttribute("data-dx-modality"),
      active: `${tag.toLowerCase()}${active?.id ? "#" + active.id : ""}`,
      editable,
      keyboardRises: (window as unknown as { __kbd: number }).__kbd,
      outlines,
      rings,
      focusInPanel: !!panel && !!active && panel.contains(active),
    };
  }, OPEN_PANEL);
}

/** The shared assertion: touch open, nothing editable focused, no outline, no ring. */
async function expectQuietOpen(page: Page, what: string) {
  const r = await report(page);
  expect(r.modality, `${what}: the modality writer saw the tap`).toBe("touch");
  expect(r.editable, `${what}: a field must not be focused by a touch open (focused ${r.active})`).toBe(false);
  expect(r.keyboardRises, `${what}: no text-entry element may take focus while the overlay opens`).toBe(0);
  expect(r.outlines, `${what}: no UA outline on the focused element or a panel`).toEqual([]);
  expect(r.rings, `${what}: no focus-ring box-shadow on the focused element or a panel`).toEqual([]);
}

async function open(page: Page, o: Overlay) {
  await gotoHydrated(page, `${BASE_URL}/component/${o.path}`, GOTO);
}

for (const o of OVERLAYS) {
  test.describe(`${o.name}: touch open`, () => {
    test("tap opens with no focused field, no outline and no ring; reopens the same way", async ({ browser }) => {
      const context = await iosLikeContext(browser);
      try {
        const page = await context.newPage();
        await open(page, o);
        const trigger = triggerOf(page, o);
        await trigger.scrollIntoViewIfNeeded();
        await trigger.tap();
        await expect.poll(() => openPanelCount(page)).toBeGreaterThan(0);
        await page.waitForTimeout(450); // past the enter animation, so geometry and focus are at rest
        await expectQuietOpen(page, `${o.name} first open`);

        // Close it, and open it again: the second open is no different.
        await page.keyboard.press("Escape");
        await expect.poll(() => openPanelCount(page)).toBe(0);
        await page.evaluate(() => {
          (window as unknown as { __kbd: number }).__kbd = 0;
        });
        await trigger.scrollIntoViewIfNeeded();
        await trigger.tap();
        await expect.poll(() => openPanelCount(page)).toBeGreaterThan(0);
        await page.waitForTimeout(450);
        await expectQuietOpen(page, `${o.name} reopen`);
      } finally {
        await context.close();
      }
    });

    test("a stale focused text field is not read as a keyboard open", async ({ browser }) => {
      // iOS keeps focus on a previously tapped text field when a button is tapped. That field always
      // matches :focus-visible, which row 206's opener check read as "opened by keyboard".
      const context = await iosLikeContext(browser);
      try {
        const page = await context.newPage();
        await open(page, o);
        const trigger = triggerOf(page, o);
        await trigger.scrollIntoViewIfNeeded();
        await page.evaluate(() => {
          const i = document.createElement("input");
          i.id = "stale-field";
          i.style.cssText = "position:fixed;top:0;left:0;width:1px;height:1px;opacity:0";
          document.body.appendChild(i);
          i.focus();
        });
        await page.evaluate(() => {
          (window as unknown as { __kbd: number }).__kbd = 0; // the seeding above is not the overlay's doing
        });
        await trigger.tap();
        await expect.poll(() => openPanelCount(page)).toBeGreaterThan(0);
        await page.waitForTimeout(450);
        const r = await report(page);
        expect(r.keyboardRises, `${o.name}: opening must not focus a text-entry element`).toBe(0);
        if (!o.leavesFocusAlone) {
          expect(r.active, `${o.name}: focus left the stale field for the panel or trigger`).not.toBe("input#stale-field");
          expect(r.editable, `${o.name}: opened with a stale field focused, then focused a field (${r.active})`).toBe(false);
        }
        expect(r.outlines).toEqual([]);
      } finally {
        await context.close();
      }
    });

    test("the mouse counterpart focuses the first control, like a keyboard open", async ({ page }) => {
      await open(page, o);
      const trigger = triggerOf(page, o);
      await trigger.scrollIntoViewIfNeeded();
      await trigger.click();
      await expect.poll(() => openPanelCount(page)).toBeGreaterThan(0);
      await page.waitForTimeout(450);
      const r = await report(page);
      expect(r.modality).toBe("mouse");
      if (o.keyboard === "inside") {
        const isPanel = await page.evaluate(
          (sel) => document.activeElement === document.querySelector(sel),
          OPEN_PANEL,
        );
        expect(r.focusInPanel, `${o.name}: a mouse open puts focus inside the panel (focused ${r.active})`).toBe(true);
        expect(isPanel, `${o.name}: a mouse open focuses a control, not the panel`).toBe(false);
        if (o.keyboardFocusesField) expect(r.editable, `${o.name}: first control is a field`).toBe(true);
      }
    });

    test("the keyboard counterpart is unchanged", async ({ page }) => {
      await open(page, o);
      const trigger = triggerOf(page, o);
      await trigger.scrollIntoViewIfNeeded();
      await trigger.focus();
      await page.keyboard.press("Enter");
      await expect.poll(() => openPanelCount(page)).toBeGreaterThan(0);
      await page.waitForTimeout(450);
      const r = await report(page);
      expect(r.modality).toBe("keyboard");
      if (o.keyboard === "inside") {
        expect(r.focusInPanel, `${o.name}: a keyboard open puts focus inside the panel (focused ${r.active})`).toBe(true);
        const isPanel = await page.evaluate(
          (sel) => document.activeElement === document.querySelector(sel),
          OPEN_PANEL,
        );
        expect(isPanel, `${o.name}: a keyboard open focuses a control, not the panel`).toBe(false);
        if (o.keyboardFocusesField) expect(r.editable, `${o.name}: first control is a field`).toBe(true);
      }
    });
  });
}

test.describe("modality writer", () => {
  test("pointer type and key presses write <html data-dx-modality>", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button/`, GOTO);
    const modality = () => page.evaluate(() => document.documentElement.getAttribute("data-dx-modality"));
    // Before any event the desktop fallback is `mouse` (no coarse pointer).
    expect(["mouse", "keyboard"]).toContain(await modality());
    await page.mouse.click(5, 5);
    expect(await modality()).toBe("mouse");
    await page.keyboard.press("Shift"); // a bare modifier is not a keyboard interaction
    expect(await modality()).toBe("mouse");
    await page.keyboard.press("Tab");
    expect(await modality()).toBe("keyboard");
    await page.mouse.click(5, 5);
    expect(await modality()).toBe("mouse");
  });

  test("a touch device reads as touch before any event, and a tap writes touch", async ({ browser }) => {
    const context = await iosLikeContext(browser);
    try {
      const page = await context.newPage();
      await gotoHydrated(page, `${BASE_URL}/component/button/`, GOTO);
      expect(await page.evaluate(() => document.documentElement.getAttribute("data-dx-modality"))).toBe("touch");
      await page.keyboard.press("Tab");
      expect(await page.evaluate(() => document.documentElement.getAttribute("data-dx-modality"))).toBe("keyboard");
      await page.touchscreen.tap(5, 5);
      expect(await page.evaluate(() => document.documentElement.getAttribute("data-dx-modality"))).toBe("touch");
    } finally {
      await context.close();
    }
  });

  test("a text field still rings when tapped (--dx-ring-field is not modality-gated)", async ({ browser }) => {
    const context = await iosLikeContext(browser);
    try {
      const page = await context.newPage();
      await gotoHydrated(page, `${BASE_URL}/component/input/`, GOTO);
      const input = page.locator("input.dx-input, input[class*='dx-input']").first();
      await input.scrollIntoViewIfNeeded();
      await input.tap();
      await expect(input).toBeFocused();
      const shadow = await input.evaluate((el) => getComputedStyle(el).boxShadow);
      const field = await page.evaluate(() => {
        const p = document.createElement("div");
        p.style.cssText = "position:fixed;left:-9999px;box-shadow:var(--dx-ring-field)";
        document.body.appendChild(p);
        const v = getComputedStyle(p).boxShadow;
        p.remove();
        return v;
      });
      expect(field).not.toBe("none");
      for (const part of field.split(/,(?![^(]*\))/).map((p) => p.trim())) expect(shadow).toContain(part);
    } finally {
      await context.close();
    }
  });
});
