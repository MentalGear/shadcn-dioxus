/**
 * Carousel: paging never scrolls the page, and only visible slides are reachable (inert).
 * Split out of `carousel.spec.ts` (row 267).
 */

import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import {
  goto,
  demoFrame,
  scrollIntoViewInstant,
  expectSnappedToBoundary,
  dragBy,
  dragHold,
  axTreeGroupNames,
  slidePitch,
} from "./carousel-helpers";

/**
 * Paging never scrolls the page (autoplay must not scroll the main page). Every
 * paging path calls `scroller.scrollBy({ left | top: delta })` directly on the carousel's own
 * scroller element -- `scrollBy` only ever writes the element it is called on, so there is no
 * ancestor for it to reach. `target.scrollIntoView()` on the newly-selected slide would walk and
 * scroll *every* scrollable ancestor between the target and the viewport, including the page
 * itself, so a click, a keypress, a tab activation, a drag release or an unattended autoplay
 * tick would silently drag a reader back to the carousel, wherever they had scrolled to. See
 * `CAROUSEL_SCROLL_TO_JS`'s own doc in `carousel.rs` for the full construction. Every test below
 * fails if a paging path uses `scrollIntoView()`: `window.scrollY` moves by hundreds to
 * thousands of pixels in each case.
 *
 * Methodology notes specific to this describe block:
 * - The demo page has its own "jump to the requested `variant=`" scroll
 *   on load (an unrelated, expected feature), which
 *   may be `smooth` (`html[data-anchor-scroll] { scroll-behavior: smooth }`,
 *   `preview/assets/main.css`, backlog row 228) -- so every test below
 *   waits for `window.scrollY` to stop changing (`waitForScrollStable`)
 *   before doing anything else, then repositions the page with an
 *   *instant* `scrollTo` (never the default/`smooth` behavior, and never
 *   Playwright's own `.click()`/`.hover()` actionability pre-scroll,
 *   which also inherits the page's scroll-behavior and can still
 *   be mid-flight when the very next assertion reads `scrollY`).
 * - Buttons are paged with `dispatchEvent("click")`, not `.click()`, so
 *   Playwright's own pre-click `scrollIntoViewIfNeeded` never runs and
 *   never contributes a scroll of its own for this test to (correctly)
 *   ignore.
 * - The `tabs` variant's activation test focuses its target tab with
 *   `el.focus({ preventScroll: true })` rather than Playwright's plain
 *   `.focus()`. This is deliberate, not a shortcut: a *plain* `.focus()`
 *   on an off-screen tab still moves the page even for a correct
 *   carousel, because focusing any off-screen element is the browser's
 *   own default, spec-mandated behavior for `element.focus()` generally
 *   -- entirely independent of this component, present for every
 *   focusable element on the web, and not something a carousel's own code
 *   can (or should) suppress from the outside. `preventScroll: true` is
 *   exactly the tool the platform gives a caller to opt out of that
 *   default when it does its own scroll positioning, which is precisely
 *   this test's situation; it isolates the thing actually under test --
 *   `CarouselIndicator`'s own `onfocus` handler
 *   (`carousel_ctx.set_selected.call(...)`) -- from that unrelated native
 *   behavior: plain `.focus()` moves the page for any carousel, while
 *   `el.focus({ preventScroll: true })` moves it only for a
 *   `scrollIntoView()`-based one, the one signal that distinguishes this
 *   bug from that unrelated mechanism.
 */
