/**
 * Shared helpers for the `carousel*.spec.ts` parts (split out of the former
 * single `carousel.spec.ts`, backlog row 267): navigation, the `demoFrame()` scoping
 * helper, drag/scroll/transform samplers and the wheel-sequence player.
 *
 * SCOPING: every variant of a "Normal"-kind component renders on the same
 * page at once, so every locator in the specs is rooted through `demoFrame()`
 * (see the header of `carousel.spec.ts` for the full write-up).
 */

import { expect } from "./fixtures";
import { type Page, type Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";


// `waitUntil: "networkidle"` alone (not the default "load") is not enough
// under the SSG lane: a prerendered page's interactive elements already
// exist in the served HTML before the wasm bundle finishes loading and
// hydrating, so an interaction issued right after goto can land on inert
// markup -- Dioxus's event delegation only attaches once hydration walks
// the tree, and a missed DOM event (a click, a mouseenter) is gone for
// good, not queued. Under `dx serve` this race cannot happen (elements are
// inserted into the DOM only once the client renders them, by which point
// listeners are already attached), so an SSG-only interaction (a first click,
// an initial hover) can silently land before hydration/mid-tick and never be
// observed as intended. `gotoHydrated` (./hydration.ts)
// waits on networkidle AND a real hydration-ready signal the app sets,
// closing the gap networkidle alone leaves open.
export const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

export async function goto(page: Page, variant: string) {
  await gotoHydrated(page, `${BASE_URL}/component/carousel/`, GOTO_OPTS);
  if (variant === "autoplay" || variant === "virtual_loop" || variant === "virtual_loop_rtl") {
    // Autoplay only runs while the carousel can be seen (`primitives/src/activity.rs`: off-screen or
    // hidden-tab autoplay is stopped and restarts with a fresh countdown), and this variant sits
    // below the fold of the stacked page. Bring it into view -- instantly, so the
    // position is settled before the next read.
    await page.evaluate((id) => {
      document.getElementById(id)?.scrollIntoView({ block: "center", behavior: "instant" });
    }, `component-preview-frame-${variant}`);
  }
}

/** See this file's own header ("SCOPING"). */
export function demoFrame(
  page: Page,
  variant:
    | "main"
    | "sizes"
    | "spacing"
    | "peek"
    | "align"
    | "api"
    | "indicators"
    | "vertical"
    | "rtl"
    | "rewind"
    | "autoplay"
    | "virtual_loop"
    | "virtual_loop_rtl"
    | "virtual_many",
): Locator {
  const id = variant === "main" ? "component-preview-frame" : `component-preview-frame-${variant}`;
  return page.locator(`#${id}`);
}

/**
 * Scroll `locator`'s element into view instantly, for tests that need a
 * settled, known position before their own subsequent real-coordinate math
 * (`page.mouse`-based drags below) -- NOT as a guard against Playwright's
 * own pre-`.hover()`/`.focus()` actionability scroll: that class
 * (dev-docs/backlog.md row 110 -- an unqualified scroll's default
 * `behavior: "auto"` deferring to an ambient
 * `html { scroll-behavior: smooth }` and animating for a couple hundred
 * ms after `.hover()` already fired, dragging the target out from under
 * the cursor) cannot happen: smooth scrolling is opt-in
 * (`html[data-anchor-scroll]`, `preview/assets/main.css`, row 228), so
 * the root scroll box is instant -- the "hovering the
 * carousel stops rotation..." test below needs no guard of its own.
 *
 * `oracle/tier2-html/top-layer.spec.ts`'s `pinNearTop` helper relies on
 * the same shared fixture, per that helper's own doc.
 */
export async function scrollIntoViewInstant(locator: Locator): Promise<void> {
  await locator.evaluate((el) =>
    el.scrollIntoView({ block: "center", inline: "center", behavior: "instant" }),
  );
}

/**
 * Poll until the given slide's inline-axis (horizontal) or block-axis
 * (vertical) start position matches its scroll container's own -- i.e.
 * scroll-snap has actually settled on this slide's boundary, not merely
 * "the click/keypress handler ran." Mirrors the exact technique
 * `primitives/src/carousel/scroll.rs`'s own `CAROUSEL_SCROLL_TRACKING_JS` uses
 * (`getBoundingClientRect`, never `scrollLeft`, so this works identically
 * under `dir="rtl"` with no sign-convention special-casing).
 *
 * The poll loop runs INSIDE the page (one `evaluate` call, a `requestAnimationFrame`
 * loop), not as repeated Node-side `expect().toPass()` round trips
 * (dev-docs/backlog.md row 112's construction: "sample in-page ... never
 * via separate round trips", row 112 instance 2). A Node-side `toPass({ timeout: 3000 })` would
 * re-issue a fresh CDP round trip (`content.boundingBox()` + `item.boundingBox()`) on every
 * poll attempt, so the 3000ms budget would have to cover both the scroll-snap settle itself AND
 * however long each Node<->browser round trip takes -- under two workers' concurrent
 * Chromium/CPU contention a slower round trip eats into the same fixed budget as a slower
 * settle, coupling a real UI wait to Node-process/IPC scheduling latency that has nothing to do
 * with whether the carousel settled in time. Driving the whole wait from a single
 * `requestAnimationFrame` loop in the page removes that coupling: the 3000ms deadline is
 * measured against `performance.now()` in the SAME realm as the animation, with zero
 * per-attempt IPC cost, so it reflects only genuine settle time.
 */
export async function expectSnappedToBoundary(content: Locator, item: Locator, orientation: "horizontal" | "vertical" = "horizontal") {
  const contentHandle = await content.elementHandle();
  const itemHandle = await item.elementHandle();
  if (!contentHandle || !itemHandle) {
    throw new Error("expectSnappedToBoundary: content or item resolved to no element");
  }
  const result = await content.page().evaluate(
    ([contentEl, itemEl, orientation]) => {
      return new Promise<{
        ok: boolean;
        delta: number;
        containerBox: { x: number; y: number };
        itemBox: { x: number; y: number };
      }>((resolve) => {
        const deadline = performance.now() + 3000;
        let lastPos = NaN;
        let stillFrames = 0;
        const check = () => {
          const containerRect = (contentEl as Element).getBoundingClientRect();
          const itemRect = (itemEl as Element).getBoundingClientRect();
          const delta =
            orientation === "horizontal"
              ? Math.abs(containerRect.x - itemRect.x)
              : Math.abs(containerRect.y - itemRect.y);
          // Settled = the slide has stopped moving: the same position on
          // three consecutive frames. Chromium 153's smooth scroll
          // spends its last frames 1px short of the snap point, which the
          // 2px tolerance below accepts, and the wheel band's own at-rest
          // test (`edgeGaps`, 0.5px) does not (backlog row 378).
          const pos = orientation === "horizontal" ? itemRect.x : itemRect.y;
          stillFrames = Math.abs(pos - lastPos) < 0.01 ? stillFrames + 1 : 0;
          lastPos = pos;
          // A couple of CSS pixels of tolerance for sub-pixel rounding
          // during a smooth-scroll settle -- not a loose bound: an
          // unsettled scroll (still mid-animation, or ending on the wrong
          // slide) misses by tens/hundreds of pixels, not 1-2.
          if (delta <= 2 && stillFrames >= 3) {
            resolve({
              ok: true,
              delta,
              containerBox: { x: containerRect.x, y: containerRect.y },
              itemBox: { x: itemRect.x, y: itemRect.y },
            });
            return;
          }
          if (performance.now() >= deadline) {
            resolve({
              ok: false,
              delta,
              containerBox: { x: containerRect.x, y: containerRect.y },
              itemBox: { x: itemRect.x, y: itemRect.y },
            });
            return;
          }
          requestAnimationFrame(check);
        };
        requestAnimationFrame(check);
      });
    },
    [contentHandle, itemHandle, orientation] as const,
  );
  expect(
    result.ok,
    `expected ${orientation} delta <= 2, got ${result.delta} -- ${JSON.stringify(result)}`,
  ).toBe(true);
}

/**
 * Drives a real pointer drag with `page.mouse` (down, N intermediate
 * moves, up) -- deliberately not a synthetic `dispatchEvent`, since
 * `CAROUSEL_DRAG_JS`'s own threshold/pointer-capture/click-suppression
 * logic all key off genuine `pointerdown`/`pointermove`/`pointerup`
 * events, which a script-fired fake event does not reliably reproduce
 * (see `primitives/src/carousel/content.rs`'s "Pointer drag" doc).
 */
export async function dragBy(page: Page, target: Locator, dx: number, dy: number, steps = 12) {
  // Without this, `boundingBox()` can return coordinates outside the
  // current viewport (`main`'s own content can sit at a
  // large negative y on first paint) -- `page.mouse` targets real
  // viewport coordinates, so a drag computed from an off-screen box
  // silently lands on nothing.
  await target.scrollIntoViewIfNeeded();
  const box = await target.boundingBox();
  if (!box) {
    throw new Error("dragBy: target has no bounding box");
  }
  const startX = box.x + box.width / 2;
  const startY = box.y + box.height / 2;
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(startX + (dx * i) / steps, startY + (dy * i) / steps);
    // A short pause between steps, matching a real mouse's ~60 Hz event
    // cadence rather than firing all steps in the same tick.
    await page.waitForTimeout(16);
  }
  await page.mouse.up();
}

