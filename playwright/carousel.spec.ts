/**
 * Carousel: smoke + a11y attributes + keyboard paging (LTR and RTL) +
 * end-of-range button state + scroll-snap landing, across every shipped
 * variant (main, sizes, peek, vertical, rtl) that has ordinary
 * Previous/Next + role=group slides -- `indicators` (the APG tablist
 * style) has neither, so it is covered by its own dedicated describe
 * block instead of this generic sweep.
 *
 * SCOPING: every variant of a "Normal"-kind component renders on the same
 * page at once (`ComponentVariantHighlight` in `preview/src/main.rs`), so
 * every locator below is rooted through `demoFrame()` -- the same
 * `#component-preview-frame`/`#component-preview-frame-<variant>`
 * construction `oracle/tier3-radix/rtl.spec.ts` already established for
 * this exact class of collision (see that file's own header doc). Each
 * variant's `Carousel` also carries its own, non-colliding `aria-label`
 * (`preview/src/components/carousel/variants/<name>/mod.rs`) as a second,
 * defense-in-depth layer -- both matter: even with unique region names,
 * every variant's `CarouselPrevious`/`CarouselNext` share the exact same
 * default "Previous slide"/"Next slide" labels by design, so scoping is
 * what actually disambiguates a role+name query across variants.
 */

import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import {
  GOTO_OPTS,
  goto,
  demoFrame,
  expectSnappedToBoundary,
  dragBy,
  slidePitch,
  sampleScrollDuring,
  passesThroughAnIntermediateValue,
} from "./carousel-helpers";

for (const variant of ["main", "sizes", "peek", "vertical", "rtl"] as const) {
  test.describe(`Carousel (${variant} variant): smoke + a11y attributes`, () => {
    test(`region has role=region, aria-roledescription=carousel, and an accessible name that doesn't contain the word "carousel"`, async ({ page }) => {
      await goto(page, variant);
      const frame = demoFrame(page, variant);
      const region = frame.getByRole("region");
      await expect(region).toHaveAttribute("aria-roledescription", "carousel");
      const name = await region.evaluate((el) => el.getAttribute("aria-label") ?? "");
      expect(name.length).toBeGreaterThan(0);
      expect(name.toLowerCase()).not.toContain("carousel");
    });

    test('first slide has role=group, aria-roledescription=slide, and a "N of M" accessible name', async ({ page }) => {
      await goto(page, variant);
      const frame = demoFrame(page, variant);
      const firstSlide = frame.locator('[role="group"][aria-roledescription="slide"]').first();
      await expect(firstSlide).toHaveAttribute("aria-label", /^1 of \d+$/);
    });

    test("Previous/Next are native buttons with real accessible names, and activating them never moves focus off itself unexpectedly", async ({ page }) => {
      await goto(page, variant);
      const frame = demoFrame(page, variant);
      const previous = frame.getByRole("button", { name: "Previous slide" });
      const next = frame.getByRole("button", { name: "Next slide" });
      await expect(previous).toHaveJSProperty("tagName", "BUTTON");
      await expect(next).toHaveJSProperty("tagName", "BUTTON");

      await next.click();
      // Native <button> semantics: activating it keeps focus on itself,
      // it is never yanked elsewhere.
      await expect(next).toBeFocused();
    });

    test("Previous is disabled at the first slide", async ({ page }) => {
      await goto(page, variant);
      const frame = demoFrame(page, variant);
      await expect(frame.getByRole("button", { name: "Previous slide" })).toBeDisabled();
    });
  });
}

/**
 * shadcn-parity geometry: 28x28px buttons, no box-shadow, and visually
 * outside the track -- never overlapping a slide -- on every variant and
 * axis. Values are shadcn/ui's own live-measured carousel
 * (`ui.shadcn.com/docs/components/carousel`, JS running, 1280x800 @2x):
 * `getBoundingClientRect()` on both buttons gives exactly 28x28,
 * `getComputedStyle().boxShadow` gives "none", and the near edge of each
 * button sits 20px clear of the track's own edge (`--dx-space-12`, 48px,
 * minus the button's own 28px). Their own vertical ("Orientation") demo
 * shows the same 20px-clear rule on the block axis too, not just
 * inferred from the horizontal case -- see
 * `preview/src/components/carousel/style.css`'s own comments for the full
 * derivation and the exact numbers this asserts against.
 *
 * Clearance is measured against `.dx-carousel-content` (the track), NOT
 * `role=region` (`.dx-carousel` itself): Previous/Next sit in a
 * `padding-inline`/`padding-block` RESERVATION inside `.dx-carousel`
 * (not a negative `inset-inline-*`/`inset-block-*` outset past its edge), so
 * the buttons sit fully inside `.dx-carousel`'s own box (that containment is
 * what the next describe block tests) -- `region`'s box does not
 * mark the track's edge, only the track itself does.
 */
