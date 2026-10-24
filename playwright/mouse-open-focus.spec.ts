/**
 * A MOUSE open focuses the first control, exactly like a keyboard open (backlog row 226 and its
 * follow-up): Base UI's default open-focus policy returns the popup only for
 * touch (`createDefaultInitialFocus`, utils/popups/popupStoreUtils.js). So, on one modality writer
 * and one policy (`dxOpenFocus`, `primitives/src/modality.rs`):
 *
 *   - mouse open of a Dialog with a form: the first FIELD is focused and paints its ring
 *     (`--dx-ring-field` is never modality-gated, the same as shadcn);
 *   - mouse open of a Popover with a field: the field is focused;
 *   - mouse open of a Command palette: the search input is focused;
 *   - mouse open of a Drawer: the same rule as a Dialog -- its first field is focused (deliberate deviation
 *     from Base UI, whose DrawerPopup always focuses the popup; our touch rule covers that case);
 *   - touch open of the same: the panel is focused and no editable element is (no software keyboard).
 *
 * The buttons and containers that take focus after a pointer open paint no ring (the non-field ring
 * tokens are transparent outside the keyboard modality): proved on the Dialog's close button case.
 */
import type { Browser, Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };

interface Case {
  name: string;
  path: string;
  trigger: string;
  panel: string;
}

const DIALOG: Case = {
  name: "dialog with a form",
  path: "dialog/",
  trigger: "Open Dialog",
  panel: "dialog.dx-dialog[open]",
};
const POPOVER: Case = {
  name: "popover with fields",
  path: "popover/",
  trigger: "Show Popover",
  panel: "dialog[popover]:popover-open",
};
const COMMAND: Case = {
  name: "command palette",
  path: "command/",
  trigger: "Open Command Palette",
  panel: "dialog.dx-command-dialog[open]",
};
const DRAWER: Case = {
  name: "drawer",
  path: "drawer/",
  trigger: "Move Goal",
  panel: "dialog.dx-drawer[open]",
};

async function focusState(page: Page, panel: string) {
  return page.evaluate((sel) => {
    const p = document.querySelector(sel) as HTMLElement | null;
    const a = document.activeElement as HTMLElement | null;
    const tag = a ? a.tagName : "none";
    const editable =
      tag === "TEXTAREA" ||
      tag === "SELECT" ||
      !!a?.isContentEditable ||
      (tag === "INPUT" && !/^(button|checkbox|radio|submit|reset|image|file|range|color|hidden)$/i.test((a as HTMLInputElement).type));
    return {
      modality: document.documentElement.getAttribute("data-dx-modality"),
      isPanel: !!p && a === p,
      inPanel: !!p && !!a && p.contains(a),
      editable,
      tag,
      id: a?.id ?? "",
    };
  }, panel);
}

async function openByMouse(page: Page, c: Case) {
  await gotoHydrated(page, `${BASE_URL}/component/${c.path}`, GOTO);
  const trigger = page.getByRole("button", { name: c.trigger, exact: true }).first();
  await trigger.scrollIntoViewIfNeeded();
  await trigger.click();
  await expect(page.locator(c.panel)).toBeVisible();
  await page.waitForTimeout(450); // past the enter animation
}

async function openByTouch(browser: Browser, c: Case) {
  const context = await browser.newContext({ hasTouch: true });
  const page = await context.newPage();
  await gotoHydrated(page, `${BASE_URL}/component/${c.path}`, GOTO);
  const trigger = page.getByRole("button", { name: c.trigger, exact: true }).first();
  await trigger.scrollIntoViewIfNeeded();
  await trigger.tap();
  await expect(page.locator(c.panel)).toBeVisible();
  await page.waitForTimeout(450);
  return { context, page };
}

