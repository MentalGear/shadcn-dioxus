/**
 * The shared windowing engine (backlog row 309): VirtualList and VirtualGrid consume one DOM
 * bridge and one "keep the focused item rendered" strategy, and each has the other's strength.
 *
 *  - VirtualGrid / VirtualWrapGrid take the `fade: ScrollFade` prop VirtualList and ScrollArea
 *    have: the `dx-scroll-fade-*` mask lands on the grid's scroll container and follows its scroll
 *    offset whatever cells are mounted (the grid's `fade` demo); `ScrollFade::Both` masks all four
 *    edges of a two-axis grid at once (`dx-scroll-fade-xy`: the block and inline gradients intersected).
 *  - A windowed VirtualList keeps the row that last received focus mounted when the window leaves
 *    it (as the grids keep their active cell): the same DOM node, drawn at its own position, and
 *    focus never falls to <body>.
 *
 * Functional spec: no millisecond thresholds (a debug build is fine). Nothing here bounds an overlay.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { demoFrame } from "./demo-frame";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const GRID_URL = `${BASE_URL}/component/virtual_grid/`;
const LIST_URL = `${BASE_URL}/component/block/?name=virtual_list&variant=windowed`;

type Edge = "crisp" | "faded";

/** The block-axis fade depths in px at the scroller's current position, and the full depth. */
function readFade(el: Locator) {
  return el.evaluate((node) => {
    const cs = getComputedStyle(node);
    const extent = node.getBoundingClientRect().height;
    const toPx = (value: string) => {
      const outer = document.createElement("div");
      outer.style.cssText = `position:absolute;visibility:hidden;pointer-events:none;height:${extent}px;`;
      const inner = document.createElement("div");
      inner.style.height = value.trim() || "0px";
      outer.appendChild(inner);
      document.body.appendChild(outer);
      const px = inner.getBoundingClientRect().height;
      outer.remove();
      return px;
    };
    return {
      start: toPx(cs.getPropertyValue("--dx-scroll-fade-t")),
      end: toPx(cs.getPropertyValue("--dx-scroll-fade-b")),
      full: Math.min(extent * 0.12, 40),
      mask: cs.maskImage || cs.webkitMaskImage || "",
    };
  });
}

async function expectEdges(el: Locator, want: { start?: Edge; end: Edge }) {
  await expect
    .poll(
      async () => {
        const f = await readFade(el);
        const kind = (px: number) => (px < 1 ? "crisp" : Math.abs(px - f.full) <= 1.5 ? "faded" : `partial(${px.toFixed(1)}/${f.full.toFixed(1)})`);
        return want.start === undefined ? kind(f.end) : `${kind(f.start)}/${kind(f.end)}`;
      },
      { timeout: 10_000, message: "block-axis fade edges" },
    )
    .toBe(want.start === undefined ? want.end : `${want.start}/${want.end}`);
}

const scrollBlockTo = (el: Locator, where: "start" | "middle" | "end") =>
  el.evaluate((n, where) => {
    const max = n.scrollHeight - n.clientHeight;
    n.scrollTop = where === "start" ? 0 : where === "middle" ? max / 2 : max + 10_000;
  }, where);

async function settleAtEnd(el: Locator) {
  await expect
    .poll(async () => {
      await scrollBlockTo(el, "end");
      return el.evaluate((n) => n.scrollHeight - n.clientHeight - Math.round(n.scrollTop) <= 2);
    }, { timeout: 15_000 })
    .toBe(true);
}

type Four = { t: Edge; b: Edge; s: Edge; e: Edge };

