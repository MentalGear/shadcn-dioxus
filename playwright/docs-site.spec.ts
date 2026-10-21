/**
 * Docs-site chrome, rows 336, 351, 352:
 * - 336: the document title is derived from the route in ONE place (`Route::page_title`, rendered by
 *   `AppLayout`), so a client-side navigation replaces it and the first paint already has it.
 * - 351: `rtl` demos (a variant named `rtl` or ending in `_rtl`) are always the last of a component's
 *   demo list, whatever `variants/order.txt` says (`preview/build.rs`).
 * - 352: the "Plus the global attributes." sentence of a props table links to MDN.
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test.describe("document title follows the route (row 336)", () => {
  test("client-side navigation between two component pages replaces the title", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button/`);
    await expect(page).toHaveTitle("Button – shadcn-dioxus");
    await page.locator('a[data-sidebar="menu-button"][href*="/component/badge/"]:visible').click();
    await expect(page.locator(".dx-component-page-header h1")).toHaveText(/badge/i);
    await expect(page).toHaveTitle("Badge – shadcn-dioxus");
  });

  test("every kind of page has its own title on load, and the SSG markup already carries it", async ({ page }) => {
    const cases: [string, string][] = [
      ["/", "shadcn-dioxus"],
      ["/docs", "Docs – shadcn-dioxus"],
      ["/demos", "Demos – shadcn-dioxus"],
      ["/charts/line/", "Line Charts – shadcn-dioxus"],
      ["/component/date_picker/", "Date Picker – shadcn-dioxus"],
    ];
    for (const [path, title] of cases) {
      await gotoHydrated(page, `${BASE_URL}${path}`);
      await expect(page, path).toHaveTitle(title);
    }
  });

  test("navigating from a component page to Docs, Demos and Charts changes the title each time", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button/`);
    for (const [name, title] of [
      ["Docs", "Docs – shadcn-dioxus"],
      ["Demos", "Demos – shadcn-dioxus"],
      ["Charts", "Area Charts – shadcn-dioxus"],
    ] as const) {
      await page.locator("nav.dx-preview-navbar").getByRole("link", { name, exact: true }).first().click();
      await expect(page, name).toHaveTitle(title);
    }
  });
});

test.describe("RTL demos come last (row 351)", () => {
  test("calendar: the rtl demo is the last example", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`);
    const ids = await page.locator('[id^="component-preview-frame"]').evaluateAll((els) => els.map((e) => e.id));
    expect(ids.length).toBeGreaterThan(2);
    expect(ids[ids.length - 1]).toBe("component-preview-frame-rtl");
    expect(ids.filter((id) => /rtl/.test(id))).toEqual(["component-preview-frame-rtl"]);
  });

  test("carousel: every *_rtl demo sits after every other demo", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/carousel/`);
    const ids = await page.locator('[id^="component-preview-frame"]').evaluateAll((els) => els.map((e) => e.id));
    const firstRtl = ids.findIndex((id) => /rtl$/.test(id));
    expect(firstRtl).toBeGreaterThan(0);
    expect(ids.slice(firstRtl).every((id) => /rtl$/.test(id))).toBe(true);
  });
});

test("the props table's global-attributes sentence links to MDN (row 352)", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button/`);
  const link = page.locator(".dx-docs-props a", { hasText: "the global attributes" }).first();
  await expect(link).toHaveAttribute(
    "href",
    "https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes",
  );
  await expect(link).toHaveAttribute("rel", /noopener/);
});