test.describe("mouse open focuses the first control", () => {
  test("Dialog with a form: the first field is focused and its ring is visible", async ({ page }) => {
    await openByMouse(page, DIALOG);
    const s = await focusState(page, DIALOG.panel);
    expect(s.modality).toBe("mouse");
    expect(s.isPanel, "the panel itself must not be focused").toBe(false);
    expect(s.editable, `first field focused (focused ${s.tag}#${s.id})`).toBe(true);
    expect(s.id).toBe("dialog-edit-name");
    const ring = await page.evaluate(() => {
      const el = document.activeElement as HTMLElement;
      const probe = document.createElement("div");
      probe.style.cssText = "position:fixed;left:-9999px;box-shadow:var(--dx-ring-field)";
      document.body.appendChild(probe);
      const field = getComputedStyle(probe).boxShadow;
      probe.remove();
      return { shadow: getComputedStyle(el).boxShadow, field };
    });
    expect(ring.field).not.toBe("none");
    for (const part of ring.field.split(/,(?![^(]*\))/).map((p) => p.trim())) expect(ring.shadow).toContain(part);
  });

  test("Popover with fields: a field is focused", async ({ page }) => {
    await openByMouse(page, POPOVER);
    const s = await focusState(page, POPOVER.panel);
    expect(s.modality).toBe("mouse");
    expect(s.isPanel, "the panel itself must not be focused").toBe(false);
    expect(s.inPanel).toBe(true);
    expect(s.editable, `a field is focused (focused ${s.tag}#${s.id})`).toBe(true);
  });

  test("Command palette: the search input is focused", async ({ page }) => {
    await openByMouse(page, COMMAND);
    const s = await focusState(page, COMMAND.panel);
    expect(s.modality).toBe("mouse");
    expect(s.inPanel).toBe(true);
    expect(s.tag).toBe("INPUT");
    expect(s.isPanel).toBe(false);
  });

  test("Drawer: the same rule as a Dialog, the first control is focused (a field when the drawer has one)", async ({ page }) => {
    await openByMouse(page, DRAWER);
    const s = await focusState(page, DRAWER.panel);
    expect(s.modality).toBe("mouse");
    expect(s.isPanel, `the drawer panel must not be focused (focused ${s.tag}#${s.id})`).toBe(false);
    expect(s.inPanel).toBe(true);
  });

  test("a control that takes focus after a mouse open paints no non-field ring", async ({ page }) => {
    // The alert dialog has no field: its first control is a button, focused by the mouse open.
    await openByMouse(page, {
      name: "alert dialog",
      path: "alert_dialog/",
      trigger: "Show Alert Dialog",
      panel: "dialog.dx-alert-dialog[open]",
    });
    const r = await page.evaluate(() => {
      const el = document.activeElement as HTMLElement;
      const probe = document.createElement("div");
      probe.style.cssText =
        "position:fixed;left:-9999px;box-shadow:0 0 0 3px color-mix(in oklab, var(--dx-ring-color) 50%, transparent)";
      document.body.appendChild(probe);
      const ring = getComputedStyle(probe).boxShadow;
      probe.remove();
      const cs = getComputedStyle(el);
      return { tag: el.tagName, shadow: cs.boxShadow, ring, outline: `${cs.outlineStyle} ${cs.outlineWidth}` };
    });
    expect(r.tag).toBe("BUTTON");
    expect(r.shadow).not.toContain(r.ring);
    expect(r.outline === "none 0px" || r.outline.startsWith("none")).toBe(true);
  });
});

test.describe("touch open focuses the panel and no editable element", () => {
  for (const c of [DIALOG, POPOVER, COMMAND, DRAWER]) {
    test(`${c.name}: tap`, async ({ browser }) => {
      const { context, page } = await openByTouch(browser, c);
      try {
        const s = await focusState(page, c.panel);
        expect(s.modality).toBe("touch");
        expect(s.isPanel, `panel focused (focused ${s.tag}#${s.id})`).toBe(true);
        expect(s.editable).toBe(false);
      } finally {
        await context.close();
      }
    });
  }
});
