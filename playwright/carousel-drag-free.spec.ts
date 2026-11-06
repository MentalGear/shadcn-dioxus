/**
 * Carousel `drag_free` (Embla `dragFree` parity, backlog row 237): no snap,
 * a released mouse drag coasts, Previous/Next still page to a slide edge,
 * `selected` is the slide nearest the start edge, RTL mirrors, and reduced
 * motion drops the coast (the track rests where it was let go).
 */
import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { dragOnTimeline } from "./pointer-timeline";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

type Variant = "drag_free" | "drag_free_rtl";

async function goto(page: Page, variant: Variant) {
  await gotoHydrated(page, `${BASE_URL}/component/carousel/`, GOTO_OPTS);
}

const frameOf = (page: Page, variant: Variant) => page.locator(`#component-preview-frame-${variant}`);

/** Offset of every slide's start edge from the track's start edge (physical px, signed toward the content). */
async function slideOffsets(page: Page, variant: Variant): Promise<{ offs: number[]; selected: number; scroll: number }> {
  return page.evaluate((v) => {
    const frame = document.querySelector(`#component-preview-frame-${v}`)!;
    const track = frame.querySelector(".dx-carousel-content") as HTMLElement;
    const rtl = getComputedStyle(track).direction === "rtl";
    const c = track.getBoundingClientRect();
    const slides = [...track.querySelectorAll('[aria-roledescription="slide"]')] as HTMLElement[];
    const offs = slides.map((s) => {
      const r = s.getBoundingClientRect();
      return rtl ? c.right - r.right : r.left - c.left;
    });
    return {
      offs,
      selected: slides.findIndex((s) => s.getAttribute("data-selected") !== null),
      scroll: Math.abs(track.scrollLeft),
    };
  }, variant);
}

/** Wait until the scroll position stops changing. */
async function settled(page: Page, variant: Variant) {
  let prev = -1;
  let stable = 0;
  await expect
    .poll(async () => {
      const { scroll } = await slideOffsets(page, variant);
      stable = Math.abs(scroll - prev) < 0.5 ? stable + 1 : 0;
      prev = scroll;
      return stable;
    }, { timeout: 8000, intervals: [100] })
    .toBeGreaterThanOrEqual(5);
}

/**
 * Records the track's scroll position at the release (capture phase, before the carousel's own handler starts the
 * coast), so "it coasted" is judged against where the pointer let go, not against when a later read happened to run.
 */
async function recordRelease(page: Page, variant: Variant) {
  await page.evaluate((v) => {
    const track = document.querySelector(`#component-preview-frame-${v} .dx-carousel-content`) as HTMLElement;
    const w = window as unknown as { __atRelease: number | null };
    w.__atRelease = null;
    window.addEventListener("pointerup", () => (w.__atRelease = Math.abs(track.scrollLeft)), { capture: true, once: true });
  }, variant);
}
const scrollAtRelease = (page: Page) =>
  page.evaluate(() => (window as unknown as { __atRelease: number | null }).__atRelease);

