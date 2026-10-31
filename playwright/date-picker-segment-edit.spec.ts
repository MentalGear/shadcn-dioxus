/**
 * DatePicker segments are `contenteditable` spans whose text is a vdom child (backlog row 473). The vdom is the
 * only writer of that text: a browser editing default action (Backspace/Delete over a selection, an IME or
 * virtual-keyboard insert, paste, cut, a formatting shortcut) must neither change what the segment shows
 * nor leave its text node detached from the vdom.
 *
 * The invariant every test ends on, `expectVdomOwnsText`:
 *   1. the segment's only child is one text node, nothing a browser edit adds (`<br>`, `<b>`, a second text node);
 *   2. its text is the formatted `aria-valuenow` (or the placeholder when there is no value);
 *   3. the vdom still reaches it: ArrowUp rewrites the text through the same node tree (a detached node would
 *      keep the stale text on screen while `aria-valuenow` moved on).
 *
 * The main demo's segments are year (min 1925), month, day, empty at load, so typed digits show zero padded.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL = `${BASE_URL}/component/date_picker/`;
const NAV_TIMEOUT = 20 * 60 * 1000;

const segment = (page: Page, name: "year" | "month" | "day") =>
  page.locator("#component-preview-frame").getByRole("spinbutton", { name });

type Shape = { kids: string[]; text: string; now: string | null };

const shapeOf = (seg: Locator): Promise<Shape> =>
  seg.evaluate((el) => ({
    kids: Array.from(el.childNodes).map((n) => n.nodeName.toLowerCase()),
    text: el.textContent ?? "",
    now: el.getAttribute("aria-valuenow"),
  }));

/** Fills the year segment with 1925 (the minimum, so focus-leave clamping never moves it) and leaves it focused. */
async function typeYear(page: Page) {
  const year = segment(page, "year");
  await year.click();
  await page.keyboard.type("1925");
  await expect(year).toHaveText("1925");
  await expect(year).toHaveAttribute("aria-valuenow", "1925");
  await year.focus(); // typing 1925 advanced focus to the month; back on the year with a fresh buffer
  await expect(year).toBeFocused();
  return year;
}

/** Selects the whole content of the focused segment, the way a double or triple click would. */
const selectAll = (seg: Locator) =>
  seg.evaluate((el) => {
    const range = document.createRange();
    range.selectNodeContents(el);
    const sel = window.getSelection()!;
    sel.removeAllRanges();
    sel.addRange(range);
  });

async function expectVdomOwnsText(page: Page, seg: Locator) {
  // The state settles one render after the event; poll until the single-text-node shape holds.
  await expect
    .poll(async () => (await shapeOf(seg)).kids, { message: "the segment holds exactly one text node" })
    .toEqual(["#text"]);
  const shape = await shapeOf(seg);
  const placeholder = shape.now === null;
  if (!placeholder) expect(shape.text).toBe(shape.now!.padStart(shape.text.length, "0"));
  // A write from the vdom must show: ArrowUp changes the value, so the text must follow it.
  const before = shape.text;
  await page.keyboard.press("ArrowUp");
  await expect.poll(async () => (await shapeOf(seg)).text).not.toBe(before);
  const after = await shapeOf(seg);
  expect(after.kids).toEqual(["#text"]);
  expect(after.text).toBe(after.now!.padStart(after.text.length, "0"));
}

