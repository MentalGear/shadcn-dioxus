/**
 * Rule sources (dev-docs/conformance-harness.md tiers):
 *
 * - WHATWG `<dialog>` modal semantics (focus trap, `showModal()`/`close()`
 *   focus restore, inertness, top layer) are inherited unchanged from
 *   `primitives/src/dialog.rs` and already covered by
 *   `oracle/tier2-html/native-dialog.spec.ts` against the unmodified
 *   `Dialog` primitive `Drawer` composes -- not re-covered here.
 * - APG "Dialog (Modal)" Pattern (tier 2): Escape closes the dialog and
 *   focus returns to the element that invoked it. Exercised here because
 *   it's the *drawer's own* integration (the trigger button, the specific
 *   first-focusable content) that needs checking, not the primitive's own
 *   escape-key/focus-restore mechanics again.
 * - shadcn/ui Drawer (Vaul)'s drag-to-dismiss gesture -- distance ratio
 *   and release-velocity thresholds, rubber-band overdrag -- is tier 3
 *   "opinion": there is no APG/WHATWG rule for a pointer drag gesture,
 *   only this library's (and Vaul's) own convention. See
 *   `primitives/src/drawer.rs`'s `DISMISS_DRAG_RATIO`/
 *   `DISMISS_VELOCITY_PX_PER_MS` doc comments for the exact values these
 *   tests assume.
 */
import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from './axe';
import { BASE_URL } from './base-url';
import { gotoHydrated } from './hydration';
import { awaitAnimationsSettled, measureAtRest, readAtRest } from './animations';
import { fixedViewportBox } from './viewport-box';
import { assertSharedBackdrop, captureBackdrop, resolveOverlayMs } from "./assert-backdrop-fade";

const URL = `${BASE_URL}/component/drawer/`;
const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

// The demos' drawers take the default side, `EdgeSide::Auto`: the bottom below the `md` breakpoint (768px)
// and a floating right panel at and above it. The drag, handle and flush-edge tests below are about the
// BOTTOM drawer, so the file runs at a phone width; the `Auto` and explicit-side tests set their own.
const PHONE = { width: 390, height: 844 };
const DESKTOP = { width: 1280, height: 800 };
test.use({ viewport: PHONE });

// The entrance slide-in keyframe runs over `--dx-motion-duration-slow`
// (200ms, preview/assets/dx-components-theme.css) with `animation-fill-mode:
// forwards`; a geometry assertion taken before it finishes reads a mid-
// transform position, not the resting one (reading
// `getBoundingClientRect()` immediately after the trigger click, with no
// wait, shows the panel still at its pre-animation `translateY(100%)`
// position). Waited for with `awaitAnimationsSettled` (./animations.ts), NOT
// a fixed `waitForTimeout`: the animation's clock starts at the first frame
// rendered after mount, which under parallel load can be hundreds of ms
// after the click, so no fixed slack is a guarantee (dev-docs/backlog.md
// row 115, `flush` test).

// primitives/src/drawer.rs's own rubber-band constants for an overdrag PAST
// the fully-open resting position (dragging away from the dismiss
// direction). The construction that keeps that overdrag from ever exposing
// bare backdrop is two-part -- see preview/src/components/drawer/style.css's
// `[data-side]` rules' own comments -- so these tests check both halves
// rather than only the DOM box's own position:
//   1. the real, damped overdrag the box's own layout position can ever
//      reach is bounded by RUBBER_BAND_MAX_PX (a small, deliberate rubber
//      band -- a protected Rust unit test requires it be non-zero, so this
//      is NOT expected to be exactly 0), and
//   2. the CSS `filter: drop-shadow(...)` extension on that same side is
//      sized at least that large, so whatever sliver (1) allows is always
//      painted over with the panel's own background rather than exposing
//      the backdrop underneath.
const RUBBER_BAND_MAX_PX = 24;
// A little slack over the exact Rust constant for CI/animation-frame jitter
// in the boundingBox() sampling below (this is a bound, not an exact-value
// assertion).
const RUBBER_BAND_ASSERT_SLACK_PX = 6;
// Subpixel layout/rendering tolerance for an at-rest "flush with the edge"
// comparison (mirrors the +/-1 tolerance context-menu.spec.ts's own
// edge-clamping assertions use for the same reason).
const FLUSH_TOLERANCE_PX = 1;

test('opens and traps focus, Escape closes and restores focus to the trigger', async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  const trigger = page.getByRole('button', { name: 'Move Goal' });
  // Opened from the keyboard, so the first control takes focus; a pointer/touch open focuses the
  // panel instead (row 206, modal-initial-focus.spec.ts).
  await trigger.focus();
  await page.keyboard.press('Enter');

  const root = page.locator('[data-slot="drawer-root"]');
  await expect(root).toHaveAttribute('data-state', 'open');

  // First focusable element inside the content in tree order -- WHATWG
  // `showModal()`'s own autofocus algorithm. `DrawerHandle` is
  // `aria-hidden` and not focusable, so the "Decrease goal" icon button
  // (the drawer's own first real control) is the expected stop.
  await expect(page.getByRole('button', { name: 'Decrease goal' })).toBeFocused();

  // APG Dialog (Modal): Escape closes and returns focus to the invoker.
  await page.keyboard.press('Escape');
  await expect(root).toHaveCount(0);
  await expect(trigger).toBeFocused();
});

