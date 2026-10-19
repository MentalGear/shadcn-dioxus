/**
 * Carousel edge fade (`class: "dx-carousel-fade"` on `Carousel`, the
 * `edge_fade` demo): a scroll-driven mask on the stationary viewport whose
 * two stop lengths (`--dx-carousel-fade-start` / `--dx-carousel-fade-end`)
 * follow the track's scroll position, so each fade is full length mid-scroll
 * and zero at its own end.
 *
 * SCOPING: every variant renders on the same page, so locators are rooted in
 * the variant's preview frame. RTL and vertical are checked by adding the
 * same class to the existing `rtl` / `vertical` demos (one carousel per demo,
 * see carousel-nav-opacity.spec.ts). Only computed style is asserted (no
 * timing), and the track is scrolled by `scrollLeft`/`scrollTop`, at rest.
 */

import { test, expect } from "./fixtures";
import { type Page, type Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

async function open(page: Page, variant: string): Promise<Locator> {
  await gotoHydrated(page, `${BASE_URL}/component/carousel/`, GOTO_OPTS);
  const id = variant === "main" ? "component-preview-frame" : `component-preview-frame-${variant}`;
  const root = page.locator(`#${id} .dx-carousel`).first();
  await root.scrollIntoViewIfNeeded();
  return root;
}

/** Fade lengths in px and the mask direction, read from the viewport. */
async function fades(root: Locator) {
  return root.evaluate((el) => {
    const vp = el.querySelector('[data-slot="carousel-viewport"]') as HTMLElement;
    const cs = getComputedStyle(vp);
    return {
      start: parseFloat(cs.getPropertyValue("--dx-carousel-fade-start")),
      end: parseFloat(cs.getPropertyValue("--dx-carousel-fade-end")),
      dir: cs.getPropertyValue("--dx-carousel-fade-dir").trim(),
      mask: cs.maskImage || cs.webkitMaskImage,
    };
  });
}

/** Scroll the track to a fraction of its range (0 = start, 1 = end). */
async function scrollTo(root: Locator, fraction: number, vertical = false) {
  await root.locator(".dx-carousel-content").evaluate(
    (el, [f, v]) => {
      const t = el as HTMLElement;
      const max = v ? t.scrollHeight - t.clientHeight : t.scrollWidth - t.clientWidth;
      const rtl = getComputedStyle(t).direction === "rtl" && !v;
      const pos = max * (f as number) * (rtl ? -1 : 1);
      if (v) t.scrollTop = pos;
      else t.scrollLeft = pos;
    },
    [fraction, vertical] as const,
  );
}

test.describe("carousel edge fade", () => {
  test("fade on both edges mid-scroll, none on the leading edge at the start", async ({ page }) => {
    const root = await open(page, "edge_fade");

    await scrollTo(root, 0);
    await expect.poll(async () => (await fades(root)).end).toBeGreaterThan(40);
    let f = await fades(root);
    expect(f.start).toBe(0);
    const size = f.end;
    expect(f.mask).toContain("linear-gradient");

    await scrollTo(root, 0.5);
    await expect.poll(async () => (await fades(root)).start).toBeGreaterThan(40);
    f = await fades(root);
    expect(f.start).toBeCloseTo(size, 0);
    expect(f.end).toBeCloseTo(size, 0);

    await scrollTo(root, 1);
    await expect.poll(async () => (await fades(root)).end).toBe(0);
    f = await fades(root);
    expect(f.start).toBeCloseTo(size, 0);
  });

  test("RTL: the start fade is on the right (gradient runs to the left)", async ({ page }) => {
    const root = await open(page, "rtl");
    await root.evaluate((el) => el.classList.add("dx-carousel-fade"));
    await scrollTo(root, 0.5);
    await expect.poll(async () => (await fades(root)).start).toBeGreaterThan(40);
    const f = await fades(root);
    expect(f.dir).toBe("to left");
    expect(f.end).toBeGreaterThan(40);
    await scrollTo(root, 0);
    await expect.poll(async () => (await fades(root)).start).toBe(0);
  });

  test("vertical: the fade runs on the block axis", async ({ page }) => {
    const root = await open(page, "vertical");
    await root.evaluate((el) => el.classList.add("dx-carousel-fade"));
    await scrollTo(root, 0.5, true);
    await expect.poll(async () => (await fades(root)).start).toBeGreaterThan(40);
    const f = await fades(root);
    expect(f.dir).toBe("to bottom");
    expect(f.end).toBeGreaterThan(40);
    await scrollTo(root, 0, true);
    await expect.poll(async () => (await fades(root)).start).toBe(0);
  });
});
