/**
 * Questionnaire height glide (backlog row 310): moving between items transitions the form's height with
 * CSS instead of snapping (`animate_height`, default on). The technique pins the old height inline, then the
 * new one, and the stylesheet's `transition: height` runs between them; the pins are cleared afterwards.
 * Reduced motion keeps the snap.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };
const URL = `${BASE_URL}/component/questionnaire/`;
const FORM = '#component-preview-frame form[data-slot="questionnaire"]';

/** Samples the form's rendered height on every frame for `ms`, plus whether an inline height was set. */
async function sampleHeights(page: Page, ms: number) {
  await page.locator(FORM).evaluate((f: HTMLFormElement, ms: number) => {
    const w = window as unknown as { __qh: number[]; __qpin: boolean };
    w.__qh = [];
    w.__qpin = false;
    const end = performance.now() + ms;
    const tick = () => {
      w.__qh.push(f.getBoundingClientRect().height);
      if (f.style.height) w.__qpin = true;
      if (performance.now() < end) requestAnimationFrame(tick);
    };
    tick();
  }, ms);
}
const sampled = (page: Page) =>
  page.evaluate(() => {
    const w = window as unknown as { __qh: number[]; __qpin: boolean };
    return { heights: w.__qh, pinned: w.__qpin };
  });

async function openMain(page: Page) {
  await gotoHydrated(page, URL, GOTO);
  const form = page.locator(FORM);
  await form.scrollIntoViewIfNeeded();
  await expect(form.locator("fieldset[data-active]")).toBeVisible({ timeout: 60_000 });
  await expect(form).toHaveAttribute("data-animate-height", "true");
  await expect.poll(() => form.evaluate((f) => getComputedStyle(f).transitionProperty)).toBe("height");
  return form;
}

/** Picks the first answer of the active item, so Next is allowed. */
async function answer(page: Page) {
  const radio = page.locator(FORM).locator("fieldset[data-active] input[data-slot='questionnaire-choice-input']").first();
  if (!(await radio.isChecked())) await radio.check({ force: true });
}
const next = (page: Page) => page.locator(FORM).locator("button[data-slot='questionnaire-next']").click();

test("the form height glides between items and the inline pins are cleared", async ({ page }) => {
  const form = await openMain(page);
  const duration = await form.evaluate((f) => parseFloat(getComputedStyle(f).transitionDuration) * 1000);
  expect(duration).toBeGreaterThan(50);
  await answer(page);
  const before = await form.evaluate((f) => f.getBoundingClientRect().height);
  await sampleHeights(page, duration + 700);
  await next(page);
  await expect(form.locator("fieldset[data-active]")).toHaveCount(1);
  await page.waitForTimeout(duration + 800);
  const { heights, pinned } = await sampled(page);
  const after = await form.evaluate((f) => f.getBoundingClientRect().height);
  expect(Math.abs(after - before)).toBeGreaterThan(5);
  expect(pinned).toBe(true);
  // At least one frame sits strictly between the old and the new height: a glide, not a snap.
  const lo = Math.min(before, after);
  const hi = Math.max(before, after);
  expect(heights.some((h) => h > lo + 1 && h < hi - 1)).toBe(true);
  // It settles on the natural (auto) height and releases the pin.
  expect(await form.evaluate((f) => f.style.height)).toBe("");
});

test("prefers-reduced-motion keeps the snap: no height is ever pinned or in between", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  const form = await openMain(page);
  await answer(page);
  const before = await form.evaluate((f) => f.getBoundingClientRect().height);
  await sampleHeights(page, 900);
  await next(page);
  await expect(form.locator("fieldset[data-active]")).toHaveCount(1);
  await page.waitForTimeout(1000);
  const { heights, pinned } = await sampled(page);
  const after = await form.evaluate((f) => f.getBoundingClientRect().height);
  expect(Math.abs(after - before)).toBeGreaterThan(5);
  expect(pinned).toBe(false);
  expect(heights.every((h) => Math.abs(h - before) < 1 || Math.abs(h - after) < 1)).toBe(true);
});
