/**
 * Popover is NON-MODAL by default (dev-docs/backlog.md row 169: shadcn / Radix / Base UI parity,
 * including page scroll). This file is the coverage for the
 * non-modal contract; `popover.spec.ts` keeps the modal arm (`variant=modal`, `is_modal: true`)
 * and `oracle/tier3-radix/scroll-lock.spec.ts` pins the modal lock/unlock on the same variant.
 *
 * The contract, on the default `main` demo ("Show Popover"):
 *   1. focus moves into the content on open and returns to the trigger on Escape;
 *   2. the page stays interactive (not inert) AND scrollable -- wheel and keyboard -- with no
 *      scroll lock at all (no `overflow: hidden`, no lock marker, no body freeze, no layout shift);
 *   3. ONE outside click closes it and that click reaches its target (no swallowed first click);
 *   4. `is_modal: true` stays available (the control test at the bottom proves the probes can
 *      tell the two apart, so a green non-modal run is not a green-by-blindness run).
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest, readStatic } from "./animations";
import { pageMetrics } from "./viewport-box";

const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app
const DEFAULT_URL = `${BASE_URL}/component/popover/`;
const MODAL_URL = `${BASE_URL}/component/popover/`;

const trigger = (page: Page) => page.getByRole("button", { name: "Show Popover", exact: true });
const dialog = (page: Page) => page.getByRole("dialog");

async function openDefault(page: Page) {
  await gotoHydrated(page, DEFAULT_URL, { timeout: NAV_TIMEOUT });
  await expect(trigger(page)).toBeVisible();
  await trigger(page).click();
  await expect(dialog(page)).toBeVisible();
}

/** A page-level button outside Dioxus' tree, bottom-right, counting its clicks on `window`. */
async function addBackgroundButton(page: Page) {
  await page.evaluate(() => {
    const w = window as unknown as { __bgClicks?: number };
    w.__bgClicks = 0;
    document.getElementById("pnm-bg")?.remove();
    const b = document.createElement("button");
    b.id = "pnm-bg";
    b.textContent = "Background target";
    b.style.cssText = "position:fixed;right:12px;bottom:12px;z-index:2147483000;padding:8px 12px;";
    b.addEventListener("click", () => {
      w.__bgClicks = (w.__bgClicks ?? 0) + 1;
    });
    document.body.appendChild(b);
  });
}

const bgClicks = (page: Page) => page.evaluate(() => (window as unknown as { __bgClicks?: number }).__bgClicks ?? 0);

/** Make sure the page is taller than the viewport, so "scrollY moved" is a real statement. */
async function ensureScrollable(page: Page) {
  const m = await pageMetrics(page);
  await page.evaluate((tooShort) => {
    if (tooShort) {
      document.body.style.minHeight = "6000px";
    }
    window.scrollTo(0, 0);
  }, m.scrollHeight < m.innerHeight * 3);
}

/** Everything a scroll lock could leave on the page, in one comparable snapshot. */
async function lockFootprint(page: Page) {
  const { clientWidth } = await pageMetrics(page);
  const rest = await page.evaluate(() => {
    const html = document.documentElement;
    const body = document.body;
    const hs = getComputedStyle(html);
    const bs = getComputedStyle(body);
    const w = window as unknown as { __dxScrollLocked?: boolean; __dxScrollLockN?: number };
    return {
      htmlOverflow: [hs.overflow, hs.overflowX, hs.overflowY],
      bodyOverflow: [bs.overflow, bs.overflowX, bs.overflowY],
      bodyPosition: bs.position,
      bodyInlineStyle: body.getAttribute("style") ?? "",
      htmlInlineStyle: html.getAttribute("style") ?? "",
      locked: w.__dxScrollLocked === true,
      lockCount: w.__dxScrollLockN ?? 0,
    };
  });
  return { ...rest, clientWidth };
}

