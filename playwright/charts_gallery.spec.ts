import { test, expect } from "./fixtures";
import { type Locator } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { fixedViewportBox } from "./viewport-box";
import fs from "node:fs";
import path from "node:path";

// The `/charts/` gallery (`preview/src/charts_gallery.rs`): one tab per chart
// type, every chart demo as a card with a Copy button and a "View Code"
// sheet/drawer. Each tab is its own prerendered route (`/charts/<slug>/`,
// `/charts/` = the Area tab), so this runs against the SSG lane (or `dx
// serve`); start the server first and point PLAYWRIGHT_BASE_URL at it.

// The page keeps a classic-scrollbar gutter while a modal is open
// (`scrollbar-gutter: stable`, main.css), which a fixed dialog's right edge
// stops short of by the scrollbar width (15px in headless Chromium).
const SCROLLBAR_SLACK = 16;

const COMPONENTS = path.join(__dirname, "../preview/src/components");

// What Copy / View Code hand out for a variant: the variant's own file, with
// its repo-relative `use super::super::component::*;` rewritten to where
// `dx components add <demo>` puts the component in the user's app
// (`preview/src/installed_source.rs`).
function installedSource(demo: string, variant: string): string {
  return fs
    .readFileSync(path.join(COMPONENTS, `${demo}/variants/${variant}/mod.rs`), "utf8")
    .replaceAll("super::super::component", `crate::components::${demo}`);
}

// Notes that belong in the repo's research logs, not in code a user copies.
const INTERNAL_NOTES = /\$S\/|refs\/ui\/|dev-docs|backlog\.md|stage-?[0-9]|clone commit/;

// Minimum card counts: shadcn's own per-tab counts, which every tab of ours
// has at least (the demos behind the cards can only grow).
const TABS = [
  { slug: "area", label: "Area Charts", demo: "area_chart", min: 10, hero: true },
  { slug: "bar", label: "Bar Charts", demo: "bar_chart", min: 11, hero: true },
  { slug: "line", label: "Line Charts", demo: "line_chart", min: 10, hero: true },
  { slug: "pie", label: "Pie Charts", demo: "pie_chart", min: 11, hero: false },
  { slug: "radar", label: "Radar Charts", demo: "radar_chart", min: 14, hero: false },
  { slug: "radial", label: "Radial Charts", demo: "radial_chart", min: 6, hero: false },
  { slug: "tooltip", label: "Tooltips", demo: "chart_tooltip", min: 9, hero: false },
] as const;