/** The pointer is at the middle of the track; the gesture is `[{ x, y, atMs }]` after the press (pointer-timeline.ts). */
async function trackCentre(page: Page, variant: Variant) {
  const track = frameOf(page, variant).locator(".dx-carousel-content");
  await track.scrollIntoViewIfNeeded();
  const box = (await track.boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

/**
 * A quick fling: `px` along the axis in eight moves 12 ms apart, released while still moving. The browser stamps the
 * events with those times (not with when each call arrived), so on every machine the release velocity is
 * `px / 96` px/ms: 1.46 px/ms for the 140 px used here, a coast of about 470 px, well past a slide boundary and well
 * short of the end of the track. `sign` is the physical direction.
 */
async function fling(page: Page, variant: Variant, px: number, sign: 1 | -1) {
  const { x, y } = await trackCentre(page, variant);
  const steps = 8;
  await dragOnTimeline(
    page,
    { x, y },
    Array.from({ length: steps }, (_, k) => ({ x: x + (sign * px * (k + 1)) / steps, y, atMs: 12 * (k + 1) })),
  );
}

const minEdgeDistance = (offs: number[]) => Math.min(...offs.map((o) => Math.abs(o)));

for (const variant of ["drag_free", "drag_free_rtl"] as const) {
  // Content follows the pointer, so reaching later slides means dragging toward the start edge:
  // leftward in LTR, rightward in RTL.
  const toward = variant === "drag_free" ? -1 : 1;

  test.describe(`carousel drag_free (${variant})`, () => {
    test("the track carries one switch: data-drag-free and no scroll-snap", async ({ page }) => {
      await goto(page, variant);
      const track = frameOf(page, variant).locator(".dx-carousel-content");
      await expect(track).toHaveAttribute("data-drag-free", "true");
      await expect(track).toHaveAttribute("data-snap", "none");
      expect(await track.evaluate((e) => getComputedStyle(e).scrollSnapType)).toBe("none");
    });

    test("a fling coasts past a slide boundary and rests off every slide edge", async ({ page }) => {
      await goto(page, variant);
      const px = 140;
      await recordRelease(page, variant);
      await fling(page, variant, px, toward as 1 | -1);
      await settled(page, variant);
      const end = await slideOffsets(page, variant);
      // Coasting keeps moving after the pointer is up.
      expect(end.scroll).toBeGreaterThan(px + 80);
      expect(end.scroll).toBeGreaterThan((await scrollAtRelease(page))! + 40);
    });

    test("a slow drag rests where it was let go, off every slide edge", async ({ page }) => {
      await goto(page, variant);
      const { x, y } = await trackCentre(page, variant);
      // Held still for 250 ms before the release: no release velocity.
      await dragOnTimeline(
        page,
        { x, y },
        [
          { x: x + toward * 10, y, atMs: 30 },
          { x: x + toward * 20, y, atMs: 60 },
          { x: x + toward * 30, y, atMs: 90 },
          { x: x + toward * 55, y, atMs: 120 },
        ],
        250,
      );
      await settled(page, variant);
      const end = await slideOffsets(page, variant);
      expect(end.scroll).toBeGreaterThan(20);
      expect(minEdgeDistance(end.offs)).toBeGreaterThan(3);
    });

    test("selected tracks the slide nearest the start edge after a coast", async ({ page }) => {
      await goto(page, variant);
      await fling(page, variant, 140, toward as 1 | -1);
      await settled(page, variant);
      await expect
        .poll(async () => {
          const { offs, selected } = await slideOffsets(page, variant);
          const nearest = offs.reduce((bi, o, i) => (Math.abs(o) < Math.abs(offs[bi]) ? i : bi), 0);
          return selected === nearest;
        })
        .toBe(true);
      expect((await slideOffsets(page, variant)).selected).toBeGreaterThan(0);
    });

    test("Next lands exactly on a slide edge, from a free position", async ({ page }) => {
      await goto(page, variant);
      await fling(page, variant, 140, toward as 1 | -1);
      await settled(page, variant);
      const before = (await slideOffsets(page, variant)).selected;
      await frameOf(page, variant).getByRole("button", { name: /next/i }).click();
      await expect
        .poll(async () => {
          const { offs, selected } = await slideOffsets(page, variant);
          return selected === before + 1 && minEdgeDistance(offs) < 1.5;
        }, { timeout: 8000 })
        .toBe(true);
      await frameOf(page, variant).getByRole("button", { name: /previous/i }).click();
      await expect
        .poll(async () => {
          const { offs, selected } = await slideOffsets(page, variant);
          return selected === before && minEdgeDistance(offs) < 1.5;
        }, { timeout: 8000 })
        .toBe(true);
    });

    test("reduced motion: the released drag does not coast", async ({ page }) => {
      await page.emulateMedia({ reducedMotion: "reduce" });
      await goto(page, variant);
      await recordRelease(page, variant);
      await fling(page, variant, 140, toward as 1 | -1);
      await page.waitForTimeout(600);
      const end = (await slideOffsets(page, variant)).scroll;
      expect(Math.abs(end - (await scrollAtRelease(page))!)).toBeLessThan(2);
    });
  });
}

test("a normal carousel snaps (control: drag_free is off by default)", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/carousel/`, GOTO_OPTS);
  const track = page.locator("#component-preview-frame-sizes .dx-carousel-content");
  await expect(track).toHaveAttribute("data-drag-free", "false");
  await expect(track).toHaveAttribute("data-snap", "x mandatory");
});