/**
 * Runs `act` (a wheel or a scrolling key) and resolves with `scrollY` once the scroll it started has ENDED: moved
 * from where it was, then unchanged for 10 frames (backlog row 407). Chromium animates wheel and keyboard scrolls,
 * so a value read as soon as `scrollY > 0` is a frame of the animation: the next key lands on a scroll still in
 * flight (an End pressed then is dropped), and a position read then is not where the page stops.
 */
async function scrollAndSettle(page: Page, act: () => Promise<void>): Promise<number> {
  const start = await page.evaluate(() => window.scrollY);
  await act();
  return page.evaluate(async (from) => {
    const frame = () => new Promise((r) => requestAnimationFrame(r));
    const t0 = performance.now();
    while (window.scrollY === from && performance.now() - t0 < 5000) await frame();
    let last = window.scrollY;
    for (let still = 0; still < 10; ) {
      await frame();
      still = window.scrollY === last ? still + 1 : 0;
      last = window.scrollY;
    }
    return last;
  }, start);
}

/**
 * How far the page may scroll with the anchor still in view. A non-modal popover stays open while the page scrolls
 * (it is placed in document coordinates, backlog row 477); a test about the page scrolling UNDER an open popover
 * scrolls less than this so the trigger stays reachable.
 */
const scrollKeepingAnchor = (page: Page) =>
  readStatic(trigger(page), (el) => Math.floor(el.getBoundingClientRect().top - 40));

/**
 * Dispatches a cancelable wheel event on the body and reports whether anything called
 * `preventDefault()` on it. The scroll lock hides the viewport scroller's overflow and listens to no wheel, so
 * a `true` is a stray blocking listener; whether the page scrolls is read from a real wheel instead.
 */
const wheelIsPrevented = (page: Page) =>
  page.evaluate(() => {
    const e = new WheelEvent("wheel", { deltaY: 400, bubbles: true, cancelable: true });
    document.body.dispatchEvent(e);
    return e.defaultPrevented;
  });

test.describe("non-modal Popover (the default): focus", () => {
  test("focus moves into the content on open and Escape returns it to the trigger", async ({ page }) => {
    await openDefault(page);
    const insideId = await page.evaluate(() => {
      const d = document.querySelector("dialog[popover]");
      return d && d.contains(document.activeElement) ? document.activeElement?.id || document.activeElement?.tagName : null;
    });
    expect(insideId, "focus must be inside the open popover content").not.toBeNull();

    await page.keyboard.press("Escape");
    await expect(dialog(page)).toHaveCount(0);
    await expect(trigger(page)).toBeFocused();

    // And it reopens, with focus moving in again: the signal is not stranded by the native close.
    await trigger(page).click();
    await expect(dialog(page)).toBeVisible();
    expect(await page.evaluate(() => !!document.querySelector("dialog[popover]")?.contains(document.activeElement))).toBe(true);
    await page.keyboard.press("Escape");
    await expect(dialog(page)).toHaveCount(0);
    await expect(trigger(page)).toBeFocused();
  });

  test("focus is not trapped: Tab walks out of the content, the popover stays open", async ({ page }) => {
    await openDefault(page);
    for (let i = 0; i < 6; i++) await page.keyboard.press("Tab");
    expect(
      await page.evaluate(() => !!document.querySelector("dialog[popover]")?.contains(document.activeElement)),
      "after more Tabs than the content has fields, focus must have left it",
    ).toBe(false);
    await expect(dialog(page)).toBeVisible();
  });

  test("it is not a modal: no :modal, no aria-modal, nothing inert", async ({ page }) => {
    await openDefault(page);
    const d = dialog(page);
    expect(await d.evaluate((el) => el.matches(":modal"))).toBe(false);
    await expect(d).not.toHaveAttribute("aria-modal", "true");
    expect(await page.evaluate(() => document.querySelectorAll("[inert]").length)).toBe(0);
    // The trigger is still a live control while the popover is open.
    await expect(trigger(page)).toHaveAttribute("aria-expanded", "true");
  });
});

