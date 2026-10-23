import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

// Fixture mirrors preview/src/components/data_table/variants/main/mod.rs's
// `PAYMENTS` (12 rows, id/status/email/amount) exactly -- single source of
// truth for the sort/filter/page expectations below is that file's own
// header comment, not re-derived here.
const goto = (page: Page) =>
  gotoHydrated(page, `${BASE_URL}/component/data_table/`, { timeout: 20 * 60 * 1000 });

const table = (page: Page) => page.locator('#component-preview-frame [data-slot="table"]');
const filterInput = (page: Page) => page.getByRole("textbox", { name: "Filter by email" });
// Scoped to the main demo's frame: the page also holds the `large` Grid demo, whose header and rows carry the same names.
const mainDemo = (page: Page) => page.locator("#component-preview-frame");
const selectAllCheckbox = (page: Page) => mainDemo(page).getByRole("checkbox", { name: "Select all" });
const rowCheckboxes = (page: Page) => mainDemo(page).getByRole("checkbox", { name: /^Select row / });
const pagination = (page: Page) => page.locator('[data-slot="data-table-pagination"]');
const selectedSummary = (page: Page) => pagination(page).locator(".dx-data-table-pagination-selected");
const pageReadout = (page: Page) => pagination(page).locator(".dx-data-table-pagination-page");
const previousButton = (page: Page) => pagination(page).getByRole("button", { name: "Previous" });
const nextButton = (page: Page) => pagination(page).getByRole("button", { name: "Next" });
// The Select trigger's role is changing elsewhere in this repo (button ->
// combobox); located by its own data-slot + accessible name, not role, so
// this spec doesn't depend on which role wins.
const pageSizeControl = (page: Page) =>
  page.locator('[data-slot="data-table-page-size"]').locator('[aria-label="Rows per page"]');

test("renders the payments demo with pagination", async ({ page }) => {
  await goto(page);

  await expect(table(page)).toBeVisible();
  await expect(table(page).locator("tbody tr")).toHaveCount(5); // page 1 of 3, 5 rows/page
  await expect(selectedSummary(page)).toHaveText("0 of 12 row(s) selected.");
  await expect(pageReadout(page)).toHaveText("Page 1 of 3");
});

test("filter narrows rows by email", async ({ page }) => {
  await goto(page);

  await filterInput(page).fill("ken");

  const rows = table(page).locator("tbody tr");
  await expect(rows).toHaveCount(1);
  await expect(rows.first()).toContainText("ken99@example.com");
  await expect(pageReadout(page)).toHaveText("Page 1 of 1");
  await expect(selectedSummary(page)).toHaveText("0 of 1 row(s) selected.");

  // Clearing the filter restores every row.
  await filterInput(page).fill("");
  await expect(table(page).locator("tbody tr")).toHaveCount(5);
  await expect(selectedSummary(page)).toHaveText("0 of 12 row(s) selected.");
});

test("filter matches a case-insensitive fragment anywhere in the email, not just a prefix", async ({ page }) => {
  await goto(page);

  // shadcn/ui's own Data Table filters the email column through TanStack
  // Table's default `includesString` filter fn -- a case-insensitive
  // substring match anywhere in the cell, not restricted to the start of
  // the string. "REK" is an uppercase, mid-to-end-of-string fragment of
  // derek@example.com (the tail of "derek"), not a prefix of any PAYMENTS
  // email -- a filter that only matched from the start would narrow to 0
  // rows here instead of 1.
  await filterInput(page).fill("REK");

  const rows = table(page).locator("tbody tr");
  await expect(rows).toHaveCount(1);
  await expect(rows.first()).toContainText("derek@example.com");
  await expect(selectedSummary(page)).toHaveText("0 of 1 row(s) selected.");
});

test("select-all checks every visible row and the selected count updates", async ({ page }) => {
  await goto(page);

  const checkboxesBefore = rowCheckboxes(page);
  await expect(checkboxesBefore).toHaveCount(5);
  for (const checkbox of await checkboxesBefore.all()) {
    await expect(checkbox).toHaveAttribute("aria-checked", "false");
  }

  await selectAllCheckbox(page).click();

  for (const checkbox of await rowCheckboxes(page).all()) {
    await expect(checkbox).toHaveAttribute("aria-checked", "true");
  }
  await expect(selectAllCheckbox(page)).toHaveAttribute("aria-checked", "true");
  await expect(selectedSummary(page)).toHaveText("5 of 12 row(s) selected.");

  // Unchecking Select All clears exactly those 5 again.
  await selectAllCheckbox(page).click();
  await expect(selectedSummary(page)).toHaveText("0 of 12 row(s) selected.");
  for (const checkbox of await rowCheckboxes(page).all()) {
    await expect(checkbox).toHaveAttribute("aria-checked", "false");
  }
});

