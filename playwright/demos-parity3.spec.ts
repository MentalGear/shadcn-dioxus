/**
 * Row 188 demo parity: the parts the parity audit lists that no other spec covers.
 *
 * - `Field { disabled }`: Nova's `group-data-[disabled=true]/field:opacity-50` on `FieldLabel`, read through
 *   Label's `--dx-label-opacity` slot (the label also wears `dx-label`, so Field declares no property on it).
 * - `InputGroupButton`: Nova's `.cn-input-group-button*` metrics (xs `h-6 px-1.5 text-sm`, icon `size-3.5`,
 *   `rounded-[calc(var(--radius)-3px)]`; icon-xs `size-6`; icon-sm `size-8`), ghost and `type="button"` by default.
 * - A Spinner inside a Badge is the Badge's 12px icon (`.cn-badge [&>svg]:size-3!`), in every stylesheet order:
 *   the Badge sizes only its direct icons and reaches the spinner through `--dx-spinner-size`, so there is no
 *   equal-specificity `svg` rule left whose winner depends on which sheet loaded last.
 *
 * Controls are located by role inside the demo frame; geometry is read at rest; axe runs in light and dark.
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
  await root.scrollIntoViewIfNeeded();
  return root;
}

const css = (el: Locator, prop: string) => el.evaluate((e, p) => getComputedStyle(e).getPropertyValue(p), prop);
const size = (el: Locator) => el.evaluate((e) => [e.getBoundingClientRect().width, e.getBoundingClientRect().height]);
// The used width/height, not the bounding box: a spinning ring's box grows with its rotation.
const used = (el: Locator) =>
  el.evaluate((e) => [parseFloat(getComputedStyle(e).width), parseFloat(getComputedStyle(e).height)]);

for (const mode of ["light", "dark"] as const) {
  test.describe(`field, input group button and badge spinner demos, ${mode}`, () => {
    test.use({ colorScheme: mode });

    test(`field/disabled: a disabled field dims its labels, not its description`, async ({ page }) => {
      const root = await open(page, "field", "disabled", mode);
      const fields = root.locator(".dx-field[data-disabled='true']");
      expect(await fields.count()).toBeGreaterThanOrEqual(2);
      const labels = fields.locator(".dx-field-label");
      expect(await labels.count()).toBeGreaterThanOrEqual(2);
      for (const label of await labels.all()) expect(await css(label, "opacity")).toBe("0.5");
      // shadcn dims only the label; the description keeps its ink.
      expect(await css(fields.first().locator(".dx-field-description"), "opacity")).toBe("1");
      // The controls are disabled themselves (the Field only styles the row).
      await expect(root.getByRole("textbox").first()).toBeDisabled();
      await expect(root.getByRole("switch").first()).toBeDisabled();
      // An enabled field next to them is not dimmed.
      const enabled = root.locator(".dx-field:not([data-disabled='true']) > .dx-field-label");
      expect(await css(enabled.first(), "opacity")).toBe("1");
      await expectNoAxeViolations(page, `field/disabled ${mode}`);
    });

    test(`input_group/button: InputGroupButton takes Nova's addon metrics`, async ({ page }) => {
      const root = await open(page, "input_group", "button", mode);
      const buttons = root.locator(".dx-input-group-addon .dx-input-group-button");
      expect(await buttons.count()).toBe(4);
      for (const b of await buttons.all()) await expect(b).toHaveAttribute("type", "button");

      // icon-xs: 24x24, radius = --radius - 3px (10px - 3px in the default theme), ghost (no fill at rest).
      const copy = root.getByRole("button", { name: /Copy link|Link copied/ });
      expect(await size(copy)).toEqual([24, 24]);
      expect(await css(copy, "border-top-left-radius")).toBe("7px");
      expect(await css(copy, "background-color")).toBe("rgba(0, 0, 0, 0)");

      // xs (the default): 24px tall, 6px inline padding, text-sm, 14px icon, 7px radius.
      const clear = root.getByRole("button", { name: "Clear" });
      expect((await size(clear))[1]).toBe(24);
      expect(await css(clear, "padding-left")).toBe("6px");
      expect(await css(clear, "font-size")).toBe("14px");
      expect(await css(clear, "border-top-left-radius")).toBe("7px");
      expect(await used(clear.locator("svg"))).toEqual([14, 14]);

      // sm: the Button's 28px row with the group's text-sm and gap-2.
      const search = root.getByRole("button", { name: "Search" });
      expect((await size(search))[1]).toBe(28);
      expect(await css(search, "font-size")).toBe("14px");

      // icon-sm: 32x32.
      const reveal = root.getByRole("button", { name: /Show password|Hide password/ });
      expect(await size(reveal)).toEqual([32, 32]);

      // Still a working button: the copy button confirms.
      await root.getByRole("button", { name: "Copy link" }).click();
      await expect(root.getByRole("button", { name: "Link copied" })).toBeVisible();
      await expectNoAxeViolations(page, `input_group/button ${mode}`);
    });

    // The site ships one CSS bundle in a fixed order, but an installed app loads each component's sheet
    // through its own `document::Link`, in render order: a Badge links its sheet before the Spinner inside it.
    // Re-append one component's rules at the end of the document to reproduce each order on the site.
    for (const last of ["dx-spinner", "dx-badge"] as const) {
      test(`badge/spinner with the ${last} rules loaded last: a Spinner in a Badge is 12px`, async ({ page }) => {
        const root = await open(page, "badge", "spinner", mode);
        await page.evaluate((prefix) => {
          const text: string[] = [];
          for (const sheet of Array.from(document.styleSheets)) {
            let rules: CSSRuleList;
            try {
              rules = sheet.cssRules;
            } catch {
              continue;
            }
            for (const rule of Array.from(rules))
              if (rule instanceof CSSStyleRule && rule.selectorText.includes(`.${prefix}`)) text.push(rule.cssText);
          }
          const style = document.createElement("style");
          style.textContent = text.join("\n");
          document.head.append(style);
        }, last);
        expect(await used(root.locator(".dx-badge .dx-spinner svg").first())).toEqual([12, 12]);
        await expectNoAxeViolations(page, `badge/spinner ${mode}`);
      });
    }

    test(`badge/main and spinner/size: the badge's own icon and a bare Spinner keep their sizes`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/badge/`, GOTO_OPTS);
      expect(await used(page.locator("#component-preview-frame .dx-badge > svg").first())).toEqual([12, 12]);
      const root = await open(page, "spinner", "size", mode);
      const rings = root.locator(".dx-spinner svg");
      expect(await Promise.all([0, 1, 2, 3].map((i) => used(rings.nth(i))))).toEqual([
        [12, 12],
        [16, 16],
        [24, 24],
        [32, 32],
      ]);
    });
  });
}