/**
 * Like `dragBy`, but leaves the pointer DOWN at the end (no `mouse.up()`) --
 * for the edge rubber-band tests below, which need to read the track's
 * transform WHILE still dragging, before release's own `bounceHome` eases
 * it back to identity. Callers must release with `page.mouse.up()`
 * themselves once they are done inspecting the held state.
 */
export async function dragHold(page: Page, target: Locator, dx: number, dy: number, steps = 12) {
  await target.scrollIntoViewIfNeeded();
  const box = await target.boundingBox();
  if (!box) {
    throw new Error("dragHold: target has no bounding box");
  }
  const startX = box.x + box.width / 2;
  const startY = box.y + box.height / 2;
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(startX + (dx * i) / steps, startY + (dy * i) / steps);
    await page.waitForTimeout(16);
  }
}

/**
 * The carousel track's own inline `transform` (mode B3's edge rubber-band,
 * `primitives/src/carousel/drag.rs`'s `CAROUSEL_DRAG_JS` and
 * `primitives/src/carousel/wheel.rs`'s `CAROUSEL_WHEEL_BOUNCE_JS`
 * -- both mutate `element.style.transform` directly via `document::eval`,
 * never through a Dioxus-rendered attribute, so this reads the live inline
 * style, not a rendered one). Empty string means identity (no bounce).
 */
