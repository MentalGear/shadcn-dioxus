/**
 * A tap on a checkbox, a radio or the label beside one does only the work of the control (backlog row 476).
 *
 * Owner report (an iPhone, the home page): checking a checkbox in the "Filter results" block, and the radios of the
 * Radio Group card, took about a second to show the new state, while the radios inside the same Filter block
 * answered at once. What the slow ones share is the `Label` beside the control (a tap on the text, the large
 * target); the fast ones are a `<button>` that contains its own text.
 *
 * Chromium cannot show a second (WebKit is not available here, row 4), so this spec pins the two mechanisms a tap
 * on a label can add to the control's own click, each of which a phone pays for and a desktop does not notice:
 *
 *   - the label's `touch-action`: a `label` the page leaves at `auto` is read by Mobile Safari as the first tap of
 *     a possible double-tap zoom (the control's `manipulation` is the control's own, not its label's);
 *   - work in the tap's hover phase: a touch synthesizes `mouseover`/`mouseenter`/`mousemove` before the click, and
 *     any signal write or script start there runs before the control's click. A touch on a label arms nothing, and
 *     a checkbox toggle runs no script (`new Function`, which is what `document::eval` costs) on the way.
 *
 * Then the bound: tap to painted state under 4x CPU throttling (`Emulation.setCPUThrottlingRate`), the median of 10
 * taps per target, with a limit far below the reported second, so only a regression of that size is red. It
 * measures from `touchend` to the animation frame after the control's `aria-checked` changes.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8090 npx playwright test --config=ssg.local.config.ts tap-latency.spec.ts
 */
import type { Locator, Page } from "@playwright/test";
import { test, expect, devices } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test.use({ ...devices["iPhone 13"], defaultBrowserType: "chromium" });

const GOTO = { timeout: 20 * 60 * 1000 };
const TAPS = 10;
const THROTTLE = 4;
/** Far below the reported second; a loaded 4-core box and a debug build both stay well under it. */
const BUDGET_MS = 600;

/** The Filter results block's checkbox + label pair, and its status radios. */
const filterCheckbox = (page: Page) => page.locator("#ft-eng");
const filterCheckboxLabel = (page: Page) => page.locator('label[for="ft-eng"]');
const filterRadios = (page: Page) =>
  page.locator('[role="radiogroup"]', { has: page.getByText("Drafts", { exact: true }) }).locator('[role="radio"]');
/** The Radio Group gallery card: three radios, each beside a `Label`. */
const radioCard = (page: Page) =>
  page.locator(".dx-component-card").filter({ has: page.locator("h3", { hasText: /^radio group$/i }) }).first();