test('data-state transitions from closed to open to closed', async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  const root = page.locator('[data-slot="drawer-root"]');
  const content = page.locator('[data-slot="drawer-content"]');

  await expect(root).toHaveCount(0);

  await page.getByRole('button', { name: 'Move Goal' }).click();
  await expect(root).toHaveAttribute('data-state', 'open');
  await expect(content).toHaveAttribute('data-state', 'open');
  await expect(content).toHaveAttribute('data-side', 'bottom');

  await page.keyboard.press('Escape');
  await expect(root).toHaveCount(0);
});

test('opens from the top side with side-specific data attributes', async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await page.getByRole('button', { name: 'Open from Top' }).click();

  const content = page.locator('[data-slot="drawer-content"]');
  await expect(content).toHaveAttribute('data-side', 'top');
  await expect(content).toHaveAttribute('data-state', 'open');

  await page.keyboard.press('Escape');
  await expect(page.locator('[data-slot="drawer-root"]')).toHaveCount(0);
});

test('the panel sits flush against its own edge at rest, bottom and top variants', async ({ page }) => {
  // Regression: the web arm's `DrawerContent` is a real, modal `<dialog>`
  // (primitives/src/dialog.rs), and Chromium's own `dialog`/`dialog:modal`
  // UA rules (`margin: auto` + `inset-block: 0` with no `inset-inline` +
  // `width`/`height: fit-content`) would center it -- both axes -- instead of
  // anchoring it to any edge: the bottom variant's computed `margin` would
  // be `223px 496.594px` (equal top/bottom AND left/right margins) on an
  // 800x1280 viewport, i.e. a floating card, not a bottom sheet.
  await gotoHydrated(page, URL, GOTO_OPTS);
  const viewport = await fixedViewportBox(page);

  await page.getByRole('button', { name: 'Move Goal' }).click();
  await awaitAnimationsSettled(page.locator('[data-slot="drawer-content"]'));
  const bottomBox = await measureAtRest(page.locator('[data-slot="drawer-content"]'));
  if (!bottomBox) throw new Error('drawer content has no bounding box');
  // Flush with the viewport's bottom edge: no gap below.
  expect(Math.abs(viewport.height - (bottomBox.y + bottomBox.height))).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
  await page.keyboard.press('Escape');
  await expect(page.locator('[data-slot="drawer-root"]')).toHaveCount(0);

  await page.getByRole('button', { name: 'Open from Top' }).click();
  await awaitAnimationsSettled(page.locator('[data-slot="drawer-content"]'));
  const topBox = await measureAtRest(page.locator('[data-slot="drawer-content"]'));
  if (!topBox) throw new Error('drawer content has no bounding box');
  // Flush with the viewport's top edge: no gap above.
  expect(Math.abs(topBox.y)).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
});

test('a drag away from the dismiss direction never exposes the viewport edge', async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  const viewport = await fixedViewportBox(page);

  await page.getByRole('button', { name: 'Move Goal' }).click();
  await awaitAnimationsSettled(page.locator('[data-slot="drawer-content"]'));

  const content = page.locator('[data-slot="drawer-content"]');
  const handleBox = await measureAtRest(page.locator('[data-slot="drawer-handle"]'));
  if (!handleBox) throw new Error('drawer handle has no bounding box');
  const startX = handleBox.x + handleBox.width / 2;
  const startY = handleBox.y + handleBox.height / 2;

  // Bottom drawer: dragging UP is away from the dismiss direction (the
  // rubber-band overdrag case). `damped_display_offset` maps a raw offset
  // to `(raw * RUBBER_BAND_RATIO).max(-RUBBER_BAND_MAX_PX)` -- with
  // RATIO=0.25, the cap only actually engages past 96px of raw movement
  // (24 / 0.25), so 160px raw comfortably pins the damped display offset
  // at its -24px cap for the later samples, not still mid-ramp.
  const rawOverdragPx = 160;
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  let maxGapBelow = 0;
  for (let i = 1; i <= 6; i++) {
    await page.mouse.move(startX, startY - (rawOverdragPx * i) / 6, { steps: 2 });
    const box = await measureAtRest(content);
    if (box) maxGapBelow = Math.max(maxGapBelow, viewport.height - (box.y + box.height));
  }
  // Read while the pointer is still down, so a `getComputedStyle` cannot land on a
  // closed/unmounted panel (backlog row 115). This gesture ends 160px above the handle,
  // over the dialog's `::backdrop`; `use_dialog_backdrop_dismiss` must not read that as a
  // backdrop click that closes the drawer (the "dragging UP closes it" report) -- the
  // dedicated tests below pin that.
  const filterValue = await content.evaluate((el) => getComputedStyle(el).filter);
  await page.mouse.up();

  // Half 1: the real layout box never moves further than the documented,
  // deliberately small rubber-band cap.
  expect(maxGapBelow).toBeLessThanOrEqual(RUBBER_BAND_MAX_PX + RUBBER_BAND_ASSERT_SLACK_PX);

  // Half 2: the CSS extension that paints over exactly that sliver is
  // present and sized at least as large as the cap it must cover -- parsed
  // from the computed `filter`, not re-typed as a second magic number, so
  // this fails if the two ever drift apart. Chromium reports each
  // `drop-shadow(...)`'s offsets/color already resolved, e.g.
  // `drop-shadow(rgb(10, 10, 10) 0px 40px 0px)`.
  const offsets = [...filterValue.matchAll(/(-?[\d.]+)px/g)].map((m) => parseFloat(m[1]));
  expect(offsets.length).toBeGreaterThan(0);
  const maxAbsOffset = Math.max(...offsets.map(Math.abs));
  expect(maxAbsOffset).toBeGreaterThanOrEqual(RUBBER_BAND_MAX_PX);
});

