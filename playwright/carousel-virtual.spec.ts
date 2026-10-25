/**
 * CarouselVirtualContent specs (window size, seamless loop, wheel wrap, a11y, re-centre,
 * paging never scrolls the page). Split out of `carousel.spec.ts` (row 267).
 */

import { test, expect } from "./fixtures";
import { type Page, Locator } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import {
  goto,
  demoFrame,
  slidePitch,
} from "./carousel-helpers";

/**
 * `CarouselVirtualContent`: the data-driven, virtualised drop-in for
 * `CarouselContent` + `CarouselItem`s (`primitives/src/carousel/virtual_content.rs`'s own
 * `CarouselVirtualContent` doc has the construction; `dev-docs/backlog.md`
 * row 91 has the report).
 *
 * Every slide here carries `data-index`/`data-position` (see
 * `CarouselItem`'s own doc for why every bridge in this module reads
 * those, never a child's array position) but never the themed
 * `.dx-carousel-item` class -- that class is applied by the themed
 * wrapper around `CarouselItem` specifically
 * (`preview/src/components/carousel/component.rs`), and
 * `CarouselVirtualContent` renders its own slide markup directly (the
 * primitive owns the wrapping div, not a nested themed component) -- so
 * every locator below scopes on `[data-position]` instead.
 *
 * `virtual_loop` (12 items, `loop: true`, `CarouselAutoplay { delay_ms:
 * 1200 }`, `CarouselIndicators` dots) and `virtual_many` (200 items,
 * `loop: false`) are the two demo variants
 * (`preview/src/components/carousel/variants/virtual_loop|virtual_many/mod.rs`),
 * both on the default `radius` (10). A looping window never repeats an
 * item, so `virtual_loop`'s is capped at `(12 - 2) / 2 = 5` per side
 * (`effective_radius`, `primitives/src/carousel/helpers.rs`): 11 slides.
 * `virtual_many`'s is the full `2 * 10 + 1 = 21` once away from its start.
 */
const VLOOP_WINDOW = 11;
const VMANY_WINDOW = 21;

test.describe("CarouselVirtualContent: the DOM never holds more than 2*radius+1 slides", () => {
  test("virtual_loop (N=12) mounts exactly 11 slides (radius capped at 5 so no item repeats)", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    await expect(frame.locator("[data-position]")).toHaveCount(VLOOP_WINDOW);
  });

  test("virtual_many (N=200) mounts exactly 21 slides once away from the (non-looping) start edge", async ({ page }) => {
    await goto(page, "virtual_many");
    const frame = demoFrame(page, "virtual_many");
    // At the very first slide the (non-wrapping) window clamps to
    // `[0, radius]` -- 11 slides, not 21 (`window()`'s own documented
    // clamping behaviour, `primitives/src/virtual/window.rs`). Page away
    // from that edge first (index >= radius) so the window is centred and
    // at its full width, the case this test is actually about.
    await expect(frame.locator("[data-position]")).toHaveCount(11);
    const next = frame.getByRole("button", { name: "Next slide" });
    for (let i = 2; i <= 11; i++) {
      await next.dispatchEvent("click");
      await expect.poll(() => frame.locator('[data-selected="true"]').getAttribute("aria-label")).toBe(
        `${i} of 200`,
      );
    }
    await expect(frame.locator("[data-position]")).toHaveCount(VMANY_WINDOW);
  });
});