/** The four fade depths in px (block edges against the box height, inline edges against its width). */
function readFour(el: Locator) {
  return el.evaluate((node) => {
    const cs = getComputedStyle(node);
    const rect = node.getBoundingClientRect();
    const toPx = (value: string, extent: number, prop: "height" | "width") => {
      const outer = document.createElement("div");
      outer.style.cssText = `position:absolute;visibility:hidden;pointer-events:none;${prop}:${extent}px;`;
      const inner = document.createElement("div");
      inner.style[prop] = value.trim() || "0px";
      outer.appendChild(inner);
      document.body.appendChild(outer);
      const px = inner.getBoundingClientRect()[prop];
      outer.remove();
      return px;
    };
    return {
      t: toPx(cs.getPropertyValue("--dx-scroll-fade-t"), rect.height, "height"),
      b: toPx(cs.getPropertyValue("--dx-scroll-fade-b"), rect.height, "height"),
      s: toPx(cs.getPropertyValue("--dx-scroll-fade-s"), rect.width, "width"),
      e: toPx(cs.getPropertyValue("--dx-scroll-fade-e"), rect.width, "width"),
      fullBlock: Math.min(rect.height * 0.12, 40),
      fullInline: Math.min(rect.width * 0.12, 40),
    };
  });
}

async function expectFour(el: Locator, want: Four) {
  await expect
    .poll(
      async () => {
        const f = await readFour(el);
        const kind = (px: number, full: number) => (px < 1 ? "crisp" : Math.abs(px - full) <= 1.5 ? "faded" : `partial(${px.toFixed(1)}/${full.toFixed(1)})`);
        return `t:${kind(f.t, f.fullBlock)} b:${kind(f.b, f.fullBlock)} s:${kind(f.s, f.fullInline)} e:${kind(f.e, f.fullInline)}`;
      },
      { timeout: 10_000, message: "the four edges of the two-axis fade" },
    )
    .toBe(`t:${want.t} b:${want.b} s:${want.s} e:${want.e}`);
}

const scrollBothTo = (el: Locator, block: "start" | "middle" | "end", inline: "start" | "middle" | "end") =>
  el.evaluate(
    (n, [block, inline]) => {
      const at = (max: number, where: string) => (where === "start" ? 0 : where === "middle" ? max / 2 : max + 10_000);
      n.scrollTop = at(n.scrollHeight - n.clientHeight, block);
      n.scrollLeft = at(n.scrollWidth - n.clientWidth, inline);
    },
    [block, inline] as const,
  );

const gridIn = (page: Page, demo: string) => demoFrame(page, "fade").locator(`[data-demo="${demo}"] [role="grid"]`);