for (const tab of TABS) {
  test(`/charts/${tab.slug}/ renders its cards with the toolbar chrome`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/${tab.slug}/`);

    await expect(page.getByRole("heading", { level: 1 })).toHaveText(/Charts/);
    await expect(page.getByRole("heading", { level: 2, name: tab.label })).toBeAttached();

    const cells = page.locator(".dx-charts-cell");
    expect(await cells.count()).toBeGreaterThanOrEqual(tab.min);
    await expect(cells.first().locator('[data-slot="chart"]').first()).toBeVisible();

    // Every card: toolbar label, an icon-only Copy button, a View Code button.
    for (const cell of await cells.all()) {
      await expect(cell.locator(".dx-charts-toolbar-label")).not.toBeEmpty();
      await expect(cell.getByRole("button", { name: /^Copy code/ })).toBeVisible();
      await expect(cell.getByRole("button", { name: /^View Code/ })).toBeVisible();
      await expect(cell.locator(".dx-chart").first()).toBeAttached();
    }

    // The interactive hero card leads and spans the full width on Area/Bar/
    // Line; nowhere else is a card full-width.
    const spans = await cells.evaluateAll((els) => els.map((el) => el.getAttribute("data-span")));
    if (tab.hero) {
      await expect(cells.first()).toHaveAttribute("data-variant", "interactive");
      expect(spans[0]).toBe("full");
      expect(spans.slice(1).every((span) => span === "single")).toBe(true);
    } else {
      expect(spans.every((span) => span === "single")).toBe(true);
    }
  });
}

test("the tab row marks the current chart type with aria-current", async ({ page }) => {
  for (const tab of TABS) {
    await gotoHydrated(page, `${BASE_URL}/charts/${tab.slug}/`);
    const nav = page.getByRole("navigation", { name: "Chart types" });
    const links = nav.getByRole("link");
    await expect(links).toHaveCount(TABS.length);
    await expect(links.nth(TABS.indexOf(tab))).toHaveText(tab.label);
    await expect(nav.locator('[aria-current="page"]')).toHaveCount(1);
    await expect(nav.getByRole("link", { name: tab.label })).toHaveAttribute("aria-current", "page");
    await expect(nav.getByRole("link", { name: tab.label })).toHaveAttribute(
      "href",
      new RegExp(`/charts/${tab.slug}/\\??$`),
    );
  }
});

test("/charts/ is the Area tab, and tab links navigate between tabs", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/charts/`);
  const nav = page.getByRole("navigation", { name: "Chart types" });
  await expect(nav.getByRole("link", { name: "Area Charts" })).toHaveAttribute("aria-current", "page");
  await expect(page.locator('.dx-charts-cell[data-kind="area"]').first()).toBeVisible();

  await nav.getByRole("link", { name: "Pie Charts" }).click();
  await expect(page).toHaveURL(/\/charts\/pie\/\??$/);
  await expect(nav.getByRole("link", { name: "Pie Charts" })).toHaveAttribute("aria-current", "page");
  await expect(nav.getByRole("link", { name: "Area Charts" })).not.toHaveAttribute("aria-current", "page");
  await expect(page.locator('.dx-charts-cell[data-kind="pie"]').first()).toBeVisible();
  await expect(page.locator('.dx-charts-cell[data-kind="area"]')).toHaveCount(0);
});