test.describe("CarouselVirtualContent: seamless loop (virtual_loop)", () => {
  function selectedLabel(frame: Locator) {
    return () => frame.locator('[data-selected="true"]').getAttribute("aria-label");
  }

  test("Next through all 12 slides wraps 12 -> 1, one slide per click, DOM count never grows", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const next = frame.getByRole("button", { name: "Next slide" });
    const label = selectedLabel(frame);

    await expect.poll(label).toBe("1 of 12");
    for (let i = 2; i <= 12; i++) {
      await next.dispatchEvent("click");
      await expect.poll(label).toBe(`${i} of 12`);
      // Structurally impossible to have "rewound" through every
      // intervening slide the way a CarouselItem-based loop would:
      // the scroller only ever holds 11 DOM slides.
      await expect(frame.locator("[data-position]")).toHaveCount(VLOOP_WINDOW);
    }
    // The 13th Next wraps physically forward (12 -> 1), not a rewind.
    await next.dispatchEvent("click");
    await expect.poll(label).toBe("1 of 12");
    await expect(frame.locator("[data-position]")).toHaveCount(VLOOP_WINDOW);
  });

  test("Previous through all 12 slides wraps 1 -> 12, one slide per click", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const previous = frame.getByRole("button", { name: "Previous slide" });
    const label = selectedLabel(frame);

    await expect.poll(label).toBe("1 of 12");
    // Previous is never `disabled` under `loop: true` -- confirms the
    // wrap is reachable via this button at all before relying on it.
    await expect(previous).toBeEnabled();
    // Starting at "1 of 12", the FIRST Previous click is itself the wrap
    // (1 -> 12); the rest count down normally back around to 1.
    for (const i of [12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]) {
      await previous.dispatchEvent("click");
      await expect.poll(label).toBe(`${i} of 12`);
      await expect(frame.locator("[data-position]")).toHaveCount(VLOOP_WINDOW);
    }
  });

  test("ArrowRight/ArrowLeft page and wrap the same way as the buttons", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    // The scroll region itself (`tabindex="0"`), not the carousel's own
    // `role="region"` root (which is never focusable) -- both carry
    // `data-orientation`, so a bare `[data-orientation]` locator's
    // `.first()` grabs the (ancestor, non-focusable) root instead.
    const content = frame.locator(".dx-carousel-content");
    const label = selectedLabel(frame);

    await content.focus();
    await expect(content).toBeFocused();
    await expect.poll(label).toBe("1 of 12");
    // A small gap between presses -- see this file's own "seamless loop"
    // describe block's "autoplay" test for why a real settle needs real
    // wall-clock time between steps (the window's own margin is not
    // unlimited): a genuinely human keyboard cadence, not a synthetic
    // zero-delay flood, is what this construction is built for.
    for (let i = 0; i < 13; i++) {
      await page.keyboard.press("ArrowRight");
      await page.waitForTimeout(120);
    }
    await expect.poll(label).toBe("2 of 12"); // 13 steps forward from 1, mod 12
    for (let i = 0; i < 2; i++) {
      await page.keyboard.press("ArrowLeft");
      await page.waitForTimeout(120);
    }
    await expect.poll(label).toBe("12 of 12");
  });

  test("autoplay advances by exactly one slide per tick and wraps 12 -> 1 with no drift", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const label = selectedLabel(frame);
    await expect.poll(label).toBe("1 of 12");

    // Poll densely (well under the 1200ms tick) rather than sampling at a
    // period close to the tick's own -- sampling every 1300ms against a
    // 1200ms tick shows an apparent "skipped" index once per run purely from
    // that beat frequency (a measurement artifact, not a product bug). This
    // instead records every distinct value seen and asserts the exact
    // sequence.
    const seen: string[] = [(await label())!];
    const deadline = Date.now() + 9000;
    while (Date.now() < deadline && seen.length < 8) {
      await page.waitForTimeout(100);
      const cur = await label();
      if (cur !== seen[seen.length - 1]) {
        seen.push(cur!);
      }
    }
    expect(seen).toEqual([
      "1 of 12",
      "2 of 12",
      "3 of 12",
      "4 of 12",
      "5 of 12",
      "6 of 12",
      "7 of 12",
      "8 of 12",
    ]);
  });

  test("the dot picker (CarouselIndicators) jumps directly to a distant slide and re-anchors instantly", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const label = selectedLabel(frame);
    await expect.poll(label).toBe("1 of 12");

    // Jump straight to slide 7 (6 away -- outside the 11-slide window, whose
    // far edges are positions -5 and +5),
    // the "distant dot click" re-anchor path (`CarouselVirtualContent`'s
    // own "Seamless loop" doc, second bullet). `CarouselIndicator`'s own
    // default accessible name is "Slide {n}" (see `component.rs`'s own doc),
    // not the custom dot-picker's "Go to slide {n}".
    await frame.getByRole("tab", { name: "Slide 7" }).click();
    await expect.poll(label).toBe("7 of 12");
    await expect(frame.locator("[data-position]")).toHaveCount(VLOOP_WINDOW);
    // Now a single Next from there still advances by exactly one, proving
    // the anchor is genuinely centred on 7, not merely displaying it.
    await frame.getByRole("button", { name: "Next slide" }).dispatchEvent("click");
    await expect.poll(label).toBe("8 of 12");
  });

  test('"k of 12" is correct exactly at the wrap in both directions', async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const next = frame.getByRole("button", { name: "Next slide" });
    const previous = frame.getByRole("button", { name: "Previous slide" });
    const label = selectedLabel(frame);

    for (let i = 2; i <= 12; i++) {
      await next.dispatchEvent("click");
      await expect.poll(label).toBe(`${i} of 12`);
    }
    await next.dispatchEvent("click");
    await expect.poll(label).toBe("1 of 12");
    await previous.dispatchEvent("click");
    await expect.poll(label).toBe("12 of 12");
  });
});