test.describe("Carousel: shadcn-parity geometry (size, shadow, outside placement)", () => {
  const BUTTON_SIZE_PX = 28;
  const MIN_CLEAR_PX = 20 - 1; // 1px tolerance for sub-pixel layout rounding

  for (const variant of ["main", "sizes", "peek", "vertical", "rtl"] as const) {
    test(`${variant}: Previous/Next are 28x28, shadow-less, 20px clear of the track`, async ({ page }) => {
      await goto(page, variant);
      const frame = demoFrame(page, variant);
      // The clipping viewport wrapper, NOT `.dx-carousel-content` itself:
      // the gap model (backlog row 91's shadcn-parity addendum) gives the
      // scroller its own negative `margin-inline-start`/`-block-start`
      // (`--dx-carousel-gap`'s own compensation), which deliberately
      // widens and shifts `.dx-carousel-content`'s own layout box by the
      // gap's width -- clipped by this wrapper, so nothing about it looks
      // different, but a raw `boundingBox()` of the scroller itself does
      // not mark the track's own VISUAL edge. The viewport wrapper's own box is
      // exactly the clipped, visual track edge, unaffected by the gap
      // model either way.
      const content = frame.locator('[data-slot="carousel-viewport"]');
      const previous = frame.getByRole("button", { name: "Previous slide" });
      const next = frame.getByRole("button", { name: "Next slide" });

      await expect(previous).toHaveCSS("box-shadow", "none");
      await expect(next).toHaveCSS("box-shadow", "none");

      const [contentBox, previousBox, nextBox] = await Promise.all([
        content.boundingBox(),
        previous.boundingBox(),
        next.boundingBox(),
      ]);
      expect(contentBox).not.toBeNull();
      expect(previousBox).not.toBeNull();
      expect(nextBox).not.toBeNull();

      for (const box of [previousBox!, nextBox!]) {
        expect(box.width).toBeCloseTo(BUTTON_SIZE_PX, 0);
        expect(box.height).toBeCloseTo(BUTTON_SIZE_PX, 0);
      }

      if (variant === "vertical") {
        // Block axis: Previous above the track, Next below it.
        expect(contentBox!.y - (previousBox!.y + previousBox!.height)).toBeGreaterThanOrEqual(MIN_CLEAR_PX);
        expect(nextBox!.y - (contentBox!.y + contentBox!.height)).toBeGreaterThanOrEqual(MIN_CLEAR_PX);
      } else if (variant === "rtl") {
        // Inline axis, mirrored: Previous (the *start* edge) resolves to
        // the physical right under dir="rtl"; Next (the *end* edge) to the
        // physical left -- the opposite pairing from every other variant.
        expect(previousBox!.x - (contentBox!.x + contentBox!.width)).toBeGreaterThanOrEqual(MIN_CLEAR_PX);
        expect(contentBox!.x - (nextBox!.x + nextBox!.width)).toBeGreaterThanOrEqual(MIN_CLEAR_PX);
      } else {
        expect(contentBox!.x - (previousBox!.x + previousBox!.width)).toBeGreaterThanOrEqual(MIN_CLEAR_PX);
        expect(nextBox!.x - (contentBox!.x + contentBox!.width)).toBeGreaterThanOrEqual(MIN_CLEAR_PX);
      }
    });
  }
});

