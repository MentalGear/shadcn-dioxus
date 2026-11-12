/**
 * The scroll fade where a browser has no scroll-driven animations (Firefox). `dx-effects.css` then computes
 * the four edge depths in its `@supports not (animation-timeline: scroll())` block from numbers the
 * scroller's runtime publishes (`--dx-scroll-top/-left`, `--dx-scroll-range-y/-x`): an edge fades only while
 * more than 1px of content lies past it.
 *
 * Chromium has the animations, so this spec stands in for the missing engine in two steps: `CSS.supports`
 * answers false for `animation-timeline` before the page runs (so the runtime takes the fallback path), and
 * the fallback block's own rules are copied out of the loaded stylesheet without their `@supports` condition,
 * with the scroller's animation switched off. What is observed is the computed edge depth at each scroll
 * position, the same observable as `scroll-fade.spec.ts`.
 *
 * Functional spec: no millisecond thresholds.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const frame = (page: Page) => page.locator("#component-preview-frame-fade").first();

async function open(page: Page) {
  await page.addInitScript(() => {
    const supports = CSS.supports.bind(CSS);
    CSS.supports = ((...args: [string] | [string, string]) =>
      /animation-timeline/.test(args.join(" ")) ? false : (supports as (...a: unknown[]) => boolean)(...args)) as typeof CSS.supports;
  });
  await gotoHydrated(page, `${BASE_URL}/component/scroll_area/`, GOTO_OPTS);
  // the fallback block, as plain rules
  await page.evaluate(() => {
    const rules: string[] = [];
    for (const sheet of Array.from(document.styleSheets)) {
      let list: CSSRuleList;
      try {
        list = sheet.cssRules;
      } catch {
        continue;
      }
      for (const rule of Array.from(list)) {
        if (rule instanceof CSSSupportsRule && /not/.test(rule.conditionText) && /animation-timeline/.test(rule.conditionText)) {
          for (const inner of Array.from(rule.cssRules)) rules.push(inner.cssText);
        }
      }
    }
    const style = document.createElement("style");
    style.id = "fallback-rules";
    style.textContent = rules.join("\n");
    document.head.appendChild(style);
  });
}

/** An edge depth in px at the scroller's current state (percentages are against the mask box). */
const depth = (el: Locator, prop: string, axis: "height" | "width") =>
  el.evaluate(
    (node, [prop, axis]) => {
      const rect = node.getBoundingClientRect();
      const outer = document.createElement("div");
      outer.style.cssText = `position:absolute;visibility:hidden;${axis}:${rect[axis as "height" | "width"]}px;`;
      const inner = document.createElement("div");
      inner.style[axis as "height" | "width"] = getComputedStyle(node).getPropertyValue(prop).trim() || "0px";
      outer.appendChild(inner);
      document.body.appendChild(outer);
      const px = inner.getBoundingClientRect()[axis as "height" | "width"];
      outer.remove();
      return px;
    },
    [prop, axis] as const,
  );

test.describe("scroll fade without scroll-driven animations", () => {
  test("the block edges fade only while there is more to scroll to, and an unscrollable list has no fade", async ({ page }) => {
    await open(page);
    const y = frame(page).locator('[data-demo="fade-y"] [data-scroll-fade="y"]');
    await y.evaluate((n) => ((n as HTMLElement).style.animation = "none"));
    const edges = async () => [await depth(y, "--dx-scroll-fade-t", "height"), await depth(y, "--dx-scroll-fade-b", "height")];
    await y.evaluate((n) => (n.scrollTop = 0));
    await expect.poll(async () => (await edges())[0]).toBe(0);
    expect((await edges())[1]).toBeGreaterThan(0);
    await y.evaluate((n) => (n.scrollTop = (n.scrollHeight - n.clientHeight) / 2));
    await expect.poll(async () => (await edges())[0]).toBeGreaterThan(0);
    expect((await edges())[1]).toBeGreaterThan(0);
    await y.evaluate((n) => (n.scrollTop = n.scrollHeight));
    await expect.poll(async () => (await edges())[1]).toBe(0);
    expect((await edges())[0]).toBeGreaterThan(0);

    const short = frame(page).locator('[data-demo="fade-short"] [data-scroll-fade="y"]');
    await short.evaluate((n) => ((n as HTMLElement).style.animation = "none"));
    await expect.poll(async () => [await depth(short, "--dx-scroll-fade-t", "height"), await depth(short, "--dx-scroll-fade-b", "height")]).toEqual([0, 0]);
  });

  test("the inline edges fade the same way", async ({ page }) => {
    await open(page);
    const x = frame(page).locator('[data-demo="fade-x"] [data-scroll-fade="x"]');
    await x.evaluate((n) => ((n as HTMLElement).style.animation = "none"));
    const edges = async () => [await depth(x, "--dx-scroll-fade-s", "width"), await depth(x, "--dx-scroll-fade-e", "width")];
    await x.evaluate((n) => (n.scrollLeft = 0));
    await expect.poll(async () => (await edges())[0]).toBe(0);
    expect((await edges())[1]).toBeGreaterThan(0);
    await x.evaluate((n) => (n.scrollLeft = (n.scrollWidth - n.clientWidth) / 2));
    await expect.poll(async () => (await edges())[0]).toBeGreaterThan(0);
    expect((await edges())[1]).toBeGreaterThan(0);
    await x.evaluate((n) => (n.scrollLeft = n.scrollWidth));
    await expect.poll(async () => (await edges())[1]).toBe(0);
    expect((await edges())[0]).toBeGreaterThan(0);
  });
});