test.describe("non-modal Popover (the default): the page behind stays interactive", () => {
  test("one outside click closes it AND reaches its target", async ({ page }) => {
    await openDefault(page);
    await addBackgroundButton(page);
    const bg = page.locator("#pnm-bg");
    const box = (await measureAtRest(bg));
    const dbox = (await measureAtRest(dialog(page)));
    const cx = box.x + box.width / 2;
    const cy = box.y + box.height / 2;
    expect(
      cx >= dbox.x && cx <= dbox.x + dbox.width && cy >= dbox.y && cy <= dbox.y + dbox.height,
      "the background target must not sit under the popover for this test to mean anything",
    ).toBe(false);

    await page.mouse.click(cx, cy);

    await expect(dialog(page)).toHaveCount(0);
    expect(await bgClicks(page), "the same click that dismissed the popover must reach its target (no swallowed first click)").toBe(1);
  });

  test("background content is focusable and clickable while the popover is open", async ({ page }) => {
    await openDefault(page);
    await addBackgroundButton(page);
    // Programmatic focus on a background control is refused when the page is inert (a modal).
    await page.evaluate(() => document.getElementById("pnm-bg")!.focus());
    expect(await page.evaluate(() => document.activeElement?.id)).toBe("pnm-bg");
  });

  test("clicking the trigger again closes it (one click, no close-and-reopen)", async ({ page }) => {
    await openDefault(page);
    await trigger(page).click();
    await expect(dialog(page)).toHaveCount(0);
    // Settle: a stray reopen would land after the exit animation.
    await page.waitForTimeout(600);
    await expect(dialog(page)).toHaveCount(0);
    await expect(trigger(page)).toHaveAttribute("aria-expanded", "false");
  });

  test("clicking inside the content keeps it open", async ({ page }) => {
    await openDefault(page);
    await dialog(page).getByRole("heading", { name: "Dimensions" }).click();
    await expect(dialog(page)).toBeVisible();
  });
});

