/**
 * Message Scroller API parity items (backlog row 213, items 1, 2, 4, 5, 6, 7), each against the real
 * demos:
 *
 * - `use_message_scroller_visibility` (`visibility` demo): the readout follows the turn being read, and
 *   the controller observes rows ONLY for the subscribers that exist (ref-counted: one demo reads it, so
 *   one IntersectionObserver is rooted in a message-scroller viewport on a page of a dozen scrollers).
 * - per-call `scroll_margin` and `ScrollBehavior::Instant` (the `visibility` demo's footer buttons).
 * - live `default_scroll_position` (`opening_position`: the tab changes the prop of a mounted scroller).
 * - the `dx-enter-*` presets of `dx-effects.css` (the `animation` demo).
 * - `InputGroupAlign::BlockStart` / `BlockEnd` (the `input_group` `block` demo).
 * - RTL: the jump button stays centred under `dir="rtl"`.
 *
 * No assertion is a millisecond threshold; positions are compared in pixels (the release-only timing
 * spec is `message-scroller-timing.spec.ts`).
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const url = (name: string) => `${BASE_URL}/component/${name}/`;
const frame = (page: Page, variant: string) =>
  page.locator(variant === "main" ? "#component-preview-frame" : `#component-preview-frame-${variant}`).first();
const viewportOf = (page: Page, variant: string) => frame(page, variant).locator("[data-message-scroller-viewport]");

async function open(page: Page, name: string, variant: string, ready: string) {
  await gotoHydrated(page, url(name), GOTO_OPTS);
  const root = frame(page, variant);
  await root.scrollIntoViewIfNeeded();
  await expect(root.locator(ready).first()).toBeVisible({ timeout: 60_000 });
  return root;
}

const offsetOf = (vp: Locator, id: string) =>
  vp.evaluate((el, id) => {
    const row = el.querySelector(`[data-message-id="${id}"]`) as HTMLElement;
    return Math.round(row.getBoundingClientRect().top - el.getBoundingClientRect().top);
  }, id);
const setScrollTop = (vp: Locator, where: "top" | "bottom") =>
  vp.evaluate((el, where) => {
    el.scrollTop = where === "top" ? 0 : el.scrollHeight;
  }, where);

test.describe("message_scroller: visibility", () => {
  test("the readout follows the turn being read", async ({ page }) => {
    const root = await open(page, "message_scroller", "visibility", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "visibility");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    const readout = root.getByTestId("visibility-readout");

    await expect(readout).toHaveText(/^Turn 1, \d+ rows in view$/);
    // A turn is current once its anchor's top reaches the reading line, not merely once it is in view: at
    // the very end the last user message is still below the line, so the previous turn is current.
    await setScrollTop(vp, "bottom");
    await expect(readout).toHaveText(/^Turn 3, \d+ rows in view$/);
    await setScrollTop(vp, "top");
    await expect(readout).toHaveText(/^Turn 1, \d+ rows in view$/);
    // A jump that puts turn 3 at the top makes it current.
    await root.getByRole("button", { name: "Jump to turn 3", exact: true }).click();
    await expect(readout).toHaveText(/^Turn 3, \d+ rows in view$/);
    await setScrollTop(vp, "top");
    await expect(readout).toHaveText(/^Turn 1, \d+ rows in view$/);
    // The count is real: the transcript is taller than the viewport, so not all 8 rows are in view.
    const rows = Number((await readout.innerText()).match(/, (\d+) rows/)![1]);
    expect(rows).toBeGreaterThan(0);
    expect(rows).toBeLessThan(8);
  });

  test("rows are observed only while a subscriber exists", async ({ page }) => {
    await page.addInitScript(() => {
      const Native = window.IntersectionObserver;
      (window as unknown as { __scrollerObservers: number }).__scrollerObservers = 0;
      window.IntersectionObserver = class extends Native {
        constructor(callback: IntersectionObserverCallback, options?: IntersectionObserverInit) {
          super(callback, options);
          const root = options?.root;
          if (root instanceof Element && root.hasAttribute("data-message-scroller-viewport")) {
            (window as unknown as { __scrollerObservers: number }).__scrollerObservers++;
          }
        }
      };
    });
    await open(page, "message_scroller", "visibility", "[data-message-scroller-viewport]");
    await expect(frame(page, "visibility").getByTestId("visibility-readout")).toHaveText(/Turn 1/);
    // Every other scroller on the page (main, long, ...) never subscribed.
    const roots = await page.locator("[data-message-scroller-viewport]").count();
    expect(roots).toBeGreaterThan(3);
    expect(await page.evaluate(() => (window as unknown as { __scrollerObservers: number }).__scrollerObservers)).toBe(1);
  });
});

test.describe("message_scroller: per-call margin and instant jumps", () => {
  test("scroll_margin shifts the landing spot by exactly its value, for that call only", async ({ page }) => {
    const root = await open(page, "message_scroller", "visibility", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "visibility");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");

    await root.getByRole("button", { name: "Jump to turn 3", exact: true }).click();
    await expect.poll(async () => (await offsetOf(vp, "visibility-q3")) > 0).toBe(true);
    await page.waitForTimeout(1200); // past the settling window
    const plain = await offsetOf(vp, "visibility-q3");

    await setScrollTop(vp, "top");
    await root.getByRole("button", { name: "Jump with margin" }).click();
    await expect.poll(async () => (await offsetOf(vp, "visibility-q3")) - plain, { timeout: 10_000 }).toBe(24);
    await page.waitForTimeout(1200);
    expect((await offsetOf(vp, "visibility-q3")) - plain).toBe(24);

    // The margin was for that call: the next plain jump is back at the provider's margin.
    await setScrollTop(vp, "top");
    await root.getByRole("button", { name: "Jump to turn 3", exact: true }).click();
    await expect.poll(async () => (await offsetOf(vp, "visibility-q3")) - plain, { timeout: 10_000 }).toBe(0);
  });

  test("Instant jumps at once even where the viewport's CSS scroll-behavior is smooth", async ({ page }) => {
    const root = await open(page, "message_scroller", "visibility", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "visibility");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    // With `smooth` here, a plain `auto` jump would animate through many scrollTop values.
    await vp.evaluate((el) => {
      el.style.scrollBehavior = "smooth";
      (el as unknown as { __tops: number[] }).__tops = [];
      el.addEventListener("scroll", () => (el as unknown as { __tops: number[] }).__tops.push(Math.round(el.scrollTop)));
    });
    await root.getByRole("button", { name: "Jump to turn 3", exact: true }).click();
    await expect.poll(async () => (await offsetOf(vp, "visibility-q3")) > 0).toBe(true);
    await page.waitForTimeout(1200);
    const tops = await vp.evaluate((el) => (el as unknown as { __tops: number[] }).__tops);
    // One jump to the target (a re-aim of a settling jump may add a value or two); never an animation.
    expect(new Set(tops).size).toBeLessThanOrEqual(3);
    expect(tops.length).toBeGreaterThan(0);
  });
});

test.describe("message_scroller: live default_scroll_position", () => {
  test("changing the prop re-applies the opening position without remounting", async ({ page }) => {
    const root = await open(page, "message_scroller", "opening_position", "[data-message-scroller-viewport]");
    const vp = viewportOf(page, "opening_position");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    await vp.evaluate((el) => el.setAttribute("data-mounted-marker", "same-node"));

    await root.getByRole("tab", { name: "start", exact: true }).click();
    await expect.poll(() => vp.evaluate((el) => Math.round(el.scrollTop)), { timeout: 10_000 }).toBe(0);
    await root.getByRole("tab", { name: "end", exact: true }).click();
    await expect
      .poll(() => vp.evaluate((el) => Math.round(el.scrollHeight - el.scrollTop - el.clientHeight)), { timeout: 10_000 })
      .toBeLessThanOrEqual(2);
    // The same DOM node throughout: a live change, not a keyed remount.
    await expect(vp).toHaveAttribute("data-mounted-marker", "same-node");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
  });
});

test.describe("message_scroller: entrance presets", () => {
  for (const [name, keyframes] of [
    ["Spring bounce", "dx-enter-spring-bounce"],
    ["Blur fade", "dx-enter-blur-fade"],
    ["Slide side", "dx-enter-slide-side"],
  ] as const) {
    test(`${name} plays ${keyframes} on the user row and animates no layout property`, async ({ page }) => {
      const root = await open(page, "message_scroller", "animation", '[data-slot="card"]');
      await root.getByRole("combobox", { name: "Animation preset" }).click();
      await page.getByRole("option", { name, exact: true }).click();
      await root.getByRole("button", { name: "Send Message" }).click();
      const row = root.locator('[data-message-id="m0"] [data-slot="message"]');
      await expect(row).toHaveClass(new RegExp(keyframes));
      const animation = await row.evaluate((el) => getComputedStyle(el).animationName);
      expect(animation).toBe(keyframes);
      // The keyframes touch only opacity / scale / translate / filter.
      const properties = await page.evaluate((keyframes) => {
        const found = new Set<string>();
        for (const sheet of Array.from(document.styleSheets)) {
          let rules: CSSRuleList;
          try {
            rules = sheet.cssRules;
          } catch {
            continue;
          }
          for (const rule of Array.from(rules)) {
            if (rule instanceof CSSKeyframesRule && rule.name === keyframes) {
              for (const frame of Array.from(rule.cssRules) as CSSKeyframeRule[]) {
                for (let i = 0; i < frame.style.length; i++) found.add(frame.style[i]);
              }
            }
          }
        }
        return [...found].sort();
      }, keyframes);
      expect(properties.length).toBeGreaterThan(0);
      for (const property of properties) {
        expect(["opacity", "scale", "translate", "filter"]).toContain(property);
      }
    });
  }
});

test.describe("input_group: block addons", () => {
  test("BlockEnd sits in a full-width row below the control, BlockStart above it", async ({ page }) => {
    const root = await open(page, "input_group", "block", ".dx-input-group");
    const groups = root.locator(".dx-input-group");
    const box = async (loc: Locator) => (await loc.boundingBox())!;

    const end = groups.nth(0);
    const endInput = await box(end.locator("input"));
    const endAddon = await box(end.locator('[data-align="block-end"]'));
    const endGroup = await box(end);
    expect(endAddon.y).toBeGreaterThanOrEqual(endInput.y + endInput.height - 1);
    expect(Math.abs(endAddon.width - endGroup.width)).toBeLessThanOrEqual(2);

    const start = groups.nth(1);
    const startInput = await box(start.locator("input"));
    const startAddon = await box(start.locator('[data-align="block-start"]'));
    expect(startAddon.y + startAddon.height).toBeLessThanOrEqual(startInput.y + 1);

    // An inline-only group is unchanged: its input and addon share a row.
    await expect(end.locator('[data-align="block-end"]')).toHaveCSS("order", "1");
    await expect(start.locator('[data-align="block-start"]')).toHaveCSS("order", "-1");
  });
});

test.describe("message_scroller: RTL", () => {
  test("the jump button stays centred in both directions", async ({ page }) => {
    await gotoHydrated(page, url("message_scroller"), GOTO_OPTS);
    const root = frame(page, "main");
    await root.scrollIntoViewIfNeeded();
    const vp = viewportOf(page, "main");
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
    // The opening pin is settled only once the viewport has published its state (`data-scrollable`, written in
    // the same frame as the pin) and the pin's 180 ms autoscroll window has closed (`data-autoscrolling` gone).
    // Absence of `data-pending-scroll` alone is not enough: it clears in that same frame, so a `scrollTop = 0`
    // issued right after lands inside the window, where a scroll up cannot release follow mode and the end
    // button stays inactive for good (backlog row 347).
    await expect(vp).toHaveAttribute("data-scrollable", /\bstart\b/);
    await expect(vp).not.toHaveAttribute("data-autoscrolling");
    const button = root.locator('[data-message-scroller-button][data-direction="end"]');
    const centred = async () => {
      await vp.evaluate((el) => (el.scrollTop = 0));
      await expect(button).toHaveAttribute("data-active", "true");
      await expect(button).toHaveCSS("opacity", "1");
      const frameBox = (await root.locator("[data-message-scroller-root]").boundingBox())!;
      const buttonBox = (await button.boundingBox())!;
      return Math.abs(buttonBox.x + buttonBox.width / 2 - (frameBox.x + frameBox.width / 2));
    };
    expect(await centred()).toBeLessThanOrEqual(1);
    await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
    expect(await centred()).toBeLessThanOrEqual(1);
  });
});

test.describe("message_scroller: follow mode and the autoscroll window", () => {
  test("a scroll up inside the pin's autoscroll window releases follow (row 347)", async ({ page }) => {
    await gotoHydrated(page, url("message_scroller"), GOTO_OPTS);
    const root = frame(page, "main");
    await root.scrollIntoViewIfNeeded();
    const vp = viewportOf(page, "main");
    await expect(vp).toHaveAttribute("data-scrollable", /\bstart\b/);
    await expect(vp).not.toHaveAttribute("data-autoscrolling");
    const button = root.locator('[data-message-scroller-button][data-direction="end"]');
    // Leave the pinned end by hand, then press the jump button: it re-arms follow and opens the 180 ms window.
    await vp.evaluate((el) => (el.scrollTop = 0));
    await expect(button).toHaveAttribute("data-active", "true");
    // Once the pin has really moved the viewport down (first scroll event of its smooth scroll, while the 180 ms
    // window is open), scroll back to the top in that same event: an upward move that lands inside the window,
    // with no timing assumption. The move must follow a downward one, or there is nothing to release.
    await vp.evaluate((el) => {
      const onScroll = () => {
        if (el.scrollTop > 0 && el.hasAttribute("data-autoscrolling")) {
          el.removeEventListener("scroll", onScroll);
          el.scrollTop = 0;
        }
      }; // listener-ok: removed by the handler itself, spec-only
      el.addEventListener("scroll", onScroll);
    });
    await button.click();
    await expect(vp).not.toHaveAttribute("data-autoscrolling");
    expect(await vp.evaluate((el) => Math.round(el.scrollTop))).toBe(0);
    // Follow was released: the viewport sits at the top and the jump button is offered again.
    await expect(button).toHaveAttribute("data-active", "true");
  });
});