test("Next/Previous move pages and disable at the ends", async ({ page }) => {
  await goto(page);

  await expect(previousButton(page), "disabled on the first page").toBeDisabled();
  await expect(nextButton(page)).toBeEnabled();

  await nextButton(page).click();
  await expect(pageReadout(page)).toHaveText("Page 2 of 3");
  await expect(previousButton(page)).toBeEnabled();
  await expect(nextButton(page)).toBeEnabled();
  await expect(table(page).locator("tbody tr")).toHaveCount(5);

  await nextButton(page).click();
  await expect(pageReadout(page)).toHaveText("Page 3 of 3");
  await expect(table(page).locator("tbody tr"), "last page is a short page (12 rows, 5/page)").toHaveCount(2);
  await expect(nextButton(page), "disabled on the last page").toBeDisabled();
  await expect(previousButton(page)).toBeEnabled();

  await previousButton(page).click();
  await expect(pageReadout(page)).toHaveText("Page 2 of 3");
});

// --- The column model: sort, selection across pages, keyboard -----------------------------
//
// Backlog row 340: the demo is one `DataTable<Payment>` (columns, row key, features) and the
// component owns sort/filter/page/selection, so these assert what the model guarantees, not
// what hand-written markup happened to do.

const header = (page: Page, name: string) => table(page).locator("th", { hasText: name });
const sortButton = (page: Page, name: string) => header(page, name).getByRole("button");
const firstRowText = (page: Page) => table(page).locator("tbody tr").first();

test("the header sort button toggles direction and moves aria-sort to the one sorted column", async ({ page }) => {
  await goto(page);

  await expect(header(page, "Email")).toHaveAttribute("aria-sort", "ascending");
  await expect(header(page, "Amount")).toHaveAttribute("aria-sort", "none");
  await expect(firstRowText(page)).toContainText("alexandra@example.com");

  await sortButton(page, "Email").click();
  await expect(header(page, "Email")).toHaveAttribute("aria-sort", "descending");
  await expect(firstRowText(page)).toContainText("zara@example.com");

  await sortButton(page, "Amount").click();
  await expect(header(page, "Email")).toHaveAttribute("aria-sort", "none");
  await expect(header(page, "Amount")).toHaveAttribute("aria-sort", "ascending");
  await expect(firstRowText(page)).toContainText("$55.00");

  await sortButton(page, "Amount").click();
  await expect(firstRowText(page)).toContainText("$999.00");

  // ID and Status are not sortable: no button, no aria-sort.
  await expect(header(page, "Status").getByRole("button")).toHaveCount(0);
  await expect(header(page, "Status")).not.toHaveAttribute("aria-sort", /.*/);
});

test("sorting and filtering return to page 1", async ({ page }) => {
  await goto(page);

  await nextButton(page).click();
  await expect(pageReadout(page)).toHaveText("Page 2 of 3");
  await sortButton(page, "Amount").click();
  await expect(pageReadout(page)).toHaveText("Page 1 of 3");

  await nextButton(page).click();
  await filterInput(page).fill("a");
  await expect(pageReadout(page)).toContainText("Page 1 of");
});

test("selection is kept by row across pages, sorting and filtering", async ({ page }) => {
  await goto(page);

  await page.getByRole("checkbox", { name: "Select row PMT-03" }).click(); // alexandra, page 1
  await expect(selectedSummary(page)).toHaveText("1 of 12 row(s) selected.");
  await expect(selectAllCheckbox(page)).toHaveAttribute("aria-checked", "mixed");
  await expect(table(page).locator("tbody tr[data-state=selected]")).toHaveCount(1);

  // Page 2 shows none of it, but the footer still counts it.
  await nextButton(page).click();
  await expect(selectAllCheckbox(page)).toHaveAttribute("aria-checked", "false");
  await expect(selectedSummary(page)).toHaveText("1 of 12 row(s) selected.");

  // Re-sort: the same row is still the selected one, wherever it lands.
  await previousButton(page).click();
  await sortButton(page, "Email").click(); // descending: alexandra is last overall
  await expect(page.getByRole("checkbox", { name: "Select row PMT-03" })).toHaveCount(0);
  await filterInput(page).fill("alexandra");
  await expect(page.getByRole("checkbox", { name: "Select row PMT-03" })).toHaveAttribute("aria-checked", "true");
  await expect(selectAllCheckbox(page)).toHaveAttribute("aria-checked", "true");
  await expect(selectedSummary(page)).toHaveText("1 of 1 row(s) selected.");
});