/**
 * carousel-narrow: Previous/Next must never extend beyond
 * their own demo frame's box, at any viewport a real consumer's container
 * could actually be, in either theme. This is the direct, black-box
 * assertion for the failure of the `rtl` variant (both arrows clipped at the
 * frame edge on a narrow viewport) -- see
 * `preview/src/components/carousel/style.css`'s own comment on the
 * horizontal/vertical placement rules for the construction and the
 * geometric derivation.
 *
 * Viewports: six as the minimum
 * (1280/900/768/640/480/390) -- this page's own layout is not simply
 * narrower at a narrower viewport (a sidebar collapses around 700-750px,
 * so the rendered frame width is NOT monotonic in viewport width; 640px
 * actually yields a wider frame than 768px does) -- so this sweeps the
 * exact set rather than a hand-picked "worst case" that could
 * miss a non-adjacent regression the way a monotonic assumption would.
 *
 * `toBeGreaterThanOrEqual(-0.5)`/`toBeLessThanOrEqual(...+0.5)`: a half
 * pixel of slack for sub-pixel layout rounding, the same kind of tolerance
 * `MIN_CLEAR_PX` above already applies -- not a loose bound: a negative-outset
 * construction overflows by 12-15px on the horizontal variants at these
 * same viewports, tens of times this tolerance.
 */
test.describe("Carousel: arrows never overflow their frame", () => {
  const VIEWPORTS = [1280, 900, 768, 640, 480, 390];
  const OVERFLOW_TOLERANCE_PX = 0.5;

  for (const dark of [false, true]) {
    for (const width of VIEWPORTS) {
      test(`${dark ? "dark" : "light"} mode, ${width}px viewport: no button escapes its frame`, async ({
        page,
      }) => {
        await page.setViewportSize({ width, height: 900 });
        // `gotoHydrated`, not a bare `page.goto`: the frames' buttons only exist once the
        // page has hydrated, and the 3 s `toPass` below would race that under load (the
        // debug build with 4 workers; row 109's class).
        await gotoHydrated(
          page,
          `${BASE_URL}/component/carousel/${dark ? "?dark_mode=true" : ""}`,
          GOTO_OPTS,
        );

        for (const variant of ["main", "sizes", "peek", "vertical", "rtl"] as const) {
          const frame = demoFrame(page, variant);
          const previous = frame.getByRole("button", { name: "Previous slide" });
          const next = frame.getByRole("button", { name: "Next slide" });

          // Polls (via `toPass`, the same retry idiom `expectSnappedToBoundary`
          // above already uses for its own settle) rather than trusting one
          // instantaneous `boundingBox()` triple: this page mounts five
          // independent carousels at once, each running its own mount-time
          // settle effect (`primitives/src/carousel.rs`'s own
          // `CAROUSEL_SCROLL_INTO_VIEW_JS` effect), and under this suite's
          // parallel workers sharing one plain `http.server` (slower to
          // answer four workers' concurrent requests than one), a read
          // racing that settle can catch the frame and a button mid-shift,
          // tens of pixels apart, even at the roomiest viewport (this repo's own
          // dev-docs name "grepping/measuring before the page has
          // actually settled" as a false-alarm source for this
          // component). A real overflow does not self-correct on retry, so
          // this still fails (after 3s) if the buttons are genuinely
          // outside their frame.
          await expect(async () => {
            const [frameBox, previousBox, nextBox] = await Promise.all([
              frame.boundingBox(),
              previous.boundingBox(),
              next.boundingBox(),
            ]);
            expect(frameBox, `${variant}: frame box`).not.toBeNull();
            expect(previousBox, `${variant}: previous box`).not.toBeNull();
            expect(nextBox, `${variant}: next box`).not.toBeNull();

            for (const [label, box] of [
              ["previous", previousBox!],
              ["next", nextBox!],
            ] as const) {
              expect(box.x, `${variant} ${label}: left edge vs frame`).toBeGreaterThanOrEqual(
                frameBox!.x - OVERFLOW_TOLERANCE_PX,
              );
              expect(box.x + box.width, `${variant} ${label}: right edge vs frame`).toBeLessThanOrEqual(
                frameBox!.x + frameBox!.width + OVERFLOW_TOLERANCE_PX,
              );
              expect(box.y, `${variant} ${label}: top edge vs frame`).toBeGreaterThanOrEqual(
                frameBox!.y - OVERFLOW_TOLERANCE_PX,
              );
              expect(box.y + box.height, `${variant} ${label}: bottom edge vs frame`).toBeLessThanOrEqual(
                frameBox!.y + frameBox!.height + OVERFLOW_TOLERANCE_PX,
              );
            }
          }).toPass({ timeout: 3000 });
        }
      });
    }
  }
});

