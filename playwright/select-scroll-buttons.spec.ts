import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { awaitAnimationsSettled, measureAtRest } from "./animations";
import { fixedViewportBox } from "./viewport-box";

// Row 268: Select scroll buttons. The `scroll` variant has 24 options, more than the popup's 18rem max-height shows.

const trigger = (page: Page) => page.getByRole("combobox", { name: "Time zone select" });
const list = (page: Page) => page.getByRole("listbox", { name: "Time zone options" });
const up = (page: Page) => list(page).locator('[data-slot="select-scroll-button"][data-direction="up"]');
const down = (page: Page) => list(page).locator('[data-slot="select-scroll-button"][data-direction="down"]');
const scrollTop = (l: Locator) => l.evaluate((el) => el.scrollTop);

async function open(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/component/select/`);
  await trigger(page).scrollIntoViewIfNeeded();
  await trigger(page).click();
  await expect(list(page)).toHaveAttribute("data-state", "open");
  await awaitAnimationsSettled(list(page));
}

test("the list scrolls inside the popup and only the down button shows at the top", async ({ page }) => {
  await open(page);
  const scrolls = await list(page).evaluate((el) => el.scrollHeight > el.clientHeight + 10);
  expect(scrolls, "24 options do not fit in the popup").toBe(true);
  await expect(down(page)).toHaveAttribute("data-visible", "true");
  await expect(up(page)).toHaveAttribute("data-visible", "false");
  await expect(down(page)).toBeVisible();
  await expect(up(page)).toBeHidden();
  // Hidden from assistive technology, and not a listbox child the options tree has to account for.
  await expect(down(page)).toHaveAttribute("aria-hidden", "true");
});

test("the buttons sit on the list's edges and never move an option", async ({ page }) => {
  await open(page);
  const first = list(page).getByRole("option").first();
  const before = await measureAtRest(first);
  const l = await measureAtRest(list(page));
  const d = await measureAtRest(down(page));
  expect(Math.abs(d.y + d.height - (l.y + l.height - 1)), "down button is at the list's bottom edge").toBeLessThanOrEqual(8);
  // Scroll to the bottom: the up button appears; the options' layout is the list's scroll, not the button's.
  await list(page).evaluate((el) => (el.scrollTop = 40));
  await expect(up(page)).toHaveAttribute("data-visible", "true");
  const u = await measureAtRest(up(page));
  expect(Math.abs(u.y - l.y), "up button is at the list's top edge").toBeLessThanOrEqual(8);
  await list(page).evaluate((el) => (el.scrollTop = 0));
  await expect(up(page)).toHaveAttribute("data-visible", "false");
  const after = await measureAtRest(first);
  expect(after.y).toBeCloseTo(before.y, 0);
});

test("scrolling to the end swaps which button shows", async ({ page }) => {
  await open(page);
  await list(page).evaluate((el) => (el.scrollTop = el.scrollHeight));
  await expect(down(page)).toHaveAttribute("data-visible", "false");
  await expect(up(page)).toHaveAttribute("data-visible", "true");
});

test("hovering the down button scrolls the list, leaving stops it, and it stops at the end", async ({ page }) => {
  await open(page);
  const l = list(page);
  expect(await scrollTop(l)).toBe(0);
  const box = await measureAtRest(down(page));
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect.poll(() => scrollTop(l), { timeout: 5000 }).toBeGreaterThan(40);
  // Away from the button: the scroll holds still.
  await page.mouse.move(box.x + box.width / 2, box.y - 120);
  const held = await scrollTop(l);
  await page.waitForTimeout(250);
  expect(Math.abs((await scrollTop(l)) - held), "leaving stops the auto-scroll").toBeLessThanOrEqual(1);
  // Back on it, all the way down: it ends at the bottom and the button hides.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect(down(page)).toHaveAttribute("data-visible", "false", { timeout: 15_000 });
  const max = await l.evaluate((el) => el.scrollHeight - el.clientHeight);
  expect(Math.abs((await scrollTop(l)) - max)).toBeLessThanOrEqual(2);
});

test("hovering the up button scrolls back to the top", async ({ page }) => {
  await open(page);
  const l = list(page);
  await l.evaluate((el) => (el.scrollTop = el.scrollHeight));
  await expect(up(page)).toHaveAttribute("data-visible", "true");
  const box = await measureAtRest(up(page));
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect(up(page)).toHaveAttribute("data-visible", "false", { timeout: 15_000 });
  expect(await scrollTop(l)).toBeLessThanOrEqual(2);
});

test("a list that fits shows no scroll button, and a closed select renders none", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/select/`);
  await expect(page.locator('[data-slot="select-scroll-button"]')).toHaveCount(0);
  const sizes = page.getByRole("combobox", { name: "Small size select" });
  await sizes.scrollIntoViewIfNeeded();
  await sizes.click();
  const l = page.getByRole("listbox", { name: "Small size options" });
  await expect(l).toHaveAttribute("data-state", "open");
  await expect(l.locator('[data-slot="select-scroll-button"][data-visible="true"]')).toHaveCount(0);
});