test('dragging over the handle or the drawer content does not select text', async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await page.getByRole('button', { name: 'Move Goal' }).click();
  await awaitAnimationsSettled(page.locator('[data-slot="drawer-content"]'));

  // Drag #1: directly over DrawerHandle itself. The handle has no text of
  // its own, so this alone can't prove much (there's nothing there TO
  // select) -- included because it's the literal case the bug report
  // named, not because it's the strongest reproduction.
  const handleBox = await measureAtRest(page.locator('[data-slot="drawer-handle"]'));
  if (!handleBox) throw new Error('drawer handle has no bounding box');
  await page.mouse.move(handleBox.x + 2, handleBox.y + handleBox.height / 2);
  await page.mouse.down();
  await page.mouse.move(handleBox.x + handleBox.width - 2, handleBox.y + handleBox.height / 2, { steps: 5 });
  await page.mouse.up();
  const handleSelection = await page.evaluate(() => window.getSelection()?.toString() ?? '');
  expect(handleSelection).toBe('');

  // Drag #2: the actual reproduction. A drag gesture starts from a
  // `pointerdown` anywhere on the content not gated out
  // (primitives/src/drawer.rs's `GESTURE_JS`), which
  // includes ordinary text -- sweep across the description paragraph the
  // same way a user's finger/mouse would while trying to grab the panel by
  // its content rather than precisely on the handle. Without the
  // gesture's selection guard this exact sweep makes `getSelection().toString()`
  // return the full sentence.
  await page.evaluate(() => window.getSelection()?.removeAllRanges());
  const desc = page.locator('[data-slot="drawer-description"]');
  const descBox = await measureAtRest(desc);
  if (!descBox) throw new Error('drawer description has no bounding box');
  const midY = descBox.y + descBox.height / 2;

  await page.mouse.move(descBox.x + 2, midY);
  await page.mouse.down();
  for (let i = 1; i <= 8; i++) {
    await page.mouse.move(descBox.x + 2 + ((descBox.width - 4) * i) / 8, midY, { steps: 2 });
  }
  await page.mouse.up();

  const contentSelection = await page.evaluate(() => window.getSelection()?.toString() ?? '');
  expect(contentSelection).toBe('');
});

test('a long, slow drag past the dismiss threshold closes the drawer', async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await page.getByRole('button', { name: 'Move Goal' }).click();

  const root = page.locator('[data-slot="drawer-root"]');
  const content = page.locator('[data-slot="drawer-content"]');
  await expect(root).toHaveAttribute('data-state', 'open');
  // `data-state="open"` is set before the panel has slid in: the handle is still below the fold
  // (y ~ 749 of a 720px viewport) and a pointer pressed there lands on nothing, so the drag never
  // starts. Measure the settled panel (backlog row 115's class; the sibling drag tests above do).
  await awaitAnimationsSettled(content);

  const contentBox = await measureAtRest(content);
  const handleBox = await measureAtRest(page.locator('[data-slot="drawer-handle"]'));
  if (!contentBox || !handleBox) throw new Error('drawer content/handle has no bounding box');

  const startX = handleBox.x + handleBox.width / 2;
  const startY = handleBox.y + handleBox.height / 2;
  // Bottom drawer: dragging DOWN (toward its own edge) dismisses.
  // Comfortably past DISMISS_DRAG_RATIO (25% of the panel's own height).
  const dragDistance = contentBox.height * 0.4;

  await page.mouse.move(startX, startY);
  await page.mouse.down();
  // Several separately-timed intermediate moves with a real pause between
  // each -- this drag must exercise the *distance* release path, not the
  // *velocity* one (shadcn/ui Drawer (Vaul) closes on either), so the
  // per-step speed has to stay well under DISMISS_VELOCITY_PX_PER_MS
  // (0.5px/ms) throughout, including the last step before release.
  const steps = 6;
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(startX, startY + (dragDistance * i) / steps, { steps: 2 });
    await page.waitForTimeout(120);
  }
  // A real pause with no further movement right before release: no new
  // pointermove sample arrives, so the release velocity is whatever the
  // slow stepped movement above already made it -- not inflated by a
  // fast final flick.
  await page.waitForTimeout(200);
  await page.mouse.up();

  await expect(content).toHaveCount(0);
  await expect(root).toHaveCount(0);
});

test('a short, slow drag snaps back without closing the drawer', async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await page.getByRole('button', { name: 'Move Goal' }).click();

  const root = page.locator('[data-slot="drawer-root"]');
  const content = page.locator('[data-slot="drawer-content"]');
  await expect(root).toHaveAttribute('data-state', 'open');
  await awaitAnimationsSettled(content); // see the long-drag test above: the handle is off-screen mid slide-in

  const handleBox = await measureAtRest(page.locator('[data-slot="drawer-handle"]'));
  if (!handleBox) throw new Error('drawer handle has no bounding box');
  const startX = handleBox.x + handleBox.width / 2;
  const startY = handleBox.y + handleBox.height / 2;

  // Short (well under the 25% distance threshold for any reasonably-sized
  // panel) and slow -- must snap back, not close.
  //
  // Slow means slow in the browser's own event time: the drawer reads the release velocity from
  // each `pointermove`'s `timeStamp` over the last 100ms before the release
  // (primitives/src/drawer.rs `GESTURE_JS`), so the two explicit moves with real pauses between
  // them (and after) are what make this a non-flick. Timing samples by when a
  // reactive effect happens to run would close this drawer in roughly 1 run in 3.
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  await page.mouse.move(startX, startY + 7, { steps: 1 });
  await page.waitForTimeout(120);
  await page.mouse.move(startX, startY + 15, { steps: 1 });
  await page.waitForTimeout(150);
  await page.mouse.up();

  await expect(root).toHaveAttribute('data-state', 'open');
  await expect(content).toHaveCount(1);
  // The live drag offset is carried by an inline `transform` (see
  // primitives/src/drawer.rs's module doc for why not `translate`) --
  // snapping back removes it, so the attribute is absent at rest.
  await expect.poll(async () => (await content.getAttribute('style')) ?? '').not.toContain('transform');
});