test.describe("DatePicker segment text is owned by the vdom (backlog row 473)", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: NAV_TIMEOUT });
    await segment(page, "year").scrollIntoViewIfNeeded();
  });

  test("Backspace over a selected segment shows the value the key effect committed", async ({ page }) => {
    const year = await typeYear(page);
    await selectAll(year);
    await page.keyboard.press("Backspace");
    await expect(year).toHaveAttribute("aria-valuenow", "192");
    await expect(year).toHaveText("0192");
    await expectVdomOwnsText(page, year);
  });

  test("Delete over a selected segment shows the value the key effect committed", async ({ page }) => {
    const year = await typeYear(page);
    await selectAll(year);
    await page.keyboard.press("Delete");
    await expect(year).toHaveAttribute("aria-valuenow", "925");
    await expect(year).toHaveText("0925");
    await expectVdomOwnsText(page, year);
  });

  test("Backspace and Delete with a collapsed caret leave one text node the vdom writes", async ({ page }) => {
    const year = await typeYear(page);
    await page.keyboard.press("Backspace");
    await expect(year).toHaveText("0192");
    await page.keyboard.press("Delete");
    await expect(year).toHaveText("0092");
    await expectVdomOwnsText(page, year);
  });

  test("Delete on an untouched segment does nothing and the page keeps responding", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    const year = segment(page, "year");
    await year.click();
    await page.keyboard.press("Delete");
    await page.keyboard.press("Backspace");
    await expect(year).toHaveText("YYYY");
    await page.keyboard.press("ArrowUp");
    await expect(year).toHaveAttribute("aria-valuenow", /^\d+$/);
    expect(errors).toEqual([]);
  });

  test("text inserted without a key (an IME or virtual keyboard commit) is typed into the segment", async ({ page }) => {
    const year = segment(page, "year");
    await year.click();
    await page.keyboard.insertText("1");
    await page.keyboard.insertText("9");
    await expect(year).toHaveText("0019");
    await expect(year).toHaveAttribute("aria-valuenow", "19");
    await expectVdomOwnsText(page, year);
  });

  test("text inserted over a filled segment replaces the digits like a typed one", async ({ page }) => {
    const year = await typeYear(page);
    await page.keyboard.insertText("2");
    await expect(year).toHaveText("0002");
    await expectVdomOwnsText(page, year);
  });

  test("a non-digit insert changes nothing", async ({ page }) => {
    const year = await typeYear(page);
    await page.keyboard.insertText("x");
    await expect(year).toHaveText("1925");
    await expect(year).toHaveAttribute("aria-valuenow", "1925");
    await expectVdomOwnsText(page, year);
  });

  test("an IME composition on a segment ends with the segment showing its value", async ({ page }) => {
    const year = await typeYear(page);
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Input.imeSetComposition", { text: "7", selectionStart: 1, selectionEnd: 1 });
    await cdp.send("Input.insertText", { text: "7" });
    await expect.poll(async () => (await shapeOf(year)).kids).toEqual(["#text"]);
    const shape = await shapeOf(year);
    if (shape.now !== null) expect(shape.text).toBe(shape.now.padStart(4, "0"));
    await expectVdomOwnsText(page, year);
  });

  test("paste and cut shortcuts do not edit the segment", async ({ page, context }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await page.evaluate(() => navigator.clipboard.writeText("55"));
    const year = await typeYear(page);
    await selectAll(year);
    await page.keyboard.press("Control+V");
    await expect(year).toHaveText("1925");
    await selectAll(year);
    await page.keyboard.press("Control+X");
    await expect(year).toHaveText("1925");
    await expect(year).toHaveAttribute("aria-valuenow", "1925");
    await expectVdomOwnsText(page, year);
  });

  test("a formatting shortcut does not wrap the segment text in an element", async ({ page }) => {
    const year = await typeYear(page);
    await selectAll(year);
    for (const key of ["Control+B", "Control+I", "Control+U"]) await page.keyboard.press(key);
    expect((await shapeOf(year)).kids).toEqual(["#text"]);
    await expect(year).toHaveText("1925");
    await expectVdomOwnsText(page, year);
  });

  test("Control+Z after typing does not rewind the text behind the vdom", async ({ page }) => {
    const year = await typeYear(page);
    await page.keyboard.press("Control+Z");
    await expect(year).toHaveText("1925");
    await expect(year).toHaveAttribute("aria-valuenow", "1925");
    await expectVdomOwnsText(page, year);
  });
});