export async function readContentTransform(content: Locator): Promise<string> {
  return content.evaluate((el) => (el as HTMLElement).style.transform);
}

/** Parses the single `translateX(...)`/`translateY(...)` px value mode B3
 * writes, or `null` if the transform is empty/identity. */
export function parseTranslatePx(transform: string): number | null {
  const match = transform.match(/translate[XY]\(([-\d.]+)px\)/);
  return match ? Number.parseFloat(match[1]) : null;
}

/**
 * The *real* accessible names of every `role="group"`/`"tabpanel"` node the
 * browser's own platform accessibility tree currently exposes (Chrome's
 * CDP `Accessibility` domain, populated from the same computation real
 * assistive tech consumes) -- deliberately not `page.getByRole`/
 * `locator.ariaSnapshot()`. This installed
 * Playwright build's own ARIA-role engine does not treat `inert` as
 * excluding a node at all -- a minimal `<div role="group" inert>` still
 * matches `getByRole("group")` and appears in `ariaSnapshot()`, while the
 * *same* CDP call this helper uses omits it entirely from the very first
 * query (not merely "present but ignored").
 * effect (removal from the accessibility tree) is a platform-level
 * property, so the platform's own tree -- not Playwright's separate,
 * DOM-based ARIA approximation -- is the correct oracle for this
 * component's own "only visible slides are exposed" contract. `getByRole`/`ariaSnapshot` remain the right
 * tool everywhere else in this file (they already correctly reflect
 * `aria-*`/native semantics for everything that isn't `inert`), and this
 * helper is reached for *only* the a11y-tree-exposure assertions below.
 *
 * Scoped to `rootSelector` via `DOM.querySelector` + `Accessibility.queryAXTree`
 * (a *subtree* query), not `getFullAXTree` for the whole page -- this file's
 * own "SCOPING" header note applies here too: every demo variant's own
 * carousel is mounted on the same page at once, and several share the exact
 * same "k of N" label shape (an `autoplay`/`indicators`/`tabs` variant sitting
 * at its own untouched slide 1 is a perfectly legitimate, *different*
 * `"1 of 5"`/`"1 of 4"` node) -- an unscoped whole-page query cannot tell
 * those apart from a stale/leftover node in the variant actually under test.
 */
