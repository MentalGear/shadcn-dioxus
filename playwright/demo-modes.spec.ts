import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// One interaction per supported mode that has its own demo (backlog row 399): each demo is opened alone
// on its block page and the mode is exercised once.
const block = (name: string, variant: string) => `${BASE_URL}/component/block/?name=${name}&variant=${variant}&`;
const goto = (page: Page, name: string, variant: string) =>
  gotoHydrated(page, block(name, variant), { timeout: 20 * 60 * 1000 });

test("data_table sort_reorder: a sort disables the drag handles, clearing it lets rows reorder", async ({ page }) => {
  await goto(page, "data_table", "sort_reorder");
  const ids = async () => (await page.locator('[data-slot="table"] tbody tr td:last-child').allTextContents()).map((t) => t.trim());
  const handle = (key: string) => page.locator(`[data-dx-reorder-handle="${key}"]`);
  const header = page.getByRole("columnheader", { name: "Header" });

  await header.getByRole("button").click();
  await expect(header).toHaveAttribute("aria-sort", "ascending");
  await expect(handle("T-1")).toHaveAttribute("title", "Clear the sort to reorder rows");
  await expect(handle("T-1")).toHaveAttribute("draggable", "false");
  // Ascending by title: Cover page, Design, Executive summary, Table of contents, Technical approach.
  expect(await ids()).toEqual(["T-1", "T-5", "T-3", "T-2", "T-4"]);

  await header.getByRole("button").click(); // descending
  await header.getByRole("button").click(); // none
  await expect(header).toHaveAttribute("aria-sort", "none");
  await expect(handle("T-1")).toHaveAttribute("draggable", "true");
  expect(await ids()).toEqual(["T-1", "T-2", "T-3", "T-4", "T-5"]);

  await handle("T-1").focus();
  await page.keyboard.press("Space");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Space");
  await expect.poll(ids).toEqual(["T-2", "T-1", "T-3", "T-4", "T-5"]);
});

const arcAngles = async (page: Page) => {
  const arcs = page.locator('[data-slot="chart-arc"]');
  await expect(arcs.first()).toBeVisible();
  const out: { start: number; end: number }[] = [];
  for (let i = 0; i < (await arcs.count()); i++) {
    out.push({
      start: parseFloat((await arcs.nth(i).getAttribute("data-start-angle")) ?? "NaN"),
      end: parseFloat((await arcs.nth(i).getAttribute("data-end-angle")) ?? "NaN"),
    });
  }
  return out;
};

test("pie_chart half: the slices span 180 degrees, not 360", async ({ page }) => {
  await goto(page, "pie_chart", "half");
  const arcs = await arcAngles(page);
  expect(arcs.length).toBe(5);
  const all = arcs.flatMap((a) => [a.start, a.end]);
  expect(Math.max(...all) - Math.min(...all)).toBeCloseTo(180, 1);
});

test("pie_chart padding: adjacent slices are 4 degrees apart", async ({ page }) => {
  await goto(page, "pie_chart", "padding");
  // The slices keep their nominal angles; the padding is in the drawn path, so the gap is measured between
  // one arc's end point and the next arc's start point, seen from the centre (the pie's origin).
  const ds = await page.locator('[data-slot="chart-arc"]').evaluateAll((els) => els.map((e) => e.getAttribute("d") ?? ""));
  expect(ds.length).toBe(5);
  const nums = (d: string) => (d.match(/-?\d+(\.\d+)?/g) ?? []).map(Number);
  const angle = (x: number, y: number) => (Math.atan2(y, x) * 180) / Math.PI;
  const start = (d: string) => angle(nums(d)[0], nums(d)[1]);
  const end = (d: string) => {
    const n = nums(d); // M x y A rx ry rot large sweep x y L 0 0 Z
    return angle(n[7], n[8]);
  };
  for (let i = 0; i < ds.length; i++) {
    const gap = Math.abs(start(ds[(i + 1) % ds.length]) - end(ds[i]));
    expect(Math.min(gap, 360 - gap)).toBeCloseTo(4, 0);
  }
});

test("line_chart gaps: a missing month breaks the line into two runs", async ({ page }) => {
  await goto(page, "line_chart", "gaps");
  const runs = page.locator('[data-slot="chart-line"]');
  await expect(runs).toHaveCount(2);
  const ds = await runs.evaluateAll((els) => els.map((e) => e.getAttribute("d") ?? ""));
  ds.forEach((d) => expect((d.match(/M/g) ?? []).length).toBe(1));
  await expect(page.locator('[data-slot="chart-dot"]')).toHaveCount(5);
});

test("input_otp complete: digits are dropped and on_complete fires when the last letter lands", async ({ page }) => {
  await goto(page, "input_otp", "complete");
  const status = page.locator("#input-otp-complete-status");
  const input = page.locator("#otp-complete");
  await input.click();
  await page.keyboard.type("12ab");
  await expect(status).toHaveText("Type four letters.");
  await page.keyboard.type("cd");
  await expect(status).toHaveText("Completed: abcd");
});

test("resizable controlled: a key step updates the app's sizes and Reset writes them back", async ({ page }) => {
  await goto(page, "resizable", "controlled");
  const sizes = page.locator("#resizable-controlled-sizes");
  await expect(sizes).toHaveText("50% / 50%");
  const handle = page.getByRole("separator", { name: "Controlled" });
  await handle.focus();
  await page.keyboard.press("ArrowRight");
  await expect(sizes).not.toHaveText("50% / 50%");
  await page.locator("#resizable-controlled-reset").click();
  await expect(sizes).toHaveText("50% / 50%");
  await expect(handle).toHaveAttribute("aria-valuenow", "50");
});

test("context_menu non_modal: an open menu does not lock page scrolling", async ({ page }) => {
  await goto(page, "context_menu", "non_modal");
  const locked = () => page.evaluate(() => (window as unknown as { __dxScrollLocked?: boolean }).__dxScrollLocked === true);
  await page.getByRole("button", { name: "Right click (non-modal)" }).click({ button: "right" });
  await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
  expect(await locked()).toBe(false);
  await page.getByRole("menuitem", { name: "Paste" }).click();
  await expect(page.locator("#context-menu-non-modal-chosen")).toHaveText("Selected: Paste");
});
