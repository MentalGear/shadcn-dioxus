/**
 * Carousel edge rubber-band, pointer half (mode B3), and the clipping viewport
 * (edge overdrag never paints outside the carousel). Split out of `carousel.spec.ts` (row 267);
 * helpers are in `carousel-helpers.ts`, `demoFrame()` scoping is explained in `carousel.spec.ts`.
 */

import { test, expect } from "./fixtures";
import { type Locator } from "@playwright/test";
import {
  goto,
  demoFrame,
  expectSnappedToBoundary,
  dragBy,
  dragHold,
  readContentTransform,
  parseTranslatePx,
  slidePitch,
  WHEEL,
  playWheel,
  expectBandGone,
  neg,
} from "./carousel-helpers";

/**
 * Edge rubber-band, pointer half (mode B3), backlog row 102 -- port of the
 * closed design in `dev-docs/archive/research/carousel-overscroll-2026-09-23.md`.
 * See `primitives/src/carousel.rs`'s own `CarouselContent` doc, "Edge
 * rubber-band" section, for the construction these tests hold to: a
 * `transform` on `.dx-carousel-content` only, never a spacer child, and
 * never a change to `selected`. The wheel/trackpad half is mode D; its
 * tests are the next describe block.
 *
 * Every test below needs the band to write `style.transform` (plain
 * clamping, mode A, never does: `readContentTransform` would always return `""`).
 */