test("hero buttons, top nav and the chart component pages link to the gallery", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/charts/area/`);
  await expect(page.getByRole("link", { name: "Browse charts" })).toHaveAttribute("href", "#charts");
  await expect(page.getByRole("link", { name: "Documentation" })).toHaveAttribute(
    "href",
    /\/component\/chart\/\??$/,
  );
  await expect(
    page.getByRole("navigation", { name: "Primary" }).getByRole("link", { name: "Charts" }),
  ).toHaveAttribute("href", /\/charts\/\??$/);
  await expect(
    page.getByRole("navigation", { name: "Footer" }).getByRole("link", { name: "Charts" }),
  ).toHaveAttribute("href", /\/charts\/\??$/);

  await gotoHydrated(page, `${BASE_URL}/component/area_chart/`);
  await page.getByRole("link", { name: "Browse all charts" }).click();
  await expect(page).toHaveURL(/\/charts\/\??$/);
});

test("Copy writes the variant's full source to the clipboard, as installed", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await gotoHydrated(page, `${BASE_URL}/charts/area/`);

  const cell = page.locator('.dx-charts-cell[data-variant="gradient"]');
  const copy = cell.getByRole("button", { name: /^Copy code/ });
  await copy.scrollIntoViewIfNeeded();
  await copy.click();

  const expected = installedSource("area_chart", "gradient");
  expect(expected).toContain("use crate::components::area_chart::*;");
  await expect
    .poll(() => page.evaluate(() => navigator.clipboard.readText()))
    .toBe(expected);
  // The check icon confirms the copy, then resets.
  await expect(copy).toHaveAttribute("data-copied", "true");
  await expect(copy).toHaveAttribute("data-copied", "false");
});

test("every card's Copy gives self-contained code: installed import path, no repo-internal notes", async ({
  page,
  context,
}) => {
  test.setTimeout(240_000);
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  for (const tab of TABS) {
    await gotoHydrated(page, `${BASE_URL}/charts/${tab.slug}/`);
    for (const cell of await page.locator(".dx-charts-cell").all()) {
      const variant = (await cell.getAttribute("data-variant"))!;
      const copy = cell.getByRole("button", { name: /^Copy code/ });
      await copy.scrollIntoViewIfNeeded();
      await copy.click();
      const expected = installedSource(tab.demo, variant);
      await expect
        .poll(() => page.evaluate(() => navigator.clipboard.readText()), {
          message: `${tab.demo}/${variant}`,
        })
        .toBe(expected);
      const copied = await page.evaluate(() => navigator.clipboard.readText());
      expect(copied, `${tab.demo}/${variant}`).not.toContain("super::super");
      expect(copied, `${tab.demo}/${variant}`).toMatch(
        new RegExp(`^use crate::components::${tab.demo}::`, "m"),
      );
      expect(copied, `${tab.demo}/${variant}`).not.toMatch(INTERNAL_NOTES);
    }
  }
});

test("View Code shows the installed import path, highlighted", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await gotoHydrated(page, `${BASE_URL}/charts/bar/`);
  const viewCode = page
    .locator('.dx-charts-cell[data-variant="main"]')
    .getByRole("button", { name: /^View Code/ });
  await viewCode.scrollIntoViewIfNeeded();
  await viewCode.click();
  const code = page.getByRole("dialog", { name: "bar_chart/variants/main/mod.rs" }).locator(
    ".dx-preview-code-theme",
  );
  await expect(code).toContainText("use crate::components::bar_chart::*;");
  await expect(code).not.toContainText("super::super");
  // The rewritten import keeps its token colors: the path segments are
  // still separate highlighted spans, not one plain run of text.
  expect(await code.locator(".dxc span").allTextContents()).toEqual(
    expect.arrayContaining(["crate", "components", "bar_chart"]),
  );
});

/** "1\n2\n...\nN": the gutter text a listing of `source` must show. */
function gutterNumbers(source: string): string {
  const lines = source.replace(/\n+$/, "").split("\n").length;
  return Array.from({ length: lines }, (_, i) => String(i + 1)).join("\n");
}

/** How far (px) the top of line `n` (1-based) sits from the top of number `n`, in a listing. */
async function lineOffset(code: Locator, n: number): Promise<number> {
  return code.evaluate((box, n) => {
    const top = (text: Node, line: number) => {
      const data = (text as Text).data;
      let at = 0;
      for (let i = 1; i < line; i++) at = data.indexOf("\n", at) + 1;
      const range = document.createRange();
      range.setStart(text, at);
      range.setEnd(text, at + 1);
      return range.getBoundingClientRect().top;
    };
    const gutter = box.querySelector(".dx-code-gutter")!.firstChild!;
    // The code's first text node holds line 1 only up to its first token; walk the
    // spans in order and find the one that starts line `n`.
    const spans = Array.from(box.querySelectorAll(".dxc code > span, .dxc code > *"));
    let seen = 1;
    for (const span of spans) {
      const text = span.firstChild;
      if (!text) continue;
      const data = (text as Text).data;
      if (n === seen) return top(text, 1) - top(gutter, n);
      const breaks = data.split("\n").length - 1;
      // Line `n` starts in this text node -- unless the node ENDS with the
      // newline before it (e.g. a lone "\n" token): then the line's first
      // character is in the next node, which the next pass picks up.
      if (n <= seen + breaks && !(n === seen + breaks && data.endsWith("\n"))) {
        return top(text, n - seen + 1) - top(gutter, n);
      }
      seen += breaks;
    }
    throw new Error(`line ${n} not found`);
  }, n);
}

test("View Code numbers its lines in a gutter that selecting and copying leave out", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.setViewportSize({ width: 1280, height: 800 });
  await gotoHydrated(page, `${BASE_URL}/charts/bar/`);
  const viewCode = page
    .locator('.dx-charts-cell[data-variant="main"]')
    .getByRole("button", { name: /^View Code/ });
  await viewCode.scrollIntoViewIfNeeded();
  await viewCode.click();
  const dialog = page.getByRole("dialog", { name: "bar_chart/variants/main/mod.rs" });
  const code = dialog.locator(".dx-preview-code-theme");
  const gutter = code.locator(".dx-code-gutter");
  const source = installedSource("bar_chart", "main");

  // One number per line, 1..N, visible, decorative, and not selectable.
  await expect(gutter).toBeVisible();
  await expect(gutter).toHaveAttribute("aria-hidden", "true");
  expect(await gutter.textContent()).toBe(gutterNumbers(source));
  expect(await gutter.evaluate((el) => getComputedStyle(el).userSelect)).toBe("none");

  // Each number is level with its line: first, middle and last.
  const lines = source.replace(/\n+$/, "").split("\n").length;
  for (const n of [1, Math.ceil(lines / 2), lines]) {
    expect(Math.abs(await lineOffset(code, n)), `line ${n}`).toBeLessThanOrEqual(1.5);
  }

  // Selecting the whole listing (and copying it) yields the source, no numbers.
  await code.evaluate((el) => {
    const range = document.createRange();
    range.selectNodeContents(el);
    const selection = getSelection()!;
    selection.removeAllRanges();
    selection.addRange(range);
  });
  const selected = await page.evaluate(() => getSelection()!.toString());
  expect(selected.trim()).toBe(source.trim());
  await page.keyboard.press("ControlOrMeta+c");
  const copied = await page.evaluate(() => navigator.clipboard.readText());
  expect(copied.trim()).toBe(source.trim());
  expect(copied).not.toMatch(/^\d+$/m);

  // The Copy button is unchanged: the source, exactly.
  await dialog.getByRole("button", { name: /^Copy code/ }).click();
  await expect.poll(() => page.evaluate(() => navigator.clipboard.readText())).toBe(source);
});

test("View Code's gutter follows the theme and stays put when long lines scroll", async ({
  page,
}) => {
  await page.setViewportSize({ width: 600, height: 800 });
  await gotoHydrated(page, `${BASE_URL}/charts/bar/`);
  for (const scheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme: scheme });
    const viewCode = page
      .locator('.dx-charts-cell[data-variant="main"]')
      .getByRole("button", { name: /^View Code/ });
    await viewCode.scrollIntoViewIfNeeded();
    await viewCode.click();
    const dialog = page.getByRole("dialog", { name: "bar_chart/variants/main/mod.rs" });
    const gutter = dialog.locator(".dx-code-gutter");
    await expect(gutter).toBeVisible();

    // Legible against its own background: not the page's text color on itself.
    const [color, background] = await gutter.evaluate((el) => {
      const style = getComputedStyle(el);
      return [style.color, style.backgroundColor];
    });
    expect(color, scheme).not.toBe(background);
    expect(background, scheme).not.toBe("rgba(0, 0, 0, 0)");

    // Scrolling the code sideways leaves the numbers where they are.
    const body = dialog.locator(".dx-charts-code-body");
    const before = (await gutter.boundingBox())!.x;
    await body.evaluate((el) => (el.scrollLeft = 120));
    expect(await body.evaluate((el) => el.scrollLeft)).toBeGreaterThan(0);
    expect(Math.abs((await gutter.boundingBox())!.x - before)).toBeLessThanOrEqual(1);

    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
  }
});

test("the component pages' CODE tab shows the installed import path too", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/pie_chart/`);
  await page.getByRole("tab", { name: "CODE" }).first().click();
  const code = page.locator(".dx-component-preview-frame .dx-preview-code-theme").first();
  await expect(code).toContainText("use crate::components::pie_chart::*;");
  await expect(code).not.toContainText("super::super");
});

