import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test.describe("homepage", () => {
  test("should not have any automatically detectable accessibility issues", async ({
    page,
  }) => {
    await gotoHydrated(page, `${BASE_URL}/`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes

    // Wait for the page to fully load
    let heroSection = page.locator("#hero");
    await heroSection.waitFor({ state: "visible" });

    // No color-contrast exclusion needed here: this round's theme-token fix
    // (dev-docs/backlog.md row 39) resolved it, and this route has no
    // `.dx-code-block` to need the one remaining, narrower exclusion for.
    await expectNoAxeViolations(page, "homepage");
  });
});


test.describe("details", () => {
  test("should not have any automatically detectable accessibility issues", async ({
    page,
  }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes

    // Wait for the page to fully load
    let componentSection = page.getByRole("heading", { name: "calendar" });
    await componentSection.waitFor({ state: "visible" });

    await expectNoAxeViolations(page, "component/calendar", {
      excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT],
    });
  });
});

test.describe("style tab", () => {
  test("shows the component's real, current CSS via the debug-build lazy fetch", async ({
    page,
  }) => {
    // Regression guard for the `css_highlight!` construction
    // (preview/src/components/mod.rs, preview/src/main.rs -- dev-docs/
    // dev-loop.md's CSS section, backlog rows 72/76): in debug builds the
    // Style tab no longer compile-time-embeds `style.css` via
    // `dioxus_code::code!()` (that macro's `include_str!` forced a full
    // rebuild on every edit); it fetches the file's own `asset!()` URL at
    // runtime instead. This asserts that fetch actually lands real CSS
    // text, not a blank/loading/error placeholder.
    await gotoHydrated(page, `${BASE_URL}/component/kbd/`);

    await page.getByRole("heading", { name: "kbd" }).waitFor({ state: "visible" });
    // The sections are the library's Accordion/Collapsible: their triggers work once hydrated.
    await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });
    // Installation and "Manual installation files" are both closed by default (rows 233, 236): the
    // reader opens Installation, then the manual files.
    const manual = page.locator(".dx-component-manual-install");
    await expect(manual).toHaveAttribute("data-open", "false");
    await page.locator("h2#installation > button").click();
    await expect(manual).toHaveAttribute("data-open", "false");
    await manual.locator("> .dx-collapsible-trigger").click();
    await expect(manual).toHaveAttribute("data-open", "true");
    await page.getByRole("tab", { name: "style.css" }).click();

    const styleCode = page.locator("pre code", { hasText: ".dx-kbd" });
    await expect(styleCode).toBeVisible({ timeout: 10_000 });
    await expect(styleCode).toContainText(".dx-kbd");
  });
});