test("the selected count only counts rows in the filtered view (row 360)", async ({ page }) => {
  await goto(page);

  await page.getByRole("checkbox", { name: "Select row PMT-03" }).click(); // alexandra
  await expect(selectedSummary(page)).toHaveText("1 of 12 row(s) selected.");

  // Filter the selected row out: the footer reads "selected in view of filtered", never "1 of 1" for a
  // row that is not in the view, and certainly not "3 of 1".
  await filterInput(page).fill("ken");
  await expect(table(page).locator("tbody tr")).toHaveCount(1);
  await expect(selectedSummary(page)).toHaveText("0 of 1 row(s) selected.");

  // The selection itself survives: clearing the filter brings it back.
  await filterInput(page).fill("");
  await expect(selectedSummary(page)).toHaveText("1 of 12 row(s) selected.");
});

test("keyboard: Enter and Space sort from the header button, Space toggles row and select-all checkboxes", async ({
  page,
}) => {
  await goto(page);

  await sortButton(page, "Amount").focus();
  await page.keyboard.press("Enter");
  await expect(header(page, "Amount")).toHaveAttribute("aria-sort", "ascending");
  await page.keyboard.press("Space");
  await expect(header(page, "Amount")).toHaveAttribute("aria-sort", "descending");
  await expect(sortButton(page, "Amount")).toBeFocused();

  await selectAllCheckbox(page).focus();
  await page.keyboard.press("Space");
  await expect(selectedSummary(page)).toHaveText("5 of 12 row(s) selected.");
  await expect(selectAllCheckbox(page)).toHaveAttribute("aria-checked", "true");

  // Tab walks header checkbox -> sort buttons -> first row checkbox.
  await page.keyboard.press("Tab");
  await expect(sortButton(page, "Email")).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(sortButton(page, "Amount")).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(rowCheckboxes(page).first()).toBeFocused();
  await page.keyboard.press("Space");
  await expect(rowCheckboxes(page).first()).toHaveAttribute("aria-checked", "false");
  await expect(selectAllCheckbox(page)).toHaveAttribute("aria-checked", "mixed");
  await expect(selectedSummary(page)).toHaveText("4 of 12 row(s) selected.");
});

test("keyboard: the filter input narrows rows and Previous/Next are reachable and operable", async ({ page }) => {
  await goto(page);

  await filterInput(page).focus();
  await page.keyboard.type("carm");
  await expect(table(page).locator("tbody tr")).toHaveCount(1); // carmella
  await expect(firstRowText(page)).toContainText("carmella@example.com");
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Backspace");
  await expect(table(page).locator("tbody tr")).toHaveCount(5);

  await nextButton(page).focus();
  await page.keyboard.press("Enter");
  await expect(pageReadout(page)).toHaveText("Page 2 of 3");
  await previousButton(page).focus();
  await page.keyboard.press("Space");
  await expect(pageReadout(page)).toHaveText("Page 1 of 3");
});

// --- Checkbox alignment and layout stability ---------------------------------
//
// Owner report: "Datatables: checkboxes are not vertically centered, and there is
// also a layout shift when they are toggled." Two root causes, measured 2026-10-04
// with the real Geist web font (the offsets differ with the fallback font):
//   1. `.dx-checkbox` was an inline-block <button>, so it sat on the cell's line-box
//      BASELINE: 2.00px above the row/cell/text centre (2.25px in the header), and
//      the row grew to 37px. Fix: the checkbox is a flex box (Nova's `flex size-4`).
//   2. The demo shrink-wrapped inside the centred preview frame, so its width was the
//      pagination row's max-content, and "0 of 12 row(s) selected." -> "1 of 12 ..."
//      (proportional digits, a `1` is ~4px narrower) moved the whole table ~2px and
//      resized it ~4px on EVERY toggle. Fix: the demo is `width: 100%`, and the
//      counters use tabular figures.
// The alignment and toggle tests below measure real boxes (getBoundingClientRect), not
// computed styles, so they stay red for any cause, not just the two above. Which tests are
// red without the fix depends on the font: the alignment test, the container-overflow guard
// and the invariant test fail in any font; the two toggle tests fail where the pagination
// row is the demo's widest child (Geist), which a wider fallback face does not reproduce.