test("the component pages' CODE tab numbers its lines, and Copy leaves the numbers out", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await gotoHydrated(page, `${BASE_URL}/component/pie_chart/`);
  await page.getByRole("tab", { name: "CODE" }).first().click();
  const block = page.locator(".dx-component-preview-frame .dx-code-block").first();
  const gutter = block.locator(".dx-code-gutter");
  await expect(gutter).toBeVisible();
  await expect(gutter).toHaveAttribute("aria-hidden", "true");
  expect(await gutter.evaluate((el) => getComputedStyle(el).userSelect)).toBe("none");
  const numbers = (await gutter.textContent())!.split("\n");
  expect(numbers[0]).toBe("1");
  expect(numbers.at(-1)).toBe(String(numbers.length));

  // The block's Copy button copies the source: one line per number, none of them a number.
  await block.locator("xpath=..").getByRole("button", { name: "Copy code" }).first().click();
  const copied = await page.evaluate(() => navigator.clipboard.readText());
  expect(copied).toMatch(/^use crate::components::pie_chart::/);
  expect(copied).not.toMatch(/^\d+$/m);
  expect(copied.replace(/\n+$/, "").split("\n").length).toBe(numbers.length);
});

test("switching tabs keeps the tab row in view and focuses the active tab", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await gotoHydrated(page, `${BASE_URL}/charts/area/`);
  const nav = page.getByRole("navigation", { name: "Chart types" });
  const rowTop = () => nav.evaluate((el) => Math.round(el.getBoundingClientRect().top));

  // Scrolled past the hero, tab row a little below the sticky navbar.
  await page.evaluate(() => {
    const row = document.querySelector(".dx-charts-tabs")!;
    window.scrollTo({ top: window.scrollY + row.getBoundingClientRect().top - 120, behavior: "instant" });
  });
  await expect.poll(rowTop).toBe(120);

  // Area -> Pie remounts the page; Pie -> Radar keeps it. Both must hold the row.
  for (const label of ["Pie Charts", "Radar Charts", "Area Charts"]) {
    await nav.getByRole("link", { name: label }).click();
    await expect(nav.getByRole("link", { name: label })).toHaveAttribute("aria-current", "page");
    await expect(nav.getByRole("link", { name: label })).toBeFocused();
    await expect.poll(rowTop, { message: `tab row after ${label}` }).toBeGreaterThanOrEqual(60);
    await expect.poll(rowTop, { message: `tab row after ${label}` }).toBeLessThan(800 - 32);
    expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);
    expect(Math.abs((await rowTop()) - 120)).toBeLessThanOrEqual(2);
  }
});

