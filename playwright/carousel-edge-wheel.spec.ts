/**
 * Carousel wheel/trackpad edge band (mode D). Split out of `carousel.spec.ts` (row 267);
 * the wheel-sequence helpers are in `carousel-helpers.ts`.
 */

import { test, expect } from "./fixtures";
import {
  goto,
  demoFrame,
  expectSnappedToBoundary,
  readContentTransform,
  slidePitch,
  WHEEL,
  playWheel,
  enableWheelDebug,
  wheelRecords,
  expectBandGone,
  maxAbsBand,
  sampleBandDuring,
  neg,
} from "./carousel-helpers";

/**
 * Wheel/trackpad edge band (mode D, the "platform-aware band") --
 * `primitives/src/carousel/wheel.rs`'s `CAROUSEL_WHEEL_BAND_JS`, ported from the
 * carousel lab's bench (`bench-rev40.html`, `mode === "onewriter"`). It is
 * the wheel path; the pointer half above is B3.
 *
 * D's contract, which every test below holds it to:
 * - the band is a `transform` on `.dx-carousel-content` (the scroller) and
 *   `""` at rest -- the same element and reader (`readContentTransform`)
 *   the B3 tests use, never a scroll-position write;
 * - ownership is decided once per gesture from where the scroller RESTS
 *   when the gesture starts: only a gesture that starts at rest at an edge
 *   and pushes into it gets a band (bounded by `LIMIT` = 0.5 x the axis);
 * - release: smooth momentum decay, cursor movement, slow-push silence
 *   (250ms), notched-wheel silence (140ms), reversal, or the 1500ms
 *   backstop; then an ease-out spring-back (340ms) to `""`;
 * - only this carousel's own axis counts.
 *
 * SAMPLING (backlog row 112's construction): every sequence is dispatched
 * IN-PAGE (`playWheel`), and the transform is sampled in-page right after
 * every event and on every animation frame of the silence that follows --
 * an animated value is never read across a CDP round trip.
 * TIMING (backlog row 407): a sequence is ONE continuous stream, dispatched
 * in a single task. D's rules read time only as silence (GAP_MS, the
 * notched / slow-push windows, the backstop), so every one of them must see
 * "no silence" inside the stream and exactly the test's `tailMs` after it.
 * A stream spaced by timers is not that input on a loaded box: one late
 * timer is a >100ms gap, which D rightly reads as a new gesture, and a tail
 * split off that way is a short slow push released by "slow-silence".
 * Dispatched in one task, no wall-clock gap can open inside the stream. Synthetic
 * `WheelEvent`s are untrusted, so they never scroll the scroller itself;
 * where a test needs the platform's own scroll (a fling arriving at slide 1)
 * it moves the scroller in-page, instantly, standing in for the compositor.
 * The telemetry this reads (`window.__dxCarouselWheel`) is D's own
 * debug-flag output (`localStorage["dx-carousel-debug"] === "1"`), enabled
 * per test by `enableWheelDebug`. Real-hardware feel remains
 * a real-device check; these pin the logic with recorded-shape sequences.
 */