test.describe("Carousel: paging never scrolls the page", () => {
  /** Poll `window.scrollY` until it stops changing -- see this describe
   * block's own "Methodology notes" above for why (the page's own
   * unrelated "jump to this variant" scroll on load). */
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

  test("several autoplay ticks never move window scroll position", async ({ page }) => {
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    const selectedLabel = async () =>
      frame.locator('[role="group"][data-selected="true"]').getAttribute("aria-label");

    await waitForScrollStable(page);
    // Instant, never the page's own default `smooth` behavior (see this
    // describe block's own "Methodology notes"). Scrolled to the carousel,
    // not to the top: autoplay only ticks while the carousel is on screen
    // (`primitives/src/activity.rs`), and the point of this test is that
    // the ticks leave a scroll position that has the carousel in view alone.
    await page.evaluate(() =>
      document
        .getElementById("component-preview-frame-autoplay")
        ?.scrollIntoView({ block: "center", behavior: "instant" }),
    );
    await waitForScrollStable(page);
    const scrollXBefore = await page.evaluate(() => window.scrollX);
    const scrollYBefore = await page.evaluate(() => window.scrollY);
    const before = await selectedLabel();

    // The `autoplay` variant's own `delay_ms: 1200` (this file's own
    // header note) -- comfortably past two full ticks.
    await page.waitForTimeout(1200 * 2 + 600);

    // The carousel did actually advance on its own -- otherwise a
    // never-firing timer would trivially pass this test for the wrong
    // reason.
    expect(await selectedLabel()).not.toBe(before);
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollYBefore);
    expect(await page.evaluate(() => window.scrollX)).toBe(scrollXBefore);
    // The slide the ticks landed on is still exactly snapped -- `scrollBy`
    // changes *how* the scroller pages, never where it lands.
    const selectedIndex = Number((await selectedLabel())!.split(" ")[0]);
    await expectSnappedToBoundary(frame.locator(".dx-carousel-content"), slide(selectedIndex));
  });

  test("a Next click and a Previous click never move window scroll position, with the page scrolled away", async ({
    page,
  }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const next = frame.getByRole("button", { name: "Next slide" });
    const previous = frame.getByRole("button", { name: "Previous slide" });
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });

    await waitForScrollStable(page);
    await page.evaluate(() => window.scrollTo({ top: 0, left: 0, behavior: "instant" }));
    const scrollYBefore = await page.evaluate(() => window.scrollY);
    const scrollXBefore = await page.evaluate(() => window.scrollX);

    // `dispatchEvent`, not `.click()` -- see this describe block's own
    // "Methodology notes" for why.
    await next.dispatchEvent("click");
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollYBefore);
    expect(await page.evaluate(() => window.scrollX)).toBe(scrollXBefore);
    await expectSnappedToBoundary(content, slide(2));

    await previous.dispatchEvent("click");
    await expect(slide(1)).toHaveAttribute("data-selected", "true");
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollYBefore);
    expect(await page.evaluate(() => window.scrollX)).toBe(scrollXBefore);
    await expectSnappedToBoundary(content, slide(1));
  });

  test("a CarouselIndicator activation never moves window scroll position (isolated from the browser's own focus-scroll)", async ({
    page,
  }) => {
    await goto(page, "indicators");
    const frame = demoFrame(page, "indicators");
    const tab = frame.getByRole("tab").nth(2);
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.locator(`[role="tabpanel"][aria-label="${n} of 5"]`);

    await waitForScrollStable(page);
    await page.evaluate(() => window.scrollTo({ top: 0, left: 0, behavior: "instant" }));
    const scrollYBefore = await page.evaluate(() => window.scrollY);
    const scrollXBefore = await page.evaluate(() => window.scrollX);

    // `{ preventScroll: true }` -- see this describe block's own
    // "Methodology notes" for why this, and not Playwright's plain
    // `.focus()`, is the correct isolation of the thing under test.
    await tab.evaluate((el) => (el as HTMLElement).focus({ preventScroll: true }));
    await expect(tab).toHaveAttribute("aria-selected", "true");
    await expect(slide(3)).toHaveAttribute("data-selected", "true");
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollYBefore);
    expect(await page.evaluate(() => window.scrollX)).toBe(scrollXBefore);
    await expectSnappedToBoundary(content, slide(3));
  });

  test("a pointer-drag release settle never moves window scroll position", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slide = (n: number) => frame.getByRole("group", { name: `${n} of 5` });
    await expect(slide(1)).toHaveAttribute("data-selected", "true");

    await waitForScrollStable(page);
    // Bring the carousel fully into view ourselves first, instantly --
    // `dragBy` needs real, stable viewport coordinates to drive
    // `page.mouse`, and would otherwise do this same positioning itself,
    // uninstrumented, the moment it is called (see this describe block's
    // own "Methodology notes"; `scrollIntoViewInstant`, this file's own
    // existing helper above, is the same technique). Establishing it here
    // means the baseline below is captured on an already-stable page, so
    // this test isolates the drag+release SETTLE itself
    // (`CAROUSEL_DRAG_JS`'s own `endDrag`) as the only thing that must not
    // move the page any further.
    await scrollIntoViewInstant(content);
    const scrollYBefore = await page.evaluate(() => window.scrollY);
    const scrollXBefore = await page.evaluate(() => window.scrollX);

    const pitch = await slidePitch(slide(1));
    await dragBy(page, content, -pitch * 0.7, 0);

    await expectSnappedToBoundary(content, slide(2));
    await expect(slide(2)).toHaveAttribute("data-selected", "true");
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollYBefore);
    expect(await page.evaluate(() => window.scrollX)).toBe(scrollXBefore);
  });
});