test("a keyboard tab switch lands on the new tab, not the page top", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await gotoHydrated(page, `${BASE_URL}/charts/area/`);
  const nav = page.getByRole("navigation", { name: "Chart types" });
  const bar = nav.getByRole("link", { name: "Bar Charts" });
  await bar.focus();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/charts\/bar\/\??$/);
  await expect(bar).toBeFocused();
  await expect(bar).toBeInViewport();
});

test("each tab has its own document title", async ({ page }) => {
  for (const tab of TABS) {
    await gotoHydrated(page, `${BASE_URL}/charts/${tab.slug}/`);
    await expect(page).toHaveTitle(`${tab.label} \u2013 shadcn-dioxus`);
  }
  await gotoHydrated(page, `${BASE_URL}/charts/`);
  await expect(page).toHaveTitle("Area Charts \u2013 shadcn-dioxus");
  // ... and a client-side switch updates it.
  await page
    .getByRole("navigation", { name: "Chart types" })
    .getByRole("link", { name: "Line Charts" })
    .click();
  await expect(page).toHaveTitle("Line Charts \u2013 shadcn-dioxus");
});

test("the interactive bar toggles have no UA button border", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/charts/bar/`);
  const toggles = page.locator(".dx-chart-interactive-toggle");
  await expect(toggles).toHaveCount(2);
  for (const toggle of await toggles.all()) {
    const widths = await toggle.evaluate((el) => {
      const style = getComputedStyle(el);
      return [style.borderTopWidth, style.borderRightWidth, style.borderBottomWidth];
    });
    expect(widths).toEqual(["0px", "0px", "0px"]);
  }
});

test("Copy shows a 'Copy code' tooltip on hover", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/charts/pie/`);
  const copy = page.locator(".dx-charts-cell").first().getByRole("button", { name: /^Copy code/ });
  await copy.hover();
  await expect(page.getByRole("tooltip", { name: "Copy code" })).toBeVisible();
});