export async function axTreeGroupNames(page: Page, rootSelector: string): Promise<string[]> {
  const client = await page.context().newCDPSession(page);
  await client.send("DOM.enable");
  await client.send("Accessibility.enable");
  const { root } = await client.send("DOM.getDocument", { depth: -1, pierce: true });
  const { nodeId } = await client.send("DOM.querySelector", { nodeId: root.nodeId, selector: rootSelector });
  const { nodes } = await client.send("Accessibility.queryAXTree", { nodeId });
  const names = (nodes as Array<{ role?: { value?: string }; name?: { value?: string }; ignored?: boolean }>)
    .filter((n) => !n.ignored && (n.role?.value === "group" || n.role?.value === "tabpanel"))
    .map((n) => n.name?.value ?? "");
  await client.detach();
  return names;
}

/**
 * The carousel track's own per-slide pitch (px, along the given axis),
 * measured live from a rendered slide's own box rather than assumed --
 * every distance-based drag test below derives its drag length from this,
 * not a hardcoded pixel constant. A hardcoded distance silently breaks
 * when a demo wrapper's rendered width changes: a distance tuned to clear
 * 50% of one pitch clears only ~42% of a wider one, so
 * `expectSnappedToBoundary` reports the drag landing exactly one slide
 * short. Measuring live at each test's own run time survives any
 * geometry change.
 */
export async function slidePitch(item: Locator, orientation: "horizontal" | "vertical" = "horizontal"): Promise<number> {
  const box = await item.boundingBox();
  if (!box) {
    throw new Error("slidePitch: item has no bounding box");
  }
  return orientation === "horizontal" ? box.width : box.height;
}

/**
 * Samples `.dx-carousel-content`'s own scroll position at animation-frame
 * cadence, starting just before `trigger()` runs, until the value has both
 * (a) changed from its own starting point and (b) gone `STABLE_FRAMES_NEEDED`
 * consecutive frames without changing again -- or `HARD_CAP_MS` elapses,
 * whichever comes first -- so a caller can tell an animated transition (the
 * position passes through a value strictly between where it started and
 * where it ended) from an instant jump (it skips straight from start to
 * end). See this file's own "actually animates" describe block below, which
 * is the reason this exists: asserting the code *chose* `behavior: 'smooth'`
 * is not the same claim as the motion actually being animated, and only
 * this sampling proves the latter.
 *
 * A fixed wall-clock sampling window (say 600ms, opened before
 * `trigger()`) is a live bug, not a hardening measure: against a `dx serve`
 * dev server the click-to-scroll latency happens to fit inside 600ms, so it
 * passes, but against a statically-served release SSG build the wasm
 * `document::eval` round trip alone can exceed it -- every sample then reads
 * the pre-scroll value, reporting a false "instant jump" on a transition
 * that animates through dozens of distinct intermediate values. A settle-condition stop is honest
 * regardless of how long the round trip actually takes on a given
 * build/server, or how long the transition itself runs -- unlike a fixed
 * window, it cannot silently start under-sampling a slower environment or
 * a shorter transition.
 */