/**
 * The seamless-loop research's own a11y contract
 * (`dev-docs/archive/research/carousel-loop-2026-09-25/loop-a11y-guidance.md` §7,
 * cross-checked against every a11y-layer-having carousel library's own
 * converged construction in that same round's `dev-docs/archive/research/carousel-loop-2026-09-25/loop-libraries.md`): only
 * the current (or, for a multi-per-view layout, every slide actually
 * inside the viewport at rest) slide is reachable by Tab or exposed to the
 * accessibility tree -- everything else is `inert`
 * (`primitives/src/carousel/item.rs`'s own "Visible slides / `inert`" doc on
 * [`CarouselItem`]). `getByRole`/`ariaSnapshot` cannot verify the
 * accessibility-tree half of this (see `axTreeGroupNames`'s own doc,
 * above, for the discrepancy between Playwright's role engine and the platform tree).
 */
test.describe("Carousel: only visible slides are reachable (inert)", () => {
  test("non-current slides are inert on the main variant; the accessibility tree exposes only the current one", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const items = frame.locator(".dx-carousel-item");
    await expect(items).toHaveCount(5);

    await expect(items.nth(0)).not.toHaveAttribute("inert");
    for (let i = 1; i < 5; i++) {
      await expect(items.nth(i)).toHaveAttribute("inert");
    }

    const namesBefore = await axTreeGroupNames(page, "#component-preview-frame [role=region]");
    expect(namesBefore).toContain("1 of 5");
    for (let n = 2; n <= 5; n++) {
      expect(namesBefore).not.toContain(`${n} of 5`);
    }

    await frame.getByRole("button", { name: "Next slide" }).click();
    await expect(items.nth(0)).toHaveAttribute("inert");
    await expect(items.nth(1)).not.toHaveAttribute("inert");

    // Chrome's own accessibility tree recomputes asynchronously (a plain
    // CDP re-query right after the DOM mutation can still observe the
    // pre-mutation tree) -- `expect.poll` retries the whole scoped query
    // until it reflects the settled DOM, the same "poll rather than assume
    // instantaneous" discipline this file already applies to geometry
    // settling (`expectSnappedToBoundary`).
    await expect
      .poll(() => axTreeGroupNames(page, "#component-preview-frame [role=region]"))
      .toContain("2 of 5");
    const namesAfter = await axTreeGroupNames(page, "#component-preview-frame [role=region]");
    expect(namesAfter).not.toContain("1 of 5");
  });

  // The `rewind` demo is not used here: it renders TWO
  // independent `Carousel` regions on one page, so a single flat
  // `.dx-carousel-item` locator across both would see the RTL row's own
  // non-inert current slide land at an index this loop's "everything past
  // 0 is inert" assumption does not hold for -- the single-region
  // assumption below is real, not incidental.
  for (const variant of ["indicators", "autoplay"] as const) {
    test(`non-current slides are inert on the ${variant} variant too`, async ({ page }) => {
      await goto(page, variant);
      const frame = demoFrame(page, variant);
      const items = frame.locator(".dx-carousel-item");
      const count = await items.count();
      expect(count).toBeGreaterThan(1);

      await expect(items.nth(0)).not.toHaveAttribute("inert");
      for (let i = 1; i < count; i++) {
        await expect(items.nth(i)).toHaveAttribute("inert");
      }
    });
  }

  test("a multi-per-view layout widens the visible set to every slide actually in the viewport (sizes variant)", async ({ page }) => {
    await goto(page, "sizes");
    const frame = demoFrame(page, "sizes");
    const items = frame.locator(".dx-carousel-item");
    const count = await items.count();

    // Measured live, not assumed -- exactly how many WHOLE slides are
    // visible at once depends on `--dx-carousel-per-view` against however
    // wide this demo's own responsive wrapper renders (this file's own
    // established "measure, don't hardcode" convention, `slidePitch`'s own
    // doc) -- 2 below the demo's own `lg` breakpoint, 3 above it. The only
    // structural invariant asserted is that the non-inert set is a
    // contiguous prefix starting at slide 0 (this demo never scrolls on
    // its own before this point) and is more than just the one selected
    // slide -- proving the widening actually happened, not merely that the
    // single-slide case still works.
    const inertFlags: boolean[] = [];
    for (let i = 0; i < count; i++) {
      inertFlags.push(await items.nth(i).evaluate((el) => el.hasAttribute("inert")));
    }
    const firstInert = inertFlags.indexOf(true);
    expect(firstInert).toBeGreaterThan(1); // more than just slide 0 is visible
    expect(inertFlags.slice(0, firstInert).every((v) => v === false)).toBe(true);
    expect(inertFlags.slice(firstInert).every((v) => v === true)).toBe(true);

    const names = await axTreeGroupNames(page, "#component-preview-frame-sizes [role=region]");
    for (let i = 0; i < firstInert; i++) {
      expect(names).toContain(`${i + 1} of ${count}`);
    }
    for (let i = firstInert; i < count; i++) {
      expect(names).not.toContain(`${i + 1} of ${count}`);
    }
  });

  test("whole-slide geometry: each visible item's width is the viewport divided by --dx-carousel-per-view, within 1px, and slides land on boundaries (sizes variant)", async ({ page }) => {
    // Pins the whole-slide construction (backlog row 91): a real flex `gap` is additive to a
    // percentage `flex-basis`, so without the gap model N items would not sum to exactly the
    // viewport width (~2.3 slides visible). Checked against the live-rendered geometry, not just
    // the CSS source, so a change in the calc (or in the gap model it depends on)
    // is caught by measurement.
    await goto(page, "sizes");
    const frame = demoFrame(page, "sizes");
    const viewport = frame.locator('[data-slot="carousel-viewport"]');
    const content = frame.locator(".dx-carousel-content");
    const items = frame.locator(".dx-carousel-item");

    const viewportBox = await viewport.boundingBox();
    const contentBox = await content.boundingBox();
    expect(viewportBox).not.toBeNull();
    expect(contentBox).not.toBeNull();
    const perView = await frame
      .locator(".dx-carousel-demo-sizes")
      .evaluate((el) => getComputedStyle(el).getPropertyValue("--dx-carousel-per-view").trim());
    const n = Number(perView);
    expect(n).toBeGreaterThanOrEqual(2);

    // Each item's own basis is `calc(100% / n)` OF THE SCROLLER's own
    // (content) width, not the clipped viewport's -- the gap model's own
    // negative-margin compensation deliberately widens the scroller by
    // `--dx-carousel-gap`'s own width (`content_gap_margin`'s own doc),
    // clipped by the viewport wrapper, so `contentBox.width`, not
    // `viewportBox.width`, is the correct denominator here.
    for (let i = 0; i < n; i++) {
      const box = await items.nth(i).boundingBox();
      expect(box).not.toBeNull();
      expect(box!.width).toBeCloseTo(contentBox!.width / n, 0);
    }
    // The N slides still visually fill the clipped viewport exactly, with
    // no overflow: the last of the N items' own right edge lands flush
    // with the viewport's own right edge (both are pinned to the same
    // physical edge by construction -- content's own right edge is never
    // shifted, only its left edge is, so this holds regardless of gap).
    const lastVisibleBox = await items.nth(n - 1).boundingBox();
    expect(lastVisibleBox).not.toBeNull();
    expect(lastVisibleBox!.x + lastVisibleBox!.width).toBeCloseTo(
      viewportBox!.x + viewportBox!.width,
      0,
    );
    // The (n+1)-th slide (0-based index n) must not be even partially
    // inside the viewport -- a whole-slide layout has no partial peek.
    const nextBox = await items.nth(n).boundingBox();
    expect(nextBox).not.toBeNull();
    expect(nextBox!.x).toBeGreaterThanOrEqual(viewportBox!.x + viewportBox!.width - 1);
  });

  test("inert state is frozen while a drag is in progress -- it only updates on release/settle", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const items = frame.locator(".dx-carousel-item");
    const count = await items.count();

    const inertSnapshot = async () => {
      const flags: boolean[] = [];
      for (let i = 0; i < count; i++) {
        flags.push(await items.nth(i).evaluate((el) => el.hasAttribute("inert")));
      }
      return flags;
    };

    const before = await inertSnapshot();
    const pitch = await slidePitch(items.first());
    await dragHold(page, content, -pitch * 0.7, 0);

    // Sample several times while still holding -- none of these may differ
    // from the pre-drag baseline (freeze rule: `dev-docs/archive/research/carousel-loop-2026-09-25/loop-a11y-guidance.md` §7 /
    // `dev-docs/archive/research/carousel-overscroll-2026-09-23.md` §3/§6, the same "never write mid-
    // gesture" discipline every other scroll-position write in this module
    // already follows).
    for (let i = 0; i < 4; i++) {
      expect(await inertSnapshot()).toEqual(before);
      await page.waitForTimeout(60);
    }

    await page.mouse.up();
    // Now it may (and, since the drag crossed a slide boundary, should)
    // update.
    await expect(async () => {
      const after = await inertSnapshot();
      expect(after).not.toEqual(before);
    }).toPass({ timeout: 3000 });
  });

  test("focus is redirected to the carousel's own content region -- never lost to <body> -- when the focused slide leaves via the root keyboard handler", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slideButton = frame.getByTestId("carousel-slide-button");

    await slideButton.focus();
    await expect(slideButton).toBeFocused();

    await page.keyboard.press("ArrowRight");
    await expect(frame.getByRole("group", { name: "2 of 5" })).toHaveAttribute("data-selected", "true");

    const contentId = await content.getAttribute("id");
    await expect
      .poll(async () =>
        page.evaluate(() => ({
          isBody: document.activeElement === document.body,
          id: document.activeElement?.id ?? null,
        })),
      )
      .toEqual({ isBody: false, id: contentId });
  });

  test("focus is redirected -- not lost to <body> -- when the focused slide leaves via clicking Next", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slideButton = frame.getByTestId("carousel-slide-button");

    await slideButton.focus();
    await frame.getByRole("button", { name: "Next slide" }).click();
    await expect(frame.getByRole("group", { name: "2 of 5" })).toHaveAttribute("data-selected", "true");

    const isBody = await page.evaluate(() => document.activeElement === document.body);
    expect(isBody).toBe(false);
  });

  test("the focus redirect never scrolls the page (focus() scrolls ancestors unless preventScroll)", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const content = frame.locator(".dx-carousel-content");
    const slideButton = frame.getByTestId("carousel-slide-button");

    // Park the page so the content region's top edge is just off-screen: a
    // bare `focus()` on it would scroll the page to reveal it.
    let previous: number | null = null;
    for (let i = 0; i < 50; i++) {
      const y = await page.evaluate(() => window.scrollY);
      if (y === previous) break;
      previous = y;
      await page.waitForTimeout(150);
    }
    const contentId = await content.getAttribute("id");
    await page.evaluate((id) => {
      const top = document.getElementById(id!)!.getBoundingClientRect().top + window.scrollY;
      window.scrollTo({ top: top + 40, left: 0, behavior: "instant" });
    }, contentId);
    await slideButton.evaluate((el) => (el as HTMLElement).focus({ preventScroll: true }));
    await expect(slideButton).toBeFocused();
    const scrollYBefore = await page.evaluate(() => window.scrollY);

    await page.keyboard.press("ArrowRight");
    await expect(frame.getByRole("group", { name: "2 of 5" })).toHaveAttribute("data-selected", "true");
    await expect.poll(() => page.evaluate(() => document.activeElement?.id ?? null)).toBe(contentId);
    expect(await page.evaluate(() => window.scrollY)).toBe(scrollYBefore);
  });

  test("Tab from the last focusable in the current slide leaves the carousel -- no keyboard trap", async ({ page }) => {
    await goto(page, "main");
    const frame = demoFrame(page, "main");
    const region = frame.locator('[role="region"]');
    const slideButton = frame.getByTestId("carousel-slide-button");

    await slideButton.focus();
    await expect(slideButton).toBeFocused();
    await page.keyboard.press("Tab");

    const regionId = await region.getAttribute("id");
    const stillInside = await page.evaluate(
      ({ regionId }) => {
        const active = document.activeElement;
        const region = regionId
          ? document.getElementById(regionId)
          : document.querySelector('[role="region"][aria-label="Featured photos"]');
        return !!(region && active && region.contains(active));
      },
      { regionId },
    );
    expect(stillInside).toBe(false);
  });

  test("autoplay does not advance while focus is inside a slide (pause-on-focus invariant, holds with inert)", async ({ page }) => {
    // None of the shipped demo variants happen to put a focusable element
    // inside an autoplay carousel's own slide content, so a plain
    // `tabindex` is added to the CURRENT slide's own element for this test
    // only -- the point under test is the Rust-side `focus_within` gate
    // (`AutoplayContext`'s own doc), not any particular demo markup. It
    // must be the current slide: a non-current one is already `inert` at
    // page load (this describe block's own contract), and an inert
    // element refuses programmatic `.focus()` too, not just pointer/Tab --
    // so focusing anything else here would silently no-op and the test
    // would pass for the wrong reason.
    await goto(page, "autoplay");
    const frame = demoFrame(page, "autoplay");
    const slide1 = frame.getByRole("group", { name: "1 of 5" });
    await slide1.evaluate((el) => el.setAttribute("tabindex", "-1"));
    await slide1.evaluate((el) => (el as HTMLElement).focus());
    await expect(async () => {
      const focused = await page.evaluate(() => document.activeElement?.getAttribute("aria-label"));
      expect(focused).toBe("1 of 5");
    }).toPass({ timeout: 1000 });

    const selectedAt = async () =>
      frame.locator('[data-selected="true"]').getAttribute("aria-label");
    const before = await selectedAt();
    expect(before).toBe("1 of 5");
    await page.waitForTimeout(2600); // past two full 1200ms ticks
    expect(await selectedAt()).toBe(before);
  });

  test("a drag starting on a partially-visible neighbour slide pages the carousel (inert content is not a hit-test target)", async ({ page }) => {
    // `peek` is the explicitly opt-in home for a genuinely partially-visible
    // neighbour slide (`--dx-carousel-peek: 20%`); the `sizes` variant shows
    // only WHOLE slides by design (the shadcn-parity rule), so it has no
    // partially-visible neighbour to drag-start on at all.
    await goto(page, "peek");
    const frame = demoFrame(page, "peek");
    const content = frame.locator(".dx-carousel-content");
    const viewport = frame.locator('[data-slot="carousel-viewport"]');
    const items = frame.locator(".dx-carousel-item");
    const count = await items.count();

    let neighbourIndex = -1;
    for (let i = 0; i < count; i++) {
      if (await items.nth(i).evaluate((el) => el.hasAttribute("inert"))) {
        neighbourIndex = i;
        break;
      }
    }
    expect(neighbourIndex, "expected at least one inert (partially visible) neighbour").toBeGreaterThan(-1);

    const neighbour = items.nth(neighbourIndex);
    // Bring the demo into the *page's* own viewport first -- `page.mouse`
    // targets real screen coordinates, and this component sits far down
    // the long component-catalog page (`dragBy`'s own doc, above, hits the
    // identical problem). Scrolling the PAGE is a different scroll
    // container from the carousel's own internal `.dx-carousel-content`
    // track, so this cannot itself reveal the neighbour slide the way
    // `scrollIntoViewIfNeeded` on the *track* would -- it only moves which
    // part of the page is on screen, not which slide is scrolled into the
    // track's own viewport.
    await scrollIntoViewInstant(viewport);
    // A point inside BOTH the neighbour's own layout box and the clipping
    // viewport's -- guaranteed actually painted on screen (not clipped)
    // without ever calling `scrollIntoViewIfNeeded` on the neighbour itself
    // (which would scroll this very slide fully into the *track's* view,
    // defeating "partially visible").
    const [viewportBox, neighbourBox] = await Promise.all([viewport.boundingBox(), neighbour.boundingBox()]);
    expect(viewportBox).not.toBeNull();
    expect(neighbourBox).not.toBeNull();
    const x = Math.max(viewportBox!.x, neighbourBox!.x) + 2;
    const y = neighbourBox!.y + neighbourBox!.height / 2;
    expect(x).toBeLessThan(viewportBox!.x + viewportBox!.width);

    const selectedBefore = await frame.locator('[data-selected="true"]').getAttribute("aria-label");
    const pitch = await slidePitch(items.first());

    await page.mouse.move(x, y);
    await page.mouse.down();
    for (let i = 1; i <= 12; i++) {
      await page.mouse.move(x - (pitch * 0.7 * i) / 12, y);
      await page.waitForTimeout(16);
    }
    await page.mouse.up();

    await expect(async () => {
      const selectedAfter = await frame.locator('[data-selected="true"]').getAttribute("aria-label");
      expect(selectedAfter).not.toBe(selectedBefore);
    }).toPass({ timeout: 3000 });
  });

  test("axe: clean with non-visible slides inert (main variant)", async ({ page }) => {
    await goto(page, "main");
    await expectNoAxeViolations(page, "carousel: main variant with inert slides", {
      include: "#component-preview-frame",
    });
  });
});