test("View Code opens the source sheet and Escape closes it, returning focus", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.setViewportSize({ width: 1280, height: 800 });
  await gotoHydrated(page, `${BASE_URL}/charts/area/`);

  const cell = page.locator('.dx-charts-cell[data-variant="gradient"]');
  const viewCode = cell.getByRole("button", { name: /^View Code/ });
  await viewCode.scrollIntoViewIfNeeded();
  await viewCode.click();

  const sheet = page.locator('[data-slot="sheet-root"]');
  await expect(sheet).toHaveAttribute("data-state", "open");
  const dialog = page.getByRole("dialog", { name: "area_chart/variants/gradient/mod.rs" });
  await expect(dialog).toBeVisible();
  // ~700px wide, flush to the right edge (once the slide-in has settled).
  await expect
    .poll(async () => {
      const box = (await dialog.boundingBox())!;
      const vw = (await fixedViewportBox(page)).width;
      return [Math.round(box.width), Math.abs(Math.round(box.x + box.width) - vw) <= SCROLLBAR_SLACK];
    })
    .toEqual([700, true]);

  // Highlighted source of THIS variant.
  await expect(dialog.locator(".dx-preview-code-theme")).toContainText("Area Chart - Gradient");
  await expect(dialog.locator(".dx-preview-code-theme .dxc span").first()).toBeAttached();

  // The sheet's own copy button copies the same text.
  await dialog.getByRole("button", { name: /^Copy code/ }).click();
  const expected = installedSource("area_chart", "gradient");
  await expect.poll(() => page.evaluate(() => navigator.clipboard.readText())).toBe(expected);

  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(viewCode).toBeFocused();

  // The close button closes it too.
  await viewCode.click();
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Close" }).click();
  await expect(dialog).toHaveCount(0);
  await expect(viewCode).toBeFocused();

  // ... and so does a click on the overlay.
  await viewCode.click();
  await expect(dialog).toBeVisible();
  await page.mouse.click(40, 400);
  await expect(dialog).toHaveCount(0);
  await expect(viewCode).toBeFocused();
});

test("View Code opens a bottom drawer on phones", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoHydrated(page, `${BASE_URL}/charts/pie/`);

  const cell = page.locator('.dx-charts-cell[data-variant="donut"]');
  const viewCode = cell.getByRole("button", { name: /^View Code/ });
  await viewCode.scrollIntoViewIfNeeded();
  await viewCode.click();

  const drawer = page.locator('[data-slot="drawer-root"]');
  await expect(drawer).toHaveAttribute("data-state", "open");
  await expect(page.locator('[data-slot="sheet-root"]')).toHaveCount(0);
  const dialog = page.getByRole("dialog", { name: "pie_chart/variants/donut/mod.rs" });
  await expect(dialog).toBeVisible();
  // Anchored to the bottom edge, full width (once the slide-in has settled).
  await expect
    .poll(async () => {
      const box = (await dialog.boundingBox())!;
      const vw = (await fixedViewportBox(page)).width;
      return [Math.abs(Math.round(box.width) - vw) <= SCROLLBAR_SLACK, Math.round(box.y + box.height)];
    })
    .toEqual([true, 844]);

  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(viewCode).toBeFocused();
});

