import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { demoFrame, demoRole } from "./demo-frame";
import { gotoHydrated } from "./hydration";

// `TableRows` (backlog row 343): the data body of a real <table> with the shared
// `virtualize: Auto | ContentVisibility | Windowed` and `auto_threshold`. The `virtualized` demo
// has three buttons (20,000 / 1,001 / 1,000 rows) around the default threshold of 1,000.
const URL = `${BASE_URL}/component/table/`;
const frame = (page: Page) => demoFrame(page, "virtualized");
const body = (page: Page) => frame(page).locator("tbody");
const rows = (page: Page) => body(page).locator("tr[data-virtual-index]");
const scroller = (page: Page) => frame(page).locator(".dx-table-container");
// virtualized demo: button 0 = 20,000 rows, 1 = 1,001 rows, 2 = 1,000 rows
const pick = (page: Page, n: number) => demoRole(page, "button", n, "virtualized").click();

async function open(page: Page) {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  await scroller(page).scrollIntoViewIfNeeded();
  await expect(body(page)).toHaveAttribute("data-virtualize", "windowed");
  await expect(rows(page).first()).toBeVisible();
}

test.describe("TableRows", () => {
  // Row 369: rows measured with a fraction. The rows sit in flow at their exact height, so the
  // bottom spacer is the exact rest: at the end it is empty, not the canvas's rounded-up padding.
  test("rows with a fractional height: the spacers add up to the rows' exact extent", async ({ page }) => {
    await open(page);
    await page.addStyleTag({ content: `tr[data-virtual-index] { height: 50.1875px !important; }` });
    await expect.poll(() => rows(page).first().evaluate((el) => el.getBoundingClientRect().height)).toBe(50.1875);
    const bottomSpacer = body(page).locator('tr[data-virtual-spacer="bottom"]');
    await expect(async () => {
      await scroller(page).evaluate((el) => (el.scrollTop = 1e9));
      await expect(rows(page).last()).toHaveAttribute("data-virtual-index", "19999", { timeout: 1000 });
      const spacer = await bottomSpacer.evaluate((el) => el.getBoundingClientRect().height);
      expect(spacer).toBeLessThan(0.01);
    }).toPass({ timeout: 30_000 });
  });

  test("Auto windows 20,000 rows: a handful in the DOM, spacers, row and table ARIA", async ({ page }) => {
    await open(page);
    const n = await rows(page).count();
    expect(n).toBeGreaterThan(5);
    expect(n).toBeLessThan(80);
    // A real table: rows are <tr> inside the one <tbody>, cells are <td>.
    await expect(frame(page).locator("table")).toHaveCount(1);
    await expect(rows(page).first().locator("td")).toHaveCount(3);
    // aria-rowindex counts the header row; aria-rowcount is the whole table, not the window.
    await expect(rows(page).first()).toHaveAttribute("data-virtual-index", "0");
    await expect(rows(page).first()).toHaveAttribute("aria-rowindex", "2");
    await expect(frame(page).locator("table")).toHaveAttribute("aria-rowcount", "20001");
    // The spacers are aria-hidden single cells spanning every column.
    for (const which of ["top", "bottom"]) {
      const spacer = body(page).locator(`tr[data-virtual-spacer="${which}"]`);
      await expect(spacer).toHaveAttribute("aria-hidden", "true");
      await expect(spacer.locator("td")).toHaveAttribute("colspan", "3");
    }
    // The fixed selection renders `data-state="selected"` on the real row.
    await expect(body(page).locator('tr[data-virtual-index="2"]')).toHaveAttribute("data-state", "selected");
    // The whole table's height is there for the scrollbar.
    const { scrollHeight, clientHeight } = await scroller(page).evaluate((el) => ({
      scrollHeight: el.scrollHeight,
      clientHeight: el.clientHeight,
    }));
    expect(scrollHeight).toBeGreaterThan(20_000 * 25);
    expect(clientHeight).toBeLessThanOrEqual(321);
  });

  test("the window follows the scroll and the <thead> stays in view", async ({ page }) => {
    await open(page);
    const box = scroller(page);
    await box.evaluate((el) => (el.scrollTop = 200_000));
    await expect
      .poll(async () => Number(await rows(page).first().getAttribute("data-virtual-index")))
      .toBeGreaterThan(1_000);
    expect(await rows(page).count()).toBeLessThan(80);
    // Sticky header: still at the top of the scroll container after scrolling 200,000 px.
    const head = frame(page).locator("thead th").first();
    await expect(head).toBeVisible();
    const [h, c] = await Promise.all([head.boundingBox(), box.boundingBox()]);
    expect(Math.abs(h!.y - c!.y)).toBeLessThanOrEqual(2);
    // The row at the top of the view is a real, numbered row of the whole table.
    const first = Number(await rows(page).first().getAttribute("data-virtual-index"));
    await expect(rows(page).first()).toHaveAttribute("aria-rowindex", String(first + 2));
    // Some mounted row sits inside the viewport (the spacers put the window where we scrolled).
    const visible = await rows(page).evaluateAll((trs, top) => {
      const c = top as number;
      return trs.filter((tr) => {
        const r = tr.getBoundingClientRect();
        return r.bottom > c && r.top < c + 300;
      }).length;
    }, c!.y);
    expect(visible).toBeGreaterThan(3);
  });

  test("Auto switches at the threshold: 1,001 windowed, 1,000 every row", async ({ page }) => {
    await open(page);
    await pick(page, 1);
    await expect(body(page)).toHaveAttribute("data-virtualize", "windowed");
    expect(await rows(page).count()).toBeLessThan(80);
    await expect(frame(page).locator("table")).toHaveAttribute("aria-rowcount", "1002");
    await pick(page, 2);
    await expect(body(page)).toHaveAttribute("data-virtualize", "content-visibility");
    await expect(rows(page)).toHaveCount(1000);
    // No spacers, and no aria-rowcount: every row is in the DOM, the UA counts them.
    await expect(body(page).locator("tr[data-virtual-spacer]")).toHaveCount(0);
    await expect(frame(page).locator("table")).not.toHaveAttribute("aria-rowcount", /.*/);
    await expect(rows(page).last()).toHaveAttribute("aria-rowindex", "1001");
    await pick(page, 0);
    await expect(body(page)).toHaveAttribute("data-virtualize", "windowed");
  });

  test("every-row mode is findable (find-in-page reaches every row); windowed is not", async ({ page, browserName }) => {
    test.skip(browserName !== "chromium", "window.find is the Chromium find-in-page stand-in");
    await open(page);
    // Windowed: row 9,000 does not exist, so nothing to find.
    const windowed = await page.evaluate(() => (window as any).find("INV009001", false, false, true));
    expect(windowed).toBe(false);
    await pick(page, 2);
    await expect(rows(page)).toHaveCount(1000);
    await page.evaluate(() => window.getSelection()?.removeAllRanges());
    const found = await page.evaluate(() => (window as any).find("INV000999", false, false, true));
    expect(found).toBe(true);
    // The match is in the real row 998.
    await expect(rows(page).nth(998)).toContainText("INV000999");
  });
});