/** A headless page cannot be backgrounded: override what the motion registry reads (`primitives/src/activity.rs`). */
async function setHidden(page: Page, hidden: boolean) {
  await page.evaluate((h) => {
    Object.defineProperty(document, "visibilityState", { configurable: true, get: () => (h ? "hidden" : "visible") });
    Object.defineProperty(document, "hidden", { configurable: true, get: () => h });
    document.dispatchEvent(new Event("visibilitychange"));
  }, hidden);
}

test("a hidden tab pauses the hover auto-scroll and a visible one resumes it", async ({ page }) => {
  await open(page);
  const l = list(page);
  const box = await measureAtRest(down(page));
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect.poll(() => scrollTop(l), { timeout: 5000 }).toBeGreaterThan(20);
  await setHidden(page, true);
  await expect(down(page)).toHaveAttribute("data-dx-motion", "paused");
  const held = await scrollTop(l);
  await page.waitForTimeout(250);
  expect(Math.abs((await scrollTop(l)) - held), "no scrolling while the tab is hidden").toBeLessThanOrEqual(1);
  await setHidden(page, false);
  await expect(down(page)).not.toHaveAttribute("data-dx-motion", "paused");
  await expect.poll(() => scrollTop(l), { timeout: 5000 }).toBeGreaterThan(held + 20);
});

// Reduced motion: the hover scroll is the user's own pointer driving the list (like holding an arrow key),
// not decorative motion, so it keeps working; the buttons still show and hide with the scroll position.
test("under reduced motion the buttons show and hovering scrolls", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await open(page);
  const l = list(page);
  await expect(down(page)).toHaveAttribute("data-visible", "true");
  await expect(up(page)).toHaveAttribute("data-visible", "false");
  const box = await measureAtRest(down(page));
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect.poll(() => scrollTop(l), { timeout: 5000 }).toBeGreaterThan(20);
  await expect(up(page)).toHaveAttribute("data-visible", "true");
});