test.describe("CarouselVirtualContent: trackpad-style wheel scrolling wraps seamlessly", () => {
  test("a forward wheel-scroll sequence advances one slide at a time, never a multi-slide jump", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const content = frame.locator("[data-orientation]").first();
    const label = () => frame.locator('[data-selected="true"]').getAttribute("aria-label");

    await expect.poll(label).toBe("1 of 12");
    const pitch = await slidePitch(frame.locator('[data-selected="true"]'));
    await content.hover();

    const seen: string[] = ["1 of 12"];
    // One wheel burst per intended step -- headless Chromium needs
    // `deltaX` (this file's own established wheel-test convention; see
    // the "wheel overdrag" describe block's own note on this), several
    // events per burst to cross one slide pitch reliably, with a settle
    // pause between bursts so each step's own settle completes before the
    // next burst starts (never mid-gesture -- `data-dragging`/the wheel
    // bridge's own idle detection would otherwise coalesce a fast stream
    // into a single multi-slide travel, which is a real behavior of
    // native scrolling, not a bug in this construction, and would defeat
    // this test's own "one slide at a time" premise).
    for (let step = 0; step < 6; step++) {
      for (let i = 0; i < 6; i++) {
        await page.mouse.wheel(pitch / 6, 0);
      }
      await page.waitForTimeout(300);
      const cur = (await label())!;
      if (cur !== seen[seen.length - 1]) {
        seen.push(cur);
      }
      await expect(frame.locator("[data-position]")).toHaveCount(VLOOP_WINDOW);
    }
    // Exactly one new slide per step -- no step is ever skipped (a
    // multi-slide jump) and none repeats (a stall).
    expect(seen).toEqual([
      "1 of 12",
      "2 of 12",
      "3 of 12",
      "4 of 12",
      "5 of 12",
      "6 of 12",
      "7 of 12",
    ]);
  });
});

