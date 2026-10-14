/**
 * The `fade: ScrollFade` prop on ScrollArea, VirtualList and MessageScrollerViewport: the
 * `dx-scroll-fade` utility of `assets/dx-effects.css`, put on the element that actually scrolls.
 *
 * What is observed. The utility is a mask whose edge depths are registered custom properties
 * (`--dx-scroll-fade-t/-b` for the block axis, `-s/-e` for the inline axis) that a scroll-driven
 * animation moves between 0 and the full depth as the scroller's own scroll offset changes. So the
 * computed value of those properties IS the fade at that scroll position; the spec resolves it to
 * pixels with a probe and compares it with the full depth (`min(12%, 40px)` of the box). Per
 * scroller:
 *
 *  - no overflow -> both edges 0 (no mask);
 *  - at the start -> the start edge crisp (0), the end edge at the full depth;
 *  - in the middle -> both at the full depth;
 *  - at the end -> the start edge at the full depth, the end edge crisp.
 *
 * Under virtualization the same must hold because the mask reads the container's scroll offset and
 * nothing else: a windowed VirtualList of 20,000 rows (a few dozen mounted), a content-visibility
 * VirtualList and MessageScroller rows, and a MessageScroller after history was prepended. RTL:
 * the horizontal ScrollArea strip flips its inline edges and the gradient direction.
 *
 * Functional spec: no millisecond thresholds (a debug build is fine). Nothing here bounds an
 * overlay in the viewport, so `fixedViewportBox` (check-spec-viewport-bound.sh) is not needed.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const url = (name: string) => `${BASE_URL}/component/?name=${name}&`;
const frame = (page: Page) => page.locator("#component-preview-frame-fade").first();

type Axis = "block" | "inline";
type Fade = { start: number; end: number; full: number; mask: string };

/** The two fade depths in px at the scroller's current scroll position, and the full depth. */
function readFade(el: Locator, axis: Axis): Promise<Fade> {
  return el.evaluate((node, axis) => {
    const cs = getComputedStyle(node);
    const rect = node.getBoundingClientRect();
    const extent = axis === "block" ? rect.height : rect.width;
    const [startProp, endProp] = axis === "block" ? ["--dx-scroll-fade-t", "--dx-scroll-fade-b"] : ["--dx-scroll-fade-s", "--dx-scroll-fade-e"];
    // Resolve a <length-percentage> (percentages are against the mask box) to px with a probe.
    const toPx = (value: string) => {
      const outer = document.createElement("div");
      outer.style.cssText = `position:absolute;visibility:hidden;pointer-events:none;${axis === "block" ? "height" : "width"}:${extent}px;`;
      const inner = document.createElement("div");
      inner.style[axis === "block" ? "height" : "width"] = value.trim() || "0px";
      outer.appendChild(inner);
      document.body.appendChild(outer);
      const px = inner.getBoundingClientRect()[axis === "block" ? "height" : "width"];
      outer.remove();
      return px;
    };
    return {
      start: toPx(cs.getPropertyValue(startProp)),
      end: toPx(cs.getPropertyValue(endProp)),
      full: Math.min(extent * 0.12, 40),
      mask: cs.maskImage || cs.webkitMaskImage || "",
    };
  }, axis);
}

/** Wait for the scroll-driven animation to catch up, then assert each edge is crisp or faded. */
async function expectEdges(el: Locator, axis: Axis, want: { start: "crisp" | "faded"; end: "crisp" | "faded" }) {
  await expect
    .poll(
      async () => {
        const f = await readFade(el, axis);
        const kind = (px: number) => (px < 1 ? "crisp" : Math.abs(px - f.full) <= 1.5 ? "faded" : `partial(${px.toFixed(1)}/${f.full.toFixed(1)})`);
        return `${kind(f.start)}/${kind(f.end)}`;
      },
      { timeout: 10_000, message: `edges of the ${axis} axis` },
    )
    .toBe(`${want.start}/${want.end}`);
}

const metrics = (el: Locator) =>
  el.evaluate((n) => ({
    sh: n.scrollHeight,
    ch: n.clientHeight,
    sw: n.scrollWidth,
    cw: n.clientWidth,
    st: Math.round(n.scrollTop),
    sl: Math.round(n.scrollLeft),
  }));

const scrollBlockTo = (el: Locator, where: "start" | "middle" | "end") =>
  el.evaluate((n, where) => {
    const max = n.scrollHeight - n.clientHeight;
    n.scrollTop = where === "start" ? 0 : where === "middle" ? max / 2 : max + 10_000;
  }, where);