// Row 268: position="item-aligned". The `aligned` variant opens with "UTC+5" selected in the 24-option list.
test.describe("position=item-aligned", () => {
  const aligned = (page: Page) => page.getByRole("combobox", { name: "Aligned zone select" });
  const alignedList = (page: Page) => page.getByRole("listbox", { name: "Aligned zone options" });

  async function openAligned(page: Page) {
    await gotoHydrated(page, `${BASE_URL}/component/select/`);
    await aligned(page).scrollIntoViewIfNeeded();
    await aligned(page).click();
    await expect(alignedList(page)).toHaveAttribute("data-state", "open");
    await expect(alignedList(page)).toHaveAttribute("data-position", "item-aligned");
    await awaitAnimationsSettled(alignedList(page));
    await expect(alignedList(page)).toHaveAttribute("data-placed", "true");
  }

  test("the selected option sits on the trigger's line and the list stays in the viewport", async ({ page }) => {
    // The 24-option list is ~634px tall with the selected option ~447px below its top, so the exact
    // alignment needs the list's top >= the 8px viewport margin: a shorter viewport clamps the list at
    // the margin (it then has nothing to scroll and the option cannot reach the trigger's line), 1000px
    // leaves ~50px of slack.
    await page.setViewportSize({ width: 1000, height: 1000 });
    await gotoHydrated(page, `${BASE_URL}/component/select/`);
    // Put the trigger mid-viewport so there is room above and below for the alignment to be exact.
    await aligned(page).evaluate((el) => {
      // rest-ok: the trigger does not animate
      const r = el.getBoundingClientRect();
      // viewport-ok: only scrolls the page, no overlay is bounded by it
      window.scrollBy(0, r.top - window.innerHeight / 2);
    });
    await aligned(page).click();
    await expect(alignedList(page)).toHaveAttribute("data-placed", "true");
    await awaitAnimationsSettled(alignedList(page));
    const selected = alignedList(page).getByRole("option", { selected: true });
    await expect(selected).toHaveText("UTC+5");
    const t = await measureAtRest(aligned(page));
    const o = await measureAtRest(selected);
    const l = await measureAtRest(alignedList(page));
    expect(Math.abs(o.y + o.height / 2 - (t.y + t.height / 2)), "option centre on trigger centre").toBeLessThanOrEqual(2);
    expect(l.width).toBeGreaterThanOrEqual(t.width - 1);
    const vp = await fixedViewportBox(page);
    expect(l.y).toBeGreaterThanOrEqual(vp.y - 0.5);
    expect(l.y + l.height).toBeLessThanOrEqual(vp.y + vp.height + 0.5);
    expect(l.x).toBeGreaterThanOrEqual(vp.x - 0.5);
    expect(l.x + l.width).toBeLessThanOrEqual(vp.x + vp.width + 0.5);
  });

  test("a trigger at the viewport's edge keeps the list inside it and the scroll buttons work", async ({ page }) => {
    await page.setViewportSize({ width: 800, height: 420 });
    await gotoHydrated(page, `${BASE_URL}/component/select/`);
    // Trigger near the bottom edge: the list cannot centre on it, so it is pushed up and the option scrolled.
    await aligned(page).evaluate((el) => {
      // rest-ok: the trigger does not animate
      const r = el.getBoundingClientRect();
      // viewport-ok: only scrolls the page, no overlay is bounded by it
      window.scrollBy(0, r.top - (window.innerHeight - 60));
    });
    await aligned(page).click();
    await expect(alignedList(page)).toHaveAttribute("data-placed", "true");
    await awaitAnimationsSettled(alignedList(page));
    const l = await measureAtRest(alignedList(page));
    const vp = await fixedViewportBox(page);
    expect(l.y).toBeGreaterThanOrEqual(vp.y - 0.5);
    expect(l.y + l.height).toBeLessThanOrEqual(vp.y + vp.height + 0.5);
    // Still an overflowing list: its edge buttons keep reporting the scroll state.
    const up = alignedList(page).locator('[data-slot="select-scroll-button"][data-direction="up"]');
    const down = alignedList(page).locator('[data-slot="select-scroll-button"][data-direction="down"]');
    const top = await scrollTop(alignedList(page));
    await expect(up).toHaveAttribute("data-visible", top > 1 ? "true" : "false");
    await expect(down).toHaveAttribute("data-visible", "true");
    // The selected option is still visible inside the list.
    const o = await measureAtRest(alignedList(page).getByRole("option", { selected: true }));
    expect(o.y).toBeGreaterThanOrEqual(l.y - 1);
    expect(o.y + o.height).toBeLessThanOrEqual(l.y + l.height + 1);
  });

  // No selected option: nothing to align, so the whole list sits beside the trigger, flipping on collision.
  const placeholder = (page: Page) => page.getByRole("combobox", { name: "Placeholder zone select" });
  const placeholderList = (page: Page) => page.getByRole("listbox", { name: "Placeholder zone options" });

  for (const [edge, side] of [
    ["bottom", "top"],
    ["top", "bottom"],
  ] as const) {
    test(`with no selection, a trigger at the ${edge} edge opens the whole list on its ${side} side`, async ({ page }) => {
      await page.setViewportSize({ width: 800, height: 420 });
      await gotoHydrated(page, `${BASE_URL}/component/select/`);
      await placeholder(page).evaluate((el, edge) => {
        // The demo sits near the page's end: room below it so the page can scroll it to the top edge
        // (a spacer, not body padding: the open select's scroll lock rewrites the body's style), and 80px,
        // not less, keeps it clear of the sticky site header, which would make the click scroll it back.
        const spacer = document.createElement("div");
        spacer.style.height = "100vh";
        document.body.appendChild(spacer);
        // rest-ok: the trigger does not animate
        const r = el.getBoundingClientRect();
        // viewport-ok: only scrolls the page, no overlay is bounded by it
        window.scrollBy(0, edge === "bottom" ? r.top - (window.innerHeight - 60) : r.top - 80);
      }, edge);
      await placeholder(page).click();
      await expect(placeholderList(page)).toHaveAttribute("data-placed", "true");
      await expect(placeholderList(page)).toHaveAttribute("data-side", side);
      await awaitAnimationsSettled(placeholderList(page));
      const t = await measureAtRest(placeholder(page));
      const l = await measureAtRest(placeholderList(page));
      const vp = await fixedViewportBox(page);
      if (side === "top") expect(l.y + l.height, "list ends above the trigger").toBeLessThanOrEqual(t.y + 0.5);
      else expect(l.y, "list starts below the trigger").toBeGreaterThanOrEqual(t.y + t.height - 0.5);
      expect(l.y).toBeGreaterThanOrEqual(vp.y - 0.5);
      expect(l.y + l.height).toBeLessThanOrEqual(vp.y + vp.height + 0.5);
      // Capped to the chosen side's room, the 24 options still scroll.
      await expect(placeholderList(page).locator('[data-slot="select-scroll-button"][data-direction="down"]')).toHaveAttribute(
        "data-visible",
        "true",
      );
    });
  }

  test("selecting works and the popper select keeps its position", async ({ page }) => {
    await openAligned(page);
    await alignedList(page).getByRole("option", { name: "UTC+8" }).click();
    await expect(aligned(page)).toContainText("UTC+8");
    await trigger(page).scrollIntoViewIfNeeded();
    await trigger(page).click();
    await expect(list(page)).toHaveAttribute("data-position", "popper");
    await expect(list(page)).not.toHaveAttribute("data-placed", "true");
  });
});