test("the code block scrolls horizontally inside the scroll-locked drawer", async ({ page }) => {
  // Long source lines must scroll sideways, not clip: the modal scroll lock
  // used to treat any purely horizontal wheel/touch gesture as a background
  // scroll and cancel it.
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoHydrated(page, `${BASE_URL}/charts/pie/`);
  const viewCode = page
    .locator('.dx-charts-cell[data-variant="donut"]')
    .getByRole("button", { name: /^View Code/ });
  await viewCode.scrollIntoViewIfNeeded();
  await viewCode.click();
  const body = page.locator(".dx-charts-code-body");
  await expect(body).toBeVisible();
  // Let the slide-in settle so the pointer lands on the body, not mid-travel.
  const dialog = page.getByRole("dialog", { name: "pie_chart/variants/donut/mod.rs" });
  await expect
    .poll(async () => Math.round(((await dialog.boundingBox())!).y + ((await dialog.boundingBox())!).height))
    .toBe(844);
  expect(await body.evaluate((el) => el.scrollWidth > el.clientWidth)).toBe(true);
  const box = (await body.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(200, 0);
  await expect.poll(() => body.evaluate((el) => el.scrollLeft)).toBeGreaterThan(0);
});

test("no horizontal page overflow; the tab row scrolls inside itself", async ({ page }) => {
  // The columns follow the room the page has beside the docs sidebar (row 303), not the viewport:
  // 2 from 640px of page content, 3 from 960px (`@container charts-page`, main.css).
  for (const [width, expected] of [
    [1440, 3], // 1360px shell - 272px sidebar - 80px padding = 1008px of content
    [1280, 2], // 1248 - 272 - 80 = 896px
    [1024, 2], // 992 - 272 - 80 = 640px
    [768, 1], // the sidebar is still docked at 768px: 768 - 272 - 80 = ~416px, one column
    [700, 2], // below 768px the sidebar is the phone drawer: 700 - 48 = 652px
    [390, 1],
    [320, 1],
  ] as const) {
    await page.setViewportSize({ width, height: 900 });
    await gotoHydrated(page, `${BASE_URL}/charts/area/`);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow, `page overflows horizontally at ${width}px`).toBeLessThanOrEqual(0);
    const { columns, content } = await page.evaluate(() => {
      const grid = document.querySelector(".dx-charts-grid")!;
      const page = document.querySelector<HTMLElement>(".dx-charts-page")!;
      const cs = getComputedStyle(page);
      return {
        columns: getComputedStyle(grid).gridTemplateColumns.split(" ").length,
        content: page.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight),
      };
    });
    expect(columns, `${width}px viewport, ${content}px of page content`).toBe(expected);
    expect(columns, `${width}px: columns derive from the container width`).toBe(
      content >= 960 ? 3 : content >= 640 ? 2 : 1,
    );
  }
  const tabs = page.locator(".dx-charts-tabs");
  const scrolls = await tabs.evaluate((el) => el.scrollWidth > el.clientWidth);
  expect(scrolls).toBe(true);
});

test.describe("Axe automated scan", () => {
  test("a gallery tab has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/area/`);
    await expectNoAxeViolations(page, "charts gallery: area");
  });

  test("the pie tab has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/pie/`);
    await expectNoAxeViolations(page, "charts gallery: pie");
  });

  test("the open code sheet has no automatically detectable a11y issues", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await gotoHydrated(page, `${BASE_URL}/charts/area/`);
    await page
      .locator('.dx-charts-cell[data-variant="gradient"]')
      .getByRole("button", { name: /^View Code/ })
      .click();
    await expect(page.locator('[data-slot="sheet-root"]')).toHaveAttribute("data-state", "open");
    await expectNoAxeViolations(page, "charts gallery: code sheet open");
  });
});

