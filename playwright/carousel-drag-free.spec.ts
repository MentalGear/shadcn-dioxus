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
      selected: slides.findIndex((s) => s.getAttribute("data-selected") === "true"),
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

/** A quick fling: `px` along the axis with no pauses (the release velocity is capped, so the coast length is bounded), released while still moving. `sign` is the physical direction. */
async function fling(page: Page, variant: Variant, px: number, sign: 1 | -1) {
  const track = frameOf(page, variant).locator(".dx-carousel-content");
  await track.scrollIntoViewIfNeeded();
  const box = (await track.boundingBox())!;
  const x0 = box.x + box.width / 2;
  const y = box.y + box.height / 2;
  await page.mouse.move(x0, y);
  await page.mouse.down();
  const steps = 8;
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(x0 + (sign * px * i) / steps, y);
  }
  await page.mouse.up();
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
      await fling(page, variant, px, toward as 1 | -1);
      // Coasting keeps moving after the pointer is up.
      const justAfter = (await slideOffsets(page, variant)).scroll;
      await settled(page, variant);
      const end = await slideOffsets(page, variant);
      expect(end.scroll).toBeGreaterThan(px + 80);
      expect(end.scroll).toBeGreaterThan(justAfter + 40);
    });

    test("a slow drag rests where it was let go, off every slide edge", async ({ page }) => {
      await goto(page, variant);
      const track = frameOf(page, variant).locator(".dx-carousel-content");
      await track.scrollIntoViewIfNeeded();
      const box = (await track.boundingBox())!;
      const x0 = box.x + box.width / 2;
      const y = box.y + box.height / 2;
      await page.mouse.move(x0, y);
      await page.mouse.down();
      await page.mouse.move(x0 + toward * 30, y, { steps: 3 });
      await page.mouse.move(x0 + toward * 55, y, { steps: 3 });
      await page.waitForTimeout(250); // held still: no release velocity
      await page.mouse.up();
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

    test("Next still lands exactly on a slide edge, from a free position", async ({ page }) => {
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
      await fling(page, variant, 140, toward as 1 | -1);
      const justAfter = (await slideOffsets(page, variant)).scroll;
      await page.waitForTimeout(600);
      const end = (await slideOffsets(page, variant)).scroll;
      expect(Math.abs(end - justAfter)).toBeLessThan(2);
    });
  });
}

test("a normal carousel still snaps (control: drag_free is off by default)", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/carousel/`, GOTO_OPTS);
  const track = page.locator("#component-preview-frame-sizes .dx-carousel-content");
  await expect(track).toHaveAttribute("data-drag-free", "false");
  await expect(track).toHaveAttribute("data-snap", "x mandatory");
});