test.describe("Carousel: paging by button and scroll-snap landing", () => {
  test("Next/Previous page one slide at a time and land exactly on the slide boundary", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const next = frame.getByRole("button", { name: "Next slide" });
    const previous = frame.getByRole("button", { name: "Previous slide" });
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    await expectSnappedToBoundary(content, slide(1));

    await next.click();
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    await expectSnappedToBoundary(content, slide(2));

    await next.click();
    await expect(slide(3)).toHaveAttribute("data-selected", "true");
    await expectSnappedToBoundary(content, slide(3));

    await previous.click();
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    await expectSnappedToBoundary(content, slide(2));
  });

  test("Next becomes disabled at the last slide, and re-enables Previous away from the first", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const next = frame.getByRole("button", { name: "Next slide" });
    const previous = frame.getByRole("button", { name: "Previous slide" });

    await expect(previous).toBeDisabled();
    await expect(next).toBeEnabled();

    for (let i = 0; i < 4; i++) {
      await next.click();
    }

    await expect(next).toBeDisabled();
    await expect(previous).toBeEnabled();

    await previous.click();
    await expect(next).toBeEnabled();
  });
});

test.describe("Carousel: root-level keyboard paging (LTR)", () => {
  test("ArrowRight moves to the next slide, ArrowLeft moves back, focus stays put", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const next = frame.getByRole("button", { name: "Next slide" });
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    // Focus somewhere inside the carousel -- the root-level handler fires
    // for a keydown anywhere inside it, not only on the buttons.
    await next.focus();

    await page.keyboard.press("ArrowRight");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    await expect(next).toBeFocused();

    await page.keyboard.press("ArrowRight");
    await expect(slide(3)).toHaveAttribute("data-selected", "true");

    await page.keyboard.press("ArrowLeft");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
  });
});

test.describe("Carousel: root-level keyboard paging (RTL, swapped per Radix's convention)", () => {
  test("ArrowLeft moves to the next slide and ArrowRight moves back -- the opposite of LTR", async ({ page }) => {
    await goto(page, "rtl");
    const frame = demoFrame(page, "rtl");
    const next = frame.getByRole("button", { name: "Next slide" });
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });

    await next.focus();
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    await page.keyboard.press("ArrowLeft");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");

    await page.keyboard.press("ArrowLeft");
    await expect(slide(3)).toHaveAttribute("data-selected", "true");

    await page.keyboard.press("ArrowRight");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
  });

  test("Next/Previous accessible names are unaffected by direction (only the arrow-key mapping swaps)", async ({ page }) => {
    await goto(page, "rtl");
    const frame = demoFrame(page, "rtl");
    await expect(frame.getByRole("button", { name: "Previous slide" })).toBeVisible();
    await expect(frame.getByRole("button", { name: "Next slide" })).toBeVisible();
  });
});

test.describe("Carousel: vertical orientation pages on the block axis", () => {
  test("ArrowDown/ArrowUp page the vertical carousel, and it snaps on the block axis", async ({ page }) => {
    await goto(page, "vertical");
    const frame = demoFrame(page, "vertical");
    const content = frame.locator(".dx-carousel-content");
    const next = frame.getByRole("button", { name: "Next slide" });
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });

    await expectSnappedToBoundary(content, slide(1), "vertical");

    await next.focus();
    await page.keyboard.press("ArrowDown");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    await expectSnappedToBoundary(content, slide(2), "vertical");

    await page.keyboard.press("ArrowUp");
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });
});