test.describe("CarouselVirtualContent: a11y", () => {
  test("only the current slide is non-inert; the rest are inert; this holds across a wrap", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const slides = frame.locator("[data-position]");

    const inertSnapshot = async () => {
      const count = await slides.count();
      const rows: { index: string | null; inert: boolean }[] = [];
      for (let i = 0; i < count; i++) {
        rows.push({
          index: await slides.nth(i).getAttribute("data-index"),
          inert: await slides.nth(i).evaluate((el) => el.hasAttribute("inert")),
        });
      }
      return rows;
    };

    let rows = await inertSnapshot();
    expect(rows.filter((r) => !r.inert)).toEqual([{ index: "0", inert: false }]);

    // Wrap forward past the last slide (12 -> 1) and re-check: still
    // exactly one non-inert slide, now data-index 0 again but a different
    // DOM node's own `data-position` (seamless, not a rewind).
    const next = frame.getByRole("button", { name: "Next slide" });
    for (let i = 0; i < 12; i++) {
      await next.dispatchEvent("click");
      await expect
        .poll(async () => (await inertSnapshot()).filter((r) => !r.inert).length)
        .toBe(1);
    }
    rows = await inertSnapshot();
    expect(rows.filter((r) => !r.inert)).toEqual([{ index: "0", inert: false }]);
  });

  test("focus is never lost to <body> across a wrap driven by the root keyboard handler", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    // The scroll region itself, not the (non-focusable) `role="region"`
    // root -- see the "ArrowRight/ArrowLeft" test's own comment, above,
    // for why a bare `[data-orientation]` locator is ambiguous here.
    const content = frame.locator(".dx-carousel-content");

    await content.focus();
    await expect(content).toBeFocused();
    for (let i = 0; i < 12; i++) {
      await page.keyboard.press("ArrowRight");
      await page.waitForTimeout(120);
    }
    const isBody = await page.evaluate(() => document.activeElement === document.body);
    expect(isBody).toBe(false);
  });

  test("virtual_many: Previous/Next are genuinely disabled at the real, non-looping ends", async ({ page }) => {
    await goto(page, "virtual_many");
    const frame = demoFrame(page, "virtual_many");
    const next = frame.getByRole("button", { name: "Next slide" });
    const previous = frame.getByRole("button", { name: "Previous slide" });
    const label = () => frame.locator('[data-selected="true"]').getAttribute("aria-label");

    await expect.poll(label).toBe("1 of 200");
    await expect(previous).toBeDisabled();
    await expect(next).toBeEnabled();

    // Paced with a small explicit wait *and* `expect.poll` after every
    // click -- see the "ArrowRight/ArrowLeft" test's own comment, above,
    // for why a zero-delay flood of 200 clicks is not this construction's
    // target use case (a settle needs real wall-clock time to land within
    // `radius`'s own margin); this is still far faster than a real user,
    // just not adversarially so. The explicit wait matters more here than
    // in the shorter loop/wrap tests above: at 200 iterations even a rare
    // (~1-in-200) settle-latency race is likely to surface at least once,
    // on the SSG lane specifically (a plain
    // `expect.poll` alone, with no wait, can miss a step around
    // i=176 there).
    for (let i = 2; i <= 200; i++) {
      await next.dispatchEvent("click");
      await page.waitForTimeout(40);
      await expect.poll(label).toBe(`${i} of 200`);
    }
    await expect(next).toBeDisabled();
    await expect(previous).toBeEnabled();
    // Clamped at the real end, never wrapping -- one more Next is a no-op.
    await next.dispatchEvent("click");
    await expect.poll(label).toBe("200 of 200");
  });

  test("axe: virtual_loop and virtual_many have no automatically detectable a11y issues", async ({ page }) => {
    // Every variant of a "Normal"-kind component (including these two)
    // renders on the same page at once -- see this file's own header
    // ("SCOPING") -- so the existing "carousel component page (every
    // variant mounted)" scan in the "Axe automated scan" describe block
    // above already covers both; this test targets them individually so a
    // regression names the right variant rather than "carousel: all
    // variants" generically.
    await goto(page, "virtual_loop");
    await expectNoAxeViolations(page, "carousel: virtual_loop variant", {
      include: "#component-preview-frame-virtual_loop",
    });
    await expectNoAxeViolations(page, "carousel: virtual_many variant", {
      include: "#component-preview-frame-virtual_many",
    });
  });

  test("virtual_loop_rtl: seamless loop wraps under RTL with the swapped arrow keys too", async ({ page }) => {
    // Cheap RTL coverage of the identical seamless-loop path -- not a full duplicate
    // of every `virtual_loop` test above, just the one thing RTL could
    // plausibly break: the key-swap composed with physical wraparound.
    await goto(page, "virtual_loop_rtl");
    const frame = demoFrame(page, "virtual_loop_rtl");
    const next = frame.getByRole("button", { name: "Next slide" });
    const label = () => frame.locator('[data-selected="true"]').getAttribute("aria-label");

    await expect.poll(label).toBe("1 of 12");
    // RTL: ArrowLeft is the swapped "next" key (Direction::resolve_horizontal).
    await next.focus();
    await page.keyboard.press("ArrowLeft");
    await expect.poll(label).toBe("2 of 12");

    await expectNoAxeViolations(page, "carousel: virtual_loop_rtl variant", {
      include: "#component-preview-frame-virtual_loop_rtl",
    });
  });
});

