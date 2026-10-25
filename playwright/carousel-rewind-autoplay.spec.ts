/**
 * Carousel loop_mode Rewind, autoplay + rotation control, and the indicators (tablist)
 * variant. Split out of `carousel.spec.ts` (row 267).
 */

import { test, expect } from "./fixtures";
import { type Locator } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import {
  goto,
  demoFrame,
  expectSnappedToBoundary,
  dragBy,
  readContentTransform,
  sampleScrollDuring,
  passesThroughAnIntermediateValue,
  sampleBandDuring,
} from "./carousel-helpers";

/**
 * `loop_mode: LoopMode::Rewind` -- the explicit opt-in (backlog row 91's
 * shadcn-parity addendum); `r#loop: true` alone (the default
 * `LoopMode::Seamless`) does nothing for the plain children API,
 * see
 * `primitives/src/carousel/options.rs`'s own `LoopMode` doc. One `rewind` demo, two rows
 * (LTR/RTL), shows the opt-in -- see
 * `preview/src/components/carousel/variants/rewind/mod.rs`.
 */
test.describe("Carousel: loop_mode Rewind (explicit opt-in wraparound)", () => {
  test("Previous and Next are never disabled, even at the first/last slide", async ({ page }) => {
    await goto(page, "rewind");
    const frame = demoFrame(page, "rewind");
    const region = frame.getByRole("region", { name: "Rewind-loop gallery", exact: true });
    const previous = region.getByRole("button", { name: /previous/i });
    const next = region.getByRole("button", { name: /next/i });

    await expect(previous).toBeEnabled();
    await expect(next).toBeEnabled();

    for (let i = 0; i < 4; i++) {
      await next.click();
    }
    await expect(region.getByRole("group", { name: "5 of 5" })).toHaveAttribute("data-selected", "true");
    // A non-loop (or non-opted-in) carousel would have `next` disabled
    // here -- `LoopMode::Rewind` never does.
    await expect(next).toBeEnabled();
    await expect(previous).toBeEnabled();
  });

  test("Next at the last slide rewinds to the first INSTANTLY; Previous at the first rewinds to the last", async ({ page }) => {
    await goto(page, "rewind");
    const frame = demoFrame(page, "rewind");
    const region = frame.getByRole("region", { name: "Rewind-loop gallery", exact: true });
    const content = region.locator(".dx-carousel-content");
    const previous = region.getByRole("button", { name: /previous/i });
    const next = region.getByRole("button", { name: /next/i });
    const slide = (n: number) => region.getByRole("group", { name: `${n} of 5` });

    // Previous from slide 1 wraps to slide 5.
    await previous.click();
    await expect(slide(5)).toHaveAttribute("data-selected", "true");
    await expectSnappedToBoundary(content, slide(5));

    // Next from slide 5 wraps back to slide 1 -- and, unlike every other
    // (animated) transition (see the "actually animates" describe block's
    // own identically-shaped test), does so as an INSTANT jump: sampling
    // `scrollLeft` across the transition must never pass through an
    // intermediate value, only the start and end positions.
    const contentId = (await content.getAttribute("id"))!;
    const samples = await sampleScrollDuring(page, contentId, "scrollLeft", () => next.click());
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    await expectSnappedToBoundary(content, slide(1));
    expect(
      passesThroughAnIntermediateValue(samples),
      `expected an instant jump (no intermediate scrollLeft between the first (${samples[0]}) and last (${samples[samples.length - 1]}) sample); got: ${JSON.stringify(samples)}`,
    ).toBe(false);

    // Root-level ArrowRight at the last slide also wraps.
    for (let i = 0; i < 4; i++) {
      await next.click();
    }
    await expect(slide(5)).toHaveAttribute("data-selected", "true");
    await next.focus();
    await page.keyboard.press("ArrowRight");
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("under RTL, loop_mode Rewind wraps both directions with the swapped arrow keys", async ({ page }) => {
    await goto(page, "rewind");
    const frame = demoFrame(page, "rewind");
    const region = frame.getByRole("region", { name: "Rewind-loop gallery (RTL)", exact: true });
    const previous = region.getByRole("button", { name: /previous/i });
    const next = region.getByRole("button", { name: /next/i });
    const slide = (n: number) => region.getByRole("group", { name: `${n} of 4` });

    await expect(previous).toBeEnabled();
    await expect(next).toBeEnabled();

    // Previous from slide 1 wraps to slide 4.
    await previous.click();
    await expect(slide(4)).toHaveAttribute("data-selected", "true");
    await expect(previous).toBeEnabled();

    // Next from slide 4 wraps back to slide 1.
    await next.click();
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    // RTL root keyboard: ArrowLeft means next (swapped), so it should wrap
    // from the last slide back to the first too.
    for (let i = 0; i < 3; i++) {
      await next.click();
    }
    await expect(slide(4)).toHaveAttribute("data-selected", "true");
    await next.focus();
    await page.keyboard.press("ArrowLeft");
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  /**
   * The true-end rule (the drag band follows the wheel band's shared
   * `dxIsTrueEnd` predicate): a looping carousel has no true end, so the drag
   * clamps at the physical edge with no band at all, and still never wraps.
   */
  test("dragging past a looping carousel's physical edge never rubber-bands and never wraps", async ({ page }) => {
    await goto(page, "rewind");
    const frame = demoFrame(page, "rewind");
    const region = frame.getByRole("region", { name: "Rewind-loop gallery", exact: true });
    const content = region.locator(".dx-carousel-content");
    const slide = (n: number) => region.getByRole("group", { name: `${n} of 5` });

    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    // A large rightward drag from slide 1 (nothing before it): no band, no
    // wrap to slide 5 -- `loop`/`loop_mode` only govern Previous/Next/the
    // root keyboard (this crate's own module doc). Sampled in-page every
    // frame for the whole drag (row 112), not read once after it.
    const samples = await sampleBandDuring(content, async () => {
      await dragBy(page, content, 250, 0);
    }, 300);
    expect(Math.max(0, ...samples.map(Math.abs)), "no band at a looping carousel's edge").toBe(0);
    expect(await readContentTransform(content)).toBe("");
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("axe: the rewind variant has no automatically detectable a11y issues", async ({ page }) => {
    await goto(page, "rewind");
    await expectNoAxeViolations(page, "carousel: rewind variant", {
      include: "#component-preview-frame-rewind",
    });
  });
});

/**
 * Autoplay + rotation control (APG "auto-rotating" carousel). The
 * `autoplay` variant's own `delay_ms: 1200` (see its own doc for why) --
 * tests below wait a bit past that, never past a second full delay, to
 * stay fast without being flaky.
 */
test.describe("Carousel: autoplay + rotation control", () => {
  test("the rotation control precedes Previous/Next/the slide content in document order, and toggles its own label", async ({ page }) => {
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const rotation = frame.getByRole("button", { name: /automatic slide show/i });
    const previous = frame.getByRole("button", { name: "Previous slide" });
    const next = frame.getByRole("button", { name: "Next slide" });
    const firstSlide = frame.getByRole("group", { name: "1 of 5" });

    await expect(rotation).toHaveAttribute("aria-label", "Stop automatic slide show");
    await expect(rotation).not.toHaveAttribute("aria-pressed");

    // Document order, matching the R4 oracle rule's own construction
    // (`playwright/oracle/tier1-apg/carousel.spec.ts`'s `precedes`) --
    // APG's own requirement that the rotation control precede everything
    // else focusable in the carousel.
    const precedes = async (a: Locator, b: Locator) => {
      const bHandle = await b.elementHandle();
      return a.evaluate(
        (elA, elB) => !!(elA.compareDocumentPosition(elB as Node) & Node.DOCUMENT_POSITION_FOLLOWING),
        bHandle,
      );
    };
    expect(await precedes(rotation, previous)).toBe(true);
    expect(await precedes(rotation, next)).toBe(true);
    expect(await precedes(rotation, firstSlide)).toBe(true);

    await rotation.click();
    await expect(rotation).toHaveAttribute("aria-label", "Start automatic slide show");
    await rotation.click();
    await expect(rotation).toHaveAttribute("aria-label", "Stop automatic slide show");
  });

  test("aria-live on the slides container toggles with rotation state", async ({ page }) => {
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const content = frame.locator(".dx-carousel-content");
    const rotation = frame.getByRole("button", { name: /automatic slide show/i });

    await expect(content).toHaveAttribute("aria-live", "off");
    await rotation.click();
    await expect(content).toHaveAttribute("aria-live", "polite");
    await rotation.click();
    // Clicking the rotation button leaves the pointer hovering the
    // carousel (the button is inside it) -- and hover legitimately pauses
    // rotation regardless of `playing` (this component's own doc
    // deliberately does NOT port the vendored reference's own
    // "ignore hover/focus once explicitly started" quirk, the same class
    // of bug already flagged and left unported for the basic reference's
    // own `hasUserActivatedPlay` latch, `dev-docs/archive/research/carousel-2026-09-19.md`
    // §1.3 point 3) -- so the pointer must move away first to observe the
    // live region reflect `playing` again.
    await page.mouse.move(5, 5);
    await expect(content).toHaveAttribute("aria-live", "off");
  });

  test("autoplay advances the slide on its own", async ({ page }) => {
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    // Relative, not absolute: under parallel load, hydration can take longer
    // than one autoplay interval, so the carousel may already have left
    // slide 1 by the time the page is ready. Whatever slide is
    //     // selected now, a different one must become selected on its own.
    const selectedLabel = () =>
      frame.locator('[role="group"][aria-roledescription="slide"][data-selected="true"]').first().getAttribute("aria-label");
    await expect.poll(selectedLabel).not.toBeNull();
    const before = await selectedLabel();
    await expect.poll(selectedLabel, { timeout: 3000 }).not.toBe(before);
  });

  test("keyboard focus entering the carousel stops rotation, and it does not resume on its own", async ({ page }) => {
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const content = frame.locator(".dx-carousel-content");
    const rotation = frame.getByRole("button", { name: /automatic slide show/i });

    // No per-call-site guard needed here (or in the hover test below):
    // Smooth scrolling is opt-in (`html[data-anchor-scroll]`,
    // dev-docs/backlog.md rows 110 and 228), so Playwright's own
    // pre-`.focus()`/`.hover()` actionability scroll
    // (an unqualified scroll, which defers to that ancestor CSS) is
    // instant -- it cannot animate out from under the cursor.
    await content.focus();
    // The rotation control's own label reflects the user's *intent*
    // (`playing`), unaffected by an ambient, temporary pause -- only the
    // live region (`rotating`) does. It stays "Stop automatic slide show"
    // throughout this whole test; asserted once here as the baseline.
    await expect(rotation).toHaveAttribute("aria-label", "Stop automatic slide show");
    await expect(content).toHaveAttribute("aria-live", "polite");

    const selectedAt = async () =>
      frame.locator('[role="group"][data-selected="true"]').getAttribute("aria-label");
    const before = await selectedAt();
    // Wait past two full ticks -- focus is still inside the content
    // element, so this must never advance regardless of elapsed time.
    await page.waitForTimeout(2600);
    expect(await selectedAt()).toBe(before);

    // Losing focus alone does not resume it either (only the rotation
    // control does) -- move focus elsewhere on the page.
    await page.keyboard.press("Tab");
    await expect(content).toHaveAttribute("aria-live", "polite");
    await page.waitForTimeout(1600);
    expect(await selectedAt()).toBe(before);
  });

  test("hovering the carousel stops rotation, and moving away resumes it", async ({ page }) => {
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const content = frame.locator(".dx-carousel-content");
    const rotation = frame.getByRole("button", { name: /automatic slide show/i });

    // No per-call-site scroll guard (dev-docs/backlog.md rows 110, 228):
    // smooth scrolling is opt-in (`html[data-anchor-scroll]`), so Playwright's
    // own pre-`.hover()` scroll is instant and cannot animate the carousel out
    // from under the cursor (which would fire a genuine `mouseleave` and
    // resume the rotation).
    await content.hover();
    // The button's own label is unaffected (reflects intent, not the
    // ambient pause) -- only the live region does. See the focus test's
    // own identical note.
    await expect(rotation).toHaveAttribute("aria-label", "Stop automatic slide show");
    await expect(content).toHaveAttribute("aria-live", "polite");

    const selectedAt = async () =>
      frame.locator('[role="group"][data-selected="true"]').getAttribute("aria-label");
    const before = await selectedAt();
    await page.waitForTimeout(1600);
    expect(await selectedAt()).toBe(before);

    // Move the mouse well away from the carousel -- resumes on its own,
    // unlike the focus case above (the vendored tabbed reference's own
    // accessibility prose: "Automatic rotation resumes when the mouse
    // moves away ... unless another condition ... has been triggered").
    await page.mouse.move(5, 5);
    await expect(content).toHaveAttribute("aria-live", "off");
    await expect
      .poll(selectedAt, { timeout: 3000 })
      .not.toBe(before);
  });

  test("Previous/Next stop rotation for good (stopOnInteraction)", async ({ page }) => {
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const rotation = frame.getByRole("button", { name: /automatic slide show/i });
    const next = frame.getByRole("button", { name: "Next slide" });

    await expect(rotation).toHaveAttribute("aria-label", "Stop automatic slide show");
    await next.click();
    await expect(rotation).toHaveAttribute("aria-label", "Start automatic slide show");
  });

  test("prefers-reduced-motion: reduce never starts rotation at all", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const rotation = frame.getByRole("button", { name: /automatic slide show/i });
    const content = frame.locator(".dx-carousel-content");

    await expect(rotation).toHaveAttribute("aria-label", "Start automatic slide show");
    await expect(content).toHaveAttribute("aria-live", "polite");

    const selectedAt = async () =>
      frame.locator('[role="group"][data-selected="true"]').getAttribute("aria-label");
    const before = await selectedAt();
    await page.waitForTimeout(1600);
    expect(await selectedAt()).toBe(before);
  });

  test("axe: the autoplay variant has no automatically detectable a11y issues", async ({ page }) => {
    await goto(page, "autoplay");
    await expectNoAxeViolations(page, "carousel: autoplay variant", {
      include: "#component-preview-frame-autoplay",
    });
  });
});

/**
 * Indicators (tablist/dot-picker) variant -- APG's "tabbed" carousel
 * style, `CarouselIndicators`/`CarouselIndicator`. The `indicators`
 * variant has 5 slides, no separate Previous/Next (matching the vendored reference's own
 * structure). A custom `use_carousel()`-built dot picker lives on the `api`
 * demo instead.
 */
test.describe("Carousel: indicators (tablist/dot-picker) variant", () => {
  test("roles: tablist/tab/tabpanel, roving tabindex, aria-selected sync with the current slide", async ({ page }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const tablist = frame.getByRole("tablist");
    const tabs = frame.getByRole("tab");
    const slide = (n: number) => frame.locator(`[role="tabpanel"][aria-label="${n} of 5"]`);

    await expect(tablist).toBeVisible();
    await expect(tabs).toHaveCount(5);
    await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "true");
    await expect(tabs.nth(0)).toHaveAttribute("tabindex", "0");
    for (let i = 1; i < 5; i++) {
      await expect(tabs.nth(i)).toHaveAttribute("aria-selected", "false");
      await expect(tabs.nth(i)).toHaveAttribute("tabindex", "-1");
    }
    await expect(slide(1)).toBeVisible();
  });

  test("clicking a tab activates its slide and moves the roving tab stop", async ({ page }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const tabs = frame.getByRole("tab");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.locator(`[role="tabpanel"][aria-label="${n} of 5"]`);

    await tabs.nth(2).click();
    await expect(tabs.nth(2)).toHaveAttribute("aria-selected", "true");
    await expect(tabs.nth(2)).toHaveAttribute("tabindex", "0");
    await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "false");
    await expect(tabs.nth(0)).toHaveAttribute("tabindex", "-1");
    await expect(slide(3)).toHaveAttribute("data-selected", "true");
    await expectSnappedToBoundary(content, slide(3));
  });

  test("ArrowRight/ArrowLeft move focus and automatically activate the newly focused tab (no Enter needed)", async ({ page }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const tabs = frame.getByRole("tab");
    const slide = (n: number) => frame.locator(`[role="tabpanel"][aria-label="${n} of 5"]`);

    await tabs.nth(0).focus();
    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(1)).toBeFocused();
    await expect(tabs.nth(1)).toHaveAttribute("aria-selected", "true");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");

    await page.keyboard.press("ArrowLeft");
    await expect(tabs.nth(0)).toBeFocused();
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("ArrowLeft/ArrowRight wrap at both ends of the tablist", async ({ page }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const tabs = frame.getByRole("tab");

    await tabs.nth(0).focus();
    await page.keyboard.press("ArrowLeft");
    await expect(tabs.nth(4)).toBeFocused();
    await expect(tabs.nth(4)).toHaveAttribute("aria-selected", "true");

    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(0)).toBeFocused();
  });

  test("Home/End move focus to the first/last tab and activate it", async ({ page }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const tabs = frame.getByRole("tab");
    const slide = (n: number) => frame.locator(`[role="tabpanel"][aria-label="${n} of 5"]`);

    await tabs.nth(0).focus();
    await page.keyboard.press("End");
    await expect(tabs.nth(4)).toBeFocused();
    await expect(slide(5)).toHaveAttribute("data-selected", "true");

    await page.keyboard.press("Home");
    await expect(tabs.nth(0)).toBeFocused();
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("each tab's aria-controls points at its matching tabpanel's id", async ({ page }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const firstTab = frame.getByRole("tab").nth(0);
    const firstPanelId = await frame.locator('[role="tabpanel"]').nth(0).getAttribute("id");
    expect(firstPanelId).toBeTruthy();
    await expect(firstTab).toHaveAttribute("aria-controls", firstPanelId!);
  });

  test("axe: the indicators variant has no automatically detectable a11y issues", async ({ page }) => {
    await goto(page, "indicators");
    await expectNoAxeViolations(page, "carousel: indicators variant", {
      include: "#component-preview-frame-indicators",
    });
  });

  test("the active dot's opacity/width transition is present normally, and near-instant under prefers-reduced-motion", async ({
    page,
  }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const firstTab = frame.getByRole("tab").nth(0);

    const transitionDuration = await firstTab.evaluate(
      (el) => getComputedStyle(el).transitionDuration,
    );
    // Two comma-separated durations (opacity, width) -- style.css's own
    // `.dx-carousel-indicator` rule -- neither near-zero under the
    // default (no reduced-motion) preference.
    const durations = transitionDuration.split(",").map((s) => parseFloat(s));
    expect(durations.length).toBeGreaterThanOrEqual(2);
    for (const d of durations) {
      expect(d).toBeGreaterThan(0.05);
    }
  });

  test("the active dot's transition is near-instant under prefers-reduced-motion", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const firstTab = frame.getByRole("tab").nth(0);

    // dx-components-theme.css's own shared reduced-motion layer forces
    // `transition-duration` to `var(--dx-motion-duration-reduced)` (0.01ms)
    // `!important` on any `[role="tab"]` -- this button qualifies without
    // this component needing its own per-rule override (see
    // `.dx-carousel-indicator`'s own style.css comment).
    const transitionDuration = await firstTab.evaluate(
      (el) => getComputedStyle(el).transitionDuration,
    );
    const durations = transitionDuration.split(",").map((s) => parseFloat(s));
    for (const d of durations) {
      expect(d).toBeLessThan(0.01);
    }
  });
});
