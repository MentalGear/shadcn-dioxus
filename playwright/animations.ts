import type { Locator } from "@playwright/test";

/** A box in CSS pixels relative to the main frame's viewport (what `Locator.boundingBox()` returns, never null). */
export type RestBox = { x: number; y: number; width: number; height: number };

/**
 * Resolves once `locator`'s element has been mounted AND every
 * animation/transition running on it (or its descendants) has finished --
 * the load-independent replacement for `page.waitForTimeout(<animation
 * duration * safety factor>)` before reading an animated value
 * (dev-docs/backlog.md row 115; same class as row 112's "asserting on an
 * animated value").
 *
 * WHY a fixed wait is wrong: a CSS animation's clock starts at the first
 * frame the page renders AFTER the element mounts, not at the click that
 * mounted it. On a starved main thread/compositor (the full suite at
 * `--workers=4` runs 4 Chromiums on 4 cores) that first frame can arrive
 * hundreds of ms late, so "200ms animation + 300ms slack" is not a
 * guarantee, only a probability. `drawer.spec.ts:110` read the panel's
 * bounding box mid-slide-in this way.
 *
 * Two rAFs first: the animation object does not exist until the style
 * recalc of the next frame, and "no running animations" is trivially true
 * before it does -- the very false-settled reading this exists to prevent.
 */
export async function awaitAnimationsSettled(locator: Locator): Promise<void> {
  await locator.waitFor({ state: "attached" });
  await locator.evaluate(async (el) => {
    const frame = () => new Promise<void>((r) => requestAnimationFrame(() => r()));
    await frame();
    await frame();
    const deadline = performance.now() + 10_000;
    for (;;) {
      // Animations that move this element's box: its own, its descendants' and its ANCESTORS' (a dialog's
      // enter scale resizes the footer inside it; reading the footer's box mid-animation is the same race as
      // reading the dialog's -- backlog row 222). An `infinite` animation (spinner, skeleton) never settles,
      // so it is not waited for: a spec that measures one reads it at the frame it happens to be on.
      const unsettled = document
        .getAnimations()
        .filter((a) => {
          if (a.playState !== "running" && (a.playState as string) !== "pending") return false;
          if (a.effect && a.effect.getComputedTiming().iterations === Infinity) return false;
          const t = a.effect && (a.effect as KeyframeEffect).target;
          return !!t && (t.contains(el) || el.contains(t));
        });
      if (unsettled.length === 0) return;
      if (performance.now() > deadline) {
        throw new Error(`animations still running after 10s: ${unsettled.map((a) => (a as CSSAnimation).animationName ?? a.id).join(", ")}`);
      }
      await Promise.all(unsettled.map((a) => a.finished.catch(() => undefined)));
      await frame();
    }
  });
}

/**
 * The ONE way a spec reads the geometry of something that animates on enter (backlog row 222): waits for
 * `awaitAnimationsSettled`, THEN reads `locator.boundingBox()`. `boundingBox()` includes transforms, so a read
 * during `dx-pop-in` (scale 0.95 -> 1), a sheet's slide or a dialog's 200 ms enter is a race against the
 * animation clock -- `select.spec.ts` read the listbox 142.16 px wide mid-animation against 145.89 at rest, and
 * `dialog.spec.ts:188` compared the footer band with the dialog during the enter (1 in 5 runs failed alone, by
 * 1.25-1.86 px). The gate `scripts/check-spec-at-rest.sh` fails a `boundingBox()` / `getBoundingClientRect()`
 * read in an overlay spec that does not go through this helper (or `readAtRest`).
 *
 * Throws when the element has no box (not rendered / `display: none`) instead of returning null, so a missing
 * overlay is a failure at the read, not a `TypeError` on `box!.width` three lines later.
 */
export async function measureAtRest(locator: Locator): Promise<RestBox> {
  await awaitAnimationsSettled(locator);
  const box = await locator.boundingBox();
  if (!box) throw new Error("measureAtRest: the element has no box (not rendered or display:none)");
  return box;
}

/**
 * `measureAtRest` for any in-page read -- `getBoundingClientRect()` of the element or of its children, a
 * computed size, `offsetWidth` -- taken only after the element's animations (and its ancestors') settled:
 * `readAtRest(panel, (el) => el.getBoundingClientRect().width)`.
 */
export async function readAtRest<R, A = undefined>(
  locator: Locator,
  read: (el: Element, arg: A) => R | Promise<R>,
  arg?: A,
): Promise<R> {
  await awaitAnimationsSettled(locator);
  return (locator.evaluate as (fn: unknown, arg?: unknown) => Promise<R>).call(locator, read, arg);
}
