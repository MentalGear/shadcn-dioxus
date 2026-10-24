import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/calendar/`, {
    timeout: 20 * 60 * 1000,
  }); // Increase timeout to 20 minutes
  await page.waitForLoadState('networkidle');

  const calendar = page.locator("#component-preview-frame").first();
  const prevButton = calendar.getByRole("button").first();
  const nextButton = calendar.getByRole("button").nth(1);

  // Assert the calendar is displayed
  await expect(calendar).toBeVisible({ timeout: 30000 });
  // Assert the current month is displayed
  const currentMonth = calendar.locator("select").first();
  let currentMonthText = await currentMonth.inputValue();

  // Click the previous button to go to the previous month
  await prevButton.click();
  // Assert the month has changed
  let previousMonthText = await currentMonth.inputValue();
  expect(previousMonthText).not.toBe(currentMonthText);

  // Click the next button to go back to the current month
  await nextButton.click();
  // Assert the month has changed back to the current month
  await expect(currentMonth).toHaveValue(currentMonthText);

  // Move focus to the calendar with tab
  await page.keyboard.press("Tab");
  const focusedDay = calendar.locator(
    '[data-month="current"]:focus'
  );
  // Assert a day is focused
  const firstDay = focusedDay.first();
  // Get the day
  const day = await firstDay.textContent();
  const dayNumber = parseInt(day || "", 10);
  // Pressing right arrow should move focus to the next day
  await page.keyboard.press("ArrowRight");
  const nextDay = focusedDay.first();
  // Assert the next day is focused
  const nextDayNumber = parseInt((await nextDay.textContent()) || "", 10);
  let current_date = new Date();
  let daysInMonth = new Date(
    current_date.getFullYear(),
    current_date.getMonth() + 1,
    0
  ).getDate();
  if (dayNumber + 1 > daysInMonth) {
    // If the next day is in the next month, it should wrap around
    expect(nextDayNumber).toBe(1);
  } else {
    expect(nextDayNumber).toBe(dayNumber + 1);
  }
  // Pressing left arrow should move focus back to the original day
  await page.keyboard.press("ArrowLeft");
  await expect(focusedDay.first()).toContainText(day || "failure");
  // Pressing down arrow should move focus to the next week
  await page.keyboard.press("ArrowDown");
  const nextWeekDay = focusedDay.first();
  // Assert the next week day is focused
  const nextWeekDayNumber = parseInt(
    (await nextWeekDay.textContent()) || "",
    10
  );
  if (dayNumber + 7 > daysInMonth) {
    // If the next week day is in the next month, it should wrap around
    expect(nextWeekDayNumber).toBe(dayNumber + 7 - daysInMonth);
  } else {
    expect(nextWeekDayNumber).toBe(dayNumber + 7);
  }
  // Pressing up arrow should move focus back to the original day
  await page.keyboard.press("ArrowUp");
  await expect(focusedDay.first()).toContainText(day || "failure");
});

test("year navigation by moving 52 weeks with arrow keys", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/calendar/`, {
    timeout: 20 * 60 * 1000,
  });

  const calendar = page.locator("#component-preview-frame").first();
  const monthSelect = calendar.locator("select").first();
  const yearSelect = calendar.locator("select").nth(1);

  // Assert the calendar is displayed
  await expect(calendar).toBeVisible({ timeout: 30000 });

  // Get the initial month and year
  const initialMonth = await monthSelect.inputValue();
  const initialYear = await yearSelect.inputValue();
  const initialYearNumber = parseInt(initialYear, 10);
  const initialMonthNumber = parseInt(initialMonth, 10);

  // Calculate the exact number of weeks needed to move to the next year
  // Start from the first day of the current month
  const startDate = new Date(initialYearNumber, initialMonthNumber - 1, 1);
  // Calculate the same date next year
  const targetDate = new Date(initialYearNumber + 1, initialMonthNumber - 1, 1);
  // Calculate the difference in days
  const daysDifference = Math.floor(
    (targetDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24)
  );
  // Calculate the number of weeks (round to nearest week)
  const weeksToMove = Math.ceil(daysDifference / 7);

  // Move focus to the calendar manually
  const firstDay = calendar
    .locator('[data-month="current"]')
    .first();
  await firstDay.focus();

  // Press ArrowDown the calculated number of times to move forward by one year
  for (let i = 0; i < weeksToMove; i++) {
    await page.keyboard.press("ArrowDown");
  }

  // Assert the year has changed to the next year
  const nextYear = await yearSelect.inputValue();
  const nextYearNumber = parseInt(nextYear, 10);
  expect(nextYearNumber).toBe(initialYearNumber + 1);

  // The month should be exactly the same
  const nextMonth = await monthSelect.inputValue();
  expect(nextMonth).toBe(initialMonth);

  // Press ArrowUp the same number of times to move back by one year
  for (let i = 0; i < weeksToMove; i++) {
    await page.keyboard.press("ArrowUp");
  }

  // Assert the year has changed back to the original year
  const currentYear = await yearSelect.inputValue();
  expect(currentYear).toBe(initialYear);

  // The month should be exactly the same
  const currentMonth = await monthSelect.inputValue();
  expect(currentMonth).toBe(initialMonth);
});

