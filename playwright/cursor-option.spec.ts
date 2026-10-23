import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoFrame, demoRole, popupItem } from "./demo-frame";

/**
 * The interactive-element cursor option (`<html data-dx-interactive-element-cursor="pointer|default">`,
 * `preview/assets/dx-components-theme.css`, "Interactive-element cursor").
 *
 * Owner 2026-10-23: "some interactive elements use a click pointer (combobox, calendar, etc), others don't --
 * make it a global consistent option and add a toggle to the demo appearance menu." The construction is ONE
 * zero-specificity theme rule over every pressable `dx-` element (`scripts/check-cursor-baseline.sh` keeps the
 * per-component declarations out). This spec pins the behaviour in the browser:
 *
 *   default (no attribute)   a Button, a Combobox's chip remove and option, a Calendar day and a Select trigger all
 *                            compute the arrow (`default`; shadcn v4's), not the hand the sheets used to pick
 *                            one by one;
 *   ="pointer"               every one of them computes `pointer`, in every sweep entry below;
 *   never changed            a text field's `text`, a disabled control's own cursor (`not-allowed`, or no
 *                            pointer for a `pointer-events: none` button), a drag handle's `grab`;
 *   the theme picker's "Cursor" control writes the attribute, persists it through a reload before first
 *   paint, and Reset clears it.
 */

const ATTR = "data-dx-interactive-element-cursor";
const KEY = "dx-interactive-element-cursor";

const cursorOf = (loc: Locator) => loc.evaluate((el) => getComputedStyle(el).cursor);

async function setOption(page: Page, value: string | null) {
  await page.evaluate(
    ([k, v]) => (window as unknown as { __dxTheme: (k: string, v: string | null) => void }).__dxTheme(k, v),
    [KEY, value] as const,
  );
}

async function open(page: Page, route: string) {
  await gotoHydrated(page, `${BASE_URL}${route}`);
}

test.describe("default: shadcn v4's arrow on every interactive element", () => {
  test("Button, Combobox chip and option, Calendar day, Select trigger compute the arrow", async ({ page }) => {
    await open(page, "/component/button/");
    expect(await cursorOf(demoRole(page, "button", 0))).toBe("default");

    await open(page, "/component/select/");
    expect(await cursorOf(demoFrame(page).locator(".dx-select-trigger").first())).toBe("default");

    await open(page, "/component/calendar/");
    const day = demoFrame(page).locator('.dx-calendar-grid-cell[data-month="current"]:not([data-disabled="true"])').first();
    expect(await cursorOf(day)).toBe("default");

    await open(page, "/component/combobox/");
    expect(await cursorOf(demoFrame(page).locator(".dx-combobox-chip-remove").first())).toBe("default");
    await demoRole(page, "combobox", 0).click();
    const option = popupItem(page, "listbox", "option", 0);
    await expect(option).toBeVisible();
    expect(await cursorOf(option)).toBe("default");
  });
});