type Box = { x: number; y: number; width: number; height: number };

// Everything that must not move when a checkbox is toggled, as live DOM rects, read AT REST: a click leaves the
// control in its `:active` release (the theme's press feedback scales it to 0.98 and eases back over
// `--dx-press-duration`), and `getBoundingClientRect()` includes `scale`, so a read right after the click saw the
// checkbox 0.3 px smaller and 0.1 px right of its layout box. This test is about LAYOUT, which the press never
// changes, so it waits for the checkbox's (and its ancestors') transitions to finish first (`readAtRest`).
const layoutBoxes = (page: Page, label: string) =>
  readAtRest(
    mainDemo(page).getByRole("checkbox", { name: label, exact: true }),
    (checkbox) => {
      const box = (el: Element | null): Box => {
        if (!el) throw new Error("missing element");
        const r = el.getBoundingClientRect();
        return { x: r.x, y: r.y, width: r.width, height: r.height };
      };
      const cell = checkbox.closest("td,th")!;
      return {
        checkbox: box(checkbox),
        cell: box(cell),
        row: box(cell.closest("tr")),
        table: box(document.querySelector('#component-preview-frame [data-slot="table"]')),
        toolbar: box(document.querySelector('[data-slot="data-table-toolbar"]')),
        pagination: box(document.querySelector('[data-slot="data-table-pagination"]')),
        // The count's own width legitimately follows its text; its origin must not move.
        selectedOrigin: (({ x, y }) => ({ x, y }))(box(document.querySelector(".dx-data-table-pagination-selected"))),
        controls: box(document.querySelector(".dx-data-table-pagination-controls")),
      };
    },
  );

const expectSameBoxes = (before: Record<string, object>, after: Record<string, object>, what: string) => {
  for (const key of Object.keys(before)) {
    const b = before[key] as Record<string, number>;
    const a = after[key] as Record<string, number>;
    for (const prop of Object.keys(b)) {
      expect(a[prop], `${what}: ${key}.${prop} moved (${b[prop]} -> ${a[prop]})`).toBeCloseTo(b[prop], 1);
    }
  }
};

const gotoStable = async (page: Page) => {
  await gotoHydrated(page, `${BASE_URL}/component/data_table/`, { timeout: 20 * 60 * 1000 });
  await expect(table(page)).toBeVisible();
  // Geist loads from Google Fonts; metrics (and so the line box) change when it lands.
  await page.evaluate(() => document.fonts.ready);
};

test("every checkbox is vertically centred in its cell, its row and beside the adjacent text", async ({ page }) => {
  await gotoStable(page);

  const offsets = await page.evaluate(() => {
    const centre = (r: { y: number; height: number }) => r.y + r.height / 2;
    return [...document.querySelectorAll('#component-preview-frame [data-slot="table"] tr')].flatMap((tr) => {
      const checkbox = tr.querySelector('[role="checkbox"]');
      if (!checkbox) return [];
      const cell = checkbox.closest("td,th")!;
      // The adjacent text: the first text node of the next cell, as laid out.
      const text = document.createTreeWalker(cell.nextElementSibling!, NodeFilter.SHOW_TEXT).nextNode()!;
      const range = document.createRange();
      range.selectNodeContents(text);
      const cb = checkbox.getBoundingClientRect();
      return [
        {
          label: checkbox.getAttribute("aria-label"),
          vsCell: centre(cb) - centre(cell.getBoundingClientRect()),
          vsRow: centre(cb) - centre(tr.getBoundingClientRect()),
          vsText: centre(cb) - centre(range.getBoundingClientRect()),
        },
      ];
    });
  });

  expect(offsets, "header + 5 row checkboxes").toHaveLength(6);
  for (const { label, vsCell, vsRow, vsText } of offsets) {
    expect(Math.abs(vsCell), `${label} vs its cell (${vsCell}px)`).toBeLessThanOrEqual(1);
    expect(Math.abs(vsRow), `${label} vs its row (${vsRow}px)`).toBeLessThanOrEqual(1);
    expect(Math.abs(vsText), `${label} vs the adjacent text (${vsText}px)`).toBeLessThanOrEqual(1);
  }
});