export async function sampleScrollDuring(
  page: Page,
  contentId: string,
  axis: "scrollLeft" | "scrollTop",
  trigger: () => Promise<void>,
): Promise<number[]> {
  const STABLE_FRAMES_NEEDED = 6;
  const HARD_CAP_MS = 3000;
  await page.evaluate(
    ({ id, axis, stableFramesNeeded, hardCapMs }) => {
      const w = window as unknown as {
        __dxCarouselSamples: number[];
        __dxCarouselDone: boolean;
      };
      w.__dxCarouselSamples = [];
      w.__dxCarouselDone = false;
      const el = document.getElementById(id) as unknown as Record<string, number>;
      const start = el[axis];
      const t0 = performance.now();
      let stableCount = 0;
      const tick = () => {
        const samples = w.__dxCarouselSamples;
        const value = el[axis];
        const previous = samples.length > 0 ? samples[samples.length - 1] : value;
        samples.push(value);
        stableCount = value === previous ? stableCount + 1 : 0;
        const hasChanged = value !== start;
        const settled = hasChanged && stableCount >= stableFramesNeeded;
        const timedOut = performance.now() - t0 > hardCapMs;
        if (settled || timedOut) {
          w.__dxCarouselDone = true;
          return;
        }
        requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    },
    { id: contentId, axis, stableFramesNeeded: STABLE_FRAMES_NEEDED, hardCapMs: HARD_CAP_MS },
  );
  await trigger();
  // A comfortable margin above the in-page HARD_CAP_MS -- that hard cap is
  // what's expected to end the wait in the overwhelming common case; this
  // outer one is only a safety net against the flag never being set at all.
  await page.waitForFunction(
    () => (window as unknown as { __dxCarouselDone: boolean }).__dxCarouselDone === true,
    undefined,
    { timeout: HARD_CAP_MS + 5000 },
  );
  return page.evaluate(() => (window as unknown as { __dxCarouselSamples: number[] }).__dxCarouselSamples);
}

/**
 * True if `samples` pass strictly through a value between its own first
 * and last entry. An instant jump never does (it sits at the start value,
 * then the end value, with nothing in between); a genuine smooth-scroll
 * transition always does -- the honest "was this actually animated"
 * check per this file's own "actually animates" describe block.
 */
export function passesThroughAnIntermediateValue(samples: number[]): boolean {
  const start = samples[0];
  const end = samples[samples.length - 1];
  const lo = Math.min(start, end);
  const hi = Math.max(start, end);
  if (hi - lo < 2) {
    return false;
  }
  return samples.some((v) => v > lo + 1 && v < hi - 1);
}

/** Delta magnitudes (px) shaped like recorded trackpad data; each test
 * applies the sign that pushes into the edge under test. */
export const WHEEL = {
  // A trackpad push: a ramp, then fingers lift and momentum decays.
  rampAndDecay: [3, 9, 21, 35, 50, 60, 62, 58, 54, 50, 46, 43, 40, 37, 34, 31, 29, 27, 25, 23, 21, 19, 18, 16, 15, 14, 13],
  // Rising, never repeating, peak >= 40px: the band holds (no decay, no
  // silence release shorter than the 1500ms backstop).
  hold: [10, 25, 45, 60, 70, 75, 78, 80, 83],
  // A native fling's momentum tail (~0.93 per event: "coasting").
  momentum: [95, 88, 82, 76, 70, 65, 61, 57, 53, 49],
  // A recorded re-push after coasting into slide 1 (bench rev 40
  // note): a dip, then a rise.
  repush: [13, 28, 67, 112, 133, 160],
  // A slow push: every step under 40px (the recorded ones are 5-16px).
  slow: [5, 8, 12, 14, 12, 13, 12, 14, 12],
};

export type WheelPlay = {
  /** Signed deltas, one event each. */
  deltas: readonly number[];
  axis?: "x" | "y";
  deltaMode?: number;
  shiftKey?: boolean;
  /** Keep sampling this long after the last event (ms). */
  tailMs?: number;
  /** After this many events, move the scroller (instantly) so its first
   * slide rests at the physical start edge -- the platform's own momentum
   * scroll carrying the track into slide 1. */
  arriveAtStartAfter?: number;
};
export type WheelRun = {
  /** Band (signed px) sampled every animation frame, `t` in ms from the first event. */
  samples: { t: number; v: number }[];
  /** Band right after each event. */
  afterEvent: number[];
  eventTimes: number[];
  axisSize: number;
  finalTransform: string;
};

export async function playWheel(content: Locator, play: WheelPlay): Promise<WheelRun> {
  return content.evaluate(async (node, p) => {
    const el = node as HTMLElement;
    const axis = p.axis ?? "x";
    const parse = (s: string) => {
      const m = s.match(/translate[XY]\(([-\d.]+)px\)/);
      return m ? Number.parseFloat(m[1]) : 0;
    };
    const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
    const samples: { t: number; v: number }[] = [];
    const afterEvent: number[] = [];
    const eventTimes: number[] = [];
    const t0 = performance.now();
    let running = true;
    const tick = () => {
      samples.push({ t: performance.now() - t0, v: parse(el.style.transform) });
      if (running) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    for (let i = 0; i < p.deltas.length; i++) {
      const d = p.deltas[i];
      el.dispatchEvent(
        new WheelEvent("wheel", {
          deltaX: axis === "x" ? d : 0,
          deltaY: axis === "y" ? d : 0,
          deltaMode: p.deltaMode ?? 0,
          shiftKey: p.shiftKey ?? false,
          bubbles: true,
          cancelable: true,
        }),
      );
      const t = performance.now() - t0;
      eventTimes.push(t);
      afterEvent.push(parse(el.style.transform));
      samples.push({ t, v: afterEvent[i] });
      if (p.arriveAtStartAfter === i + 1) {
        const horizontal = el.getAttribute("data-orientation") !== "vertical";
        const c = el.getBoundingClientRect();
        let lo = Infinity;
        for (const child of Array.from(el.children)) {
          const r = child.getBoundingClientRect();
          lo = Math.min(lo, horizontal ? r.left : r.top);
        }
        const delta = lo - (horizontal ? c.left : c.top);
        el.scrollBy({ left: horizontal ? delta : 0, top: horizontal ? 0 : delta, behavior: "instant" });
      }
    }
    await sleep(p.tailMs ?? 0);
    running = false;
    const track = el.parentElement as HTMLElement;
    const horizontal = el.getAttribute("data-orientation") !== "vertical";
    return {
      samples,
      afterEvent,
      eventTimes,
      axisSize: horizontal ? track.clientWidth : track.clientHeight,
      finalTransform: el.style.transform,
    };
  }, play);
}

/** Turn on D's debug telemetry for the next navigation (read once at mount). */
export async function enableWheelDebug(page: Page): Promise<void> {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem("dx-carousel-debug", "1");
    } catch {
      // A storage-less context simply records nothing; tests that need it fail loudly.
    }
  });
}