const scrollInlineTo = (el: Locator, where: "start" | "middle" | "end") =>
  el.evaluate((n, where) => {
    const max = n.scrollWidth - n.clientWidth;
    const sign = getComputedStyle(n).direction === "rtl" ? -1 : 1;
    n.scrollLeft = sign * (where === "start" ? 0 : where === "middle" ? max / 2 : max + 10_000);
  }, where);

/** Pin the scroller at its end until it stops moving (a windowed list re-measures as it goes). */
async function settleAtEnd(el: Locator) {
  await expect
    .poll(async () => {
      await scrollBlockTo(el, "end");
      const m = await metrics(el);
      return m.sh - m.ch - m.st <= 2;
    }, { timeout: 15_000 })
    .toBe(true);
}

test.describe("ScrollArea fade", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, url("scroll_area"), GOTO_OPTS);
    await frame(page).scrollIntoViewIfNeeded();
  });

  test("the class lands on the scrolling element, never on a wrapper", async ({ page }) => {
    const y = frame(page).locator('[data-demo="fade-y"] [data-scroll-fade="y"]');
    await expect(y).toHaveClass(/dx-scroll-fade-y/);
    await expect(y).toHaveClass(/dx-scroll-area-auto-hide/); // the component's own class survives the merge
    expect(await y.evaluate((n) => getComputedStyle(n).overflowY)).toBe("auto");
    await expect(frame(page).locator('[data-demo="fade-y"]')).not.toHaveClass(/dx-scroll-fade/);
    expect(await y.evaluate((n) => (n.parentElement as HTMLElement).classList.toString())).not.toContain("dx-scroll-fade");
    // The unfaded variants of the same component carry no class at all.
    const plain = page.locator("#component-preview-frame-releases [data-scroll-direction]").first();
    await expect(plain).not.toHaveAttribute("data-scroll-fade", /.*/);
    expect(await plain.evaluate((n) => n.className)).not.toContain("dx-scroll-fade");
  });

  test("vertical: top, middle and end", async ({ page }) => {
    const y = frame(page).locator('[data-demo="fade-y"] [data-scroll-fade="y"]');
    await expect(y).toBeVisible();
    const m = await metrics(y);
    expect(m.sh).toBeGreaterThan(m.ch + 200); // room for a middle that is clear of both reveal ranges
    expect((await readFade(y, "block")).mask).toContain("linear-gradient");
    await scrollBlockTo(y, "start");
    await expectEdges(y, "block", { start: "crisp", end: "faded" });
    await scrollBlockTo(y, "middle");
    await expectEdges(y, "block", { start: "faded", end: "faded" });
    await scrollBlockTo(y, "end");
    await expectEdges(y, "block", { start: "faded", end: "crisp" });
    await scrollBlockTo(y, "start");
    await expectEdges(y, "block", { start: "crisp", end: "faded" });
  });

  test("a list that fits has no fade", async ({ page }) => {
    const short = frame(page).locator('[data-demo="fade-short"] [data-scroll-fade="y"]');
    await expect(short).toBeVisible();
    const m = await metrics(short);
    expect(m.sh).toBeLessThanOrEqual(m.ch);
    await expectEdges(short, "block", { start: "crisp", end: "crisp" });
  });

  test("a list that stops overflowing loses its fade, even from the middle of the scroll range", async ({ page }) => {
    // Chromium keeps the last value of a filled scroll-driven animation when its timeline turns
    // inactive; the utility has no fill mode for that reason (dx-effects.css).
    const y = frame(page).locator('[data-demo="fade-y"] [data-scroll-fade="y"]');
    await scrollBlockTo(y, "middle");
    await expectEdges(y, "block", { start: "faded", end: "faded" });
    await y.evaluate((n) => ((n as HTMLElement).style.height = "4000px"));
    expect((await metrics(y)).sh).toBeLessThanOrEqual((await metrics(y)).ch);
    await expectEdges(y, "block", { start: "crisp", end: "crisp" });
    // And it comes back with the overflow.
    await y.evaluate((n) => ((n as HTMLElement).style.height = "12rem"));
    await scrollBlockTo(y, "start");
    await expectEdges(y, "block", { start: "crisp", end: "faded" });
  });

  test("horizontal: the strip fades at its inline edges", async ({ page }) => {
    const x = frame(page).locator('[data-demo="fade-x"] [data-scroll-fade="x"]');
    await expect(x).toHaveClass(/dx-scroll-fade-x/);
    const m = await metrics(x);
    expect(m.sw).toBeGreaterThan(m.cw + 200);
    expect((await readFade(x, "inline")).mask).toContain("to right");
    await scrollInlineTo(x, "start");
    await expectEdges(x, "inline", { start: "crisp", end: "faded" });
    await scrollInlineTo(x, "middle");
    await expectEdges(x, "inline", { start: "faded", end: "faded" });
    await scrollInlineTo(x, "end");
    await expectEdges(x, "inline", { start: "faded", end: "crisp" });
  });

  test("horizontal in RTL: the inline start is the right edge and the gradient mirrors", async ({ page }) => {
    const x = frame(page).locator('[data-demo="fade-x"] [data-scroll-fade="x"]');
    await frame(page).getByRole("button", { name: "Right-to-left strip" }).click();
    await expect(x).toHaveAttribute("dir", "rtl");
    expect((await readFade(x, "inline")).mask).toContain("to left");
    await scrollInlineTo(x, "start");
    expect(await x.evaluate((n) => n.scrollLeft)).toBeLessThanOrEqual(0);
    await expectEdges(x, "inline", { start: "crisp", end: "faded" });
    await scrollInlineTo(x, "middle");
    await expectEdges(x, "inline", { start: "faded", end: "faded" });
    await scrollInlineTo(x, "end");
    expect(await x.evaluate((n) => n.scrollLeft)).toBeLessThan(-100);
    await expectEdges(x, "inline", { start: "faded", end: "crisp" });
  });
});

