import { test, expect } from "./fixtures";
import { type Locator, type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

/**
 * A toast is as tall as its content plus a constant padding (backlog row 376), never a fixed box:
 *  - its content sits inside it with the same padding top and bottom, whether it holds one line or many;
 *  - collapsed, every toast behind the front one takes the FRONT toast's height and hides its content,
 *    so the pile looks even whatever each one holds, and the peek offset is the gap;
 *  - expanded (hover or focus), each toast has its own height and the toasts do not overlap.
 * Layout heights are read as computed `height` (a transform such as the pile's `scale()` would change a
 * bounding box), at rest.
 */
const ML = "Multiline toast notifications";
const LOADING = "Loading toast notifications";

const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => gotoHydrated(page, `${BASE_URL}/component/toast/`);

/**
 * The toast's border box and the union of every element inside it, in one read: the padding the content
 * has above and below (border excluded) and whether anything is clipped by the box.
 */
function fit(toast: Locator) {
  return readAtRest(toast, (el) => {
    const box = el.getBoundingClientRect(); // rest-ok: read inside readAtRest
    const cs = getComputedStyle(el);
    let top = Infinity;
    let bottom = -Infinity;
    el.querySelectorAll("*").forEach((c) => {
      const r = c.getBoundingClientRect(); // rest-ok: read inside readAtRest
      if (r.height === 0) return;
      top = Math.min(top, r.top);
      bottom = Math.max(bottom, r.bottom);
    });
    const bt = parseFloat(cs.borderTopWidth);
    const bb = parseFloat(cs.borderBottomWidth);
    return {
      height: parseFloat(cs.height),
      padTop: top - box.top - bt,
      padBottom: box.bottom - bottom - bb,
      clipped: el.scrollHeight > el.clientHeight + 0.5,
    };
  });
}

/** The toast's layout height and the opacity of its content row (0 while a collapsed pile hides it). */
function size(toast: Locator) {
  return readAtRest(toast, (el) => {
    const cs = getComputedStyle(el);
    const body = el.querySelector("[data-toast-body]") as HTMLElement;
    return { height: parseFloat(cs.height), contentOpacity: parseFloat(getComputedStyle(body).opacity) };
  });
}

async function add(page: Page, button: string, count = 1) {
  const b = page.getByRole("button", { name: button });
  const before = await toasts(page, ML).count();
  for (let i = 0; i < count; i++) await b.click();
  await expect(toasts(page, ML)).toHaveCount(before + count);
}

test.describe("toast: height follows the content (row 376)", () => {
  test("the loading demo's title and wrapped description sit inside the toast, padded evenly", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await page.getByRole("button", { name: "Show loading" }).click();
    const toast = toasts(page, LOADING).first();
    await expect(toast).toBeVisible();
    const f = await fit(toast);
    expect(f.clipped, `nothing is clipped (${JSON.stringify(f)})`).toBe(false);
    expect(f.padTop, `room above the content (${JSON.stringify(f)})`).toBeGreaterThanOrEqual(12);
    expect(Math.abs(f.padTop - f.padBottom), `the same room below (${JSON.stringify(f)})`).toBeLessThan(1);
  });

  test("one line or several: the same padding, a height that follows the content", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add short toast");
    const short = await fit(toasts(page, ML).first());
    await add(page, "Add long toast");
    // Hover the pile so both are expanded and each is measured at its own height.
    const t = toasts(page, ML);
    await t.first().hover();
    await expect.poll(() => region(page, ML).evaluate((el) => el.matches(":hover"))).toBe(true);
    const long = await fit(t.first());
    const shortAgain = await fit(t.nth(1));
    for (const f of [short, long, shortAgain]) {
      expect(f.clipped, `nothing is clipped (${JSON.stringify(f)})`).toBe(false);
      expect(f.padTop, `room above the content (${JSON.stringify(f)})`).toBeGreaterThanOrEqual(12);
      expect(Math.abs(f.padTop - f.padBottom), `evenly padded (${JSON.stringify(f)})`).toBeLessThan(1);
    }
    expect(Math.abs(long.padTop - short.padTop), "the padding does not depend on the number of lines").toBeLessThan(1);
    expect(long.height - short.height, "the long toast is taller by its extra lines").toBeGreaterThan(40);
    expect(Math.abs(shortAgain.height - short.height), "a toast keeps its own height when the pile expands").toBeLessThan(1);
  });
});