test.describe("Carousel: edge rubber-band, pointer (mode B3)", () => {
  test("pointer overdrag past the first slide produces a bounded, nonzero transform that returns to identity on release", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    expect(await readContentTransform(content)).toBe("");

    const pitch = await slidePitch(slide(1));
    // Positive dx at slide 1 (the start) asks for a "previous" slide that
    // does not exist -- see `CAROUSEL_DRAG_JS`'s own "Pointer drag" doc for
    // why a positive drag is the physical-previous direction in this
    // (LTR, horizontal) variant.
    const rawTravel = pitch * 1.5;
    await dragHold(page, content, rawTravel, 0);

    const transform = await readContentTransform(content);
    const depth = parseTranslatePx(transform);
    expect(depth, `expected a nonzero translateX, got "${transform}"`).not.toBeNull();
    expect(depth!).toBeGreaterThan(0);
    // Damped: the visible depth is strictly less than the raw pointer
    // travel that produced it -- the rubber function's whole point.
    expect(depth!).toBeLessThan(rawTravel);

    await page.mouse.up();
    await expect(async () => {
      expect(await readContentTransform(content)).toBe("");
    }).toPass({ timeout: 2000 });

    // Never a second source of truth for the index (research doc's own
    // framing): the bounce never moved `selected`.
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    await expect(frame.getByRole("button", { name: "Previous slide" })).toBeDisabled();
  });

  test("pointer overdrag past the last slide produces a bounded, nonzero transform that returns to identity on release", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    const next = frame.getByRole("button", { name: "Next slide" });

    for (let i = 0; i < 4; i++) {
      await next.click();
      await expectSnappedToBoundary(content, slide(i + 2));
    }
    await expect(slide(5)).toHaveAttribute("data-selected", "true");
    await expect(next).toBeDisabled();

    const pitch = await slidePitch(slide(5));
    const rawTravel = pitch * 1.5;
    // Negative dx at the last slide asks for a "next" slide that does not
    // exist.
    await dragHold(page, content, -rawTravel, 0);

    const transform = await readContentTransform(content);
    const depth = parseTranslatePx(transform);
    expect(depth, `expected a nonzero translateX, got "${transform}"`).not.toBeNull();
    expect(depth!).toBeLessThan(0);
    expect(Math.abs(depth!)).toBeLessThan(rawTravel);

    await page.mouse.up();
    await expect(async () => {
      expect(await readContentTransform(content)).toBe("");
    }).toPass({ timeout: 2000 });

    await expect(slide(5)).toHaveAttribute("data-selected", "true");
    await expect(next).toBeDisabled();
  });

  test("pointer overdrag at the start boundary works under dir=rtl", async ({ page }) => {
    await goto(page, "rtl");
    const frame = demoFrame(page, "rtl");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    const pitch = await slidePitch(slide(1));
    const rawTravel = pitch * 1.5;
    // Negative dx is the physical-previous direction under RTL (mirrored
    // from LTR -- see the "pointer drag" describe block's own rtl test,
    // which drags positive to advance).
    await dragHold(page, content, -rawTravel, 0);

    const transform = await readContentTransform(content);
    const depth = parseTranslatePx(transform);
    expect(depth, `expected a nonzero translateX, got "${transform}"`).not.toBeNull();
    expect(depth!).toBeLessThan(0);
    expect(Math.abs(depth!)).toBeLessThan(rawTravel);

    await page.mouse.up();
    await expect(async () => {
      expect(await readContentTransform(content)).toBe("");
    }).toPass({ timeout: 2000 });

    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("pointer overdrag at the start boundary works in the vertical variant", async ({ page }) => {
    await goto(page, "vertical");
    const frame = demoFrame(page, "vertical");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    const pitch = await slidePitch(slide(1), "vertical");
    const rawTravel = pitch * 1.5;
    // Positive dy is the physical-previous direction on the block axis
    // (the "pointer drag" describe block's own vertical test drags
    // negative to advance).
    await dragHold(page, content, 0, rawTravel);

    const transform = await readContentTransform(content);
    const depth = parseTranslatePx(transform);
    expect(depth, `expected a nonzero translateY, got "${transform}"`).not.toBeNull();
    expect(depth!).toBeGreaterThan(0);
    expect(depth!).toBeLessThan(rawTravel);

    await page.mouse.up();
    await expect(async () => {
      expect(await readContentTransform(content)).toBe("");
    }).toPass({ timeout: 2000 });

    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("a mid-range pointer drag produces no transform and pages correctly", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    const pitch = await slidePitch(slide(1));
    await dragBy(page, content, -pitch * 0.7, 0);

    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    expect(await readContentTransform(content)).toBe("");
  });
});

/** The presentational clipping wrapper `CarouselContent` renders around its
 * scroller (`primitives/src/carousel.rs`'s `CarouselContent` doc, "Edge
 * rubber-band (mode B3)" section, "Correction" paragraph). */
function viewportLocator(frame: Locator): Locator {
  return frame.locator('[data-slot="carousel-viewport"]');
}

/**
 * True if the topmost element actually painted at page coordinate `(x, y)`
 * -- `document.elementFromPoint`, not rect math -- is `content` itself or
 * one of its descendants (a slide). Rect math on the transformed element
 * would flag the same translated box as "outside the viewport" whether or
 * not anything actually clips it; asking the browser what it painted there
 * is the only check that distinguishes "clipped" from "not."
 */
async function paintsAt(content: Locator, x: number, y: number): Promise<boolean> {
  return content.evaluate((contentEl, { x, y }) => {
    const el = document.elementFromPoint(x, y);
    return !!el && contentEl.contains(el);
  }, { x, y });
}

/**
 * Clipping viewport (backlog row 102): `CarouselContent` translates its
 * scroller for the edge rubber-band, but an element's own `overflow` clips
 * relative to ITS OWN box, and a `transform` moves that box -- clip region
 * included -- as one rigid unit. A scroller alone therefore never clips its
 * own overdrag: on real hardware a trackpad overdrag pushes the whole
 * visible slide track ~750px past the carousel, unclipped, overlapping the
 * rest of the page (`primitives/src/carousel.rs`'s `CarouselContent` doc has
 * the full "Correction" writeup). So a stationary wrapper
 * (`div[data-slot="carousel-viewport"]`, `overflow: clip`) surrounds the
 * scroller, and translating the scroller moves content inside a clip
 * boundary that never itself moves. Every "does not paint outside the
 * viewport" assertion below fails on a scroller without that wrapper.
 */
test.describe("Carousel: clipping viewport (edge overdrag never paints outside the carousel)", () => {
  test("the viewport wraps the scroller, clips with overflow: clip (not hidden), and the transformed track is its descendant", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const viewport = viewportLocator(frame);
    const content = frame.locator(".dx-carousel-content");

    await expect(viewport).toHaveCount(1);
    const overflow = await viewport.evaluate((el) => {
      const cs = getComputedStyle(el);
      return { x: cs.overflowX, y: cs.overflowY };
    });
    // `clip`, never `hidden`: a `hidden` box is still a scroll container,
    // which would make the paging path's `scrollIntoView` on a slide scroll
    // this wrapper too and permanently offset the whole track.
    expect(overflow.x).toBe("clip");
    expect(overflow.y).toBe("clip");

    const isDescendant = await viewport.evaluate((vpEl) => vpEl.querySelector(".dx-carousel-content") !== null);
    expect(isDescendant).toBe(true);
  });

  test("a pointer overdrag past the first slide (horizontal, LTR) never paints a slide past the viewport's edge", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const viewport = viewportLocator(frame);
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    const pitch = await slidePitch(slide(1));
    const rawTravel = pitch * 1.5;
    // Positive dx at slide 1 -- see the "edge rubber-band" describe block's
    // identical test above for why this is the physical-previous direction.
    await dragHold(page, content, rawTravel, 0);
    await expect(async () => {
      expect(parseTranslatePx(await readContentTransform(content))).not.toBeNull();
    }).toPass({ timeout: 2000 });

    const box = await viewport.boundingBox();
    if (!box) {
      throw new Error("viewport has no bounding box");
    }
    // Just past the viewport's right edge, inside the padding-inline
    // gutter but before Previous/Next's own 20px-clear zone
    // (`preview/src/components/carousel/style.css`'s own derivation) --
    // this is squarely "the page, where the translated track would be" if
    // it were not clipped, and clear of the button so a hit there can only
    // mean the track itself painted through.
    const x = box.x + box.width + 10;
    const y = box.y + box.height * 0.2;
    expect(await paintsAt(content, x, y), "a slide painted past the viewport's own right edge").toBe(false);

    await page.mouse.up();
    await expect(async () => {
      expect(await readContentTransform(content)).toBe("");
    }).toPass({ timeout: 2000 });
  });

  test("a pointer overdrag at the start boundary under dir=rtl never paints a slide past the viewport's edge", async ({
    page,
  }) => {
    await goto(page, "rtl");
    const frame = demoFrame(page, "rtl");
    const viewport = viewportLocator(frame);
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });

    const pitch = await slidePitch(slide(1));
    const rawTravel = pitch * 1.5;
    // Negative dx is the physical-previous direction under RTL -- see the
    // "edge rubber-band" describe block's identical test above.
    await dragHold(page, content, -rawTravel, 0);
    await expect(async () => {
      expect(parseTranslatePx(await readContentTransform(content))).not.toBeNull();
    }).toPass({ timeout: 2000 });

    const box = await viewport.boundingBox();
    if (!box) {
      throw new Error("viewport has no bounding box");
    }
    // The track moved left this time (negative translateX), so it is the
    // LEFT edge that would escape.
    const x = box.x - 10;
    const y = box.y + box.height * 0.2;
    expect(await paintsAt(content, x, y), "a slide painted past the viewport's own left edge").toBe(false);

    await page.mouse.up();
    await expect(async () => {
      expect(await readContentTransform(content)).toBe("");
    }).toPass({ timeout: 2000 });
  });

  test("a pointer overdrag at the start boundary in the vertical variant never paints a slide past the viewport's edge", async ({
    page,
  }) => {
    await goto(page, "vertical");
    const frame = demoFrame(page, "vertical");
    const viewport = viewportLocator(frame);
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });

    const pitch = await slidePitch(slide(1), "vertical");
    const rawTravel = pitch * 1.5;
    // Positive dy -- see the "edge rubber-band" describe block's identical
    // test above.
    await dragHold(page, content, 0, rawTravel);
    await expect(async () => {
      expect(parseTranslatePx(await readContentTransform(content))).not.toBeNull();
    }).toPass({ timeout: 2000 });

    const box = await viewport.boundingBox();
    if (!box) {
      throw new Error("viewport has no bounding box");
    }
    // The track moved down (positive translateY), so it is the BOTTOM edge
    // that would escape. `x` stays away from the horizontal center, where
    // Previous/Next sit (`inset-inline-start: 50%`).
    const x = box.x + box.width * 0.1;
    const y = box.y + box.height + 10;
    expect(await paintsAt(content, x, y), "a slide painted past the viewport's own bottom edge").toBe(false);

    await page.mouse.up();
    await expect(async () => {
      expect(await readContentTransform(content)).toBe("");
    }).toPass({ timeout: 2000 });
  });

  // Both wheel tests drive a push that HOLDS its band (rising, never
  // repeating, peak >= 40px: only the 1500ms backstop or a cursor move
  // releases it -- see the mode D describe block's own header), so the
  // static held band is what `paintsAt` probes; nothing animated is read
  // across a round trip. The page's mouse is never moved during the probe
  // (a cursor move is D's "fingers lifted" signal).
  test("a wheel overdrag at the start boundary never paints a slide past the viewport's edge", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const viewport = viewportLocator(frame);
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    const run = await playWheel(content, { deltas: neg(WHEEL.hold) });
    const depth = parseTranslatePx(run.finalTransform);
    expect(depth, "expected a held, nonzero translateX").not.toBeNull();
    expect(depth!).toBeGreaterThan(0);

    const box = await viewport.boundingBox();
    if (!box) {
      throw new Error("viewport has no bounding box");
    }
    const x = box.x + box.width + 10;
    const y = box.y + box.height * 0.2;
    expect(await paintsAt(content, x, y), "a slide painted past the viewport's own right edge").toBe(false);

    await expectBandGone(content, 3000);
  });

  test("a wheel overdrag at the end boundary never paints a slide past the viewport's edge", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const viewport = viewportLocator(frame);
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    const next = frame.getByRole("button", { name: "Next slide" });

    for (let i = 0; i < 4; i++) {
      await next.click();
      await expectSnappedToBoundary(content, slide(i + 2));
    }
    await expect(slide(5)).toHaveAttribute("data-selected", "true");

    const run = await playWheel(content, { deltas: WHEEL.hold });
    const depth = parseTranslatePx(run.finalTransform);
    expect(depth, "expected a held, nonzero translateX").not.toBeNull();
    expect(depth!).toBeLessThan(0);

    const box = await viewport.boundingBox();
    if (!box) {
      throw new Error("viewport has no bounding box");
    }
    const x = box.x - 10;
    const y = box.y + box.height * 0.2;
    expect(await paintsAt(content, x, y), "a slide painted past the viewport's own left edge").toBe(false);

    await expectBandGone(content, 3000);
  });
});
