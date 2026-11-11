/**
 * The safe triangle (backlog row 499): a hover-opened panel stays open while the pointer crosses the gap toward it, and
 * closes at the host's own short delay (NavigationMenu 50 ms, submenus 0 ms: Base UI's `closeDelay`s) once the pointer
 * leaves the triangle.
 *
 * Every verdict reads the app's timers on the page clock (`app-clock.ts`), so a loaded machine cannot move it. The
 * panel's rect is measured BEFORE the clock is paused (a paused clock never finishes its open animation). Each pointer
 * step is one `mouse.move` plus `page.clock.runFor(STEP_MS)`, which also runs the fake animation frame the bridge's
 * `mousemove` stream is coalesced on. Chromium only (the real-time wait after the leave lets the bridge install its
 * `mousemove` listener, which the paused page clock does not control).
 */
import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole } from "./demo-frame";
import { awaitAnimationsSettled, measureAtRest } from "./animations";
import { pauseAppClock } from "./app-clock";

const GOTO = { timeout: 20 * 60 * 1000 };
/** One pointer step of a crossing; well under the 100 ms stall. */
const STEP_MS = 30;
/** The crossing outlasts the close delays, so only a held panel survives it. */
const STEPS = 20;

const nav = (page: Page) => page.getByRole("navigation", { name: "Component navigation menu" });

/** Move the pointer in `STEPS` equal steps, each followed by `STEP_MS` of page clock. */
async function cross(page: Page, from: { x: number; y: number }, to: { x: number; y: number }, onStep?: () => Promise<void>) {
  for (let i = 1; i <= STEPS; i++) {
    await page.mouse.move(from.x + ((to.x - from.x) * i) / STEPS, from.y + ((to.y - from.y) * i) / STEPS);
    await page.clock.runFor(STEP_MS);
    if (i === 1) {
      // The bridge installs its `mousemove` listener (a real-time round trip) after the rect read that follows the leave.
      await page.waitForTimeout(60); // timing-bound-ok: real-time bridge install; the page clock is paused
    }
    await onStep?.();
  }
}

test.describe("NavigationMenu: hover close (50 ms) and the safe triangle", () => {
  async function openPanel(page: Page) {
    await gotoHydrated(page, `${BASE_URL}/component/navigation_menu/`, GOTO);
    const trigger = nav(page).getByRole("button", { name: "Getting started" });
    await trigger.hover();
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
    const link = nav(page).getByRole("link", { name: "Component Library" });
    await expect(link).toBeVisible();
    const panelEl = page.locator(`#${await trigger.getAttribute("aria-controls")}`);
    await awaitAnimationsSettled(panelEl);
    return { trigger, panelEl, tb: await measureAtRest(trigger), pb: await measureAtRest(panelEl) };
  }

  test("a slow diagonal across the gap to the panel keeps it open", async ({ page }) => {
    const { trigger, panelEl, tb, pb } = await openPanel(page);
    await pauseAppClock(page);
    const from = { x: tb.x + 10, y: tb.y + tb.height - 2 };
    await page.mouse.move(from.x, from.y);
    const to = { x: pb.x + pb.width * 0.8, y: pb.y + 8 };
    await cross(page, from, to, async () => {
      await expect(trigger, "open all the way across the gap").toHaveAttribute("aria-expanded", "true");
    });
    // In the panel: the pointer is home, and resting there keeps it open past every close delay.
    await page.clock.runFor(500);
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
    await expect(panelEl).toBeVisible();
    await page.clock.resume();
  });

  test("a move out of the triangle closes it at the 50 ms delay", async ({ page }) => {
    const { trigger, tb } = await openPanel(page);
    await pauseAppClock(page);
    const x = tb.x + 10;
    const gapY = tb.y + tb.height + 2;
    // Out of the trigger into the gap, then sideways: off the path to the panel.
    await page.mouse.move(x, tb.y + tb.height - 2);
    await page.mouse.move(x, gapY);
    await page.waitForTimeout(60); // timing-bound-ok: real-time bridge install; the page clock is paused
    await page.mouse.move(x + 300, gapY);
    // The app's close timer starts when the move's message arrives (real time; at most one animation frame, 16 ms,
    // of page time later).
    await page.waitForTimeout(60); // timing-bound-ok: real-time message delivery; the page clock is paused
    await page.clock.runFor(40);
    await expect(trigger, "open before the 50 ms delay has run").toHaveAttribute("aria-expanded", "true");
    // 70 ms after the move: past the 50 ms delay (and a frame), short of the 100 ms stall that would also close it.
    await page.clock.runFor(30);
    await expect(trigger, "closed once the 50 ms delay has run").toHaveAttribute("aria-expanded", "false");
    await page.clock.resume();
  });

  test("a pointer that stops in the gap closes it", async ({ page }) => {
    const { trigger, tb } = await openPanel(page);
    await pauseAppClock(page);
    const x = tb.x + 10;
    await page.mouse.move(x, tb.y + tb.height - 2);
    await page.mouse.move(x, tb.y + tb.height + 2);
    await page.clock.runFor(60);
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
    await page.clock.runFor(200);
    await expect(trigger, "a stall in the gap is not a path to the panel").toHaveAttribute("aria-expanded", "false");
    await page.clock.resume();
  });
});

