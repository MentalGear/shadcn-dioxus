/**
 * The Utilities section of /docs: the standalone `dx-scroll-fade` and `dx-shimmer` classes of
 * `assets/dx-effects.css`, on plain elements (no ScrollArea, Marker or Attachment around them).
 * `scroll-fade.spec.ts` owns the components' `fade` prop; this spec owns what that one does not:
 * the bare classes, the one-edge and `-none` switches, the depth variable, and the shimmer.
 *
 * What is observed. scroll-fade: the fade depths are registered custom properties
 * (`--dx-scroll-fade-t/-b` block, `-s/-e` inline) that a scroll-driven animation moves between 0 and
 * the full depth, so their computed value IS the fade at that scroll position (resolved to px with a
 * probe). shimmer: the computed `animation-name` / `animation-play-state` / `background-image` /
 * `-webkit-text-fill-color` of the status line, and the `data-dx-motion` attribute the `use_motion`
 * gate puts on it. Functional spec: no millisecond thresholds, a debug build is fine.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const demo = (page: Page, name: string) => page.locator(`[data-demo="${name}"]`);

type Axis = "block" | "inline";

function readFade(el: Locator, axis: Axis, fullPx?: number) {
  return el.evaluate(
    (node, [axis, fullPx]) => {
      const cs = getComputedStyle(node);
      const rect = node.getBoundingClientRect();
      const extent = axis === "block" ? rect.height : rect.width;
      const [s, e] = axis === "block" ? ["--dx-scroll-fade-t", "--dx-scroll-fade-b"] : ["--dx-scroll-fade-s", "--dx-scroll-fade-e"];
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
        start: toPx(cs.getPropertyValue(s)),
        end: toPx(cs.getPropertyValue(e)),
        full: (fullPx as number | undefined) ?? Math.min(extent * 0.12, 40),
        mask: cs.maskImage || cs.webkitMaskImage || "",
      };
    },
    [axis, fullPx] as [Axis, number | undefined],
  );
}

async function expectEdges(el: Locator, axis: Axis, want: { start: "crisp" | "faded"; end: "crisp" | "faded" }, fullPx?: number) {
  await expect
    .poll(
      async () => {
        const f = await readFade(el, axis, fullPx);
        const kind = (px: number) => (px < 1 ? "crisp" : Math.abs(px - f.full) <= 1.5 ? "faded" : `partial(${px.toFixed(1)}/${f.full.toFixed(1)})`);
        return `${kind(f.start)}/${kind(f.end)}`;
      },
      { timeout: 10_000, message: `edges of the ${axis} axis` },
    )
    .toBe(`${want.start}/${want.end}`);
}

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

test.describe("scroll-fade utility on plain elements", () => {
  let y: Locator;
  let x: Locator;
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/docs`, GOTO_OPTS);
    y = demo(page, "util-fade-y").locator('[data-scroller="y"]');
    x = demo(page, "util-fade-x").locator('[data-scroller="x"]');
    await y.scrollIntoViewIfNeeded();
  });

  test("the bare class fades only a box that overflows, and follows the scroll position", async () => {
    await expect(y).toHaveClass(/dx-scroll-fade\b/);
    expect(await y.evaluate((n) => n.scrollHeight - n.clientHeight)).toBeGreaterThan(200);
    expect((await readFade(y, "block")).mask).toContain("linear-gradient");
    await scrollBlockTo(y, "start");
    await expectEdges(y, "block", { start: "crisp", end: "faded" });
    await scrollBlockTo(y, "middle");
    await expectEdges(y, "block", { start: "faded", end: "faded" });
    await scrollBlockTo(y, "end");
    await expectEdges(y, "block", { start: "faded", end: "crisp" });
    // Nothing left to scroll: no fade, even coming from the middle of the range.
    await scrollBlockTo(y, "middle");
    await y.evaluate((n) => ((n as HTMLElement).style.height = "4000px"));
    await expectEdges(y, "block", { start: "crisp", end: "crisp" });
  });

  test("dx-scroll-fade-none turns the mask off, whatever the class order", async () => {
    await scrollBlockTo(y, "middle");
    await y.evaluate((n) => n.classList.add("dx-scroll-fade-none"));
    expect(await y.evaluate((n) => getComputedStyle(n).maskImage)).toBe("none");
    await y.evaluate((n) => (n.className = "dx-scroll-fade-none dx-scroll-fade"));
    expect(await y.evaluate((n) => getComputedStyle(n).maskImage)).toBe("none");
    await y.evaluate((n) => (n.className = "dx-scroll-fade"));
    await expectEdges(y, "block", { start: "faded", end: "faded" });
  });

  test("a one-edge class fades that edge only", async () => {
    await y.evaluate((n) => (n.className = "dx-scroll-fade-t"));
    await scrollBlockTo(y, "start");
    await expectEdges(y, "block", { start: "crisp", end: "crisp" });
    await scrollBlockTo(y, "middle");
    await expectEdges(y, "block", { start: "faded", end: "crisp" });
    await y.evaluate((n) => (n.className = "dx-scroll-fade-b"));
    await scrollBlockTo(y, "middle");
    await expectEdges(y, "block", { start: "crisp", end: "faded" });
  });

  test("--dx-scroll-fade-size sets the depth", async () => {
    await y.evaluate((n) => (n as HTMLElement).style.setProperty("--dx-scroll-fade-size", "20px"));
    await scrollBlockTo(y, "middle");
    await expectEdges(y, "block", { start: "faded", end: "faded" }, 20);
  });

  test("the horizontal strip fades at its inline edges, mirrored in RTL", async () => {
    await x.scrollIntoViewIfNeeded();
    expect(await x.evaluate((n) => n.scrollWidth - n.clientWidth)).toBeGreaterThan(200);
    expect((await readFade(x, "inline")).mask).toContain("to right");
    await scrollInlineTo(x, "start");
    await expectEdges(x, "inline", { start: "crisp", end: "faded" });
    await scrollInlineTo(x, "middle");
    await expectEdges(x, "inline", { start: "faded", end: "faded" });
    await scrollInlineTo(x, "end");
    await expectEdges(x, "inline", { start: "faded", end: "crisp" });

    await x.evaluate((n) => n.setAttribute("dir", "rtl"));
    expect((await readFade(x, "inline")).mask).toContain("to left");
    await scrollInlineTo(x, "start");
    await expectEdges(x, "inline", { start: "crisp", end: "faded" });
    await scrollInlineTo(x, "end");
    await expectEdges(x, "inline", { start: "faded", end: "crisp" });
  });
});

test.describe("shimmer utility", () => {
  const line = (page: Page) => demo(page, "util-shimmer").locator('[data-shimmer="line"]');
  const style = (el: Locator) =>
    el.evaluate((n) => {
      const cs = getComputedStyle(n);
      const a = n.getAnimations().find((x) => (x as CSSAnimation).animationName === "dx-shimmer-sweep");
      return {
        name: cs.animationName,
        playState: a ? a.playState : "none",
        image: cs.backgroundImage,
        fill: cs.webkitTextFillColor,
        color: cs.color,
        motion: n.getAttribute("data-dx-motion"),
      };
    });

  test("a gated status line sweeps while on screen", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/docs`, GOTO_OPTS);
    const l = line(page);
    await l.scrollIntoViewIfNeeded();
    await expect(l).toHaveClass(/dx-shimmer\b/);
    await expect(l).toHaveAttribute("data-dx-motion-key", /.+/);
    await expect(l).not.toHaveAttribute("data-dx-motion", "paused");
    await expect.poll(async () => (await style(l)).playState).toBe("running");
    const s = await style(l);
    expect(s.name).toBe("dx-shimmer-sweep");
    expect(s.image).toContain("linear-gradient");
    expect(s.fill).toBe("rgba(0, 0, 0, 0)"); // text painted by the clipped background
  });

  test("the sweep is paused while the line is off screen, and resumes", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/docs`, GOTO_OPTS);
    const l = line(page);
    await l.scrollIntoViewIfNeeded();
    await expect.poll(async () => (await style(l)).playState).toBe("running");
    await page.evaluate(() => window.scrollTo(0, 0));
    await expect(l).toHaveAttribute("data-dx-motion", "paused", { timeout: 10_000 });
    await expect.poll(async () => (await style(l)).playState).toBe("paused");
    await l.scrollIntoViewIfNeeded();
    await expect(l).not.toHaveAttribute("data-dx-motion", "paused", { timeout: 10_000 });
    await expect.poll(async () => (await style(l)).playState).toBe("running");
  });

  test("a data-dx-motion=paused ancestor pauses a bare .dx-shimmer", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/docs`, GOTO_OPTS);
    const l = line(page);
    await l.scrollIntoViewIfNeeded();
    const frame = demo(page, "util-shimmer");
    await frame.evaluate((n) => n.setAttribute("data-dx-motion", "paused"));
    await expect.poll(async () => (await style(l)).playState).toBe("paused");
    await frame.evaluate((n) => n.removeAttribute("data-dx-motion"));
    await expect.poll(async () => (await style(l)).playState).toBe("running");
  });

  test("dx-shimmer-none renders the text normally", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/docs`, GOTO_OPTS);
    const l = line(page);
    await l.scrollIntoViewIfNeeded();
    await demo(page, "util-shimmer").getByRole("button").click();
    await expect(l).toHaveClass(/dx-shimmer-none/);
    const s = await style(l);
    expect(s.image).toBe("none");
    expect(s.fill).toBe(s.color);
    await demo(page, "util-shimmer").getByRole("button").click();
    await expect(l).not.toHaveClass(/dx-shimmer-none/);
    expect((await style(l)).image).toContain("linear-gradient");
  });

  test("reduced motion turns the shimmer off", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await gotoHydrated(page, `${BASE_URL}/docs`, GOTO_OPTS);
    const l = line(page);
    await l.scrollIntoViewIfNeeded();
    await expect(l).toHaveClass(/dx-shimmer\b/);
    const s = await style(l);
    expect(s.name).toBe("none");
    expect(s.playState).toBe("none");
    expect(s.image).toBe("none");
    expect(s.fill).toBe(s.color);
  });
});