test.describe("Carousel: a custom picker built on use_carousel()'s CarouselApi", () => {
  // Lives on the `api` demo; the `indicators` variant is the APG tablist demo
  // (`CarouselIndicators`/`CarouselIndicator`); see playwright/carousel.spec.ts's own
  // "tablist (dot-picker) variant" describe block for that one instead.
  test("clicking a dot indicator jumps directly to that slide, and the active dot tracks the selection", async ({ page }) => {
    await goto(page, "api");
    const frame = demoFrame(page, "api");
    const indicators = frame.locator('[aria-label="Slide picker"] button');
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    await expect(indicators).toHaveCount(5);
    await expect(indicators.nth(0)).toHaveAttribute("data-active", "true");

    await indicators.nth(2).click();
    await expect(slide(3)).toHaveAttribute("data-selected", "true");
    await expect(indicators.nth(2)).toHaveAttribute("data-active", "true");
    await expect(indicators.nth(0)).toHaveAttribute("data-active", "false");
  });

  test("the visible 'Slide n of m' counter tracks the selection", async ({ page }) => {
    await goto(page, "api");
    const frame = demoFrame(page, "api");
    const counter = frame.locator("p", { hasText: /^Slide \d+ of \d+$/ });

    await expect(counter).toHaveText("Slide 1 of 5");
    await frame.getByRole("button", { name: "Next slide" }).click();
    await expect(counter).toHaveText("Slide 2 of 5");
  });
});

test.describe("Carousel: light and dark mode render without error", () => {
  for (const dark of [false, true]) {
    test(`main variant loads in ${dark ? "dark" : "light"} mode`, async ({ page }) => {
      await gotoHydrated(page, 
        `${BASE_URL}/component/carousel/${dark ? "?dark_mode=true" : ""}`,
        GOTO_OPTS,
      );
      const frame = demoFrame(page, "main");
      await expect(frame.getByRole("region")).toBeVisible();
    });
  }
});

test.describe("Axe automated scan", () => {
  test("carousel component page (every variant mounted) has no automatically detectable a11y issues", async ({ page }) => {
    await goto(page, "main");
    await expectNoAxeViolations(page, "carousel: all variants", {
      excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT],
    });
  });
});

/**
 * The first paged transition (slide 1 to slide 2) animates like every later one -- see
 * `primitives/src/carousel.rs`'s mount-time effect doc for the construction. The failure it
 * guards: every click, the first included, sends `CAROUSEL_SCROLL_INTO_VIEW_JS`
 * `{instant: false, behavior: 'smooth'}` while the test browser reports
 * `prefers-reduced-motion: no-preference` throughout -- so this asserts the actual, sampled
 * scroll motion rather than "the code chose smooth", which a wording-only check could pass by
 * coincidence.
 */
test.describe("Carousel: the first paged transition actually animates", () => {
  test("clicking Next from slide 1 passes through an intermediate scroll position, not an instant jump", async ({ page }) => {
    await goto(page, "main");
    const reducedMotion = await page.evaluate(
      () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    );
    test.skip(
      reducedMotion,
      "test browser reports prefers-reduced-motion: reduce -- CAROUSEL_SCROLL_INTO_VIEW_JS deliberately forces an instant scroll in that case (WCAG 2.3.3 territory), so whether the transition animates cannot be asked honestly in this environment",
    );

    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const contentId = (await content.getAttribute("id"))!;
    const next = frame.getByRole("button", { name: "Next slide" });

    const samples = await sampleScrollDuring(page, contentId, "scrollLeft", () => next.click());

    expect(
      passesThroughAnIntermediateValue(samples),
      `expected an intermediate scrollLeft strictly between the first (${samples[0]}) and last (${samples[samples.length - 1]}) sample; got: ${JSON.stringify(samples)}`,
    ).toBe(true);
  });
});

/**
 * Mouse/pen drag-to-scroll on the track, layered on the
 * same scroll-snap construction (see `primitives/src/carousel.rs`'s own
 * "Pointer drag" doc). Touch is deliberately never exercised here -- it
 * already scrolls natively and Playwright's own synthetic touch input
 * would not exercise this code path anyway (`CAROUSEL_DRAG_JS` explicitly
 * ignores `pointerType === 'touch'`).
 */