test.describe('data-dx-interactive-element-cursor="pointer": the hand on every one of them', () => {
  test("Button, Combobox chip and option, Calendar day, Select trigger compute pointer", async ({ page }) => {
    await open(page, "/component/button/");
    await setOption(page, "pointer");
    expect(await cursorOf(demoRole(page, "button", 0))).toBe("pointer");

    await open(page, "/component/select/");
    await setOption(page, "pointer");
    expect(await cursorOf(demoFrame(page).locator(".dx-select-trigger").first())).toBe("pointer");

    await open(page, "/component/calendar/");
    await setOption(page, "pointer");
    const day = demoFrame(page).locator('.dx-calendar-grid-cell[data-month="current"]:not([data-disabled="true"])').first();
    expect(await cursorOf(day)).toBe("pointer");

    await open(page, "/component/combobox/");
    await setOption(page, "pointer");
    expect(await cursorOf(demoFrame(page).locator(".dx-combobox-chip-remove").first())).toBe("pointer");
    await demoRole(page, "combobox", 0).click();
    const option = popupItem(page, "listbox", "option", 0);
    await expect(option).toBeVisible();
    expect(await cursorOf(option)).toBe("pointer");
  });

  // One pressable of each kind, so a sheet that grows its own declaration (or a list the rule lost an entry
  // from) shows up as a named row. [route, selector inside the main demo frame].
  const SWEEP: [string, string][] = [
    ["switch", ".dx-switch"],
    ["checkbox", ".dx-checkbox"],
    ["radio_group", ".dx-radio-group-item"],
    ["tabs", '.dx-tabs-trigger:not([data-disabled="true"])'],
    ["toggle", ".dx-toggle"],
    ["toggle_group", ".dx-toggle-group-item"],
    ["slider", ".dx-slider-thumb"],
    ["accordion", ".dx-accordion-trigger"],
    ["collapsible", ".dx-collapsible-trigger"],
    ["dropdown_menu", ".dx-dropdown-menu-trigger"],
    ["menubar", ".dx-menubar-trigger"],
    ["native_select", ".dx-native-select"],
    ["date_picker", ".dx-date-picker-popover-trigger"],
    ["toolbar", ".dx-toolbar-button"],
    ["carousel", ".dx-carousel-previous"],
  ];
  for (const [route, selector] of SWEEP) {
    test(`${route}: ${selector} follows the option`, async ({ page }) => {
      await open(page, `/component/${route}/`);
      const el = demoFrame(page).locator(selector).first();
      await expect(el).toBeAttached();
      expect(await cursorOf(el), "default").not.toBe("pointer");
      await setOption(page, "pointer");
      expect(await cursorOf(el), "pointer").toBe("pointer");
      await setOption(page, null);
      expect(await cursorOf(el), "cleared").not.toBe("pointer");
    });
  }

  test("an interactive element inside a nested container can opt back out", async ({ page }) => {
    await open(page, "/component/button/");
    await setOption(page, "pointer");
    const cursor = await page.evaluate(() => {
      const host = document.createElement("div");
      host.setAttribute("data-dx-interactive-element-cursor", "default");
      host.innerHTML = '<button class="dx-button" id="nested-cursor-probe">x</button>';
      document.body.append(host);
      return getComputedStyle(host.querySelector("button")!).cursor;
    });
    expect(cursor).toBe("default");
  });
});

test.describe("what the option never changes", () => {
  for (const mode of ["default", "pointer"] as const) {
    test(`${mode}: a text field keeps the text cursor`, async ({ page }) => {
      await open(page, "/component/input/");
      if (mode === "pointer") await setOption(page, "pointer");
      expect(await cursorOf(demoFrame(page).locator(".dx-input").first())).toBe("text");
    });

    test(`${mode}: the combobox's input keeps the text cursor`, async ({ page }) => {
      await open(page, "/component/combobox/");
      if (mode === "pointer") await setOption(page, "pointer");
      expect(await cursorOf(demoFrame(page).locator(".dx-combobox-input").first())).toBe("text");
    });

    test(`${mode}: a disabled button never shows the hand (its recipe is pointer-events: none)`, async ({ page }) => {
      await open(page, "/component/button/");
      if (mode === "pointer") await setOption(page, "pointer");
      const button = demoRole(page, "button", 0);
      await button.evaluate((el) => ((el as HTMLButtonElement).disabled = true));
      const style = await button.evaluate((el) => {
        const s = getComputedStyle(el);
        return { cursor: s.cursor, events: s.pointerEvents };
      });
      expect(style.events).toBe("none");
      expect(style.cursor).not.toBe("pointer");
    });

    test(`${mode}: a disabled tab keeps its not-allowed cursor`, async ({ page }) => {
      await open(page, "/component/tabs/");
      if (mode === "pointer") await setOption(page, "pointer");
      const tab = demoFrame(page).locator('.dx-tabs-trigger[data-disabled="true"]').first();
      await expect(tab).toBeAttached();
      expect(await cursorOf(tab)).toBe("not-allowed");
    });

    test(`${mode}: a drag handle keeps its grab cursor`, async ({ page }) => {
      await open(page, "/component/drag_and_drop_list/");
      if (mode === "pointer") await setOption(page, "pointer");
      expect(await cursorOf(demoFrame(page).locator(".dx-drag-and-drop-list-item").first())).toBe("grab");
    });

    test(`${mode}: a resize handle keeps its resize cursor`, async ({ page }) => {
      await open(page, "/component/resizable/");
      if (mode === "pointer") await setOption(page, "pointer");
      const handle = demoFrame(page).locator('.dx-resizable-handle').first();
      await expect(handle).toBeAttached();
      expect(await cursorOf(handle)).toMatch(/^(col|row)-resize$/);
    });
  }
});

