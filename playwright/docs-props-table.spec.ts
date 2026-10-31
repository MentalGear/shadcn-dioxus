import { test, expect } from "./fixtures";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { catalogNames } from "./catalog";
import { gotoHydrated } from "./hydration";

/**
 * Row 165: every component docs page ends with a "Props reference" accordion item that `preview/build.rs`
 * generates from the `#[derive(Props)]` structs (`preview/props_scan.rs`), and `scripts/check-props-docs.sh`
 * keeps every field documented. This spec is the browser half: the section exists on every component
 * page, a known prop shows its type, default and doc, and the opened section passes axe.
 */
const NAV = { waitUntil: "load" as const, timeout: 20 * 60 * 1000 };

test("every component page carries the Props reference section", async ({ request }) => {
  const names = catalogNames();
  expect(names.length).toBeGreaterThan(50);
  const missing: string[] = [];
  for (const name of names) {
    const response = await request.get(`${BASE_URL}/component/${name}/`);
    expect(response.ok(), `${name}: HTTP ${response.status()}`).toBe(true);
    const html = await response.text();
    if (!html.includes('id="docs-props-reference"')) {
      missing.push(name);
    }
  }
  expect(missing, "component pages without a props table").toEqual([]);
});

test("a known prop shows its display type, default and doc comment", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/switch/`, NAV);
  const section = page.locator(".dx-docs-accordion > .dx-accordion-item", { has: page.locator("#docs-props-reference") });
  await expect(section.locator("> h3")).toHaveText("Props reference");
  await section.locator("> h3 > button").click();
  await expect(section).toHaveAttribute("data-open", "true");

  // The styled wrapper's table, then the primitive's: both list the same prop.
  // The library's Table classes, so the docs table looks like the Table component.
  const table = section.locator('table[class~="dx-docs-props-table"][class~="dx-table"]').first();
  await expect(table.locator("thead.dx-table-header th.dx-table-head").first()).toHaveText("Prop");
  const row = table.locator('tr[data-prop="default_checked"]');
  await expect(row.locator("td").nth(1)).toHaveText("bool"); // type
  await expect(row.locator("td").nth(2)).toHaveText("false"); // `#[props(default = false)]`
  await expect(row.locator("td").nth(3)).toContainText("The default checked state when uncontrolled.");

  // `ReadSignal<Option<bool>>` shows the inner type.
  const checked = table.locator('tr[data-prop="checked"]');
  await expect(checked.locator("td").nth(1)).toHaveText("bool");
  await expect(checked.locator("td").nth(2)).toHaveText("None");
  // `extends = GlobalAttributes` is a sentence, not a row.
  await expect(table.locator('tr[data-prop="attributes"]')).toHaveCount(0);
  await expect(section.getByText("Plus the global attributes.").first()).toBeVisible();
  // The wrapper's own prop is documented too.
  await expect(table.locator('tr[data-prop="size"] td').nth(3)).toContainText("Sm");
});

test("the opened Props reference passes axe on a derive-struct and fn-argument components", async ({ page }) => {
  for (const name of ["select", "badge", "button"]) {
    await gotoHydrated(page, `${BASE_URL}/component/${name}/`, NAV);
    const section = page.locator(".dx-docs-accordion > .dx-accordion-item", { has: page.locator("#docs-props-reference") });
    await section.locator("> h3 > button").click();
    await expect(section.locator("table").first()).toBeVisible();
    await expectNoAxeViolations(page, `${name}: props reference open`, {
      include: ".dx-docs-accordion > .dx-accordion-item:has(#docs-props-reference)",
    });
  }
});