test.describe("DropdownMenu submenu: hover close (0 ms) and the safe triangle", () => {
  const rootItem = (page: Page, n: number) => page.getByRole("menu").first().getByRole("menuitem").nth(n);
  const submenuOf = (page: Page) => page.getByRole("menu").last();

  async function openSubmenu(page: Page) {
    await gotoHydrated(page, `${BASE_URL}/component/dropdown_menu/`, GOTO);
    await demoRole(page, "button").click();
    const sub = rootItem(page, 3);
    await awaitAnimationsSettled(sub);
    await sub.hover();
    await expect(sub).toHaveAttribute("aria-expanded", "true");
    const submenu = submenuOf(page);
    await expect(submenu).toBeVisible();
    await awaitAnimationsSettled(submenu);
    return { sub, submenu, tb: await measureAtRest(sub), sb: await measureAtRest(submenu) };
  }

  test("a slow move along the sibling items to the submenu keeps it open", async ({ page }) => {
    const { sub, submenu, tb, sb } = await openSubmenu(page);
    await pauseAppClock(page);
    const y = tb.y + tb.height + 3;
    const from = { x: tb.x + tb.width * 0.3, y };
    await page.mouse.move(from.x, tb.y + tb.height / 2);
    await page.mouse.move(from.x, from.y);
    await cross(page, from, { x: sb.x + 10, y }, async () => {
      await expect(sub, "open on the way").toHaveAttribute("aria-expanded", "true");
    });
    await page.clock.runFor(300);
    await expect(sub).toHaveAttribute("aria-expanded", "true");
    await expect(submenu).toBeVisible();
    await page.clock.resume();
  });

  test("a move out of the triangle closes it at once (0 ms)", async ({ page }) => {
    const { sub, tb } = await openSubmenu(page);
    await pauseAppClock(page);
    const x = tb.x + tb.width * 0.3;
    await page.mouse.move(x, tb.y + tb.height + 3);
    await page.waitForTimeout(60); // timing-bound-ok: real-time bridge install; the page clock is paused
    // Away from the submenu (to the left of the sub-trigger's start).
    await page.mouse.move(tb.x - 20, tb.y + tb.height + 3);
    await page.waitForTimeout(60); // timing-bound-ok: real-time message delivery; the page clock is paused
    await page.clock.runFor(20);
    await expect(sub, "closed within a frame of leaving the triangle, short of the 100 ms stall").toHaveAttribute("aria-expanded", "false");
    await page.clock.resume();
  });
});
