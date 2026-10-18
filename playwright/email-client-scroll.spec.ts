import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// The email client demo keeps one scroll offset per view (backlog row 225): the message list per folder / tab /
// filter, a thread per message. Leaving a view saves where its own scroll container was, entering it puts that
// back, and a view never visited opens at the top. The container is the list's or the thread's own, never the window.
const URL = `${BASE_URL}/dashboard/email-client`;
const GOTO = { timeout: 20 * 60 * 1000 };

const list = (page: Page) => page.locator(".ec-list-scroll");
const thread = (page: Page) => page.locator(".ec-read-body");
const top = (loc: ReturnType<typeof list>) => loc.evaluate((el) => Math.round(el.scrollTop));

async function open(page: Page) {
  await gotoHydrated(page, URL, GOTO);
  await expect(page.locator(".ec-row").first()).toBeVisible({ timeout: 60_000 });
  // The effect that restores a view marks the container once it has run; scrolling is only remembered after that.
  await expect(list(page)).toHaveAttribute("data-scroll-restored", /^list:/);
}

/** Scroll a container and wait until the scroll has settled and been saved (a frame later). */
async function scrollTo(page: Page, loc: ReturnType<typeof list>, y: number) {
  await loc.evaluate((el, y) => (el.scrollTop = y), y);
  await expect.poll(() => top(loc)).toBeGreaterThan(y - 3);
  await page.waitForTimeout(250);
}

/** Open the n-th message that is actually inside the list's viewport, with a real click (Playwright's own
 * scroll-into-view would move the list, which a user clicking a visible row never does). */
async function openVisibleRow(page: Page, n: number) {
  const rows = page.locator(".ec-row-open");
  const box = await list(page).boundingBox();
  const count = await rows.count();
  let seen = 0;
  for (let i = 0; i < count; i++) {
    const b = await rows.nth(i).boundingBox();
    if (b && box && b.y >= box.y && b.y + b.height <= box.y + box.height) {
      if (seen++ === n) {
        await page.mouse.click(b.x + b.width / 2, b.y + b.height / 2);
        return;
      }
    }
  }
  throw new Error("no visible row");
}

const near = (a: number, b: number) => Math.abs(a - b) <= 2;

test("a message list returns to where it was left after opening a message and going back", async ({ page }) => {
  await open(page);
  await scrollTo(page, list(page), 1500);
  const before = await top(list(page));
  // The page itself does not scroll: the list does.
  expect(await page.evaluate(() => Math.round(window.scrollY))).toBe(0);

  await openVisibleRow(page, 1);
  await expect(page.locator(".ec-main")).toHaveClass(/ec-reading/);
  await expect(thread(page)).toBeVisible();
  await page.locator(".ec-read-toolbar button").first().click();
  await expect(page.locator(".ec-main")).not.toHaveClass(/ec-reading/);

  await expect.poll(async () => near(await top(list(page)), before)).toBe(true);
});

test("on a narrow screen the hidden list is put back where it was", async ({ page }) => {
  await page.setViewportSize({ width: 420, height: 800 });
  await open(page);
  await scrollTo(page, list(page), 1200);
  const before = await top(list(page));

  await openVisibleRow(page, 1);
  await expect(page.locator(".ec-main")).toHaveClass(/ec-reading/);
  // The list pane is display:none while reading, which discards its offset ...
  await expect(list(page)).toBeHidden();
  await page.locator(".ec-read-toolbar button").first().click();
  await expect(list(page)).toBeVisible();
  // ... and coming back restores it.
  await expect.poll(async () => near(await top(list(page)), before)).toBe(true);
});

test("a folder opened for the first time starts at the top, and each folder keeps its own place", async ({ page }) => {
  await open(page);
  const inbox = 1500;
  await scrollTo(page, list(page), inbox);

  await page.getByRole("button", { name: "Sent", exact: true }).click();
  await expect(list(page)).toHaveAttribute("data-scroll-view", /^list:sent:/);
  await expect.poll(() => top(list(page))).toBe(0); // first visit
  // Sent is long enough that "top" is a decision, not just the clamped end of a short list.
  expect(await list(page).evaluate((el) => el.scrollHeight - el.clientHeight)).toBeGreaterThan(700);
  await scrollTo(page, list(page), 700);

  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  await expect(list(page)).toHaveAttribute("data-scroll-view", /^list:inbox:/);
  await expect.poll(async () => near(await top(list(page)), inbox)).toBe(true);

  await page.getByRole("button", { name: "Sent", exact: true }).click();
  await expect.poll(async () => near(await top(list(page)), 700)).toBe(true);

  // Another folder never visited: top again.
  await page.getByRole("button", { name: "Archive", exact: true }).click();
  await expect(list(page)).toHaveAttribute("data-scroll-view", /^list:archive:/);
  await expect.poll(() => top(list(page))).toBe(0);
});

test("a tab is its own view too", async ({ page }) => {
  await open(page);
  await scrollTo(page, list(page), 1500);
  const before = await top(list(page));
  await page.getByRole("tab", { name: /^Unread/ }).click();
  await expect(list(page)).toHaveAttribute("data-scroll-view", /^list:inbox:unread:/);
  await expect.poll(() => top(list(page))).toBe(0);
  await page.getByRole("tab", { name: /^All/ }).click();
  await expect(list(page)).toHaveAttribute("data-scroll-view", /^list:inbox:all:/);
  await expect.poll(async () => near(await top(list(page)), before)).toBe(true);
});

test("a thread opens at the top the first time and returns to where it was left", async ({ page }) => {
  await open(page);
  await page.locator(".ec-row-open").nth(2).click({ force: true });
  await expect(thread(page)).toBeVisible();
  const first = await thread(page).getAttribute("data-scroll-view");
  await expect(thread(page)).toHaveAttribute("data-scroll-restored", first!);
  const room = await thread(page).evaluate((el) => el.scrollHeight - el.clientHeight);
  test.skip(room < 120, `the thread is not tall enough to scroll here (${room}px)`);
  expect(await top(thread(page))).toBe(0);
  await scrollTo(page, thread(page), Math.min(room, 160));
  const left = await top(thread(page));

  // Another message: its own first visit, at the top.
  await page.locator(".ec-row-open").nth(3).click({ force: true });
  await expect(thread(page)).not.toHaveAttribute("data-scroll-view", first!);
  await expect.poll(() => top(thread(page))).toBe(0);

  // Back to the first one: where it was left.
  await page.locator(".ec-row-open").nth(2).click({ force: true });
  await expect(thread(page)).toHaveAttribute("data-scroll-view", first!);
  await expect.poll(async () => near(await top(thread(page)), left)).toBe(true);
});
