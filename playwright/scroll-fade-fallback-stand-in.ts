/**
 * The stand-in for a browser without scroll-driven animations (Firefox), for specs that observe the scroll
 * fade's fallback path on Chromium. The runtime (`use_scroll_fade`) installs its fallback where
 * `CSS.supports('animation-timeline: scroll()')` is false and publishes the scroll numbers; `dx-effects.css`
 * computes the four edge depths from them in its `@supports not (animation-timeline: scroll())` block.
 * Chromium applies neither the false answer nor that block, so the stand-in does three things:
 *
 *  1. `CSS.supports` answers false for `animation-timeline`, before any page script runs;
 *  2. the fallback block's own rules are copied out of the loaded stylesheet without their `@supports`
 *     condition, so the stylesheet path is what is read, not a spec-side formula;
 *  3. the scroll-driven animations are switched off, so they do not override the computed depths.
 */
import type { Page } from "@playwright/test";
import { gotoHydrated } from "./hydration";

/** Opens `url` as a browser without scroll-driven animations would show it. */
export async function gotoWithoutScrollTimeline(page: Page, url: string, options: { timeout: number }) {
  await page.addInitScript(() => {
    const supports = CSS.supports.bind(CSS);
    CSS.supports = ((...args: string[]) =>
      /animation-timeline/.test(args.join(" ")) ? false : (supports as (...a: string[]) => boolean)(...args)) as typeof CSS.supports;
  });
  await gotoHydrated(page, url, options);
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
    if (!rules.length) throw new Error("the fallback block of dx-effects.css is not in the page");
    const style = document.createElement("style");
    style.id = "scroll-fade-fallback-rules";
    style.textContent = rules.join("\n") + '\n[class*="dx-scroll-fade"] { animation: none !important; }';
    document.head.appendChild(style);
  });
}