// ---------------------------------------------------------------------------------------
// Drag direction, threshold and flick (the report: "a mouse drag UPWARDS on the
// handle closes a bottom drawer"). Only movement toward the drawer's own closing edge may
// dismiss it -- down for the bottom drawer, up for the top one -- and only past
// DISMISS_DRAG_RATIO of its size or as a flick (>= 0.5px/ms over the last 100ms, after at
// least 10px). Everything else snaps back, and a drag the other way rubber-bands.
// ---------------------------------------------------------------------------------------

type PointerKind = 'mouse' | 'touch' | 'pen';

/**
 * Drags from the centre of `selector` by `(dx, dy)` with synthetic Pointer Events of the given
 * type -- the only way to drive touch and pen deterministically, and the same events a real
 * device raises: the drawer listens to `pointerdown` on its root and `pointermove`/`pointerup`
 * on the window, whatever the pointer type. The velocity is read from each event's
 * `timeStamp`, which a synthetic event takes when it is CREATED, so:
 *   - a FLICK (`flick: true`) is dispatched in ONE synchronous task, spinning `stepMs` between
 *     events -- a busy page (a debug build re-rendering the panel on every move) cannot stretch
 *     the gesture and turn a flick into a slow drag;
 *   - a slow drag awaits real timers between events (`stepMs`), and `pauseBeforeUpMs` holds the
 *     pointer still before it is released (a pause is no flick). Jank only makes these slower.
 * Resolves after the release.
 */
async function pointerDrag(
  page: import('@playwright/test').Page,
  selector: string,
  opts: { kind: PointerKind; dx: number; dy: number; steps: number; stepMs: number; pauseBeforeUpMs?: number; flick?: boolean },
) {
  await page.evaluate(
    async ({ selector, kind, dx, dy, steps, stepMs, pauseBeforeUpMs, flick }) => {
      const el = document.querySelector(selector) as HTMLElement;
      const r = el.getBoundingClientRect(); // rest-ok: places the pointer on a drawer the caller has already opened and settled (a drag helper, not a measurement)
      const x0 = r.x + r.width / 2;
      const y0 = r.y + r.height / 2;
      const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
      const spin = (ms: number) => {
        const t = performance.now();
        while (performance.now() - t < ms) { /* hold the task: deterministic event spacing */ }
      };
      const init = (x: number, y: number) => ({
        bubbles: true, cancelable: true, composed: true, pointerId: 41, pointerType: kind,
        isPrimary: true, clientX: x, clientY: y, button: 0, buttons: 1,
      });
      el.dispatchEvent(new PointerEvent('pointerdown', init(x0, y0)));
      for (let i = 1; i <= steps; i++) {
        if (flick) spin(stepMs);
        else await sleep(stepMs);
        el.dispatchEvent(new PointerEvent('pointermove', init(x0 + (dx * i) / steps, y0 + (dy * i) / steps)));
      }
      if (pauseBeforeUpMs) await sleep(pauseBeforeUpMs);
      el.dispatchEvent(new PointerEvent('pointerup', { ...init(x0 + dx, y0 + dy), buttons: 0 }));
    },
    { selector, ...opts },
  );
}

const CONTENT = '[data-slot="drawer-content"]';
const HANDLE = '[data-slot="drawer-handle"]';
const ROOT = '[data-slot="drawer-root"]';

async function openBottom(page: import('@playwright/test').Page) {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await page.getByRole('button', { name: 'Move Goal' }).click();
  await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
  await awaitAnimationsSettled(page.locator(CONTENT));
}

async function openTop(page: import('@playwright/test').Page) {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await page.getByRole('button', { name: 'Open from Top' }).click();
  await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
  await awaitAnimationsSettled(page.locator(CONTENT));
}

/** The drawer is still open, still modal, and back at rest -- after a gesture that must not dismiss it. */
async function expectStillOpenAtRest(page: import('@playwright/test').Page) {
  // Longer than the exit (200ms) plus the driver's wait, so a dismissal that was merely slow cannot pass.
  await page.waitForTimeout(900);
  await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
  await expect(page.locator(CONTENT)).toHaveCount(1);
  expect(await page.locator(CONTENT).evaluate((el) => (el as HTMLDialogElement).matches(':modal'))).toBe(true);
  await expect.poll(async () => (await page.locator(CONTENT).getAttribute('style')) ?? '').not.toContain('transform');
}

test("a mouse drag UP on a bottom drawer's handle never closes it, however far, even released over the backdrop", async ({ page }) => {
  await openBottom(page);
  const handleBox = await measureAtRest(page.locator(HANDLE));
  if (!handleBox) throw new Error('drawer handle has no bounding box');
  const startX = handleBox.x + handleBox.width / 2;
  const startY = handleBox.y + handleBox.height / 2;

  // Slow, long, and ending far above the panel -- on the dialog's `::backdrop`. The release's
  // `click` has the dialog as its target, outside its box: that must not count as a backdrop
  // dismiss, whichever way the drawer was dragged.
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  for (let i = 1; i <= 8; i++) {
    await page.mouse.move(startX, startY - (40 * i), { steps: 2 });
    await page.waitForTimeout(20);
  }
  await page.mouse.up();
  await expectStillOpenAtRest(page);

  // And a FAST fling upward is no dismissal either (velocity counts only toward the closing edge).
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  await page.mouse.move(startX, startY - 120, { steps: 3 });
  await page.mouse.up();
  await expectStillOpenAtRest(page);
});