test.describe("VirtualGrid fade (shared with VirtualList and ScrollArea)", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, GRID_URL, GOTO_OPTS);
    await demoFrame(page, "fade").scrollIntoViewIfNeeded();
  });

  test("wrap grid: the fade follows the scroll position over 500 windowed rows", async ({ page }) => {
    const grid = gridIn(page, "fade-wrap");
    await expect(grid).toHaveAttribute("data-scroll-fade", "y");
    await expect(grid).toHaveClass(/dx-scroll-fade-y/);
    await expect(grid).toHaveClass(/dx-virtual-grid/); // merged, not replaced
    await expect(grid.locator('[role="gridcell"]').first()).toBeVisible({ timeout: 60_000 });
    expect(await grid.locator('[role="row"]').count()).toBeLessThan(40);

    await scrollBlockTo(grid, "start");
    await expectEdges(grid, { start: "crisp", end: "faded" });

    await scrollBlockTo(grid, "middle");
    await expectEdges(grid, { start: "faded", end: "faded" });
    // The window really moved, and stayed small.
    await expect
      .poll(() => grid.locator('[role="row"]').last().getAttribute("aria-rowindex").then(Number), { timeout: 10_000 })
      .toBeGreaterThan(100);
    expect(await grid.locator('[role="row"]').count()).toBeLessThan(40);

    await settleAtEnd(grid);
    await expectEdges(grid, { start: "faded", end: "crisp" });
    await expect(grid.locator('[data-grid-index="1999"]')).toBeVisible();
  });

  test("data grid: fade Both masks all four edges, each only while there is content past it", async ({ page }) => {
    const grid = gridIn(page, "fade-grid");
    await expect(grid).toHaveAttribute("data-scroll-fade", "both");
    await expect(grid).toHaveClass(/dx-scroll-fade-xy/);
    await expect(grid).toHaveClass(/dx-virtual-grid/); // merged, not replaced
    await expect(grid.locator('[data-grid-cell="0:0"]')).toBeVisible({ timeout: 60_000 });

    // One mask of four layers (dx-effects.css, "The mask is a stack of layers"): two solid ones that keep the
    // scrollbar gutters opaque, then the block gradient and the inline gradient, intersected.
    const mask = await grid.evaluate((n) => {
      const cs = getComputedStyle(n);
      return { image: cs.maskImage || cs.webkitMaskImage || "", composite: cs.maskComposite };
    });
    expect((mask.image.match(/linear-gradient/g) ?? []).length).toBe(4);
    expect(mask.image).toContain("to right"); // the inline layer; the block layer is the default direction, to bottom
    expect(mask.composite).toMatch(/^exclude, exclude, intersect(, intersect)*$/); // one operation per layer

    // At the origin: top and inline-start crisp, bottom and inline-end faded.
    await scrollBothTo(grid, "start", "start");
    await expectFour(grid, { t: "crisp", b: "faded", s: "crisp", e: "faded" });
    // Scrolled on one axis only: that axis' start edge fades in, the other axis is untouched.
    await scrollBothTo(grid, "middle", "start");
    await expectFour(grid, { t: "faded", b: "faded", s: "crisp", e: "faded" });
    await scrollBothTo(grid, "start", "middle");
    await expectFour(grid, { t: "crisp", b: "faded", s: "faded", e: "faded" });
    // In the middle of both: all four faded. Rows and columns are still windowed under the mask.
    await scrollBothTo(grid, "middle", "middle");
    await expectFour(grid, { t: "faded", b: "faded", s: "faded", e: "faded" });
    expect(await grid.locator("[data-grid-cell]").count()).toBeLessThan(600);
    // At the far corner: the end edges sharpen, the start edges stay faded.
    await scrollBothTo(grid, "end", "end");
    await expectFour(grid, { t: "faded", b: "crisp", s: "faded", e: "crisp" });
  });

  test("fade Both follows the reading direction: RTL mirrors the inline gradient", async ({ page }) => {
    const grid = gridIn(page, "fade-grid");
    await expect(grid.locator('[data-grid-cell="0:0"]')).toBeVisible({ timeout: 60_000 });
    const ltr = await grid.evaluate((n) => getComputedStyle(n).maskImage || "");
    expect(ltr).toContain("to right");
    expect(ltr).not.toContain("to left");
    const rtl = await grid.evaluate((n) => {
      const wrapper = n.parentElement!;
      wrapper.setAttribute("dir", "rtl");
      const mask = getComputedStyle(n).maskImage || "";
      wrapper.removeAttribute("dir");
      return mask;
    });
    expect((rtl.match(/linear-gradient/g) ?? []).length).toBe(4); // two solid gutter layers and the two fades
    expect(rtl).toContain("to left");
    expect(rtl).not.toContain("to right");
  });
});