/**
 * The re-centre correction (`CarouselVirtualContent`'s own "Seamless
 * loop" doc): once a paging step settles, the window re-renders around
 * the new anchor and a dedicated effect instantly re-aligns the scroller
 * to compensate for the resulting one-slide-width DOM shift. This must
 * never be visible -- sampled every animation frame across a step that
 * triggers it, the currently-selected slide's own rect must ease
 * smoothly to rest and never move again afterward (no jump back-and-forth,
 * no post-settle correction visible as a second motion).
 */
test.describe("CarouselVirtualContent: the re-centre never paints a visible jump", () => {
  test("the current slide's rect eases to a stable rest position with no jump after settling", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const content = frame.locator("[data-orientation]").first();
    const next = frame.getByRole("button", { name: "Next slide" });

    await content.evaluate((el) => {
      const w = window as unknown as { __samples: number[]; __stopSampling: () => void };
      w.__samples = [];
      let raf: number;
      const sample = () => {
        const sel = el.querySelector('[data-selected="true"]');
        if (sel) {
          w.__samples.push(sel.getBoundingClientRect().left);
        }
        raf = requestAnimationFrame(sample);
      };
      raf = requestAnimationFrame(sample);
      w.__stopSampling = () => cancelAnimationFrame(raf);
    });

    await next.dispatchEvent("click");
    // Comfortably past the paging animation, the settle, and the instant
    // re-centre correction that follows it.
    await page.waitForTimeout(1200);

    const samples: number[] = await content.evaluate(() => {
      const w = window as unknown as { __samples: number[]; __stopSampling: () => void };
      w.__stopSampling();
      return w.__samples;
    });
    expect(samples.length).toBeGreaterThan(10);

    // The final quarter of samples (well after the animation should have
    // finished) must be perfectly flat -- if the re-centre's own instant
    // correction were visible as a second motion after the eased scroll
    // already looked settled, it would show up here as a nonzero diff.
    const tail = samples.slice(Math.floor(samples.length * 0.75));
    for (let i = 1; i < tail.length; i++) {
      expect(Math.abs(tail[i] - tail[i - 1])).toBeLessThan(1);
    }
  });
});

test.describe("CarouselVirtualContent: paging never scrolls the page", () => {
  async function waitForScrollStable(page: Page): Promise<number> {
    let previous: number | null = null;
    for (let i = 0; i < 50; i++) {
      const y = await page.evaluate(() => window.scrollY);
      if (y === previous) {
        return y;
      }
      previous = y;
      await page.waitForTimeout(150);
    }
    throw new Error("waitForScrollStable: window.scrollY never settled");
  }

  test("Next/Previous through a full wrap never moves window scroll position", async ({ page }) => {
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const next = frame.getByRole("button", { name: "Next slide" });
    const label = () => frame.locator('[data-selected="true"]').getAttribute("aria-label");

    await waitForScrollStable(page);
    await page.evaluate(() => window.scrollTo({ top: 0, left: 0, behavior: "instant" }));
    const scrollYBefore = await page.evaluate(() => window.scrollY);
    const scrollXBefore = await page.evaluate(() => window.scrollX);

    // Paced with `expect.poll` per click -- see the "ArrowRight/ArrowLeft"
    // test's own comment (`CarouselVirtualContent: seamless loop` describe
    // block, above) for why a zero-delay flood of clicks is not this
    // construction's target use case.
    for (let i = 2; i <= 13; i++) {
      await next.dispatchEvent("click");
      await expect.poll(label).toBe(`${((i - 1) % 12) + 1} of 12`);
    }
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollYBefore);
    expect(await page.evaluate(() => window.scrollX)).toBe(scrollXBefore);
  });
});