export type WheelRecord = {
  id: string;
  startedAtEdge: string;
  /** The edge the gesture started at rest against, when `isTrueEnd` refused it. */
  falseEnd: string | null;
  owned: boolean | null;
  notched: boolean;
  deltas: number[];
  releasedBy: string | null;
  peakBand: number;
  lastEventAt: number;
  releasedAt: number | null;
  endedByRepush: boolean;
};

/** D's telemetry records for this scroller only (every variant shares the page). */
export async function wheelRecords(content: Locator): Promise<WheelRecord[]> {
  return content.evaluate((el) => {
    const log = (window as unknown as { __dxCarouselWheel?: WheelRecord[] }).__dxCarouselWheel ?? [];
    return JSON.parse(JSON.stringify(log.filter((r) => r.id === el.id)));
  });
}

/** Waits (polling a static end state, not sampling an animation) for the band to be gone. */
export async function expectBandGone(content: Locator, timeout = 3000): Promise<void> {
  await expect.poll(() => readContentTransform(content), { timeout }).toBe("");
}

export function maxAbsBand(run: WheelRun): number {
  return Math.max(0, ...run.samples.map((s) => Math.abs(s.v)));
}

/** In-page rAF sampler around real (CDP) input, for the tests that must use
 * trusted events -- same "sample in-page" construction as `playWheel`. */
export async function sampleBandDuring(content: Locator, action: () => Promise<void>, tailMs = 0): Promise<number[]> {
  await content.evaluate((node) => {
    const el = node as HTMLElement;
    const w = window as unknown as { __dxBandSamples: number[]; __dxBandRun: boolean };
    w.__dxBandSamples = [];
    w.__dxBandRun = true;
    const tick = () => {
      const m = el.style.transform.match(/translate[XY]\(([-\d.]+)px\)/);
      w.__dxBandSamples.push(m ? Number.parseFloat(m[1]) : 0);
      if (w.__dxBandRun) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
  await action();
  if (tailMs) await content.page().waitForTimeout(tailMs);
  return content.evaluate(() => {
    const w = window as unknown as { __dxBandSamples: number[]; __dxBandRun: boolean };
    w.__dxBandRun = false;
    return w.__dxBandSamples;
  });
}

export const neg = (xs: number[]) => xs.map((x) => -x);