async function open(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/`, GOTO);
}

/** Milliseconds from `touchend` to the frame after `control`'s `aria-checked` changes, for one tap of `target`. */
async function tapToPaint(page: Page, control: Locator, target: Locator): Promise<number> {
  await target.scrollIntoViewIfNeeded();
  await page.waitForTimeout(250);
  const handle = await control.elementHandle();
  await page.evaluate((el) => {
    const w = window as unknown as { __tap: { touchEnd?: number; painted?: number } };
    const tap = (w.__tap = {});
    const before = el!.getAttribute("aria-checked");
    document.addEventListener("touchend", () => (tap.touchEnd = performance.now()), { capture: true, once: true });
    new MutationObserver((_, observer) => {
      if (el!.getAttribute("aria-checked") === before) return;
      observer.disconnect();
      requestAnimationFrame(() => {
        const channel = new MessageChannel();
        channel.port1.onmessage = () => (tap.painted = performance.now());
        channel.port2.postMessage(0);
      });
    }).observe(el!, { attributes: true });
  }, handle);
  await target.tap();
  await page.waitForFunction(() => (window as unknown as { __tap: { painted?: number } }).__tap.painted !== undefined);
  return page.evaluate(() => {
    const tap = (window as unknown as { __tap: { touchEnd: number; painted: number } }).__tap;
    return tap.painted - tap.touchEnd;
  });
}

const median = (xs: number[]) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];

test("the label of a control is not left at touch-action auto", async ({ page }) => {
  await open(page);
  const labels = await page.evaluate(() =>
    [...document.querySelectorAll("label")].map((l) => ({ text: (l.textContent ?? "").trim(), action: getComputedStyle(l).touchAction })),
  );
  expect(labels.length).toBeGreaterThan(10);
  expect(labels.filter((l) => l.action !== "manipulation")).toEqual([]);
});

test("a tap on a label arms no script and writes no signal", async ({ page }) => {
  await open(page);
  const card = radioCard(page);
  await card.scrollIntoViewIfNeeded();
  const live = () =>
    page.evaluate(() => [...(((window as any).__dxJsListeners?.live as Map<number, string>) ?? new Map()).values()].filter((t) => t === "mousedown").length);
  const before = await live();
  await card.locator("label").nth(2).tap();
  await expect(card.locator('[role="radio"]').nth(2)).toHaveAttribute("aria-checked", "true");
  await page.waitForTimeout(300);
  expect(await live()).toBe(before);
});

test("toggling a checkbox runs no script", async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as unknown as { __functions: number };
    w.__functions = 0;
    const F = window.Function;
    window.Function = new Proxy(F, {
      construct(target, args, newTarget) {
        w.__functions++;
        return Reflect.construct(target, args, newTarget);
      },
    });
  });
  await open(page);
  const box = filterCheckbox(page);
  await box.scrollIntoViewIfNeeded();
  await page.waitForTimeout(500);
  const before = await page.evaluate(() => (window as any).__functions as number);
  for (const state of ["true", "false", "true"]) {
    await box.tap();
    await expect(box).toHaveAttribute("aria-checked", state);
  }
  await page.waitForTimeout(300);
  expect(await page.evaluate(() => (window as any).__functions as number)).toBe(before);
});

test.describe("tap to painted state under CPU throttling", () => {
  test.beforeEach(async ({ page }) => {
    await open(page);
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Emulation.setCPUThrottlingRate", { rate: THROTTLE });
  });

  /** The index of a radio that is not selected, so a tap on it changes the state. */
  const unselected = (radios: Locator) =>
    radios.evaluateAll((els) => els.findIndex((el) => el.getAttribute("aria-checked") === "false"));

  async function bound(page: Page, name: string, pick: () => Promise<{ control: Locator; target: Locator }>) {
    const times: number[] = [];
    for (let i = 0; i < TAPS; i++) {
      const { control, target } = await pick();
      times.push(await tapToPaint(page, control, target));
    }
    const m = median(times);
    test.info().annotations.push({ type: name, description: `median ${m.toFixed(0)} ms, max ${Math.max(...times).toFixed(0)} ms` });
    expect(m, `${name}: ${times.map((t) => t.toFixed(0)).join(", ")}`).toBeLessThan(BUDGET_MS);
  }

  test("Filter results checkbox, by its box and by its label", async ({ page }) => {
    const box = filterCheckbox(page);
    await bound(page, "checkbox box", async () => ({ control: box, target: box }));
    await bound(page, "checkbox label", async () => ({ control: box, target: filterCheckboxLabel(page) }));
  });

  test("Filter results radios", async ({ page }) => {
    const radios = filterRadios(page);
    await bound(page, "filter radio", async () => {
      const radio = radios.nth(await unselected(radios));
      return { control: radio, target: radio };
    });
  });

  test("Radio Group card radios, by the radio and by its label", async ({ page }) => {
    const card = radioCard(page);
    const radios = card.locator('[role="radio"]');
    const labels = card.locator("label");
    await bound(page, "card radio", async () => {
      const radio = radios.nth(await unselected(radios));
      return { control: radio, target: radio };
    });
    await bound(page, "card radio label", async () => {
      const at = await unselected(radios);
      return { control: radios.nth(at), target: labels.nth(at) };
    });
  });
});