test.describe("non-modal Popover (the default): the page keeps scrolling and holds no scroll lock", () => {
  test("wheel scrolls the page with the popover open; no lock marker, overflow, body freeze or shift", async ({ page }) => {
    await gotoHydrated(page, DEFAULT_URL, { timeout: NAV_TIMEOUT });
    await ensureScrollable(page);
    const before = await lockFootprint(page);

    await trigger(page).click();
    await expect(dialog(page)).toBeVisible();

    const open = await lockFootprint(page);
    expect(open.locked, "a non-modal popover must not set the scroll-lock marker").toBe(false);
    expect(open.lockCount).toBe(0);
    // The site's own CSS already clips the x axis (`html { overflow-x: hidden }`), so compare with the
    // pre-open footprint instead of asserting no `hidden` anywhere; the y axis is what a lock takes.
    expect(open.htmlOverflow, "html overflow is untouched by opening").toEqual(before.htmlOverflow);
    expect(open.bodyOverflow, "body overflow is untouched by opening").toEqual(before.bodyOverflow);
    expect(open.htmlOverflow[2]).not.toBe("hidden");
    expect(open.bodyOverflow[2]).not.toBe("hidden");
    expect(open.bodyPosition, "no compensated body freeze").not.toBe("fixed");
    expect(open.bodyInlineStyle, "the body's inline style is untouched by opening").toBe(before.bodyInlineStyle);
    expect(open.htmlInlineStyle).toBe(before.htmlInlineStyle);
    expect(open.clientWidth, "no scrollbar-gutter shift on open").toBe(before.clientWidth);
    expect(await wheelIsPrevented(page), "nothing may preventDefault a wheel event").toBe(false);

    const room = await scrollKeepingAnchor(page);
    expect(room, "the trigger must sit low enough for a real scroll that keeps it in view").toBeGreaterThan(100);
    await page.mouse.move(400, 300);
    expect(await scrollAndSettle(page, () => page.mouse.wheel(0, room))).toBeGreaterThan(0);
    // Past the close-on-scroll quiet window (300 ms after the open's focus move), so a close would have landed.
    await page.waitForTimeout(500);
    await expect(dialog(page), "scrolling with the anchor in view does not close a non-modal popover").toBeVisible();
  });

  test("keyboard scrolls the page with the popover open", async ({ page }) => {
    await gotoHydrated(page, DEFAULT_URL, { timeout: NAV_TIMEOUT });
    await ensureScrollable(page);
    await trigger(page).click();
    await expect(dialog(page)).toBeVisible();
    // Focus is inside the content's fields on open; scrolling keys must act on the page, so park
    // focus on the body first (a modal's lock would swallow PageDown/End either way).
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    // Each key waits for the previous scroll to end: a key pressed into a scroll still in flight is dropped.
    expect(await scrollAndSettle(page, () => page.keyboard.press("PageDown"))).toBeGreaterThan(0);
    // End must reach the bottom. Not "further than after PageDown": on a short page PageDown alone
    // can already reach the bottom.
    const endY = await scrollAndSettle(page, () => page.keyboard.press("End"));
    const pm = await pageMetrics(page);
    const maxY = pm.scrollHeight - pm.clientHeight;
    expect(endY).toBeGreaterThanOrEqual(maxY - 1);
  });

  test("closing has nothing to undo: the scroll position is kept, no footprint is left", async ({ page }) => {
    await gotoHydrated(page, DEFAULT_URL, { timeout: NAV_TIMEOUT });
    await ensureScrollable(page);
    const before = await lockFootprint(page);
    await trigger(page).click();
    await expect(dialog(page)).toBeVisible();

    // Escape is what closes it here, so the anchor stays in view; the position is read once the scroll has ended.
    const room = await scrollKeepingAnchor(page);
    expect(room).toBeGreaterThan(100);
    await page.mouse.move(400, 300);
    const scrolled = await scrollAndSettle(page, () => page.mouse.wheel(0, room));
    expect(scrolled).toBeGreaterThan(100);
    await expect(dialog(page)).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(dialog(page)).toHaveCount(0);
    await page.waitForTimeout(400);

    // A lock restores the pre-open scroll position on release; nothing was taken, so nothing moves.
    expect(await page.evaluate(() => window.scrollY), "closing must not jump the page back").toBe(scrolled);
    const after = await lockFootprint(page);
    expect(after.bodyInlineStyle).toBe(before.bodyInlineStyle);
    expect(after.htmlInlineStyle).toBe(before.htmlInlineStyle);
    expect(after.locked).toBe(false);
    expect(await wheelIsPrevented(page)).toBe(false);
  });
});

test.describe("modal Popover (is_modal: true) is available -- and the probes can tell it apart", () => {
  test("the modal variant is :modal, holds the scroll lock while open and releases it on Escape", async ({ page }) => {
    await gotoHydrated(page, MODAL_URL, { timeout: NAV_TIMEOUT });
    await ensureScrollable(page);
    expect(await wheelIsPrevented(page), "no wheel listener before opening").toBe(false);

    await page.getByRole("button", { name: "Open modal popover" }).click();
    await expect(dialog(page)).toBeVisible();
    expect(await dialog(page).evaluate((el) => el.matches(":modal"))).toBe(true);
    expect((await lockFootprint(page)).locked, "the modal arm sets the lock marker").toBe(true);
    const scrollYBefore = await page.evaluate(() => window.scrollY);
    await page.mouse.move(400, 300);
    await page.mouse.wheel(0, 400);
    await page.waitForTimeout(200);
    expect(await page.evaluate(() => window.scrollY), "the modal arm's lock stops a wheel scroll").toBe(scrollYBefore);

    await page.keyboard.press("Escape");
    await expect(dialog(page)).toHaveCount(0);
    await expect.poll(() => lockFootprint(page).then((f) => f.locked)).toBe(false);
    expect(await wheelIsPrevented(page)).toBe(false);
  });
});