test("shift + arrow keys navigation", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/calendar/`, {
    timeout: 20 * 60 * 1000,
  });

  const calendar = page.locator("#component-preview-frame").first();
  const monthSelect = calendar.locator("select").first();
  const yearSelect = calendar.locator("select").nth(1);

  // Assert the calendar is displayed
  await expect(calendar).toBeVisible({ timeout: 30000 });

  // Get the initial month and year
  const initialMonth = await monthSelect.inputValue();
  const initialYear = await yearSelect.inputValue();
  const initialYearNumber = parseInt(initialYear, 10);
  const initialMonthNumber = parseInt(initialMonth, 10);

  // Move focus to the calendar
  const firstDay = calendar
    .locator('[data-month="current"]')
    .first();
  await firstDay.focus();

  // Test Shift + ArrowDown - should move forward by one month
  await page.keyboard.press("Shift+ArrowDown");

  let currentMonth = await monthSelect.inputValue();
  let currentYear = await yearSelect.inputValue();
  let expectedMonth = initialMonthNumber === 12 ? 1 : initialMonthNumber + 1;
  let expectedYear =
    initialMonthNumber === 12 ? initialYearNumber + 1 : initialYearNumber;

  expect(parseInt(currentMonth, 10)).toBe(expectedMonth);
  expect(parseInt(currentYear, 10)).toBe(expectedYear);

  // Test Shift + ArrowUp - should move back to the initial month
  await page.keyboard.press("Shift+ArrowUp");

  currentMonth = await monthSelect.inputValue();
  currentYear = await yearSelect.inputValue();
  expect(currentMonth).toBe(initialMonth);
  expect(currentYear).toBe(initialYear);
});

async function testArrowKeyNavigation(
  page: any,
  arrowKey: "ArrowRight" | "ArrowLeft",
  startPosition: "first" | "last",
  expectedOrder: "ascending" | "descending"
) {
  await gotoHydrated(page, `${BASE_URL}/component/calendar/`, {
    timeout: 20 * 60 * 1000,
  });

  const calendar = page.locator("#component-preview-frame").first();
  const monthSelect = calendar.locator("select").first();
  const yearSelect = calendar.locator("select").nth(1);

  // Assert the calendar is displayed
  await expect(calendar).toBeVisible({ timeout: 30000 });

  // Get the current month and year to calculate days in month
  const currentMonthValue = await monthSelect.inputValue();
  const currentYearValue = await yearSelect.inputValue();
  const monthNumber = parseInt(currentMonthValue, 10);
  const yearNumber = parseInt(currentYearValue, 10);

  // Calculate the number of days in the current month
  const daysInMonth = new Date(yearNumber, monthNumber, 0).getDate();

  // Move focus to the starting day of the current month
  const startDay = calendar
    .locator('[data-month="current"]')
    [startPosition]();
  await startDay.focus();

  // Get the focused day selector
  const focusedDay = calendar.locator(
    '[data-month="current"]:focus'
  );

  // Array to track all days visited
  const daysVisited = [];

  // Get the starting day number
  let dayText = await focusedDay.first().textContent();
  let dayNumber = parseInt(dayText || "", 10);
  daysVisited.push(dayNumber);

  // Press arrow key to navigate through all remaining days of the month
  for (let i = 1; i < daysInMonth; i++) {
    await page.keyboard.press(arrowKey);

    // Get the new focused day
    dayText = await focusedDay.first().textContent();
    dayNumber = parseInt(dayText || "", 10);
    daysVisited.push(dayNumber);
  }

  // Assert that we visited the correct number of days
  expect(daysVisited.length).toBe(daysInMonth);

  // Sort the days visited to check we got all days from 1 to daysInMonth
  const sortedDays = [...daysVisited].sort((a, b) => a - b);

  // Create the expected array [1, 2, 3, ..., daysInMonth]
  const expectedDays = Array.from({ length: daysInMonth }, (_, i) => i + 1);

  // Assert that we visited every day exactly once
  expect(sortedDays).toEqual(expectedDays);

  // Verify we traversed in the expected order
  if (expectedOrder === "ascending") {
    expect(daysVisited).toEqual(expectedDays);
  } else {
    const expectedReverseDays = Array.from(
      { length: daysInMonth },
      (_, i) => daysInMonth - i
    );
    expect(daysVisited).toEqual(expectedReverseDays);
  }
}

test("right arrow key navigates through all days of the month", async ({
  page,
}) => {
  await testArrowKeyNavigation(page, "ArrowRight", "first", "ascending");
});

test.describe("Axe automated scan", () => {
  // Calendar renders inline (no overlay); a day is focused/"selected" by
  // keyboard, so that state, plus the plain loaded state, are scanned.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    await expectNoAxeViolations(page, "calendar: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("a day focused via keyboard has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    const calendar = page.locator("#component-preview-frame").first();
    // Click the prev/next nav buttons first, exactly as the file's own main
    // test does, so Tab lands inside the calendar grid rather than on the
    // first tabbable element on the whole page (e.g. the docs sidebar).
    const prevButton = calendar.getByRole("button").first();
    const nextButton = calendar.getByRole("button").nth(1);
    await prevButton.click();
    await nextButton.click();
    await page.keyboard.press("Tab");
    await expect(page.locator('[data-month="current"]:focus').first()).toBeVisible();
    await expectNoAxeViolations(page, "calendar: a day focused", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

test("left arrow key navigates through all days of the month in reverse", async ({
  page,
}) => {
  await testArrowKeyNavigation(page, "ArrowLeft", "last", "descending");
});

test.describe("selected/range state outranks :hover (user report, calendar-state lane)", () => {
  // Dark mode specifically: the bug this guards against (line255's old
  // `[data-selected="true"]:hover` re-declaring background-color/color
  // instead of leaving the plain [data-selected="true"] rule in place) only
  // showed up in dark mode -- its light-mode branch happened to match the
  // plain selected color already. Light mode alone would stay green on the
  // unfixed code and not actually guard against a regression.
  test("a selected day keeps its selected background while the pointer is still over it (dark mode)", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/?dark_mode=true`, {
      timeout: 20 * 60 * 1000,
    });
    await page.waitForLoadState("networkidle");

    const calendar = page.locator("#component-preview-frame").first();
    const day = calendar
      .locator('.dx-calendar-grid-cell[data-month="current"]:not([data-disabled="true"])')
      .first();
    const otherDay = calendar
      .locator('.dx-calendar-grid-cell[data-month="current"]:not([data-disabled="true"])')
      .nth(1);

    // Read the plain (unselected) hover color first, from a day that will
    // never be selected in this test -- this is the "wrong answer" the bug
    // produces on the day that WAS just selected.
    await otherDay.hover();
    const plainHoverBg = await otherDay.evaluate((el) => getComputedStyle(el).backgroundColor);

    // Click a day: Playwright's .click() leaves the pointer resting on the
    // element afterwards, exactly the "clicked it, mouse hasn't moved" case
    // the user asked for ("date picker: active state on selected date must
    // be over hover state, otherwise active states are not visible after
    // click").
    await day.click();
    await expect(day).toHaveAttribute("data-selected", "true");
    expect(await day.evaluate((el) => el.matches(":hover"))).toBe(true);
    const selectedAndHoveredBg = await day.evaluate((el) => getComputedStyle(el).backgroundColor);

    // Move the pointer off the day and read the "true" selected background.
    const dayHeader = calendar.locator(".dx-calendar-grid-day-header").first();
    await dayHeader.hover();
    const selectedAtRestBg = await day.evaluate((el) => getComputedStyle(el).backgroundColor);

    expect(selectedAndHoveredBg).toBe(selectedAtRestBg); // selected look survives hover
    expect(selectedAndHoveredBg).not.toBe(plainHoverBg); // and isn't just the plain hover color
    // Still visually selected once the pointer has moved on, in the
    // strongest sense: the data-selected attribute AND its rendered color.
    await expect(day).toHaveAttribute("data-selected", "true");
  });

  // A range's in-between days (not an endpoint) have their own hover companion: without it the generic
  // day-hover rule would out-rank [data-selection-between="true"] on specificity and a hovered in-range day
  // would look like a plain single-selected endpoint instead of keeping its muted "between" look.
  // /component/block/... isolates one variant per page, avoiding the multi-panel ambiguity every other test
  // in this file works around with #component-preview-frame.first().
  //
  // dev-docs/backlog.md row 113 -- the locator below ("waiting for
  // locator(...:not([data-disabled="true"])).nth(4)") must resolve on every lane and on every
  // day. It does not depend on today's date: the `range` demo fixes its own `view_date` to
  // 2026-05-15 and its min/max (1995-2035) leave May 2026 entirely available, so the locator
  // resolves to all 31 May cells whatever day this runs on. What makes it wait out the whole
  // test timeout, deterministically, is the URL: `/component/block/calendar/range/` (the
  // path-segment form) is only ever served by the dev server's SPA fallback; the static SSG
  // server has no prerendered file for it and answers 404, so on that lane the calendar is
  // never rendered and the first `.click()` waits out the timeout. Two measures keep both
  // causes out:
  //  1. the legacy query form `/component/block/?name=calendar&variant=range&`,
  //     which both lanes serve (every other block-page spec here uses it);
  //  2. the date is pinned with Playwright's clock (`Date`/`new Date()` only --
  //     timers keep running) to a day OUTSIDE the demo's May 2026 view, so
  //     `data-today` can never land on one of the cells below whatever day this
  //     runs on, and a precondition (31 cells) fails in seconds, with a clear
  //     message, instead of burning the timeout.
  test("a range's in-between day keeps its muted look while hovered, not the endpoint look", async ({ page }) => {
    await page.clock.setFixedTime(new Date("2026-09-26T12:00:00Z"));
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=calendar&variant=range&`, {
      timeout: 20 * 60 * 1000,
    });
    await page.waitForLoadState("networkidle");

    const days = page.locator('.dx-calendar-grid-cell[data-month="current"]:not([data-disabled="true"])');
    // May 2026 (the demo's own fixed view): 31 current-month days. Fail fast, and say why.
    await expect(days, "the range demo should render May 2026's 31 days").toHaveCount(31, { timeout: 30_000 });
    const start = days.nth(4); // day "5"
    const end = days.nth(11); // day "12"
    const between = days.nth(7); // day "8", strictly inside 5..12

    await start.click();
    await end.click();
    await expect(between).toHaveAttribute("data-selection-between", "true");

    const restBg = await between.evaluate((el) => getComputedStyle(el).backgroundColor);
    const endpointBg = await start.evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(restBg).not.toBe(endpointBg); // between and endpoint are meant to look different

    await between.hover();
    const hoveredBg = await between.evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(hoveredBg).toBe(restBg); // hovering must not snap it to the endpoint look
    expect(hoveredBg).not.toBe(endpointBg);
  });
});

test.describe("aria-disabled on unavailable/disabled days -- backlog row 84 finding 1", () => {
  test("a day inside a disabled range carries aria-disabled=true; an available day still carries an explicit aria-disabled=false", async ({ page }) => {
    // Unavailable days have no native `disabled` attribute (deliberately, so
    // the grid keeps a single roving tabstop -- `handle_day_select`
    // early-returns instead); with only `data-disabled`/`data-unavailable`
    // plus CSS, assistive tech would have no signal at all that the day
    // cannot be chosen. So `calendar_day_attributes` (primitives/src/
    // calendar.rs) emits `aria-disabled`, mirroring `data-disabled` exactly
    // (same source value, unconditionally present) -- `aria-disabled` (unlike a real `disabled`
    // attribute) does not remove the element from the tab order, matching
    // this crate's own "focusable but not operable" convention elsewhere
    // (e.g. combobox/select/command options).
    //
    // `unavailable_dates`'s own demo (preview/src/components/calendar/
    // variants/unavailable_dates/mod.rs) opens directly on May 2026 with
    // 2026-05-15..2026-05-18 disabled, so no navigation is needed and
    // "Friday, May 15, 2026" is a fixed, known-unavailable date (matches
    // date-picker.spec.ts's own already-verified value for the same date).
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");

    const frame = page.locator("#component-preview-frame-unavailable_dates");
    const may15 = frame.locator('[data-unavailable="true"]').first();
    await expect(may15).toHaveAttribute("aria-label", "Friday, May 15, 2026");
    await expect(may15).toHaveAttribute("aria-disabled", "true");
    // Not a native disabled control -- still focusable/tabbable, per the
    // roving-tabstop design the calendar keeps for unavailable dates.
    await expect(may15).toHaveJSProperty("disabled", false);

    const may20 = frame
      .locator('.dx-calendar-grid-cell[data-month="current"]')
      .filter({ hasText: /^20$/ });
    await expect(may20).not.toHaveAttribute("data-unavailable", "true");
    await expect(may20).toHaveAttribute("aria-disabled", "false");
  });
});

/**
 * Caption layouts and week numbers (backlog row 155 group 8: shadcn's `captionLayout` and
 * `showWeekNumber`). The plain `simple` variant is the default caption (`label`); `caption` shows
 * all four layouts side by side; `week_numbers` is `showWeekNumber`.
 */
test.describe("caption layouts (captionLayout)", () => {
  const caption = (page: any, name: string) => page.locator(`[data-caption-layout="${name}"]`);

  async function gotoCaptionVariant(page: any) {
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=calendar&variant=caption&`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    await expect(caption(page, "Label")).toBeVisible({ timeout: 30000 });
  }

  test("the default is a plain label: 'May 2026' between the arrows, no selects", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=calendar&variant=simple&`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    const calendar = page.locator(".dx-calendar").first();
    await expect(calendar).toBeVisible({ timeout: 30000 });
    await expect(calendar.locator("select")).toHaveCount(0);
    const title = calendar.locator(".dx-calendar-month-title");
    await expect(title).toHaveText("May 2026");
    await expect(title).toHaveAttribute("role", "status");
    await expect(title).toHaveAttribute("aria-live", "polite");
    // The arrows still page it, and the label follows.
    await calendar.locator(".dx-calendar-nav-next").click();
    await expect(title).toHaveText("June 2026");
    await calendar.locator(".dx-calendar-nav-prev").click();
    await calendar.locator(".dx-calendar-nav-prev").click();
    await expect(title).toHaveText("April 2026");
  });

  test("label: no selects; dropdown: Month + Year; dropdown-months: Month + the year as text; dropdown-years: Year + the month as text", async ({ page }) => {
    await gotoCaptionVariant(page);

    const label = caption(page, "Label");
    await expect(label.locator("select")).toHaveCount(0);
    await expect(label.locator(".dx-calendar-month-title")).toHaveText("May 2026");

    const both = caption(page, "Dropdown");
    await expect(both.getByRole("combobox", { name: "Month" })).toHaveValue("5");
    await expect(both.getByRole("combobox", { name: "Year" })).toHaveValue("2026");
    await expect(both.locator("select")).toHaveCount(2);

    const months = caption(page, "Dropdown months");
    await expect(months.getByRole("combobox", { name: "Month" })).toHaveValue("5");
    await expect(months.getByRole("combobox", { name: "Year" })).toHaveCount(0);
    await expect(months.locator(".dx-calendar-caption-text")).toHaveText("2026");

    const years = caption(page, "Dropdown years");
    await expect(years.getByRole("combobox", { name: "Year" })).toHaveValue("2026");
    await expect(years.getByRole("combobox", { name: "Month" })).toHaveCount(0);
    await expect(years.locator(".dx-calendar-caption-text")).toHaveText("May");
  });

  test("the dropdowns are real selects: focusable, they move the grid, and the live region says where it landed", async ({ page }) => {
    await gotoCaptionVariant(page);
    const calendar = caption(page, "Dropdown");
    const month = calendar.getByRole("combobox", { name: "Month" });
    const year = calendar.getByRole("combobox", { name: "Year" });
    const firstDay = calendar.locator('.dx-calendar-grid-cell[data-month="current"]').first();

    await month.focus();
    await expect(month).toBeFocused();
    await month.selectOption("9");
    await expect(firstDay).toHaveAttribute("aria-label", /September 1, 2026$/);
    await year.selectOption("2031");
    await expect(firstDay).toHaveAttribute("aria-label", /September 1, 2031$/);
    // The visually hidden full caption is the polite live region for a select-driven change.
    const status = calendar.locator(".dx-calendar-sr-only");
    await expect(status).toHaveText("September 2031");
    await expect(status).toHaveAttribute("role", "status");
    await expect(status).toHaveAttribute("aria-live", "polite");
    // ...and it is not painted.
    const box = await status.boundingBox();
    expect(box === null || (box.width <= 1 && box.height <= 1)).toBe(true);

    // The plain-text half follows the same view.
    const months = caption(page, "Dropdown months");
    await months.getByRole("combobox", { name: "Month" }).selectOption("2");
    await expect(months.locator('.dx-calendar-grid-cell[data-month="current"]').first()).toHaveAttribute("aria-label", /February 1, 2026$/);
    await months.locator(".dx-calendar-nav-next").click();
    await expect(months.getByRole("combobox", { name: "Month" })).toHaveValue("3");
  });

  test("keyboard: Tab reaches Month then Year; a day key still moves within the grid", async ({ page }) => {
    await gotoCaptionVariant(page);
    const calendar = caption(page, "Dropdown");
    await calendar.locator(".dx-calendar-nav-prev").focus();
    await page.keyboard.press("Tab");
    await expect(calendar.getByRole("combobox", { name: "Month" })).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(calendar.getByRole("combobox", { name: "Year" })).toBeFocused();
    await page.keyboard.press("Tab"); // next-month arrow
    await page.keyboard.press("Tab"); // the grid's roving tab stop
    const focused = calendar.locator('[data-month="current"]:focus');
    await expect(focused).toHaveCount(1);
    const day = parseInt((await focused.textContent()) || "", 10);
    await page.keyboard.press("ArrowRight");
    await expect(focused).toHaveText(String(day + 1));
  });

  test("a year dropdown is limited to min_date..max_date", async ({ page }) => {
    await gotoCaptionVariant(page);
    const years = caption(page, "Dropdown").getByRole("combobox", { name: "Year" }).locator("option");
    // min_date is 1995-07-21 and the view is May: May 1995 is before it, so the first pick is 1996.
    await expect(years.first()).toHaveText("1996");
    await expect(years.last()).toHaveText("2035");
  });

  test("axe: the caption layouts have no automatically detectable a11y issues", async ({ page }) => {
    // The scan needs the full docs page (main + h1); its `caption` variant frame is the four layouts.
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    await expect(page.locator("#component-preview-frame-caption")).toBeVisible({ timeout: 30000 });
    await expectNoAxeViolations(page, "calendar: caption layouts", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

test.describe("week numbers (showWeekNumber)", () => {
  async function gotoWeekNumbers(page: any) {
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=calendar&variant=week_numbers&`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    await expect(page.locator(".dx-calendar").first()).toBeVisible({ timeout: 30000 });
  }

  test("a labelled week-number cell starts every row; the first week is 1 (the week holding January 1st)", async ({ page }) => {
    await gotoWeekNumbers(page);
    const rows = page.locator(".dx-calendar-grid-week");
    const count = await rows.count();
    expect(count).toBeGreaterThanOrEqual(5);
    for (let i = 0; i < count; i++) {
      const number = rows.nth(i).locator(".dx-calendar-grid-weeknum");
      await expect(number).toHaveText(String(i + 1));
      await expect(number).toHaveAttribute("aria-label", `Week ${i + 1}`);
      await expect(number).toHaveAttribute("role", "rowheader");
      // The week number is the FIRST cell, then seven days.
      await expect(rows.nth(i).locator("td")).toHaveCount(8);
      await expect(rows.nth(i).locator("td").first()).toHaveClass(/dx-calendar-grid-weeknum/);
    }
    // An empty header cell holds the column open above them.
    await expect(page.locator(".dx-calendar-grid-header .dx-calendar-grid-weeknum")).toHaveCount(1);
  });

  test("the column is a cell wide, sits in front of the days, and the month does not overflow", async ({ page }) => {
    await gotoWeekNumbers(page);
    const firstRow = page.locator(".dx-calendar-grid-week").first();
    const number = await firstRow.locator(".dx-calendar-grid-weeknum").evaluate((el) => el.getBoundingClientRect().toJSON());
    const firstDay = await firstRow.locator(".dx-calendar-grid-cell").first().evaluate((el) => el.getBoundingClientRect().toJSON());
    expect(Math.abs(number.width - 28)).toBeLessThanOrEqual(1);
    expect(firstDay.left, "days start after the week-number column").toBeGreaterThanOrEqual(number.right - 0.5);
    const overflow = await page.locator(".dx-calendar-view").first().evaluate((el) => el.scrollWidth - el.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    // The seven weekday headers still line up over the seven day columns.
    const headers = page.locator(".dx-calendar-grid-day-header");
    await expect(headers).toHaveCount(7);
    const h0 = await headers.first().evaluate((el) => el.getBoundingClientRect().left);
    expect(Math.abs(h0 - firstDay.left)).toBeLessThanOrEqual(1);
  });

  test("the day grid keeps its single tab stop and arrow keys", async ({ page }) => {
    await gotoWeekNumbers(page);
    const day = page.locator('.dx-calendar-grid-cell[data-month="current"]').first();
    await day.focus();
    await page.keyboard.press("ArrowRight");
    await expect(page.locator('[data-month="current"]:focus')).toHaveText("2");
    await page.keyboard.press("ArrowDown");
    await expect(page.locator('[data-month="current"]:focus')).toHaveText("9");
    // The week-number cells are not tab stops.
    expect(await page.locator(".dx-calendar-grid-weeknum[tabindex]").count()).toBe(0);
  });

  test("a calendar without showWeekNumber renders no column", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=calendar&variant=simple&`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    await expect(page.locator(".dx-calendar").first()).toBeVisible({ timeout: 30000 });
    await expect(page.locator(".dx-calendar-grid-weeknum")).toHaveCount(0);
    await expect(page.locator(".dx-calendar-grid-week").first().locator("td")).toHaveCount(7);
  });

  test("axe: week numbers have no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
    await expect(page.locator("#component-preview-frame-week_numbers .dx-calendar-grid-weeknum").first()).toBeVisible({ timeout: 30000 });
    await expectNoAxeViolations(page, "calendar: week numbers", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// dev-docs/backlog.md row 380: the page is prerendered on the build day. A calendar that read the clock
// while rendering kept the build day's "today" mark and the build year after hydration; both now come
// from the visitor's clock, read after mount. The clock is pinned (`Date` only, timers keep running).
test.describe("today is the visitor's date, not the build day", () => {
  // The prerendered page itself (the `?name=` block form renders on the client and would hide the bug).
  async function gotoFrozen(page: any, now: string) {
    await page.clock.setFixedTime(new Date(now));
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`, { timeout: 20 * 60 * 1000 });
    await page.waitForLoadState("networkidle");
  }
  const frame = (page: any, variant: string) => page.locator(`#component-preview-frame-${variant}`);

  test("the today mark follows the visitor's day (cell_size, which shows October 2026)", async ({ page }) => {
    await gotoFrozen(page, "2026-10-20T12:00:00Z");
    const today = frame(page, "cell_size").locator('.dx-calendar-grid-cell[data-today="true"]');
    await expect(today.first()).toHaveAttribute("aria-label", /October 20, 2026$/);
    for (const label of await today.evaluateAll((els) => els.map((el) => el.getAttribute("aria-label")))) {
      expect(label).toMatch(/October 20, 2026$/);
    }
  });

  test("week_numbers opens on January of the visitor's year", async ({ page }) => {
    await gotoFrozen(page, "2031-03-12T12:00:00Z");
    const calendar = frame(page, "week_numbers");
    await expect(calendar.locator(".dx-calendar-month-title").first()).toHaveText("January 2031");
    await expect(calendar.locator('.dx-calendar-grid-cell[data-selected="true"]')).toHaveAttribute("aria-label", /January 12, 2031$/);
  });
});
