/**
 * Drawer snap points (backlog row 156; Base UI Drawer `snapPoints`). Demo: `variants/snap_points`, a bottom
 * drawer with 148px / 355px / full snap points and a CONTROLLED active point.
 *
 * Construction under test: ONE resting offset. The primitive writes the active point's extent and the
 * largest one as two custom properties on the panel; the stylesheet derives `--dx-drawer-rest` and the
 * panel's `transform` reads it. So every assertion measures the panel's visible height at rest and compares
 * it with the snap point, never a number copied from the implementation.
 *
 * Tier-3 "opinion" rules (no APG rule for a drag gesture): settle on the nearest point to the release
 * position projected by the release velocity (`SNAP_PROJECTION_MS`, capped), close only by dragging below
 * the lowest point by the drawer's one closing rule (25% of its visible extent, or a flick).
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole } from "./demo-frame";
import { awaitAnimationsSettled, measureAtRest } from "./animations";
import { fixedViewportBox } from "./viewport-box";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";

const URL = `${BASE_URL}/component/drawer/`;
const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const PHONE = { width: 390, height: 844 };
test.use({ viewport: PHONE });

const VARIANT = "snap_points";
const CONTENT = '[data-slot="drawer-content"]';
const HANDLE = '[data-slot="drawer-handle"]';
const ROOT = '[data-slot="drawer-root"]';
const ACTIVE = '[data-testid="active-snap"]';
// The demo's snap points, in px (the last one is `Fraction(1.0)`: the whole viewport).
const LOW = 148;
const MID = 355;

async function open(page: Page) {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await demoRole(page, "button", 0, VARIANT).click();
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
  await awaitAnimationsSettled(page.locator(CONTENT));
}

/** How much of the panel is on screen, at rest: the viewport's bottom edge minus the panel's top. */
async function visible(page: Page): Promise<number> {
  const viewport = await fixedViewportBox(page);
  const box = await measureAtRest(page.locator(CONTENT));
  return viewport.y + viewport.height - box.y;
}

async function expectVisible(page: Page, px: number) {
  const viewport = await fixedViewportBox(page);
  const want = px === Infinity ? viewport.height : px;
  await expect.poll(async () => Math.round(await visible(page)), { timeout: 10_000 }).toBeGreaterThanOrEqual(Math.round(want) - 1);
  expect(Math.abs((await visible(page)) - want)).toBeLessThanOrEqual(1);
}

type Kind = "mouse" | "touch" | "pen";

/** Same driver as drawer.spec.ts: synthetic Pointer Events, a flick in ONE synchronous task. */
async function drag(
  page: Page,
  selector: string,
  opts: { kind?: Kind; dy: number; steps?: number; stepMs?: number; pauseMs?: number; flick?: boolean },
) {
  await page.evaluate(
    async ({ selector, kind, dy, steps, stepMs, pauseMs, flick }) => {
      const el = document.querySelector(selector) as HTMLElement;
      const r = el.getBoundingClientRect(); // rest-ok: places the pointer on a drawer the caller has settled
      const x0 = r.x + r.width / 2;
      const y0 = r.y + r.height / 2;
      const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
      const spin = (ms: number) => {
        const t = performance.now();
        while (performance.now() - t < ms) { /* deterministic event spacing */ }
      };
      const init = (y: number) => ({
        bubbles: true, cancelable: true, composed: true, pointerId: 43, pointerType: kind,
        isPrimary: true, clientX: x0, clientY: y, button: 0, buttons: 1,
      });
      el.dispatchEvent(new PointerEvent("pointerdown", init(y0)));
      for (let i = 1; i <= steps; i++) {
        if (flick) spin(stepMs);
        else await sleep(stepMs);
        el.dispatchEvent(new PointerEvent("pointermove", init(y0 + (dy * i) / steps)));
      }
      if (pauseMs) await sleep(pauseMs);
      el.dispatchEvent(new PointerEvent("pointerup", { ...init(y0 + dy), buttons: 0 }));
    },
    { selector, kind: opts.kind ?? "mouse", dy: opts.dy, steps: opts.steps ?? 6, stepMs: opts.stepMs ?? 40, pauseMs: opts.pauseMs ?? 0, flick: opts.flick ?? false },
  );
}

/** A slow drag (no velocity at release: the pointer rests before letting go). `dy < 0` is up, toward open. */
const slow = (page: Page, dy: number) => drag(page, HANDLE, { dy, steps: 6, stepMs: 50, pauseMs: 200 });
/** A flick: `dy` px in ~12ms, well over the projection cap. */
const flick = (page: Page, dy: number) => drag(page, HANDLE, { dy, steps: 3, stepMs: 4, flick: true });

test("opens at the first snap point, sized by the largest", async ({ page }) => {
  await open(page);
  await expectVisible(page, LOW);
  await expect(page.locator(ACTIVE)).toHaveText("148px");
  await expect(page.locator(CONTENT)).toHaveAttribute("data-snap", "partial");
  // The panel is as tall as the largest snap point (the whole viewport here) and rests translated.
  const viewport = await fixedViewportBox(page);
  const box = await measureAtRest(page.locator(CONTENT));
  expect(Math.abs(box.height - viewport.height)).toBeLessThanOrEqual(1);
});