// Where a browser has no scroll-driven animations, an edge must still show only while there is more content
// past it: the one fallback script (`use_scroll_fade`) sets the depth properties from the scroll position.
// Forced here on Chromium: `CSS.supports` says no for that query, and the CSS animations are switched off so
// the script's inline values are what is read (the `@supports not` stylesheet branch is only the static default).
test.describe("scroll fade without scroll-driven animations (the fallback script)", () => {
  test("every edge of a two-axis grid and of a wrapping gallery tracks the scroll position", async ({ page }) => {
    await page.addInitScript(() => {
      const supports = CSS.supports.bind(CSS);
      CSS.supports = ((...args: string[]) => (String(args[0]).includes("animation-timeline") ? false : (supports as (...a: string[]) => boolean)(...args))) as typeof CSS.supports;
    });
    await gotoHydrated(page, GRID_URL, GOTO_OPTS);
    await page.addStyleTag({ content: '[class*="dx-scroll-fade"] { animation: none !important; }' });
    await demoFrame(page, "fade").scrollIntoViewIfNeeded();

    const grid = gridIn(page, "fade-grid");
    await expect(grid.locator('[data-grid-cell="0:0"]')).toBeVisible({ timeout: 60_000 });
    await scrollBothTo(grid, "start", "start");
    await expectFour(grid, { t: "crisp", b: "faded", s: "crisp", e: "faded" });
    await scrollBothTo(grid, "middle", "middle");
    await expectFour(grid, { t: "faded", b: "faded", s: "faded", e: "faded" });
    await scrollBothTo(grid, "end", "end");
    await expectFour(grid, { t: "faded", b: "crisp", s: "faded", e: "crisp" });

    const wrap = gridIn(page, "fade-wrap");
    await expect(wrap.locator('[role="gridcell"]').first()).toBeVisible({ timeout: 60_000 });
    await scrollBlockTo(wrap, "start");
    await expectEdges(wrap, { start: "crisp", end: "faded" });
    await scrollBlockTo(wrap, "middle");
    await expectEdges(wrap, { start: "faded", end: "faded" });
    await settleAtEnd(wrap);
    await expectEdges(wrap, { start: "faded", end: "crisp" });
  });
});

test.describe("VirtualList keeps its focused row rendered (shared with the grids' active cell)", () => {
  test("a focused row survives the window leaving it, as the same node, and rejoins the flow", async ({ page }) => {
    await gotoHydrated(page, LIST_URL, GOTO_OPTS);
    const list = page.locator('[data-vl-demo="windowed"] [role="list"]');
    await expect(list).toHaveAttribute("data-virtualize", "windowed");
    const row = list.locator('[data-virtual-index="3"]');
    await expect(row).toBeVisible({ timeout: 60_000 });

    // Focus the row (as a link or button inside it would be) and mark the node.
    await row.evaluate((el) => {
      (el as HTMLElement).tabIndex = -1;
      (el as HTMLElement).focus();
      (el as unknown as { __probe: number }).__probe = 309;
    });
    const before = await row.evaluate((el) => el.getBoundingClientRect().top - el.closest('[role="list"]')!.firstElementChild!.getBoundingClientRect().top);

    // Scroll far away: the window moves past row 1,000 ...
    await list.evaluate((el) => (el.scrollTop = 500_000));
    await expect
      .poll(() => list.locator('[role="listitem"]:not([data-virtual-pinned])').first().getAttribute("data-virtual-index").then(Number), { timeout: 15_000 })
      .toBeGreaterThan(1000);
    // ... and row 3 is still there: pinned, the same node, still focused, at its own position.
    await expect(row).toHaveAttribute("data-virtual-pinned", "true");
    expect(await row.evaluate((el) => (el as unknown as { __probe?: number }).__probe)).toBe(309);
    expect(await row.evaluate((el) => document.activeElement === el)).toBe(true);
    const pinnedAt = await row.evaluate((el) => el.getBoundingClientRect().top - el.closest('[role="list"]')!.firstElementChild!.getBoundingClientRect().top);
    expect(Math.abs(pinnedAt - before)).toBeLessThan(300);
    // One extra row, not a growing DOM.
    expect(await list.getByRole("listitem").count()).toBeLessThan(80);

    // Back to the top: the row rejoins the flow without remounting, and keeps focus.
    await list.evaluate((el) => (el.scrollTop = 0));
    await expect(row).not.toHaveAttribute("data-virtual-pinned", /.*/, { timeout: 15_000 });
    expect(await row.evaluate((el) => (el as unknown as { __probe?: number }).__probe)).toBe(309);
    expect(await row.evaluate((el) => document.activeElement === el)).toBe(true);
    expect(await row.evaluate((el) => getComputedStyle(el).position)).toBe("static");
  });
});