test("the checkbox's hidden mirror input does not make the table container scrollable", async ({ page }) => {
  await gotoStable(page);

  // The primitive follows each checkbox with a hidden `position: absolute` <input>. Behind a
  // block-level checkbox it sits below the box, and `overflow: hidden` on the cell does not
  // clip it (the container is its containing block) -- a 4px vertical scroll range.
  const overflow = await page.locator('#component-preview-frame [data-slot="table-container"]').evaluate((el) => ({
    vertical: el.scrollHeight - el.clientHeight,
    horizontal: el.scrollWidth - el.clientWidth,
  }));
  expect(overflow).toEqual({ vertical: 0, horizontal: 0 });
});

test("the demo spans its frame and the counters use tabular figures, so no text width can move the table", async ({ page }) => {
  await gotoStable(page);

  // The preview frame centres its children and shrink-wraps them, so a demo without its own
  // `width` is as wide as its widest child -- here the pagination row, whose width follows the
  // "N of 12 row(s) selected." text. Whether that child is the widest depends on the font in use
  // (Geist: yes, a wider fallback: no), which is why the two behavioural tests below can pass
  // without it; this is the font-independent invariant that rules the whole class out.
  const { demo, frame } = await page.evaluate(() => {
    const demo = document.querySelector(".dx-data-table")!;
    const frame = demo.parentElement!;
    const cs = getComputedStyle(frame);
    return {
      demo: demo.getBoundingClientRect().width,
      frame: frame.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight),
    };
  });
  expect(demo, "demo width vs the preview frame's content width").toBeCloseTo(frame, 0);

  // The counters are the only text that changes on a toggle / page turn: digits keep one width.
  for (const counter of [selectedSummary(page), pageReadout(page)]) {
    await expect(counter).toHaveCSS("font-variant-numeric", "tabular-nums");
  }
});

test("toggling a checkbox moves and resizes nothing", async ({ page }) => {
  await gotoStable(page);

  const labels = ["Select all", ...(await rowCheckboxes(page).evaluateAll((els) => els.map((el) => el.getAttribute("aria-label")!)))];
  expect(labels).toHaveLength(6);

  for (const label of labels) {
    const checkbox = mainDemo(page).getByRole("checkbox", { name: label, exact: true });
    const before = await layoutBoxes(page, label);
    await checkbox.click();
    await expect(checkbox).not.toHaveAttribute("aria-checked", "false");
    await page.mouse.move(0, 0); // park the pointer so a :hover fill is not part of the comparison
    const after = await layoutBoxes(page, label);
    expectSameBoxes(before, after, `checking "${label}"`);

    await checkbox.click();
    await page.mouse.move(0, 0);
    expectSameBoxes(before, await layoutBoxes(page, label), `unchecking "${label}"`);
  }
});

test("the selection count gaining a digit moves nothing but its own text", async ({ page }) => {
  await gotoStable(page);

  // 5 selected on page 1, then select-all on page 2: "5 of 12 ..." -> "10 of 12 ...", one
  // character wider in ANY font, so tabular figures cannot hide it -- only a layout that
  // does not depend on the text's width (the demo spanning its frame) can.
  await selectAllCheckbox(page).click();
  await expect(selectedSummary(page)).toHaveText("5 of 12 row(s) selected.");
  await nextButton(page).click();
  await expect(pageReadout(page)).toHaveText("Page 2 of 3");

  const before = await layoutBoxes(page, "Select all");
  await selectAllCheckbox(page).click();
  await expect(selectedSummary(page)).toHaveText("10 of 12 row(s) selected.");
  await page.mouse.move(0, 0);
  expectSameBoxes(before, await layoutBoxes(page, "Select all"), "selecting page 2");
});

test.describe("Axe automated scan", () => {
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await goto(page);
    await expect(table(page)).toBeVisible();
    await expectNoAxeViolations(page, "data_table: loaded", {
      excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT],
    });
  });

  test("rows-per-page select open has no automatically detectable a11y issues", async ({ page }) => {
    await goto(page);
    await pageSizeControl(page).click();
    await expect(page.getByRole("listbox")).toBeVisible();
    await expectNoAxeViolations(page, "data_table: rows-per-page select open", {
      excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT],
    });
  });
});