test("a slow drag settles on the nearest snap point by distance", async ({ page }) => {
  await open(page);
  const viewport = await fixedViewportBox(page);

  // 130px up from 148 is 278 visible: nearer 355 than 148 (the midpoint is 251).
  await slow(page, -130);
  await expectVisible(page, MID);
  await expect(page.locator(ACTIVE)).toHaveText("355px");
  await expect(page.locator(CONTENT)).toHaveAttribute("data-snap", "partial");

  // 300px up from 355 is 655: nearer full (844) than 355 (the midpoint is 599).
  await slow(page, -300);
  await expectVisible(page, viewport.height);
  await expect(page.locator(ACTIVE)).toHaveText("full");
  await expect(page.locator(CONTENT)).toHaveAttribute("data-snap", "full");

  // 60px down from full stays (nearer full); 400px down lands on 355 (visible 444: nearer 355 than 844).
  await slow(page, 60);
  await expectVisible(page, viewport.height);
  await slow(page, 400);
  await expectVisible(page, MID);
});

test("a short slow drag snaps back to the same point", async ({ page }) => {
  await open(page);
  await slow(page, -40);
  await expectVisible(page, LOW);
  await expect(page.locator(ACTIVE)).toHaveText("148px");
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
});

test("a fast flick settles by velocity: the same distance that stays when slow skips to another point", async ({ page }) => {
  await open(page);
  const viewport = await fixedViewportBox(page);
  // 60px slow stays (208 visible is nearer 148 than 355) ...
  await slow(page, -60);
  await expectVisible(page, LOW);
  // ... the same 60px thrown at ~5px/ms projects 450px further: full, skipping 355.
  await flick(page, -60);
  await expectVisible(page, viewport.height);
  await expect(page.locator(ACTIVE)).toHaveText("full");

  // A flick DOWN from 355 lands on the lowest point and never closes, even though it is fast.
  await slow(page, 400); // back to 355
  await expectVisible(page, MID);
  await flick(page, 30);
  await expectVisible(page, LOW);
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
});

test("dragging below the lowest snap point closes the drawer; a short pull does not", async ({ page }) => {
  await open(page);
  // 20px of 148 visible is 14%: stays.
  await slow(page, 20);
  await expectVisible(page, LOW);
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
  // 100px is 68%: closes.
  await slow(page, 100);
  await expect(page.locator(ROOT)).toHaveCount(0);
});

test("a flick down from the lowest snap point closes it", async ({ page }) => {
  await open(page);
  await flick(page, 30);
  await expect(page.locator(ROOT)).toHaveCount(0);
});

test("a long drag from the top point that stays above the lowest never closes", async ({ page }) => {
  await open(page);
  // Expand through the footer button first (the controlled path), then drag down to between the middle and lowest points (never below the lowest).
  await page.locator(CONTENT).getByRole("button", { name: "Expand" }).click();
  const viewport = await fixedViewportBox(page);
  await expectVisible(page, viewport.height);
  await slow(page, viewport.height - LOW - 200);
  await expectVisible(page, MID);
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
});

test("the active snap point is controlled: a footer button moves the panel and the demo reports it", async ({ page }) => {
  await open(page);
  await page.locator(CONTENT).getByRole("button", { name: "Expand" }).click();
  const viewport = await fixedViewportBox(page);
  await expectVisible(page, viewport.height);
  await expect(page.locator(ACTIVE)).toHaveText("full");
  await expect(page.locator(CONTENT)).toHaveAttribute("data-snap", "full");
  // A drag reports through `on_active_snap_point_change`, which the demo writes back.
  await slow(page, 700);
  await expectVisible(page, LOW);
  await expect(page.locator(ACTIVE)).toHaveText("148px");
});

test("content scrolls only at the largest point, and a scrolled panel does not start a drag", async ({ page }) => {
  await open(page);
  const content = page.locator(CONTENT);
  // Partial: clipped, not scrollable.
  expect(await content.evaluate((el) => getComputedStyle(el).overflowY)).toBe("hidden");

  await content.getByRole("button", { name: "Expand" }).click();
  const viewport = await fixedViewportBox(page);
  await expectVisible(page, viewport.height);
  expect(await content.evaluate((el) => getComputedStyle(el).overflowY)).toBe("auto");

  const box = await measureAtRest(content);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, 200);
  await expect.poll(() => content.evaluate((el) => el.scrollTop)).toBeGreaterThan(0);

  // Pressing in scrolled content scrolls it; it does not drag the drawer (Base UI).
  await drag(page, '[data-slot="drawer-header"]', { dy: 300, steps: 6, stepMs: 40, pauseMs: 100 });
  await page.waitForTimeout(500);
  await expectVisible(page, viewport.height);
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
});

test("Escape closes a snap drawer opened at a partial point", async ({ page }) => {
  await open(page);
  await page.keyboard.press("Escape");
  await expect(page.locator(ROOT)).toHaveCount(0);
});

test.describe("reduced motion", () => {
  test("a settle lands on the snap point, with the reduced duration", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await open(page);
    const reduced = await page.evaluate(() => {
      const probe = document.createElement("div");
      probe.style.transitionDuration = "var(--dx-motion-duration-reduced)";
      document.body.appendChild(probe);
      const d = getComputedStyle(probe).transitionDuration;
      probe.remove();
      return d;
    });
    const first = (await page.locator(CONTENT).evaluate((el) => getComputedStyle(el).transitionDuration)).split(",")[0].trim();
    expect(first).toBe(reduced);
    await slow(page, -130);
    await expectVisible(page, MID);
  });
});

test("a touch drag settles the same way", async ({ page }) => {
  await open(page);
  await drag(page, HANDLE, { kind: "touch", dy: -130, steps: 6, stepMs: 50, pauseMs: 200 });
  await expectVisible(page, MID);
});

test("axe: a snap drawer open at a partial point has no violations", async ({ page }) => {
  await open(page);
  await expectNoAxeViolations(page, "drawer: snap points, open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
});