test("window.__dxTheme('dx-interactive-element-cursor', v) accepts only pointer and default", async ({ page }) => {
  await open(page, "/docs");
  const seen = await page.evaluate(
    ([k, a]) => {
      const w = window as unknown as { __dxTheme: (k: string, v: string | null) => void };
      const html = document.documentElement;
      const out: (string | null)[] = [];
      for (const v of ["pointer", "bogus", "default", "", null, "Pointer", "pointer "]) {
        w.__dxTheme(k, v);
        out.push(html.getAttribute(a));
      }
      return out;
    },
    [KEY, ATTR] as const,
  );
  expect(seen).toEqual(["pointer", null, "default", null, null, null, null]);
});

test("the cookie seeds the attribute before first paint", async ({ page, context }) => {
  await context.addCookies([{ name: `dx_${KEY.replace("-", "_")}`, value: "pointer", url: BASE_URL }]);
  await gotoHydrated(page, `${BASE_URL}/component/button/`);
  expect(await page.evaluate((a) => document.documentElement.getAttribute(a), ATTR)).toBe("pointer");
  expect(await cursorOf(demoRole(page, "button", 0))).toBe("pointer");
});

test("the theme picker's Cursor control writes the attribute, persists through a reload, and Reset clears it", async ({ page }) => {
  await open(page, "/docs");
  const html = page.locator("html");
  await expect(html).not.toHaveAttribute(ATTR, /.+/);

  await page.getByRole("button", { name: "Theme", exact: true }).click();
  const picker = page.locator(".dx-theme-picker");
  const group = picker.locator('[aria-labelledby^="dx-theme-cursor-label"]');
  const chip = (label: string) => group.locator("button", { hasText: new RegExp(`^${label}$`) });
  // "Default" (no attribute) is the pressed one to begin with.
  await expect(chip("Default")).toHaveAttribute("aria-pressed", "true");

  await chip("Pointer").click();
  await expect(html).toHaveAttribute(ATTR, "pointer");
  expect((await page.context().cookies()).some((c) => c.name === `dx_${KEY.replace("-", "_")}` && c.value === "pointer")).toBe(true);
  // The picker's own chips are interactive dx- elements: they follow the option they just set.
  expect(await cursorOf(chip("Default"))).toBe("pointer");

  // Written by the pre-paint snippet from the cookie: present at DOMContentLoaded, before hydration.
  await page.reload({ waitUntil: "domcontentloaded" });
  expect(await page.evaluate((a) => document.documentElement.getAttribute(a), ATTR)).toBe("pointer");
  await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });

  await page.getByRole("button", { name: "Theme", exact: true }).click();
  await expect(chip("Pointer")).toHaveAttribute("aria-pressed", "true");
  await chip("Default").click();
  await expect(html).not.toHaveAttribute(ATTR, /.+/);
  expect(await cursorOf(chip("Default"))).toBe("default");

  await chip("Pointer").click();
  await expect(html).toHaveAttribute(ATTR, "pointer");
  await picker.getByRole("button", { name: "Reset" }).click();
  await expect(html).not.toHaveAttribute(ATTR, /.+/);
  await expect(chip("Default")).toHaveAttribute("aria-pressed", "true");
});

test("the docs page documents the option", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/docs`);
  const heading = page.locator("h3#interactive-element-cursor");
  await expect(heading).toBeVisible();
  await expect(page.locator("code", { hasText: `${ATTR}="pointer"` }).first()).toBeVisible();
});