test('a text selection or drag that starts inside a dialog and ends on its backdrop does not dismiss it', async ({ page }) => {
  // The class behind the drawer report, on the shared hook: a press inside the panel and a
  // release over the `::backdrop` produce a `click` on the dialog itself. Only a press that
  // ALSO began on the backdrop is a dismissal.
  await openBottom(page);
  const title = page.locator('[data-slot="drawer-title"]');
  const box = await measureAtRest(title);
  if (!box) throw new Error('drawer title has no bounding box');
  await page.mouse.move(box.x + 4, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + 4, 20, { steps: 6 });
  await page.mouse.up();
  await expectStillOpenAtRest(page);

  // A real backdrop click (press and release both on it) still dismisses.
  await page.mouse.click(2, 2);
  await expect(page.locator(ROOT)).toHaveCount(0);
});

for (const kind of ['mouse', 'touch', 'pen'] as const) {
  test.describe(`${kind} pointer on a bottom drawer`, () => {
    test(`dragging UP, far and fast, never closes it`, async ({ page }) => {
      await openBottom(page);
      await pointerDrag(page, HANDLE, { kind, dx: 0, dy: -300, steps: 6, stepMs: 8, flick: true });
      await expectStillOpenAtRest(page);
    });

    test(`dragging DOWN past the threshold closes it`, async ({ page }) => {
      await openBottom(page);
      const h = (await measureAtRest(page.locator(CONTENT))).height;
      // 40% of the panel, slowly, then held still before release: the distance rule, not a flick.
      await pointerDrag(page, HANDLE, { kind, dx: 0, dy: h * 0.4, steps: 8, stepMs: 60, pauseBeforeUpMs: 200 });
      await expect(page.locator(ROOT)).toHaveCount(0);
    });

    test(`a short, slow drag DOWN snaps back`, async ({ page }) => {
      await openBottom(page);
      await pointerDrag(page, HANDLE, { kind, dx: 0, dy: 24, steps: 4, stepMs: 60, pauseBeforeUpMs: 150 });
      await expectStillOpenAtRest(page);
    });

    test(`a flick DOWN closes it even though it is short`, async ({ page }) => {
      await openBottom(page);
      const h = (await measureAtRest(page.locator(CONTENT))).height;
      // 15% of the panel (under the 25% distance threshold) in ~12ms: only the velocity can close it.
      await pointerDrag(page, HANDLE, { kind, dx: 0, dy: h * 0.15, steps: 3, stepMs: 4, flick: true });
      await expect(page.locator(ROOT)).toHaveCount(0);
    });

    test(`a twitch of a few pixels is no flick`, async ({ page }) => {
      await openBottom(page);
      await pointerDrag(page, HANDLE, { kind, dx: 0, dy: 4, steps: 2, stepMs: 2, flick: true });
      await expectStillOpenAtRest(page);
    });
  });
}

test.describe('top drawer: the closing edge is UP', () => {
  const GRAB = '[data-slot="drawer-title"]';

  test('dragging DOWN (toward the open side) never closes it', async ({ page }) => {
    await openTop(page);
    await pointerDrag(page, GRAB, { kind: 'mouse', dx: 0, dy: 300, steps: 6, stepMs: 8, flick: true });
    await page.waitForTimeout(900);
    await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
    await expect(page.locator(CONTENT)).toHaveCount(1);
  });

  test('dragging UP past the threshold closes it', async ({ page }) => {
    await openTop(page);
    const h = (await measureAtRest(page.locator(CONTENT))).height;
    await pointerDrag(page, GRAB, { kind: 'mouse', dx: 0, dy: -h * 0.5, steps: 8, stepMs: 60, pauseBeforeUpMs: 200 });
    await expect(page.locator(ROOT)).toHaveCount(0);
  });

  test('a flick UP closes it', async ({ page }) => {
    await openTop(page);
    const h = (await measureAtRest(page.locator(CONTENT))).height;
    await pointerDrag(page, GRAB, { kind: 'touch', dx: 0, dy: -h * 0.15, steps: 3, stepMs: 4, flick: true });
    await expect(page.locator(ROOT)).toHaveCount(0);
  });
});

