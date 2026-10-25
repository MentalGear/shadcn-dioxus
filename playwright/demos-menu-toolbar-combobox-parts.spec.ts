/**
 * Rows 398 (part B) and 188: the placement, orientation and combobox parts the docs review and the
 * parity audit found missing.
 *
 * - DropdownMenu `side` / `align` / `side_offset` (`dropdown_menu/placement`): each menu sits on its
 *   side of the trigger, aligned and gapped as asked, and publishes the side as `data-placed-side`.
 * - Toolbar `horizontal: false` (`toolbar/vertical`): a column of full-width buttons, ArrowDown/ArrowUp
 *   roving focus, a horizontal separator.
 * - Single Combobox `name` / `item_to_value` / `show_clear` (`combobox/clear_form`): the form entry,
 *   the clear button, the reset.
 * - `ComboboxGroup` (`combobox/groups`), `auto_highlight` (`combobox/auto_highlight`), `invalid`
 *   (`combobox/invalid`) and `addon` (`combobox/input_group`).
 *
 * Axe runs on every demo in light and dark.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const frame = (page: Page, variant: string) => page.locator(`#component-preview-frame-${variant}`).first();

async function open(page: Page, name: string, variant: string, mode: "light" | "dark") {
  await gotoHydrated(page, `${BASE_URL}/component/${name}/`, GOTO_OPTS);
  await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
  const root = frame(page, variant);
  // Centred, so a menu has room on every side and never flips.
  await root.evaluate((e) => e.scrollIntoView({ block: "center" }));
  return root;
}

const rect = (el: Locator) =>
  el.evaluate((e) => {
    const r = e.getBoundingClientRect();
    return { top: r.top, right: r.right, bottom: r.bottom, left: r.left, width: r.width, height: r.height };
  });
const openList = (page: Page) => page.locator("[role='listbox'][data-state='open']");
const formEntries = (root: Locator, name: string) =>
  root.locator("form").evaluate((form, n) => new FormData(form as HTMLFormElement).getAll(n), name);

for (const mode of ["light", "dark"] as const) {
  test.describe(`menu placement, vertical toolbar and combobox parts, ${mode}`, () => {
    test.use({ colorScheme: mode });

    test(`dropdown_menu/placement: side, align and side_offset place the menu`, async ({ page }) => {
      const root = await open(page, "dropdown_menu", "placement", mode);
      const cases = [
        { name: "Opens above", side: "top" },
        { name: "Opens on the right", side: "right" },
        { name: "Opens below, end-aligned", side: "bottom" },
        { name: "Opens on the left", side: "left" },
      ];
      for (const { name, side } of cases) {
        const trigger = root.getByRole("button", { name });
        await trigger.click();
        const menu = page.locator("[role='menu'][data-state='open']");
        await expect(menu).toBeVisible();
        await expect(menu).toHaveAttribute("data-placed-side", side);
        // The pop-in scales the panel: measure once it has settled.
        await page.waitForTimeout(300);
        const t = await rect(trigger);
        const m = await rect(menu);
        if (side === "top") {
          expect(Math.abs(t.top - 4 - m.bottom)).toBeLessThanOrEqual(1.5);
          expect(Math.abs(m.left - t.left)).toBeLessThanOrEqual(1.5);
        } else if (side === "right") {
          expect(Math.abs(t.right + 4 - m.left)).toBeLessThanOrEqual(1.5);
          expect(Math.abs(m.top - t.top)).toBeLessThanOrEqual(1.5);
        } else if (side === "bottom") {
          expect(Math.abs(t.bottom + 12 - m.top)).toBeLessThanOrEqual(1.5);
          expect(Math.abs(m.right - t.right)).toBeLessThanOrEqual(1.5);
        } else {
          expect(Math.abs(t.left - 4 - m.right)).toBeLessThanOrEqual(1.5);
          expect(Math.abs(m.top + m.height / 2 - (t.top + t.height / 2))).toBeLessThanOrEqual(1.5);
        }
        await expectNoAxeViolations(page, `dropdown_menu/placement ${name} ${mode}`);
        await page.keyboard.press("Escape");
        await expect(menu).toHaveCount(0);
      }
    });

    test(`toolbar/vertical: a column with up/down roving focus and a horizontal separator`, async ({ page }) => {
      const root = await open(page, "toolbar", "vertical", mode);
      const toolbar = root.getByRole("toolbar", { name: "Clipboard" });
      await expect(toolbar).toHaveAttribute("aria-orientation", "vertical");
      const cut = await rect(root.getByRole("button", { name: "Cut" }));
      const copy = await rect(root.getByRole("button", { name: "Copy" }));
      const redo = await rect(root.getByRole("button", { name: "Redo" }));
      expect(copy.top).toBeGreaterThanOrEqual(cut.bottom - 0.5);
      expect(Math.abs(copy.left - cut.left)).toBeLessThanOrEqual(0.5);
      expect(Math.abs(redo.width - cut.width)).toBeLessThanOrEqual(0.5);
      const separator = await rect(toolbar.locator(".dx-toolbar-separator"));
      expect(separator.height).toBeLessThanOrEqual(1.5);
      expect(separator.width).toBeGreaterThan(cut.width / 2);

      await root.getByRole("button", { name: "Cut" }).focus();
      await page.keyboard.press("ArrowDown");
      await expect(root.getByRole("button", { name: "Copy" })).toBeFocused();
      await page.keyboard.press("ArrowUp");
      await expect(root.getByRole("button", { name: "Cut" })).toBeFocused();
      await expectNoAxeViolations(page, `toolbar/vertical ${mode}`);
    });

    test(`combobox/clear_form: name submits the value, the clear button and the reset`, async ({ page }) => {
      const root = await open(page, "combobox", "clear_form", mode);
      const input = root.getByRole("combobox", { name: "Form framework" });
      await expect(input).toHaveValue("Astro");
      expect(await formEntries(root, "framework")).toEqual(["astro"]);

      await root.getByRole("button", { name: "Clear" }).click();
      await expect(input).toHaveValue("");
      await expect(input).toBeFocused();
      await expect(root.getByRole("button", { name: "Clear" })).toHaveCount(0);
      expect(await formEntries(root, "framework")).toEqual([]);

      await input.press("Escape");
      await input.click();
      await openList(page).getByRole("option", { name: "Remix" }).click();
      await expect(input).toHaveValue("Remix");
      expect(await formEntries(root, "framework")).toEqual(["remix"]);

      await root.getByRole("button", { name: "Reset" }).click();
      await expect(input).toHaveValue("Astro");
      expect(await formEntries(root, "framework")).toEqual(["astro"]);
      await expectNoAxeViolations(page, `combobox/clear_form ${mode}`);
    });

    test(`combobox/groups: labelled groups, one key order, empty groups hide`, async ({ page }) => {
      const root = await open(page, "combobox", "groups", mode);
      const input = root.getByRole("combobox", { name: "Grouped food" });
      await input.click();
      const list = openList(page);
      await expect(list.getByRole("group")).toHaveCount(3);
      await expect(list.getByRole("group", { name: "Fruits" }).getByRole("option")).toHaveCount(4);
      await expectNoAxeViolations(page, `combobox/groups open ${mode}`);

      // ArrowDown from the last fruit lands on the first vegetable: the groups share one order.
      for (let i = 0; i < 5; i++) await input.press("ArrowDown");
      await expect(list.getByRole("option", { name: "Broccoli" })).toHaveAttribute("data-highlighted", "true");

      await input.fill("car");
      await expect(list.getByRole("group")).toHaveCount(1);
      await expect(list.getByRole("group", { name: "Vegetables" })).toBeVisible();
      await expect(list.getByText("Fruits")).toBeHidden();
    });

    test(`combobox/auto_highlight: typing highlights the first match, Enter picks it`, async ({ page }) => {
      const root = await open(page, "combobox", "auto_highlight", mode);
      const input = root.getByRole("combobox", { name: "Auto-highlight language" });
      await input.click();
      await input.pressSequentially("ru");
      const rust = openList(page).getByRole("option", { name: "Rust" });
      await expect(rust).toHaveAttribute("data-highlighted", "true");
      await expect(input).toHaveAttribute("aria-activedescendant", (await rust.getAttribute("id"))!);
      await input.pressSequentially("b");
      await expect(openList(page).getByRole("option", { name: "Ruby" })).toHaveAttribute("data-highlighted", "true");
      await input.press("Enter");
      await expect(input).toHaveValue("Ruby");
      await expectNoAxeViolations(page, `combobox/auto_highlight ${mode}`);
    });

    test(`combobox/invalid: the input is aria-invalid until a value is picked`, async ({ page }) => {
      const root = await open(page, "combobox", "invalid", mode);
      const input = root.getByRole("combobox", { name: "Country" });
      await expect(input).toHaveAttribute("aria-invalid", "true");
      const ring = await input.evaluate((e) => getComputedStyle(e).boxShadow);
      await expect(root.getByText("Pick a country to continue.")).toBeVisible();
      await expectNoAxeViolations(page, `combobox/invalid ${mode}`);

      await input.click();
      await openList(page).getByRole("option", { name: "Japan" }).click();
      await expect(input).not.toHaveAttribute("aria-invalid", "true");
      await expect(root.getByText("Pick a country to continue.")).toHaveCount(0);
      expect(await input.evaluate((e) => getComputedStyle(e).boxShadow)).not.toBe(ring);
    });

    test(`combobox/input_group: the addon icon sits inside the input, before its text`, async ({ page }) => {
      const root = await open(page, "combobox", "input_group", mode);
      const input = root.getByRole("combobox", { name: "Time zone" });
      const icon = root.locator(".dx-combobox-addon svg");
      const i = await rect(input);
      const a = await rect(icon);
      expect(a.width).toBeCloseTo(16, 0);
      expect(a.left).toBeGreaterThan(i.left);
      expect(a.top).toBeGreaterThan(i.top);
      expect(a.bottom).toBeLessThan(i.bottom);
      const padStart = await input.evaluate((e) => parseFloat(getComputedStyle(e).paddingInlineStart));
      expect(i.left + padStart).toBeGreaterThan(a.right);
      // The addon never takes the click: the input opens.
      await icon.click({ force: true });
      await expect(openList(page)).toBeVisible();
      await expectNoAxeViolations(page, `combobox/input_group ${mode}`);
    });
  });
}