test.describe("VirtualList fade", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, url("virtual_list"), GOTO_OPTS);
    await frame(page).scrollIntoViewIfNeeded();
  });

  test("windowed: the fade follows the scroll position over 20,000 rows", async ({ page }) => {
    const list = frame(page).locator('[data-demo="fade-windowed"] [role="list"]');
    await expect(list).toHaveAttribute("data-virtualize", "windowed");
    await expect(list).toHaveAttribute("data-scroll-fade", "y");
    await expect(list).toHaveClass(/dx-scroll-fade-y/);
    await expect(list).toHaveClass(/dx-virtual-list-container/); // merged, not replaced
    await expect(list.locator('[role="listitem"]').first()).toBeVisible({ timeout: 60_000 });
    // Virtualized: a few dozen rows are mounted, not 20,000.
    expect(await list.locator('[role="listitem"]').count()).toBeLessThan(100);

    await scrollBlockTo(list, "start");
    await expectEdges(list, "block", { start: "crisp", end: "faded" });

    await scrollBlockTo(list, "middle");
    await expectEdges(list, "block", { start: "faded", end: "faded" });
    // The window really moved: the first mounted row is far from row 0.
    await expect
      .poll(() => list.locator('[role="listitem"]').first().getAttribute("data-virtual-index").then(Number), { timeout: 10_000 })
      .toBeGreaterThan(1000);
    expect(await list.locator('[role="listitem"]').count()).toBeLessThan(100);

    await settleAtEnd(list);
    await expectEdges(list, "block", { start: "faded", end: "crisp" });
    await expect
      .poll(() => list.locator('[role="listitem"]').last().getAttribute("data-virtual-index").then(Number), { timeout: 10_000 })
      .toBeGreaterThan(19_900);

    await scrollBlockTo(list, "start");
    await expectEdges(list, "block", { start: "crisp", end: "faded" });
  });

  test("windowed: three rows fit, so there is no fade", async ({ page }) => {
    const list = frame(page).locator('[data-demo="fade-windowed"] [role="list"]');
    await expect(list.locator('[role="listitem"]').first()).toBeVisible({ timeout: 60_000 });
    await frame(page).getByRole("button", { name: "3 rows", exact: true }).click();
    await expect(list.locator('[role="listitem"]')).toHaveCount(3);
    const m = await metrics(list);
    expect(m.sh).toBeLessThanOrEqual(m.ch);
    await expectEdges(list, "block", { start: "crisp", end: "crisp" });
    // And back: the fade returns with the overflow.
    await frame(page).getByRole("button", { name: "20000 rows", exact: true }).click();
    await expect(list.locator('[role="listitem"]').nth(10)).toBeAttached({ timeout: 30_000 });
    await scrollBlockTo(list, "start");
    await expectEdges(list, "block", { start: "crisp", end: "faded" });
  });

  test("content-visibility: every row is in the DOM and the fade still follows the scroll", async ({ page }) => {
    const list = frame(page).locator('[data-demo="fade-content-visibility"] [role="list"]');
    await expect(list).toHaveAttribute("data-virtualize", "content-visibility");
    await expect(list).toHaveClass(/dx-scroll-fade-y/);
    await expect(list.locator('[role="listitem"]')).toHaveCount(300);
    await scrollBlockTo(list, "start");
    await expectEdges(list, "block", { start: "crisp", end: "faded" });
    await scrollBlockTo(list, "middle");
    await expectEdges(list, "block", { start: "faded", end: "faded" });
    await settleAtEnd(list);
    await expectEdges(list, "block", { start: "faded", end: "crisp" });
  });
});