// Row 302: the pill on a chart-type tab. The theme's press feedback paints a rounded wash while a link is
// `:active`, which read as "the pill that does not stay"; the active tab now keeps a `muted` pill, a hovered
// one shows a lighter pill, and the focus ring is keyboard-only.
test.describe("chart-type tabs keep the pill on the active tab", () => {
  const rgba = (el: Element) => getComputedStyle(el).backgroundColor;
  const isClear = (c: string) => c === "rgba(0, 0, 0, 0)" || c === "transparent";

  test("the active tab has a filled pill, the others none, and it follows a tab switch", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/area/`);
    const nav = page.getByRole("navigation", { name: "Chart types" });
    const area = nav.getByRole("link", { name: "Area Charts" });
    const pie = nav.getByRole("link", { name: "Pie Charts" });

    const filled = await area.evaluate(rgba);
    expect(isClear(filled), `active tab background ${filled}`).toBe(false);
    expect(isClear(await pie.evaluate(rgba)), "an inactive tab has no pill").toBe(true);
    // The pill is the pill shape, not a square wash.
    expect(await area.evaluate((el) => parseFloat(getComputedStyle(el).borderTopLeftRadius))).toBeGreaterThan(
      (await area.boundingBox())!.height / 2 - 1,
    );

    await pie.click();
    await expect(page).toHaveURL(/\/charts\/pie\/\??$/);
    const active = nav.getByRole("link", { name: "Pie Charts" });
    await expect(active).toHaveAttribute("aria-current", "page");
    // The transition is a short colour fade; wait for it to settle, then the pill is on the new tab.
    await expect.poll(async () => isClear(await active.evaluate(rgba))).toBe(false);
    await expect.poll(async () => isClear(await nav.getByRole("link", { name: "Area Charts" }).evaluate(rgba))).toBe(true);
  });

  test("the pill stays while the pointer is held down, and a clicked tab draws no focus ring", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/area/`);
    const nav = page.getByRole("navigation", { name: "Chart types" });
    const area = nav.getByRole("link", { name: "Area Charts" });
    const box = (await area.boundingBox())!;
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    expect(isClear(await area.evaluate(rgba)), "pressed active tab keeps its pill").toBe(false);
    await page.mouse.up();

    // Switch with the mouse: the router focuses the new active tab by script; under a mouse that is no ring.
    await nav.getByRole("link", { name: "Bar Charts" }).click();
    await expect(page).toHaveURL(/\/charts\/bar\/\??$/);
    const bar = nav.getByRole("link", { name: "Bar Charts" });
    await expect(bar).toBeFocused();
    expect(await bar.evaluate((el) => getComputedStyle(el).boxShadow), "no ring after a click").toMatch(
      /^none$|rgba\(0, 0, 0, 0\) 0px 0px 0px 0px/,
    );

    // Keyboard: the ring is back.
    await page.keyboard.press("Tab");
    await page.keyboard.press("Shift+Tab");
    await expect(bar).toBeFocused();
    expect(await bar.evaluate((el) => getComputedStyle(el).boxShadow), "a ring for the keyboard").not.toMatch(
      /^none$|rgba\(0, 0, 0, 0\) 0px 0px 0px 0px$/,
    );
  });

  test("hover shows a lighter pill on an inactive tab (fine pointers only)", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/area/`);
    const nav = page.getByRole("navigation", { name: "Chart types" });
    const pie = nav.getByRole("link", { name: "Pie Charts" });
    await pie.hover();
    await expect.poll(async () => isClear(await pie.evaluate(rgba))).toBe(false);
    const hovered = await pie.evaluate(rgba);
    const active = await nav.getByRole("link", { name: "Area Charts" }).evaluate(rgba);
    expect(hovered, "hover is not the active pill").not.toBe(active);
    // Only compositor-cheap properties animate.
    const props = await pie.evaluate((el) => getComputedStyle(el).transitionProperty);
    expect(props).toContain("background-color");
    expect(props).not.toMatch(/\ball\b|width|height|padding/);
  });
});

// Row 303: the gallery wears the docs shell like every component page.
test.describe("the charts and demos pages have the docs sidebar", () => {
  for (const [path, label] of [
    ["/charts/", "Charts"],
    ["/charts/pie/", "Charts"],
    ["/demos/", "Demos"],
  ] as const) {
    test(`${path} renders the sidebar, its trigger, and marks "${label}"`, async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 900 });
      await gotoHydrated(page, `${BASE_URL}${path}`);
      const sidebar = page.locator('[data-slot="sidebar-inner"]');
      await expect(sidebar).toBeVisible();
      await expect(page.locator(".dx-navbar-sidebar-trigger")).toBeVisible();
      await expect(sidebar.locator("a[aria-current]:visible")).toHaveText(label);
      // One landmark: the shell's inset is the page's <main>, and the skip link still lands on it.
      await expect(page.locator("main")).toHaveCount(1);
      await expect(page.locator("main#main-content")).toHaveCount(1);
      // The page content sits beside the sidebar, not under it.
      const sidebarBox = (await sidebar.boundingBox())!;
      const contentBox = (await page.locator(path.startsWith("/charts") ? ".dx-charts-page" : ".dx-home-page").boundingBox())!;
      expect(contentBox.x).toBeGreaterThanOrEqual(sidebarBox.x + sidebarBox.width - 1);
    });
  }

  test("at a phone width the trigger opens the sidebar as a drawer and nothing overflows", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    for (const path of ["/charts/", "/demos/"]) {
      await gotoHydrated(page, `${BASE_URL}${path}`);
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow, `${path} overflows horizontally at 390px`).toBeLessThanOrEqual(0);
      await page.locator(".dx-navbar-sidebar-trigger").click();
      await expect(page.getByRole("dialog").getByRole("link", { name: "Charts", exact: true })).toBeVisible();
    }
  });
});