test('a real-mouse flick closes a bottom drawer; a drag-close carries on from the release point instead of easing back first', async ({ page }) => {
  await openBottom(page);
  const content = page.locator(CONTENT);
  const box = (await measureAtRest(content));
  const handleBox = (await measureAtRest(page.locator(HANDLE)));
  const startX = handleBox.x + handleBox.width / 2;
  const startY = handleBox.y + handleBox.height / 2;

  // Sample the panel's top edge from the release on. The panel was let go ~35% down; the exit
  // must carry it on from there (monotonically toward the bottom edge), not ease it back to rest
  // while the exit keyframe slides it out (the hitch that read as the Drawer "glitching" on close).
  await page.mouse.move(startX, startY);
  await page.mouse.down();
  for (let i = 1; i <= 5; i++) {
    await page.mouse.move(startX, startY + (box.height * 0.35 * i) / 5, { steps: 2 });
    await page.waitForTimeout(60);
  }
  await page.waitForTimeout(200);
  await page.evaluate((selector) => {
    const w = window as unknown as { __tops: number[]; __run: boolean };
    w.__tops = [];
    w.__run = true;
    const tick = () => {
      const el = document.querySelector(selector);
      // Once the exit is over the dialog is `display: none` (rect all zeros) until it unmounts: not a frame of the slide.
      const r = el?.getBoundingClientRect(); // rest-ok: samples the exit slide on every frame on purpose
      if (r && r.height > 0) w.__tops.push(r.top);
      if (w.__run) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }, CONTENT);
  await page.mouse.up();
  await expect(page.locator(ROOT)).toHaveCount(0);
  const tops = await page.evaluate(() => {
    const w = window as unknown as { __tops: number[]; __run: boolean };
    w.__run = false;
    return w.__tops;
  });
  expect(tops.length).toBeGreaterThan(1);
  let worstBackslide = 0;
  for (let i = 1; i < tops.length; i++) worstBackslide = Math.max(worstBackslide, tops[i - 1] - tops[i]);
  expect(worstBackslide, `the panel moved back toward the open position by ${worstBackslide}px after release: ${tops.map(Math.round).join(',')}`).toBeLessThanOrEqual(1);
});

test.describe('Axe automated scan', () => {
  test('loaded (drawer closed) has no automatically detectable a11y issues', async ({ page }) => {
    await gotoHydrated(page, URL, GOTO_OPTS);
    // Wait for render before scanning -- see input.spec.ts's identical
    // comment for why (avoids a false pre-hydration "no main"/"no h1").
    await expect(page.getByRole('button', { name: 'Move Goal' })).toBeVisible();
    await expectNoAxeViolations(page, 'drawer: loaded', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test('open has no automatically detectable a11y issues', async ({ page }) => {
    await gotoHydrated(page, URL, GOTO_OPTS);
    await page.getByRole('button', { name: 'Move Goal' }).click();
    await expect(page.locator('[data-slot="drawer-root"]')).toHaveAttribute('data-state', 'open');
    // Named via DrawerTitle so axe's `aria-dialog-name` rule passes -- see
    // primitives/src/drawer.rs's `DrawerTitle` doc for the lesson this
    // repeats from `primitives/src/command.rs`'s `CommandDialog`.
    await expectNoAxeViolations(page, 'drawer: open', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test('top variant, dark mode, open has no automatically detectable a11y issues', async ({ page }) => {
    // Regression: `DrawerClose`'s default (no `as` override) rendering --
    // exactly the "Close" button in this component's own top-side demo --
    // needs a CSS rule of its own, or its `background-color` is the
    // UA's `ButtonFace`, which never responds to `data-theme`: in dark mode
    // `color: rgb(212, 212, 212)` (correctly inverted, inherited from
    // `.dx-drawer`) on `background-color: rgb(239, 239, 239)` (NOT inverted,
    // the same value as light mode) -- light text on a near-white
    // background.
    await gotoHydrated(page, `${URL}?dark_mode=true`, GOTO_OPTS);
    await page.getByRole('button', { name: 'Open from Top' }).click();
    await expect(page.locator('[data-slot="drawer-root"]')).toHaveAttribute('data-state', 'open');
    await expectNoAxeViolations(page, 'drawer: top variant, dark mode, open', {
      excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT],
    });
  });
});

// The scrim every modal overlay shares (assert-backdrop-fade.ts has the full
// claim; popover/dialog/alert-dialog/sheet/drawer specs all run the identical
// assertion): the dialog's own `::backdrop` is the only painter, black/10 with a
// 4px blur, and it fades in and out for the same length. Before this, Drawer
// stacked its own animated wrapper scrim on the UA's un-animated one and the
// exit snapped.
test.describe("Scrim", () => {
  test("Drawer paints the shared scrim and fades it in and out", async ({ page }) => {
    await gotoHydrated(page, URL, GOTO_OPTS);
    const trigger = page.getByRole("button", { name: "Move Goal", exact: true });
    await expect(trigger).toBeVisible();
    const capture = await captureBackdrop(
      page,
      () => trigger.click(),
      () => page.keyboard.press("Escape"),
    );
    assertSharedBackdrop(capture, await resolveOverlayMs(page), "drawer");
  });
});

// ---------------------------------------------------------------------------------------
// One edge panel, two drawers' worth of sides (backlog rows 207 and 208). `Auto` -- the default -- is
// the bottom below the `md` breakpoint (768px) and the right at and above it; an explicit side is
// exactly that side at every width. A bottom drawer is docked flush and rounds the corners facing the
// page; a left/right one floats inset from the viewport edge with all four corners rounded. Tier 3
// ("opinion"): shadcn's demo is a bottom Vaul drawer, and `Auto` is the chosen default.
// ---------------------------------------------------------------------------------------

/** The panel's resolved placement: the edge gaps to the fixed viewport box, the radii and the handle. */
async function placement(page: import('@playwright/test').Page) {
  const viewport = await fixedViewportBox(page);
  return readAtRest(page.locator(CONTENT), (el, vp) => {
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    const handle = el.querySelector('[data-slot="drawer-handle"]') as HTMLElement | null;
    return {
      side: el.getAttribute('data-side'),
      top: r.top - vp.y,
      right: vp.x + vp.width - r.right,
      bottom: vp.y + vp.height - r.bottom,
      left: r.left - vp.x,
      radius: {
        tl: parseFloat(cs.borderTopLeftRadius),
        tr: parseFloat(cs.borderTopRightRadius),
        br: parseFloat(cs.borderBottomRightRadius),
        bl: parseFloat(cs.borderBottomLeftRadius),
      },
      handleShown: handle ? getComputedStyle(handle).display !== 'none' : null,
    };
  }, viewport);
}

test.describe('Auto side (the default)', () => {
  test('is a flush bottom drawer with a rounded top edge and a handle below 768px', async ({ page }) => {
    await page.setViewportSize(PHONE);
    await openBottom(page);
    const p = await placement(page);
    expect(p.side).toBe('bottom');
    expect(Math.abs(p.bottom)).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
    expect(Math.abs(p.left)).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
    expect(Math.abs(p.right)).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
    expect(p.radius.tl).toBeGreaterThan(0);
    expect(p.radius.tr).toBeGreaterThan(0);
    expect(p.radius.bl).toBe(0);
    expect(p.radius.br).toBe(0);
    expect(p.handleShown).toBe(true);
  });

  test('is a floating right drawer with all four corners rounded and no handle at 768px and above', async ({ page }) => {
    await page.setViewportSize(DESKTOP);
    await openBottom(page);
    const p = await placement(page);
    expect(p.side).toBe('right');
    // Inset from the viewport edge on the three sides it floats off (the theme's `--dx-space-2`, 8px)...
    for (const gap of [p.top, p.right, p.bottom]) expect(Math.abs(gap - 8)).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
    // ...three quarters of the viewport wide, capped at 24rem (shadcn's `w-3/4 sm:max-w-sm`).
    expect((await measureAtRest(page.locator(CONTENT))).width).toBeLessThanOrEqual(384 + FLUSH_TOLERANCE_PX);
    for (const r of Object.values(p.radius)) expect(r).toBeGreaterThan(0);
    // The grab bar is the bottom drawer's.
    expect(p.handleShown).toBe(false);
  });

  test('breaks exactly at the md breakpoint (767px bottom, 768px right) and follows a resize while closed', async ({ page }) => {
    await page.setViewportSize({ width: 767, height: 800 });
    await gotoHydrated(page, URL, GOTO_OPTS);
    const trigger = page.getByRole('button', { name: 'Move Goal' });
    await trigger.click();
    await expect(page.locator(CONTENT)).toHaveAttribute('data-side', 'bottom');
    await page.keyboard.press('Escape');
    await expect(page.locator(ROOT)).toHaveCount(0);

    // No reload: the listener on the media query moves the next open to the other side.
    await page.setViewportSize({ width: 768, height: 800 });
    await trigger.click();
    await expect(page.locator(CONTENT)).toHaveAttribute('data-side', 'right');
    await page.keyboard.press('Escape');
    await expect(page.locator(ROOT)).toHaveCount(0);
  });

  test('a drag toward the right edge closes the auto-resolved right drawer; away from it never does', async ({ page }) => {
    await page.setViewportSize(DESKTOP);
    await openBottom(page);
    const w = (await measureAtRest(page.locator(CONTENT))).width;
    // The grab surface is the panel itself (the handle is hidden on a side drawer).
    await pointerDrag(page, '[data-slot="drawer-title"]', { kind: 'mouse', dx: -300, dy: 0, steps: 6, stepMs: 8, flick: true });
    await page.waitForTimeout(900);
    await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
    await pointerDrag(page, '[data-slot="drawer-title"]', { kind: 'mouse', dx: w * 0.5, dy: 0, steps: 8, stepMs: 60, pauseBeforeUpMs: 200 });
    await expect(page.locator(ROOT)).toHaveCount(0);
  });
});

test.describe('An explicit side is exactly that side', () => {
  for (const [name, viewport] of [['phone', PHONE], ['desktop', DESKTOP]] as const) {
    test(`Top is a flush top drawer with a rounded bottom edge and no handle (${name})`, async ({ page }) => {
      await page.setViewportSize(viewport);
      await openTop(page);
      const p = await placement(page);
      expect(p.side).toBe('top');
      expect(Math.abs(p.top)).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
      expect(p.radius.bl).toBeGreaterThan(0);
      expect(p.radius.br).toBeGreaterThan(0);
      expect(p.radius.tl).toBe(0);
      expect(p.radius.tr).toBe(0);
    });

    test(`Left floats inset with all corners rounded (${name})`, async ({ page }) => {
      await page.setViewportSize(viewport);
      await gotoHydrated(page, URL, GOTO_OPTS);
      await page.getByRole('button', { name: 'Open from Left' }).click();
      await awaitAnimationsSettled(page.locator(CONTENT));
      const p = await placement(page);
      expect(p.side).toBe('left');
      for (const gap of [p.top, p.left, p.bottom]) expect(Math.abs(gap - 8)).toBeLessThanOrEqual(FLUSH_TOLERANCE_PX);
      for (const r of Object.values(p.radius)) expect(r).toBeGreaterThan(0);
    });
  }
});

// ---------------------------------------------------------------------------------------
// Nested drawers (Vaul's `NestedRoot`): a `Drawer` inside another `DrawerContent`. Opening the child
// steps the parent back (a shrink + offset), closing it restores the parent; Escape, a drag and a
// scrim tap act on the top-most only; the page's scroll lock stays one lock.
// ---------------------------------------------------------------------------------------
test.describe('Nested drawers', () => {
  // The child is a DOM descendant of the parent, so its title is also "inside" the parent's content: match it
  // by its own text.
  const CHILD_TITLE = '[data-slot="drawer-title"]:text-is("Nested drawer")';
  const CHILD_HANDLE = '[data-slot="drawer-content"] [data-slot="drawer-content"] [data-slot="drawer-handle"]';

  const scrollLocked = (page: import('@playwright/test').Page) =>
    page.evaluate(() => (window as unknown as { __dxScrollLocked?: boolean }).__dxScrollLocked === true);
  const openModals = (page: import('@playwright/test').Page) =>
    page.evaluate(() => [...document.querySelectorAll('dialog')].filter((d) => d.open && d.matches(':modal')).length);
  const parentState = (page: import('@playwright/test').Page) =>
    page.evaluate(() => {
      const el = document.querySelector('dialog[data-slot="drawer-content"][data-nested-open]') as HTMLElement;
      const cs = getComputedStyle(el);
      return { nested: el.getAttribute('data-nested-open'), scale: cs.scale, transform: cs.transform, open: el.getAttribute('data-state') };
    });

  async function openParent(page: import('@playwright/test').Page) {
    await gotoHydrated(page, URL, GOTO_OPTS);
    await page.getByRole('button', { name: 'Open nested drawers' }).click();
    await expect(page.getByText('Parent drawer', { exact: true })).toBeVisible();
    await awaitAnimationsSettled(page.locator(CONTENT));
  }
  async function openChild(page: import('@playwright/test').Page) {
    await page.getByRole('button', { name: 'Open nested drawer', exact: true }).click();
    await expect(page.locator(CHILD_TITLE)).toBeVisible();
    await awaitAnimationsSettled(page.locator(CHILD_HANDLE));
  }

  test('opening the child scales and offsets the parent; closing it restores the parent', async ({ page }) => {
    await openParent(page);
    expect(await parentState(page)).toMatchObject({ nested: 'false', scale: 'none' });

    await openChild(page);
    // The parent steps back: `scale` 0.96 and a vertical offset, once its transition has played.
    await expect.poll(async () => (await parentState(page)).nested).toBe('true');
    await expect.poll(async () => parseFloat((await parentState(page)).scale)).toBeCloseTo(0.96, 2);
    await expect.poll(async () => (await parentState(page)).transform).not.toBe('none');
    expect(await openModals(page)).toBe(2);

    // Escape closes only the top-most (the child); the parent is restored the moment the child starts to go.
    await page.keyboard.press('Escape');
    await expect(page.locator(CHILD_TITLE)).toHaveCount(0);
    expect(await openModals(page)).toBe(1);
    await expect.poll(async () => parentState(page)).toMatchObject({ nested: 'false', scale: 'none', transform: 'none', open: 'open' });
    // Focus is back on the button that opened the child, inside the still-open parent.
    await expect(page.getByRole('button', { name: 'Open nested drawer', exact: true })).toBeFocused();
  });

  test('Escape, then Escape again: child first, parent second; the scroll lock stays one lock until the last closes', async ({ page }) => {
    await openParent(page);
    expect(await scrollLocked(page)).toBe(true);
    await openChild(page);
    expect(await scrollLocked(page)).toBe(true);

    await page.keyboard.press('Escape');
    await expect(page.locator(CHILD_TITLE)).toHaveCount(0);
    // The parent still holds the lock: closing the child must not release it.
    expect(await scrollLocked(page)).toBe(true);
    await expect(page.getByText('Parent drawer', { exact: true })).toBeVisible();

    await page.keyboard.press('Escape');
    await expect(page.locator(ROOT)).toHaveCount(0);
    expect(await scrollLocked(page)).toBe(false);
    await expect(page.getByRole('button', { name: 'Open nested drawers' })).toBeFocused();
  });

  test('a drag on the child closes only the child, however far; the parent never starts a drag', async ({ page }) => {
    await openParent(page);
    await openChild(page);
    const h = await readAtRest(page.locator(CHILD_HANDLE), (el) => (el.closest('dialog') as HTMLElement).getBoundingClientRect().height);
    // 50% of the CHILD, slowly: the same gesture would close the (taller) parent too if its listener took it.
    await pointerDrag(page, CHILD_HANDLE, { kind: 'mouse', dx: 0, dy: h * 0.5, steps: 8, stepMs: 60, pauseBeforeUpMs: 200 });
    await expect(page.locator(CHILD_TITLE)).toHaveCount(0);
    await page.waitForTimeout(900);
    await expect(page.getByText('Parent drawer', { exact: true })).toBeVisible();
    expect(await openModals(page)).toBe(1);
    expect(await scrollLocked(page)).toBe(true);
    await expect.poll(async () => parentState(page)).toMatchObject({ nested: 'false', scale: 'none', open: 'open' });
    // And now the parent, alone on top, drags normally.
    const ph = (await measureAtRest(page.locator(CONTENT))).height;
    await pointerDrag(page, '[data-slot="drawer-content"] > [data-slot="drawer-handle"]', { kind: 'mouse', dx: 0, dy: ph * 0.5, steps: 8, stepMs: 60, pauseBeforeUpMs: 200 });
    await expect(page.locator(ROOT)).toHaveCount(0);
    expect(await scrollLocked(page)).toBe(false);
  });

  test('a tap on the child scrim closes the child, not the parent', async ({ page }) => {
    await openParent(page);
    await openChild(page);
    await page.mouse.click(195, 8); // far above both panels: the child's own scrim
    await expect(page.locator(CHILD_TITLE)).toHaveCount(0);
    await page.waitForTimeout(500);
    await expect(page.getByText('Parent drawer', { exact: true })).toBeVisible();
    expect(await scrollLocked(page)).toBe(true);
  });

  test('has no automatically detectable a11y issues with both drawers open', async ({ page }) => {
    await openParent(page);
    await openChild(page);
    await expectNoAxeViolations(page, 'drawer: nested, open', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
