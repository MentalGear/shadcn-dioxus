/**
 * The scroll fade's edge depths ease when they switch. `--dx-scroll-fade-t/-b/-s/-e` are registered
 * `<length-percentage>` properties (`assets/dx-effects.css`), so a `transition` on them animates an
 * edge growing in as the reader scrolls away from it and shrinking as they reach it. Where scroll-driven
 * animations exist the depth is already an animation of the scroll position and an animation outranks a
 * transition, so the transition is for the browsers without them (the scroller's script sets the depth
 * to its full size or 0 there).
 *
 * What is observed: the computed `transition-*` of a faded ScrollArea (the four properties, the
 * theme's base duration, ease-out; none under `prefers-reduced-motion: reduce`), and the CSSTransition
 * objects the browser creates when a depth changes (read through the Web Animations API and sought to a
 * fixed time, so no wall-clock threshold). The scroll-driven animation is switched off on the element
 * in the behavioural tests (`animation: none`) to stand in for a browser without it, and the depth is
 * set the way the fallback script sets it: as an inline custom property.
 *
 * Functional spec: no millisecond thresholds. Nothing here bounds an overlay in the viewport.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const url = `${BASE_URL}/component/scroll_area/`;
const PROPS = ["--dx-scroll-fade-t", "--dx-scroll-fade-b", "--dx-scroll-fade-s", "--dx-scroll-fade-e"];

const scroller = (page: Page) => page.locator('#component-preview-frame-fade [data-demo="fade-y"] [data-scroll-fade="y"]');

async function open(page: Page): Promise<Locator> {
  await gotoHydrated(page, url, GOTO_OPTS);
  const el = scroller(page);
  await el.scrollIntoViewIfNeeded();
  await expect(el).toHaveClass(/dx-scroll-fade-y/);
  return el;
}

const transitionStyle = (el: Locator) =>
  el.evaluate((n) => {
    const cs = getComputedStyle(n);
    return { property: cs.transitionProperty, duration: cs.transitionDuration, timing: cs.transitionTimingFunction };
  });

/** Stand-in for a browser without scroll-driven animations: the depth is a plain, inline value. */
const withoutScrollTimeline = (el: Locator) => el.evaluate((n) => ((n as HTMLElement).style.animation = "none"));

/** Set one depth inline, as the fallback script does; return the transition the browser started for it. */
const setDepth = (el: Locator, prop: string, value: string) =>
  el.evaluate(
    (n, [prop, value]) => {
      (n as HTMLElement).style.setProperty(prop, value);
      getComputedStyle(n).getPropertyValue(prop); // flush style so the change is seen
      const running = n
        .getAnimations()
        .filter((a) => a instanceof CSSTransition && (a as CSSTransition).transitionProperty === prop) as CSSTransition[];
      return { count: running.length, duration: running[0]?.effect?.getTiming().duration ?? null };
    },
    [prop, value] as const,
  );

/** The depth in px with every running transition of it sought to `ms` (paused there). */
const depthAt = (el: Locator, prop: string, ms: number) =>
  el.evaluate(
    (n, [prop, ms]) => {
      for (const a of n.getAnimations()) {
        if (a instanceof CSSTransition && a.transitionProperty === prop) {
          a.pause();
          a.currentTime = ms;
        }
      }
      return parseFloat(getComputedStyle(n).getPropertyValue(prop));
    },
    [prop, ms] as const,
  );

test.describe("scroll fade edge transition", () => {
  test("each depth transitions with the theme's base duration and ease-out", async ({ page }) => {
    const el = await open(page);
    const { property, duration, timing } = await transitionStyle(el);
    const props = property.split(",").map((s) => s.trim());
    for (const p of PROPS) expect(props, `transition-property lists ${p}`).toContain(p);
    expect(props).toHaveLength(PROPS.length);
    expect(new Set(duration.split(",").map((s) => s.trim()))).toEqual(new Set(["0.15s"]));
    expect(new Set(timing.split(",").map((s) => s.trim()))).toEqual(new Set(["ease-out"]));
  });

  test("a depth that switches grows in and shrinks out over the transition, not at once", async ({ page }) => {
    const el = await open(page);
    await withoutScrollTimeline(el);
    for (const prop of PROPS) {
      // Settle at 0, then grow to 40px: the browser starts one 150ms transition and the middle is between.
      await el.evaluate((n, p) => (n as HTMLElement).style.setProperty(p, "0px"), prop);
      await el.evaluate((n) => n.getAnimations().forEach((a) => a.finish()));
      const grow = await setDepth(el, prop, "40px");
      expect(grow, `${prop} grows through a transition`).toEqual({ count: 1, duration: 150 });
      const half = await depthAt(el, prop, 75);
      expect(half, `${prop} half way (${half}px)`).toBeGreaterThan(0);
      expect(half).toBeLessThan(40);
      expect(await depthAt(el, prop, 150), `${prop} arrives at 40px`).toBeCloseTo(40, 3);
      await el.evaluate((n) => n.getAnimations().forEach((a) => a.finish()));
      // And back: shrinking is eased the same way.
      const shrink = await setDepth(el, prop, "0px");
      expect(shrink, `${prop} shrinks through a transition`).toEqual({ count: 1, duration: 150 });
      const back = await depthAt(el, prop, 75);
      expect(back).toBeGreaterThan(0);
      expect(back).toBeLessThan(40);
      await el.evaluate((n) => n.getAnimations().forEach((a) => a.finish()));
    }
  });

  test("under prefers-reduced-motion the transition is off and a depth switches at once", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    const el = await open(page);
    const { property, duration } = await transitionStyle(el);
    expect(property).toBe("none");
    expect(duration).toBe("0s");
    await withoutScrollTimeline(el);
    for (const prop of PROPS) {
      const grow = await setDepth(el, prop, "40px");
      expect(grow.count, `${prop} has no transition`).toBe(0);
      expect(parseFloat(await el.evaluate((n, p) => getComputedStyle(n).getPropertyValue(p), prop))).toBeCloseTo(40, 3);
    }
  });

  test("where the scroll timeline drives the depth, scrolling starts no transition", async ({ page }) => {
    const el = await open(page);
    const supported = await page.evaluate(() => CSS.supports("animation-timeline: scroll()"));
    test.skip(!supported, "needs scroll-driven animations");
    await el.evaluate((n) => ((n as HTMLElement).scrollTop = 120));
    await expect
      .poll(() => el.evaluate((n) => parseFloat(getComputedStyle(n).getPropertyValue("--dx-scroll-fade-t"))), {
        message: "the start edge faded in with the scroll",
      })
      .toBeGreaterThan(1);
    const transitions = await el.evaluate((n) => n.getAnimations().filter((a) => a instanceof CSSTransition).length);
    expect(transitions).toBe(0);
  });
});