test.describe("Carousel: pointer drag (mouse/pen)", () => {
  test("a drag past the threshold advances the slide, and selected (buttons, N of M label) stays correct", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    await expect(frame.getByRole("button", { name: "Previous slide" })).toBeDisabled();

    const pitch = await slidePitch(slide(1));
    await dragBy(page, content, -pitch * 0.7, 0);

    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    // The exact same bridge a click/keypress uses (`use_carousel_scroll_tracking`)
    // is what a drag settles through too -- Previous's disabled state is
    // one of the things that would desync if a drag had its own, second
    // source of truth for the index instead.
    await expect(frame.getByRole("button", { name: "Previous slide" })).toBeEnabled();
  });

  test("a click inside a slide works when the pointer never crosses the drag threshold", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const button = frame.getByTestId("carousel-slide-button");
    await expect(button).toHaveAttribute("data-clicked", "false");

    await button.click();

    await expect(button).toHaveAttribute("data-clicked", "true");
  });

  test("a real drag starting on a slide's own button pages the carousel and suppresses that button's own click", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const button = frame.getByTestId("carousel-slide-button");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    // Pitch measured from the slide itself, not `button` -- `button` is a
    // small element *inside* slide 1, and its own box is far narrower than
    // the track's actual per-slide pitch (see `slidePitch`'s own doc). The
    // drag still *originates* on `button`, exercising the same "a drag
    // starting on interactive content still pages, and suppresses that
    // content's own click" behavior this test is for.
    const pitch = await slidePitch(slide(1));
    await dragBy(page, button, -pitch * 0.7, 0);

    await expectSnappedToBoundary(frame.locator(".dx-carousel-content"), slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    await expect(button).toHaveAttribute("data-clicked", "false");
  });

  /**
   * A drag that ends clamped against an edge comes to rest exactly on a slide,
   * so its release needs no settle scroll -- and every `scrollend` during the
   * drag is dropped while `data-dragging` is set. The tracking bridge
   * therefore treats the end of a drag as the end of a scroll and reports where
   * it rested: dragging from slide 2 hard past slide 1 must select slide 1
   * (not leave "2 of 5" at scrollLeft 0).
   */
  test("a drag from slide 2 hard past the first slide selects slide 1 (the drag's end reports where it rested)", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await frame.getByRole("button", { name: "Next slide" }).click();
    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");

    await dragBy(page, content, (await slidePitch(slide(2))) * 2.5, 0, 20);
    await expectSnappedToBoundary(content, slide(1));
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    await expect(frame.getByRole("button", { name: "Previous slide" })).toBeDisabled();
  });

  test("a short movement below the threshold does not page", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    await dragBy(page, content, -2, 0, 2);
    await page.waitForTimeout(300);

    await expect(slide(1)).toHaveAttribute("data-selected", "true");
  });

  test("drag works under dir=rtl -- the physical direction that means 'next' mirrors, matching the keyboard's own RTL swap", async ({ page }) => {
    await goto(page, "rtl");
    const frame = demoFrame(page, "rtl");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });

    // Rightward, not leftward -- see primitives/src/carousel.rs's own
    // "Pointer drag" doc for why this mirrors LTR (the same swap
    // `ArrowLeft`/`ArrowRight` already gets at the carousel root).
    const pitch = await slidePitch(slide(1));
    await dragBy(page, content, pitch * 0.7, 0);

    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
  });

  test("drag works in the vertical variant", async ({ page }) => {
    await goto(page, "vertical");
    const frame = demoFrame(page, "vertical");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 4` });

    const pitch = await slidePitch(slide(1), "vertical");
    await dragBy(page, content, 0, -pitch * 0.7);

    await expectSnappedToBoundary(content, slide(2), "vertical");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
  });
});

/**
 * A drag release settles on the nearest slide through the same `scrollIntoView` paging path
 * `CarouselPrevious`/`CarouselNext` use (see `primitives/src/carousel.rs`'s `CarouselContent` doc,
 * "Release settle" section, for the construction), so it animates. Restoring `scroll-snap-type`
 * and trusting Chromium's own re-snap would not: sampling `scrollLeft` across such a release
 * yields exactly two distinct values (the mid-drag position and the final slide), both landing
 * instantly, whatever the drag length.
 *
 * Sampling starts only once the drag has already stopped moving and is
 * sitting at a raw, unsnapped position -- `page.mouse.up()` is the only
 * action inside `sampleScrollDuring`'s own `trigger`, not the whole drag
 * (unlike the "pointer drag" describe block above). The drag's own
 * per-move `scrollBy` calls already produce many intermediate values on
 * their own, which would make an assertion over the *whole* gesture pass
 * even if the release itself were an instant jump -- isolating the
 * release is what exercises the settle path, the same way the "first
 * paged transition actually animates" describe block isolates a single
 * button click.
 */
test.describe("Carousel: a drag release actually animates to the nearest slide", () => {
  test("releasing a drag passes through several intermediate scrollLeft values, not an instant jump", async ({
    page,
  }) => {
    await goto(page, "main");
    const reducedMotion = await page.evaluate(
      () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    );
    test.skip(
      reducedMotion,
      "test browser reports prefers-reduced-motion: reduce -- the release deliberately forces an instant scroll in that case, so whether it animates cannot be asked honestly in this environment",
    );

    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const contentId = (await content.getAttribute("id"))!;
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    await content.scrollIntoViewIfNeeded();
    const box = await content.boundingBox();
    if (!box) {
      throw new Error("content has no bounding box");
    }
    const pitch = await slidePitch(slide(1));
    const startX = box.x + box.width / 2;
    const startY = box.y + box.height / 2;
    const dx = -pitch * 0.7;
    const steps = 12;

    // Drag most of the way to slide 2 but stop short of releasing, so
    // sampling begins from wherever the drag currently sits (see this
    // describe block's own header for why the release must be isolated).
    await page.mouse.move(startX, startY);
    await page.mouse.down();
    for (let i = 1; i <= steps; i++) {
      await page.mouse.move(startX + (dx * i) / steps, startY);
      await page.waitForTimeout(16);
    }

    const samples = await sampleScrollDuring(page, contentId, "scrollLeft", () => page.mouse.up());

    expect(
      passesThroughAnIntermediateValue(samples),
      `expected an intermediate scrollLeft strictly between the first (${samples[0]}) and last (${samples[samples.length - 1]}) sample; got: ${JSON.stringify(samples)}`,
    ).toBe(true);
    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
  });
});

/**
 * `scroll-snap-stop: always` (`primitives/src/carousel.rs`, a plain inline
 * `style` declaration on each slide, right alongside `scroll-snap-align`
 * -- see that file's own `CarouselContent` doc, "scroll-snap-stop"
 * section) requires this track to stop at the first snap position a
 * scroll operation would otherwise pass, rather than skipping over
 * several to settle on whichever is numerically nearest. Measured to
 * genuinely cap a browser-animated smooth scroll -- a caller's own
 * `scrollBy(..., {behavior: 'smooth'})`, and by the same CSS Scroll Snap
 * Spec language a native wheel/trackpad fling -- to one slide of travel
 * regardless of the requested distance, which is what this test exercises
 * directly.
 *
 * The distance matters. At 2.5 slide widths the assertion "lands on
 * slide 2" is a false green, because "settle on whichever snap point is
 * numerically nearest" (the un-capped behaviour, indistinguishable from
 * this property being entirely inert) *also* lands on slide 2 at that
 * distance. Three slide widths discriminates: uncapped, an exact
 * 3.0-pitch scroll has no rounding ambiguity and settles on slide 4;
 * capped, the scroll must stop at the very first snap position it would
 * otherwise pass, landing on slide 2 -- two different, separately
 * checkable predictions (see `CarouselContent`'s own doc for why an inert
 * `scroll-snap-stop` must fail this test).
 *
 * Deliberately **not** exercised here via this crate's own pointer-drag
 * gesture (`dragBy`): this crate's own drag path does *not* get this
 * guarantee, because it moves the track with many independently-instant,
 * unsnapped `scrollBy` calls while `scroll-snap-type` is suspended for the
 * gesture's own duration, and restoring that property afterward is a
 * fresh, static re-evaluation of an already-stationary position -- not a
 * "scrolling operation" this property constrains on this engine
 * (Chromium); a drag-driven version of this test settles 2 slides from the
 * origin, not 1. The test covers the mechanism the property actually
 * delivers (a smooth scroll). See `CarouselContent`'s own doc for the
 * full write-up of that gap.
 */
test.describe("Carousel: scroll-snap-stop caps a smooth scroll to one slide", () => {
  test("a smooth scroll covering three slide widths lands exactly one slide away, not three", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    const pitch = await slidePitch(slide(1));
    const contentId = await content.getAttribute("id");
    // 3 slides' worth -- see this describe block's own header for why this
    // distance is the one that discriminates a working `scroll-snap-stop`
    // from an inert one.
    await page.evaluate(
      ({ id, distance }) => {
        document.getElementById(id)!.scrollBy({ left: distance, behavior: "smooth" });
      },
      { id: contentId, distance: pitch * 3 },
    );

    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    await expect(slide(3)).toHaveAttribute("data-selected", "false");
    await expect(slide(4)).toHaveAttribute("data-selected", "false");
  });
});


/**
 * True if `samples` pass strictly through a value between their own first
 * and last entry, on a scale-relative tolerance.
 *
 * Deliberately NOT `passesThroughAnIntermediateValue`: that one guards with
 * `hi - lo < 2` and bands with `lo + 1`/`hi - 1`, which are pixel
 * thresholds. Opacity's whole range is 0..1, so a 1 -> 0.5 fade has a span
 * of 0.5 and would fail that guard unconditionally -- the test would be red
 * no matter how well the fade worked. This one takes a minimum span and
 * pads by a fraction of the observed span instead.
 */
function passesThroughAnIntermediateFraction(samples: number[], minSpan = 0.05): boolean {
  if (samples.length < 3) {
    return false;
  }
  const start = samples[0];
  const end = samples[samples.length - 1];
  const lo = Math.min(start, end);
  const hi = Math.max(start, end);
  if (hi - lo < minSpan) {
    return false;
  }
  const pad = (hi - lo) * 0.1;
  return samples.some((v) => v > lo + pad && v < hi - pad);
}

/** Sample an element's computed opacity every frame across `trigger`. */
async function sampleOpacityDuring(
  page: Page,
  selector: string,
  trigger: () => Promise<void>,
  windowMs = 1500,
): Promise<number[]> {
  await page.evaluate(
    ({ selector, windowMs }) => {
      const w = window as unknown as { __dxOpacity: number[]; __dxOpacityDone: boolean };
      w.__dxOpacity = [];
      w.__dxOpacityDone = false;
      const el = document.querySelector(selector) as HTMLElement | null;
      if (!el) {
        w.__dxOpacityDone = true;
        return;
      }
      const t0 = performance.now();
      const tick = () => {
        w.__dxOpacity.push(parseFloat(getComputedStyle(el).opacity));
        if (performance.now() - t0 > windowMs) {
          w.__dxOpacityDone = true;
          return;
        }
        requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    },
    { selector, windowMs },
  );
  await trigger();
  await page.waitForFunction(
    () => (window as unknown as { __dxOpacityDone: boolean }).__dxOpacityDone === true,
    undefined,
    { timeout: windowMs + 5000 },
  );
  return page.evaluate(() => (window as unknown as { __dxOpacity: number[] }).__dxOpacity);
}

/**
 * The disabled-state fade: Previous/Next are
 * genuinely `disabled` at the first/last slide, and that opacity change must
 * not apply instantly. The transition lives on the BASE rule rather than
 * inside `:disabled` -- scoped to `:disabled` it would animate entering the
 * state but not leaving it, since the non-disabled style would then carry no
 * transition of its own. So both directions are asserted here.
 */
test.describe("Carousel: the disabled-state opacity change actually fades", () => {
  test("Next fades both entering and leaving :disabled", async ({ page }) => {
    await goto(page, "main");
    const reducedMotion = await page.evaluate(
      () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    );
    test.skip(
      reducedMotion,
      "test browser reports prefers-reduced-motion: reduce -- the shared theme layer deliberately suppresses this fade, so whether it animates cannot be asked honestly here",
    );

    const frame = demoFrame(page, "main");
    const next = frame.getByRole("button", { name: "Next slide" });
    const previous = frame.getByRole("button", { name: "Previous slide" });

    // Walk to the second-to-last slide so the next click disables Next.
    for (let i = 0; i < 3; i++) {
      await next.click();
      await page.waitForTimeout(700);
    }

    const disabling = await sampleOpacityDuring(page, ".dx-carousel-next", () => next.click());
    await expect(next).toBeDisabled();
    expect(
      passesThroughAnIntermediateFraction(disabling),
      `entering :disabled should pass through an intermediate opacity; got ${JSON.stringify(disabling)}`,
    ).toBe(true);

    await page.waitForTimeout(700);

    const enabling = await sampleOpacityDuring(page, ".dx-carousel-next", () => previous.click());
    await expect(next).toBeEnabled();
    expect(
      passesThroughAnIntermediateFraction(enabling),
      `leaving :disabled should pass through an intermediate opacity; got ${JSON.stringify(enabling)}`,
    ).toBe(true);
  });
});