test.describe("MessageScroller fade", () => {
  const demo = (page: Page) => frame(page).locator('[data-demo="fade"]');
  const viewportOf = (page: Page) => demo(page).locator("[data-message-scroller-viewport]");
  const button = (page: Page, name: string) => frame(page).getByRole("button", { name, exact: true });

  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, url("message_scroller"), GOTO_OPTS);
    const vp = viewportOf(page);
    await vp.scrollIntoViewIfNeeded();
    await expect(vp).not.toHaveAttribute("data-pending-scroll");
  });

  test("the class lands on the viewport, never on the frame", async ({ page }) => {
    const vp = viewportOf(page);
    await expect(vp).toHaveClass(/dx-scroll-fade-t/);
    await expect(vp).toHaveClass(/dx-message-scroller-viewport/);
    expect(await vp.evaluate((n) => getComputedStyle(n).overflowY)).toBe("auto");
    await expect(demo(page).locator('[data-slot="message-scroller"]')).not.toHaveClass(/dx-scroll-fade/);
    await expect(demo(page).locator('[data-slot="message-scroller-content"]')).toHaveAttribute("data-virtualize", "content-visibility");
  });

  test("history above fades: crisp only at the very top", async ({ page }) => {
    const vp = viewportOf(page);
    // The thread opens at the newest messages, far from the top.
    await expect.poll(async () => {
      const m = await metrics(vp);
      return m.sh - m.ch - m.st <= 2;
    }).toBe(true);
    const m = await metrics(vp);
    expect(m.sh).toBeGreaterThan(m.ch + 600);
    // A top-only fade: the block gradient is the one-edge mask and the start edge is faded.
    await expectEdges(vp, "block", { start: "faded", end: "crisp" });
    await scrollBlockTo(vp, "middle");
    await expectEdges(vp, "block", { start: "faded", end: "crisp" });
    await scrollBlockTo(vp, "start");
    await expectEdges(vp, "block", { start: "crisp", end: "crisp" });
  });

  test("after prepending history the fade still follows the scroll position", async ({ page }) => {
    const vp = viewportOf(page);
    await scrollBlockTo(vp, "start");
    await expectEdges(vp, "block", { start: "crisp", end: "crisp" });
    const before = await metrics(vp);
    await button(page, "Load earlier messages").click();
    // Prepending keeps the reader's place: the viewport moves down by the prepended height, so the
    // top edge is no longer the start of the scroll range and fades in.
    await expect.poll(async () => (await metrics(vp)).sh, { timeout: 10_000 }).toBeGreaterThan(before.sh + 100);
    await expect.poll(async () => (await metrics(vp)).st, { timeout: 10_000 }).toBeGreaterThan(96);
    await expectEdges(vp, "block", { start: "faded", end: "crisp" });
    // Scrolling back to the new top: crisp again.
    await scrollBlockTo(vp, "start");
    await expectEdges(vp, "block", { start: "crisp", end: "crisp" });
  });

  test("both edges, the bottom edge and none", async ({ page }) => {
    const vp = viewportOf(page);
    await expect.poll(async () => {
      const m = await metrics(vp);
      return m.sh - m.ch - m.st <= 2;
    }).toBe(true);

    await button(page, "Both").click();
    await expect(vp).toHaveClass(/dx-scroll-fade-y/);
    await expect(vp).not.toHaveClass(/dx-scroll-fade-t/);
    await scrollBlockTo(vp, "end");
    await expectEdges(vp, "block", { start: "faded", end: "crisp" });
    await scrollBlockTo(vp, "middle");
    await expectEdges(vp, "block", { start: "faded", end: "faded" });
    await scrollBlockTo(vp, "start");
    await expectEdges(vp, "block", { start: "crisp", end: "faded" });

    await button(page, "Bottom").click();
    await expect(vp).toHaveClass(/dx-scroll-fade-b/);
    await scrollBlockTo(vp, "middle");
    await expectEdges(vp, "block", { start: "crisp", end: "faded" });

    await button(page, "None").click();
    await expect(vp).not.toHaveAttribute("data-scroll-fade", /.*/);
    expect(await vp.evaluate((n) => n.className)).not.toContain("dx-scroll-fade");
    expect(await vp.evaluate((n) => getComputedStyle(n).maskImage)).toBe("none");
  });

  test("a transcript that fits has no fade", async ({ page }) => {
    const vp = viewportOf(page);
    await button(page, "Both").click();
    // Grow the frame past the content: nothing is left to scroll.
    await demo(page).evaluate((n) => ((n as HTMLElement).style.height = "20000px"));
    await expect.poll(async () => {
      const m = await metrics(vp);
      return m.sh <= m.ch;
    }).toBe(true);
    await expectEdges(vp, "block", { start: "crisp", end: "crisp" });
  });
});