test.describe("toast: the stack with different heights (row 376)", () => {
  /** A 3-line toast behind a 1-line one: the long one first, so the short one is the front. */
  async function pile(page: Page) {
    await open(page);
    await page.mouse.move(2, 2);
    await add(page, "Add long toast");
    await add(page, "Add short toast");
    const t = toasts(page, ML);
    return { front: t.first(), back: t.nth(1) };
  }

  test("collapsed: the back toast takes the front toast's height and hides its content", async ({ page }) => {
    const { front, back } = await pile(page);
    const f = await size(front);
    const b = await size(back);
    expect(Math.abs(b.height - f.height), `back ${b.height} px, front ${f.height} px`).toBeLessThan(1);
    expect(f.contentOpacity, "the front toast shows its content").toBe(1);
    expect(b.contentOpacity, "the back toast hides its content").toBe(0);
    // The peek is the gap, not a function of the heights: the back toast's centre sits one gap above the front's.
    const centre = async (l: Locator) =>
      readAtRest(l, (el) => {
        const r = el.getBoundingClientRect(); // rest-ok: read inside readAtRest
        return r.top + r.height / 2;
      });
    expect((await centre(front)) - (await centre(back)), "peek offset").toBeGreaterThan(10);
    expect((await centre(front)) - (await centre(back)), "peek offset").toBeLessThan(14);
  });

  test("expanded: each toast has its own height, shows its content and the toasts do not overlap", async ({ page }) => {
    const { front, back } = await pile(page);
    const collapsedFront = await size(front);
    await front.focus();
    await expect.poll(() => region(page, ML).evaluate((el) => el.matches(":focus-within"))).toBe(true);
    const f = await size(front);
    const b = await size(back);
    expect(b.height - f.height, "the long toast is taller than the short one").toBeGreaterThan(40);
    expect(Math.abs(f.height - collapsedFront.height), "the front toast keeps its height").toBeLessThan(1);
    expect(b.contentOpacity, "the back toast shows its content again").toBe(1);
    const rect = (l: Locator) =>
      readAtRest(l, (el) => {
        const r = el.getBoundingClientRect(); // rest-ok: read inside readAtRest
        return { top: r.top, bottom: r.bottom };
      });
    const fr = await rect(front);
    const br = await rect(back);
    expect(fr.top - br.bottom, "the back toast ends one gap above the front one").toBeGreaterThan(10);
    expect(fr.top - br.bottom, "and no further").toBeLessThan(14);
    expect((await fit(back)).clipped, "the long toast shows all of its content").toBe(false);
  });

  test("collapsing again takes the back toast back to the front's height", async ({ page }) => {
    const { front, back } = await pile(page);
    await front.focus();
    await expect.poll(() => region(page, ML).evaluate((el) => el.matches(":focus-within"))).toBe(true);
    await front.blur();
    await page.mouse.move(2, 2);
    await expect.poll(async () => (await size(back)).contentOpacity).toBe(0);
    const f = await size(front);
    const b = await size(back);
    expect(Math.abs(b.height - f.height)).toBeLessThan(1);
  });

  test("closing the front toast: the next one becomes the front and the pile follows its height", async ({ page }) => {
    const { front, back } = await pile(page);
    const t = toasts(page, ML);
    const longHeight = await (async () => {
      await front.focus();
      await expect.poll(() => region(page, ML).evaluate((el) => el.matches(":focus-within"))).toBe(true);
      return (await size(back)).height;
    })();
    await front.getByRole("button", { name: "close", exact: true }).click();
    await expect(t).toHaveCount(1);
    await page.mouse.move(2, 2);
    await region(page, ML).evaluate((el) => (el.contains(document.activeElement) ? (document.activeElement as HTMLElement).blur() : undefined));
    const only = await size(t.first());
    expect(Math.abs(only.height - longHeight), "the long toast, alone, is its own height").toBeLessThan(1);
    expect(only.contentOpacity, "and shows its content").toBe(1);
  });
});
