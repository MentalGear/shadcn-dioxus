import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoFrame, demoRole } from "./demo-frame";
import { DEFAULT_OVERLAY_MS } from "./assert-backdrop-fade";

/**
 * The modal scrim under `prefers-reduced-motion: reduce` (backlog row 441).
 *
 * The scrim has two painters on the web arm: the wrapper `div` that carries `data-dx-scrim` (it
 * animates `dx-fade-in`/`dx-fade-out`, its own background is transparent) and the `<dialog>`'s
 * top-layer `::backdrop` (a `transition`, the scrim itself). Both must come up and go away without
 * their fade under reduced motion, and both must fade without the emulation (so the claim is not
 * vacuous: the same probe reads the full 200 ms without it).
 *
 * Read from computed style, never from geometry: a fade changes opacity, not the box, and a
 * `boundingBox()` read mid-animation would be the row-222 flake.
 */

type ScrimStyle = {
  wrapperAnimationName: string;
  wrapperAnimationDuration: string;
  wrapperOpacity: string;
  backdropTransitionDuration: string;
};

/** A CSS time (`0.2s`, `200ms`, `1e-05s`, `0.2s, 0.2s`) as milliseconds, one number per comma-separated entry. */
function timesMs(value: string): number[] {
  return value.split(",").map((part) => {
    const t = part.trim();
    const n = parseFloat(t);
    return t.endsWith("ms") ? n : n * 1000;
  });
}

/** The reduced-motion token resolves to 0.01 ms; anything at or under that is "no fade". */
const NO_FADE_MS = 0.011;

async function readScrim(page: Page): Promise<ScrimStyle> {
  return page.evaluate(() => {
    const wrapper = document.querySelector<HTMLElement>(
      '#component-preview-frame [data-dx-scrim][data-state="open"]',
    );
    const dialog = document.querySelector<HTMLDialogElement>("dialog[open]");
    if (!wrapper || !dialog) throw new Error("no open scrim wrapper or dialog");
    const w = getComputedStyle(wrapper);
    const b = getComputedStyle(dialog, "::backdrop");
    return {
      wrapperAnimationName: w.animationName,
      wrapperAnimationDuration: w.animationDuration,
      wrapperOpacity: w.opacity,
      backdropTransitionDuration: b.transitionDuration,
    };
  });
}

async function openDemoDialog(page: Page) {
  // The Dialog demo's trigger is the first button of the main demo ("Show Dialog").
  await demoRole(page, "button", 0).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(demoFrame(page).locator('[data-dx-scrim][data-state="open"]')).toHaveCount(1);
  return dialog;
}

test.describe("modal scrim motion", () => {
  test("under reduced motion the scrim appears and leaves without its fade", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await gotoHydrated(page, `${BASE_URL}/component/dialog/`, { timeout: 20 * 60 * 1000 });

    // Record the wrapper's computed animation as its data-state turns "closed", before the
    // dialog is removed. A MutationObserver reads the style at the moment of the change.
    await page.evaluate(() => {
      const wrapper = document.querySelector<HTMLElement>("#component-preview-frame [data-dx-scrim]");
      if (!wrapper) throw new Error("no scrim wrapper in the demo frame");
      const w = window as unknown as { __scrimClosed?: { name: string; duration: string } | null };
      w.__scrimClosed = null;
      new MutationObserver(() => {
        if (wrapper.dataset.state === "closed" && !w.__scrimClosed) {
          const cs = getComputedStyle(wrapper);
          w.__scrimClosed = { name: cs.animationName, duration: cs.animationDuration };
        }
      }).observe(wrapper, { attributes: true, attributeFilter: ["data-state"] });
    });

    await openDemoDialog(page);

    // The open scrim: no fade (the duration is the reduced token), the backdrop has no transition,
    // and the scrim is fully visible at rest (an `animation: none` alone would leave the base
    // `opacity: 0` in place, which is why the rule shortens the duration instead).
    const open = await readScrim(page);
    expect(timesMs(open.wrapperAnimationDuration)[0]).toBeLessThanOrEqual(NO_FADE_MS);
    for (const ms of timesMs(open.backdropTransitionDuration)) {
      expect(ms).toBeLessThanOrEqual(NO_FADE_MS);
    }
    await expect
      .poll(async () => (await readScrim(page)).wrapperOpacity)
      .toBe("1");

    // Escape closes the dialog; the closed scrim must not fade either.
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);
    const closed = await page.evaluate(
      () => (window as unknown as { __scrimClosed?: { name: string; duration: string } | null }).__scrimClosed,
    );
    expect(closed, "the scrim's closed state was never observed").not.toBeNull();
    expect(timesMs(closed!.duration)[0]).toBeLessThanOrEqual(NO_FADE_MS);
  });

  test("without the emulation the same scrim fades for the full overlay duration", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await gotoHydrated(page, `${BASE_URL}/component/dialog/`, { timeout: 20 * 60 * 1000 });
    await openDemoDialog(page);

    const open = await readScrim(page);
    expect(open.wrapperAnimationName).toContain("dx-fade-in");
    expect(timesMs(open.wrapperAnimationDuration)[0]).toBeCloseTo(DEFAULT_OVERLAY_MS, 3);
    for (const ms of timesMs(open.backdropTransitionDuration)) {
      expect(ms).toBeCloseTo(DEFAULT_OVERLAY_MS, 3);
    }
  });
});