test.describe("Carousel: wheel/trackpad edge band (mode D, platform-aware)", () => {
  test("an owned push at rest at the start edge draws a band bounded by 0.5 x the axis, then springs back to identity", async ({
    page,
  }) => {
    await enableWheelDebug(page);
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    expect(await readContentTransform(content)).toBe("");

    // Negative deltaX at slide 1 asks for more "previous" than exists.
    const run = await playWheel(content, { deltas: neg(WHEEL.rampAndDecay), tailMs: 900 });
    const peak = maxAbsBand(run);
    expect(peak, "an owned push must draw a band").toBeGreaterThan(5);
    expect(Math.min(...run.afterEvent), "pushing right, the band translates right").toBeGreaterThanOrEqual(0);
    expect(peak).toBeLessThanOrEqual(run.axisSize * 0.5 + 0.5);
    expect(run.finalTransform, "springs back to identity after the release").toBe("");

    const [rec] = (await wheelRecords(content)).slice(-1);
    expect(rec.startedAtEdge).toBe("left");
    expect(rec.owned).toBe(true);
    expect(rec.releasedBy, "the momentum tail is what released it").toBe("smooth-decay");
    // THE RULE: the band never moved `selected`.
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("an owned push at the end edge draws a band in the other direction and springs back", async ({ page }) => {
    await enableWheelDebug(page);
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

    const run = await playWheel(content, { deltas: WHEEL.rampAndDecay, tailMs: 900 });
    expect(Math.min(...run.samples.map((s) => s.v)), "pushing left past the end, the band translates left").toBeLessThan(-5);
    expect(maxAbsBand(run)).toBeLessThanOrEqual(run.axisSize * 0.5 + 0.5);
    expect(run.finalTransform).toBe("");
    const [rec] = (await wheelRecords(content)).slice(-1);
    expect(rec.startedAtEdge).toBe("right");
    expect(rec.owned).toBe(true);
    await expect(slide(5)).toHaveAttribute("data-selected", "true");
  });

  /**
   * Sustained push (the wheel counterpart of B3's "a sustained constant stream
   * grows sub-linearly"). A perfectly constant stream is, correctly, a notched wheel
   * (see the notched test below) and releases after 140ms, so the same two
   * curve properties are checked on a sustained push whose steps vary the
   * way a real hand's do: the band stays under the asymptote (`LIMIT` x the
   * axis -- 0.5, tighter than B3's 1.0), and doubling the input less
   * than doubles the depth (WebKit's concave curve). Same two properties as
   * B3, with a tighter bound.
   */
  test("a sustained hard push stays within 0.5 x the axis and grows sub-linearly", async ({ page }) => {
    await enableWheelDebug(page);
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    const sustained = Array.from({ length: 20 }, (_, i) => -(150 + (i % 3) * 7));
    // Momentum all the way down, as a real tail runs (~0.93 per event, to a
    // few px): D's decay test only counts once the peak is above 3 x the
    // smallest step this device has sent, and a page that has only ever
    // seen 100px+ steps has not learned its quantum yet.
    const tail = Array.from({ length: 34 }, (_, i) => -Math.max(3, Math.round(140 * 0.93 ** i)));
    const run = await playWheel(content, { deltas: [...sustained, ...tail], tailMs: 900 });
    const depth10 = Math.abs(run.afterEvent[9]);
    const depth20 = Math.abs(run.afterEvent[19]);
    expect(depth10).toBeGreaterThan(0);
    expect(depth20, "continued pushing should grow the depth").toBeGreaterThan(depth10);
    expect(depth20, "doubling the input should not double the depth (sub-linear curve)").toBeLessThan(depth10 * 2);
    expect(maxAbsBand(run)).toBeLessThanOrEqual(run.axisSize * 0.5 + 0.5);
    expect(run.finalTransform).toBe("");
    const [rec] = (await wheelRecords(content)).slice(-1);
    expect(rec.releasedBy).toBe("smooth-decay");
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("a fling that arrives at slide 1 from another slide with momentum draws NO band (the platform's edge)", async ({
    page,
  }) => {
    await enableWheelDebug(page);
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await frame.getByRole("button", { name: "Next slide" }).click();
    await expectSnappedToBoundary(content, slide(2));

    // Starts at slide 2 (not at rest at an edge); the "platform" carries the
    // track into slide 1 after four events and the tail keeps arriving there.
    const run = await playWheel(content, { deltas: neg(WHEEL.momentum), arriveAtStartAfter: 4, tailMs: 300 });
    expect(maxAbsBand(run), "D never draws on a gesture that did not start at rest at the edge").toBe(0);
    expect(run.finalTransform).toBe("");
    const recs = await wheelRecords(content);
    expect(recs.length).toBeGreaterThan(0);
    const [rec] = recs.slice(-1);
    expect(rec.startedAtEdge).toBe("none");
    expect(rec.owned).toBe(false);
  });

  test("after coasting into slide 1, a dip-then-rise re-push starts a new, owned gesture and draws a band", async ({
    page,
  }) => {
    await enableWheelDebug(page);
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await frame.getByRole("button", { name: "Next slide" }).click();
    await expectSnappedToBoundary(content, slide(2));

    // ..., -57, -53, -49 (momentum), then -13, -28, -67, -112, -133, -160
    // (new fingers) -- one continuous stream, never a GAP_MS silence, so only
    // the coast-then-rise rule (bench rev 40) can split it.
    const deltas = neg([...WHEEL.momentum, ...WHEEL.repush]);
    const run = await playWheel(content, { deltas, arriveAtStartAfter: 4, tailMs: 50 });
    const nMomentum = WHEEL.momentum.length;
    // The stream is one task, so the arrival's samples are its events' own (by index, not by time).
    const before = run.afterEvent.slice(0, nMomentum);
    expect(Math.max(0, ...before.map((v) => Math.abs(v))), "no band during the platform's own arrival").toBe(0);
    expect(Math.max(...run.afterEvent.slice(nMomentum)), "the re-push is owned: D draws its band").toBeGreaterThan(5);

    const recs = await wheelRecords(content);
    const owned = recs.filter((r) => r.owned === true);
    expect(owned.length).toBe(1);
    expect(owned[0].startedAtEdge).toBe("left");
    // The record before it is the arrival, ended by the re-push rule (not a gap).
    const i = recs.indexOf(owned[0]);
    expect(recs[i - 1].owned).toBe(false);
    expect(recs[i - 1].endedByRepush).toBe(true);
    await expectBandGone(content);
  });

  test("a slow push (steps under 16px) holds the band, then releases ~250ms after its last event", async ({ page }) => {
    await enableWheelDebug(page);
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");

    const run = await playWheel(content, { deltas: neg(WHEEL.slow), tailMs: 1100 });
    const last = run.eventTimes[run.eventTimes.length - 1];
    const held = run.afterEvent[run.afterEvent.length - 1];
    expect(held).toBeGreaterThan(0);
    // No release inside the first 200ms of silence: the band stays exactly put.
    const early = run.samples.filter((s) => s.t > last && s.t < last + 200);
    expect(early.length).toBeGreaterThan(0);
    for (const s of early) expect(s.v).toBeCloseTo(held, 5);
    expect(run.finalTransform, "released and home well before the 1500ms backstop").toBe("");

    const [rec] = (await wheelRecords(content)).slice(-1);
    expect(rec.releasedBy).toBe("slow-silence");
    const silence = (rec.releasedAt ?? 0) - rec.lastEventAt;
    expect(silence).toBeGreaterThanOrEqual(230);
    expect(silence).toBeLessThan(700);
  });

  for (const [label, play] of [
    ["deltaMode 1 (lines)", { deltas: [-3, -3, -3, -3], deltaMode: 1 }],
    ["deltaMode 0, repeated identical 120px steps", { deltas: [-120, -120, -120, -120] }],
  ] as const) {
    test(`a notched mouse wheel -- ${label} -- releases fast, not after the 1500ms backstop`, async ({ page }) => {
      await enableWheelDebug(page);
      await goto(page, "main");
      const frame = demoFrame(page, "main");
      const content = frame.locator(".dx-carousel-content");

      const run = await playWheel(content, { ...play, tailMs: 900 });
      expect(maxAbsBand(run)).toBeGreaterThan(0);
      expect(run.finalTransform, "home within ~140ms + the 340ms spring-back").toBe("");
      const [rec] = (await wheelRecords(content)).slice(-1);
      expect(rec.owned).toBe(true);
      expect(rec.notched).toBe(true);
      expect(rec.releasedBy).toBe("notch-silence");
      const silence = (rec.releasedAt ?? 0) - rec.lastEventAt;
      expect(silence).toBeGreaterThanOrEqual(120);
      expect(silence, "faster than even the slow-push release").toBeLessThan(250);
    });
  }

  test("a cursor movement after the push counts as the fingers lifting and releases the band", async ({ page }) => {
    await enableWheelDebug(page);
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const box = await content.boundingBox();
    if (!box) throw new Error("content has no bounding box");
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);

    const run = await playWheel(content, { deltas: neg(WHEEL.hold) });
    expect(run.finalTransform, "the band is held (no silence release this short)").not.toBe("");
    await page.mouse.move(box.x + box.width / 2 + 20, box.y + box.height / 2);
    await page.mouse.move(box.x + box.width / 2 + 40, box.y + box.height / 2);
    await expectBandGone(content, 1200);
    const [rec] = (await wheelRecords(content)).slice(-1);
    expect(rec.releasedBy).toBe("cursor-moved");
  });

  test("vertical wheel input over a horizontal carousel at slide 1 draws no band, and scrolls the page", async ({
    page,
  }) => {
    await enableWheelDebug(page);
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");

    for (const sign of [-1, 1]) {
      const run = await playWheel(content, { deltas: WHEEL.rampAndDecay.map((x) => sign * x), axis: "y", tailMs: 100 });
      expect(maxAbsBand(run), `deltaY ${sign < 0 ? "up" : "down"} is not this carousel's axis`).toBe(0);
    }
    // Ignored entirely: not even a gesture record.
    expect(await wheelRecords(content)).toEqual([]);

    // Real (trusted) vertical wheel over the carousel scrolls the page.
    await content.hover();
    const before = await page.evaluate(() => window.scrollY);
    const samples = await sampleBandDuring(content, async () => {
      await page.mouse.wheel(0, 200);
      await page.mouse.wheel(0, 200);
    }, 300);
    expect(Math.max(0, ...samples.map(Math.abs))).toBe(0);
    await expect.poll(() => page.evaluate(() => window.scrollY)).toBeGreaterThan(before);
  });

  test("Shift+wheel (reported as deltaY with shiftKey) counts as horizontal intent", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const run = await playWheel(content, { deltas: neg(WHEEL.rampAndDecay), axis: "y", shiftKey: true, tailMs: 900 });
    expect(maxAbsBand(run)).toBeGreaterThan(5);
    expect(run.finalTransform).toBe("");
  });

  test("vertical orientation: an owned push at the top draws a translateY band and ignores horizontal input", async ({
    page,
  }) => {
    await enableWheelDebug(page);
    await goto(page, "vertical");
    const frame = demoFrame(page, "vertical");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    const sideways = await playWheel(content, { deltas: neg(WHEEL.rampAndDecay), axis: "x", tailMs: 100 });
    expect(maxAbsBand(sideways), "deltaX is not a vertical carousel's axis").toBe(0);

    const run = await playWheel(content, { deltas: neg(WHEEL.rampAndDecay), axis: "y", tailMs: 900 });
    expect(Math.min(...run.afterEvent), "pushing down at the top, the band translates down").toBeGreaterThanOrEqual(0);
    expect(maxAbsBand(run)).toBeGreaterThan(5);
    expect(maxAbsBand(run)).toBeLessThanOrEqual(run.axisSize * 0.5 + 0.5);
    expect(run.finalTransform).toBe("");
    const recs = await wheelRecords(content);
    expect(recs.map((r) => r.startedAtEdge)).toContain("top");
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("RTL: an owned push at slide 1 (the physical right edge) draws a leftward band", async ({ page }) => {
    await enableWheelDebug(page);
    await goto(page, "rtl");
    const frame = demoFrame(page, "rtl");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    // Under RTL, slide 1 rests at the physical RIGHT; positive deltaX pushes
    // past it. All geometry is physical, so nothing here is mirrored by hand.
    const run = await playWheel(content, { deltas: WHEEL.rampAndDecay, tailMs: 900 });
    expect(Math.max(...run.afterEvent), "the band translates left").toBeLessThanOrEqual(0);
    expect(maxAbsBand(run)).toBeGreaterThan(5);
    expect(maxAbsBand(run)).toBeLessThanOrEqual(run.axisSize * 0.5 + 0.5);
    expect(run.finalTransform).toBe("");
    const [rec] = (await wheelRecords(content)).slice(-1);
    expect(rec.startedAtEdge).toBe("right");
    expect(rec.owned).toBe(true);
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  /**
   * The band only engages at a TRUE end of the
   * data (`isTrueEnd` in `CAROUSEL_WHEEL_BAND_JS`), and a looping carousel
   * has none -- including `LoopMode::Rewind`, whose ends are physical but
   * wrap, so Rewind does not band like a physical edge.
   */
  test("loop_mode Rewind never bands, at its first, middle or last slide", async ({ page }) => {
    await enableWheelDebug(page);
    await goto(page, "rewind");
    const frame = demoFrame(page, "rewind");
    const region = frame.getByRole("region", { name: "Rewind-loop gallery", exact: true });
    const content = region.locator(".dx-carousel-content");
    const slide = (n: number) => region.getByRole("group", { name: `${n} of 5` });
    const next = region.getByRole("button", { name: "Next slide" });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    expect(await content.getAttribute("data-loop")).toBe("true");

    // Slide 1 (rest at the physical start), slide 3 (middle), slide 5 (end).
    let current = 1;
    for (const [n, pushes] of [
      [1, [-1]],
      [3, [-1, 1]],
      [5, [1]],
    ] as const) {
      while (current < n) {
        await next.click();
        current += 1;
        await expectSnappedToBoundary(content, slide(current));
      }
      for (const sign of pushes) {
        const run = await playWheel(content, { deltas: WHEEL.rampAndDecay.map((x) => sign * x), tailMs: 400 });
        expect(maxAbsBand(run), `slide ${n}, push ${sign}`).toBe(0);
      }
    }
    // The pushes at slides 1 and 5 really started at rest against an edge,
    // and were refused as not a true end -- not merely never at an edge.
    const refused = (await wheelRecords(content)).map((r) => r.falseEnd).filter(Boolean);
    expect(refused).toEqual(expect.arrayContaining(["left", "right"]));
  });

  test("a CarouselVirtualContent seamless loop has no true end, so no band even at its window's physical start", async ({
    page,
  }) => {
    await enableWheelDebug(page);
    await goto(page, "virtual_loop");
    const frame = demoFrame(page, "virtual_loop");
    const content = frame.locator(".dx-carousel-content");
    await expect.poll(() => frame.locator('[data-selected="true"]').getAttribute("aria-label")).toBe("1 of 12");

    // Jump to the window's own physical start and push into it in the same
    // task, before any re-anchor can run: the bridge is attached, and it is
    // `isTrueEnd` (a looping carousel has no true end) that refuses it.
    const result = await content.evaluate((node) => {
      const el = node as HTMLElement;
      // Physical min over every rendered slide (the window is keyed by
      // position, so DOM order alone says nothing about which is leftmost).
      const startGap = () => {
        let lo = Infinity;
        for (const child of Array.from(el.children)) lo = Math.min(lo, child.getBoundingClientRect().left);
        return lo - el.getBoundingClientRect().left;
      };
      // Test setup only: snapping (with `scroll-snap-stop: always`) holds
      // this scroller on its current slide against an instant jump, so
      // suspend it just for the placement, and restore it after.
      const snap = el.style.scrollSnapType;
      el.style.scrollSnapType = "none";
      el.scrollBy({ left: startGap(), behavior: "instant" });
      const gap = startGap();
      for (let i = 0; i < 8; i++) {
        el.dispatchEvent(new WheelEvent("wheel", { deltaX: -(20 + i * 5), bubbles: true, cancelable: true }));
      }
      const transform = el.style.transform;
      el.style.scrollSnapType = snap;
      return { gap, transform };
    });
    expect(Math.abs(result.gap), `the scroller really was at its window's start (gap ${result.gap}px)`).toBeLessThan(1);
    expect(result.transform).toBe("");
    const [rec] = (await wheelRecords(content)).slice(-1);
    expect(rec.falseEnd, "at rest against the window's start, refused as not a true end").toBe("left");
    expect(rec.owned).not.toBe(true);
  });

  test("prefers-reduced-motion: reduce draws no wheel band", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const run = await playWheel(content, { deltas: neg(WHEEL.rampAndDecay), tailMs: 100 });
    expect(maxAbsBand(run)).toBe(0);
    expect(run.finalTransform).toBe("");
  });

  test("telemetry is off unless the debug flag is set: nothing is recorded", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const run = await playWheel(content, { deltas: neg(WHEEL.rampAndDecay), tailMs: 100 });
    expect(maxAbsBand(run), "the band itself still works").toBeGreaterThan(5);
    expect(await page.evaluate(() => (window as unknown as { __dxCarouselWheel?: unknown }).__dxCarouselWheel)).toBeUndefined();
  });

  test("real (trusted) wheel input at the start and end edges draws a band that springs back, without paging", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    const next = frame.getByRole("button", { name: "Next slide" });

    // `deltaX`: a plain vertical delta on a horizontal carousel is not its
    // axis (see the vertical-input test above). 120px identical steps are a
    // notched wheel, so each burst releases fast.
    await content.hover();
    const start = await sampleBandDuring(content, async () => {
      for (let i = 0; i < 6; i++) await page.mouse.wheel(-120, 0);
    });
    expect(Math.max(...start), "a band at the start edge").toBeGreaterThan(0);
    await expectBandGone(content);
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    for (let i = 0; i < 4; i++) {
      await next.click();
      await expectSnappedToBoundary(content, slide(i + 2));
    }
    await content.hover();
    const end = await sampleBandDuring(content, async () => {
      for (let i = 0; i < 6; i++) await page.mouse.wheel(120, 0);
    });
    expect(Math.min(...end), "a band at the end edge").toBeLessThan(0);
    await expectBandGone(content);
    await expect(slide(5)).toHaveAttribute("data-selected", "true");
  });

  test("a mid-range wheel scroll produces no lingering transform and pages correctly", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    await content.hover();
    const pitch = await slidePitch(slide(1));
    for (let i = 0; i < 8; i++) {
      await page.mouse.wheel(pitch / 6, 0);
    }

    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    await expectBandGone(content, 2000);
  });
});
