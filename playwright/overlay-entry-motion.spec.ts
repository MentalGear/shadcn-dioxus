import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoFrame, demoRole } from "./demo-frame";
import { installFrameRecorder, readFrames } from "./frame-recorder";

/**
 * Every overlay fades in on open, and its entry does not depend on `@starting-style`.
 *
 * An entry written as a `transition` from `@starting-style` has to animate out of `display: none` (a `<dialog>`
 * before `showModal()`, a popover before `showPopover()`, a `::backdrop` before the dialog is modal). An engine that
 * does not animate out of `display: none` plays the exit and snaps the entry. Popovers and menus enter with a
 * `@keyframes` animation bound to the open state (`--dx-floating-enter`), which starts when the element starts being
 * rendered in every engine. The modal dialog family (Dialog, AlertDialog, CommandDialog and the scrim) enters with the
 * same `transition` it exits with: the `<dialog>` is rendered a frame before it gets `data-open`, so the transition
 * starts from a value the engine has rendered, and a close in the middle of the entry reverses it (a cancelled
 * animation would start no transition and the panel would snap shut).
 *
 * Two claims per overlay kind, opened by real input:
 *   - behaviour: the opacity (the scrim's alpha) of the frame sequence starts below rest, never falls and ends at
 *     rest (the sequence is asserted, no duration is bounded, so the cases carry no timing tag); under `prefers-reduced-motion: reduce` it never shows an intermediate value;
 *   - mechanism, standing in for the engines this suite cannot run: no style sheet carries an `@starting-style`
 *     rule that matches the element, and the entry is the mechanism of its kind: a keyframe animation (a `CSSAnimation`
 *     with a name) for a popover or menu; for the modal dialog, a `CSSTransition` of `opacity` (and of the scrim's
 *     `background-color`), the first frame rendered at rest (opacity 0) WITHOUT `data-open`, and no keyframe entry.
 *
 * The frames come from `installFrameRecorder` (every animation frame from the first byte of the document), so no
 * frame between the input and the end of the fade is missed. The values are computed style, never geometry.
 */

const GOTO = { timeout: 20 * 60 * 1000 };

type Frame = {
  /** The element's computed opacity. */
  o: number;
  /** The `::backdrop`'s background alpha (a modal `<dialog>` only), else `null`. */
  bd: number | null;
  /** Every animation whose effect targets the element or its `::backdrop`. */
  anims: { name: string | null; transition: string | null; pseudo: string | null }[];
  /** Whether the element carries `data-open` in this frame. */
  dataOpen: boolean;
};

type Kind = {
  name: string;
  url: string;
  open: (page: Page) => Promise<void>;
  /** Whether the element is a modal `<dialog>` with a scrim. */
  scrim: boolean;
  /** The keyframes the entry names, or `null` when the entry is a transition (the modal dialog). */
  keyframes: RegExp | null;
};

const KINDS: Kind[] = [
  {
    name: "modal dialog and its scrim",
    url: `${BASE_URL}/component/dialog/`,
    open: async (page) => {
      // dialog demo: "Show Dialog" is the first button of the main demo.
      await demoRole(page, "button", 0).click();
    },
    scrim: true,
    keyframes: null,
  },
  {
    name: "popover opened by click",
    url: `${BASE_URL}/component/popover/`,
    open: async (page) => {
      await demoRole(page, "button", 0).click();
    },
    scrim: false,
    keyframes: /^dx-[a-z-]+-in$/,
  },
  {
    name: "hover card opened by hover",
    url: `${BASE_URL}/component/hover_card/`,
    open: async (page) => {
      await demoFrame(page).locator("i").first().hover();
    },
    scrim: false,
    keyframes: /^dx-[a-z-]+-in$/,
  },
  {
    name: "menu opened by click",
    url: `${BASE_URL}/component/dropdown_menu/`,
    open: async (page) => {
      await demoRole(page, "button", 0).click();
    },
    scrim: false,
    keyframes: /^dx-[a-z-]+-in$/,
  },
];

const isNonDecreasing = (xs: number[]) => xs.every((x, i) => i === 0 || x >= xs[i - 1] - 0.001);

const FRAMES = "__overlayEntryFrames";

async function record(page: Page) {
  await installFrameRecorder(page, FRAMES, () => {
    const el = document.querySelector<HTMLElement>('dialog[open], [data-dx-surface="floating"][data-open]');
    if (!el) return null;
    const alpha = (c: string) => {
      const m = /rgba?\(([^)]+)\)/.exec(c);
      if (m) {
        const p = m[1].split(/[ ,/]+/).filter(Boolean);
        return p.length > 3 ? Number(p[3]) : 1;
      }
      const c4 = /color\(.*\/\s*([\d.]+)\)/.exec(c);
      return c4 ? Number(c4[1]) : c.startsWith("color(") ? 1 : 0;
    };
    const bd = el.tagName === "DIALOG" ? alpha(getComputedStyle(el, "::backdrop").backgroundColor) : null;
    const anims = document
      .getAnimations()
      .filter((a) => (a.effect as KeyframeEffect | null)?.target === el)
      .map((a) => ({
        name: (a as CSSAnimation).animationName ?? null,
        transition: (a as CSSTransition).transitionProperty ?? null,
        pseudo: (a.effect as KeyframeEffect).pseudoElement ?? null,
      }));
    return { o: Number(getComputedStyle(el).opacity), bd, anims, dataOpen: el.hasAttribute("data-open") } as Frame;
  });
}

/** The `@starting-style` selectors that match the open overlay (the `::backdrop` ones match its dialog). */
async function startingStyleSelectors(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const el = document.querySelector<HTMLElement>('dialog[open][data-open], [data-dx-surface="floating"][data-open]');
    if (!el) throw new Error("no open overlay");
    const hits: string[] = [];
    const walk = (rules: CSSRuleList, inStarting: boolean) => {
      for (const rule of Array.from(rules)) {
        const starting = inStarting || rule.cssText.startsWith("@starting-style");
        const nested = (rule as CSSGroupingRule).cssRules;
        if (nested) walk(nested, starting);
        const selector = (rule as CSSStyleRule).selectorText;
        if (starting && selector) {
          for (const part of selector.split(",")) {
            const bare = part.replace(/::?backdrop/, "").trim();
            try {
              if (el.matches(bare || "*")) hits.push(part.trim());
            } catch {
              /* a selector this engine cannot parse cannot match */
            }
          }
        }
      }
    };
    for (const sheet of Array.from(document.styleSheets)) {
      try {
        walk(sheet.cssRules, false);
      } catch {
        /* a cross-origin sheet is not ours */
      }
    }
    return hits;
  });
}

for (const kind of KINDS) {
  test.describe(`${kind.name}: entry`, () => {
    test("starts below rest, rises and ends at rest", async ({ page }) => {
      await page.emulateMedia({ reducedMotion: "no-preference" });
      await record(page);
      await gotoHydrated(page, kind.url, GOTO);
      await kind.open(page);
      await expect.poll(async () => (await readFrames<Frame>(page, FRAMES)).length, { timeout: 10_000 }).toBeGreaterThan(0);
      // Wait for the fade to finish: the last frame is at rest.
      await expect
        .poll(async () => (await readFrames<Frame>(page, FRAMES)).at(-1)?.o, { timeout: 10_000 })
        .toBeCloseTo(1, 2);
      const frames = await readFrames<Frame>(page, FRAMES);
      // The sequence, not a duration: the first frame the overlay is open is below rest, the values never fall, the last is at rest.
      const opacities = frames.map((f) => f.o);
      expect(opacities[0], `the first open frame is already at rest: ${JSON.stringify(opacities)}`).toBeLessThan(0.98);
      expect(isNonDecreasing(opacities), `opacity fell during the entry: ${JSON.stringify(opacities)}`).toBe(true);
      if (kind.scrim) {
        await expect
          .poll(async () => (await readFrames<Frame>(page, FRAMES)).at(-1)?.bd, { timeout: 10_000 })
          .toBeGreaterThan(0.09);
        const alphas = (await readFrames<Frame>(page, FRAMES)).map((f) => f.bd ?? 0);
        expect(alphas[0], `the scrim is already at rest on the first open frame: ${JSON.stringify(alphas)}`).toBeLessThan(0.098);
        expect(isNonDecreasing(alphas), `the scrim alpha fell during the entry: ${JSON.stringify(alphas)}`).toBe(true);
      }
    });

    test("shows no intermediate value under reduced motion", async ({ page }) => {
      await page.emulateMedia({ reducedMotion: "reduce" });
      await record(page);
      await gotoHydrated(page, kind.url, GOTO);
      await kind.open(page);
      await expect
        .poll(async () => (await readFrames<Frame>(page, FRAMES)).at(-1)?.o, { timeout: 10_000 })
        .toBeCloseTo(1, 2);
      const frames = await readFrames<Frame>(page, FRAMES);
      expect(frames.filter((f) => f.o > 0.02 && f.o < 0.98).map((f) => f.o)).toEqual([]);
      if (kind.scrim) {
        expect(frames.filter((f) => (f.bd ?? 0) > 0.002 && (f.bd ?? 0) < 0.098).map((f) => f.bd)).toEqual([]);
      }
    });

    test(kind.keyframes ? "is a keyframe animation, not a `@starting-style` transition" : "is a transition from a frame rendered before `data-open`, not a `@starting-style` transition", async ({ page }) => {
      await page.emulateMedia({ reducedMotion: "no-preference" });
      await record(page);
      await gotoHydrated(page, kind.url, GOTO);
      await kind.open(page);
      await expect.poll(async () => (await readFrames<Frame>(page, FRAMES)).length, { timeout: 10_000 }).toBeGreaterThan(0);
      await expect
        .poll(async () => (await readFrames<Frame>(page, FRAMES)).at(-1)?.o, { timeout: 10_000 })
        .toBeCloseTo(1, 2);

      expect(await startingStyleSelectors(page), "a `@starting-style` rule matches the open overlay").toEqual([]);

      const frames = await readFrames<Frame>(page, FRAMES);
      const named = (pseudo: string | null, pattern: RegExp) =>
        frames.some((f) => f.anims.some((a) => a.pseudo === pseudo && a.name !== null && pattern.test(a.name)));
      const transitioned = (pseudo: string | null, property: string) =>
        frames.some((f) => f.anims.some((a) => a.pseudo === pseudo && a.transition === property));
      if (kind.keyframes) {
        expect(named(null, kind.keyframes), `no running keyframe animation on the element: ${JSON.stringify(frames[0]?.anims)}`).toBe(true);
      } else {
        expect(named(null, /./), `the element's entry is a keyframe animation: ${JSON.stringify(frames[0]?.anims)}`).toBe(false);
        expect(transitioned(null, "opacity"), `no running opacity transition on the element: ${JSON.stringify(frames.map((f) => f.anims))}`).toBe(true);
        // The first frame the dialog is rendered is the base rule: opacity 0, no `data-open`.
        console.log(`first frames (opacity, data-open): ${JSON.stringify(frames.slice(0, 4).map((f) => [f.o, f.dataOpen]))}`);
        expect(frames[0].dataOpen, "the first rendered frame already carries `data-open`").toBe(false);
        expect(frames[0].o, "the first rendered frame is not at the base rule").toBe(0);
        expect(frames.some((f) => f.dataOpen), "`data-open` never landed").toBe(true);
      }
      if (kind.scrim) {
        expect(transitioned("::backdrop", "background-color"), "no running background-color transition on the ::backdrop").toBe(true);
        expect(named("::backdrop", /./), "the scrim's entry is a keyframe animation").toBe(false);
      }
    });
  });
}

test.describe("modal dialog: a close in the middle of the entry", () => {
  test("reverses the entry transition and keeps the dialog open for the exit", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await gotoHydrated(page, `${BASE_URL}/component/dialog/`, GOTO);
    // A long exit makes "still open" measurable; the entry shares the token.
    await page.addStyleTag({ content: ":root { --dx-overlay-duration: 1200ms !important; }" });
    await demoRole(page, "button", 0).click();
    await expect(page.locator("dialog[open]")).toHaveCount(1);
    // Mid-entry: the panel is visibly on its way in (a reversal is shortened by how far it got, so a close
    // at the first frames would have almost nothing to reverse).
    await expect
      .poll(() => page.evaluate(() => Number(getComputedStyle(document.querySelector("dialog[open]")!).opacity)))
      .toBeGreaterThan(0.3);
    await page.keyboard.press("Escape");
    await page.waitForTimeout(60);
    const state = await page.evaluate(() => {
      const dialog = document.querySelector<HTMLDialogElement>("dialog[open]");
      if (!dialog) return { open: false, running: [] as string[] };
      return {
        open: true,
        running: document
          .getAnimations()
          .filter((a) => (a.effect as KeyframeEffect | null)?.target === dialog && a.playState === "running")
          .map((a) => `${a.constructor.name}:${(a as CSSTransition).transitionProperty ?? (a as CSSAnimation).animationName}`),
      };
    });
    expect(state.open, "the dialog snapped shut instead of playing the exit").toBe(true);
    expect(state.running.length, `no running transition on the dialog 60 ms after Escape: ${JSON.stringify(state)}`).toBeGreaterThan(0);
    await expect(page.locator("dialog[open]")).toHaveCount(0, { timeout: 10_000 });
  });
});

test.describe("modal dialog: the entering frame", () => {
  // The long token keeps the entry (`data-entering` then `data-open`) measurable; the first frames are the window.
  const openSlowly = async (page: Page) => {
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await gotoHydrated(page, `${BASE_URL}/component/dialog/`, GOTO);
    await page.addStyleTag({ content: ":root { --dx-overlay-duration: 1200ms !important; }" });
  };

  /** Clicks the trigger and runs `probe` in the page on the first animation frame where the `<dialog>` is modal. */
  const onEnteringFrame = async <T>(page: Page, probe: string): Promise<T> => {
    const result = page.evaluate(
      (body) =>
        new Promise<unknown>((resolve) => {
          const probeFn = new Function("dialog", `return (${body})(dialog);`) as (d: HTMLDialogElement) => unknown;
          const tick = () => {
            const dialog = document.querySelector<HTMLDialogElement>("dialog:modal");
            if (dialog) resolve(probeFn(dialog));
            else requestAnimationFrame(tick);
          };
          tick();
        }),
      probe,
    );
    await demoRole(page, "button", 0).click();
    return (await result) as T;
  };

  test("a backdrop press on the first frame after open dismisses it", async ({ page }) => {
    await openSlowly(page);
    const pressed = await onEnteringFrame<{ entering: boolean; hit: boolean }>(
      page,
      `(dialog) => {
        const entering = dialog.hasAttribute("data-entering");
        const hit = document.elementFromPoint(2, 2) === dialog;
        for (const type of ["pointerdown", "pointerup"]) {
          dialog.dispatchEvent(new PointerEvent(type, { bubbles: true, composed: true, clientX: 2, clientY: 2, pointerType: "mouse", button: 0, isPrimary: true }));
        }
        return { entering, hit };
      }`,
    );
    expect(pressed.entering, "the press missed the entering frame").toBe(true);
    expect(pressed.hit, "the backdrop press did not land on the <dialog> on the entering frame").toBe(true);
    await expect(page.locator("dialog[open]")).toHaveCount(0, { timeout: 10_000 });
  });

  test("the entering frame takes pointer events", async ({ page }) => {
    await openSlowly(page);
    const frame = await onEnteringFrame<{ entering: boolean; pointerEvents: string }>(
      page,
      `(dialog) => ({ entering: dialog.hasAttribute("data-entering"), pointerEvents: getComputedStyle(dialog).pointerEvents })`,
    );
    expect(frame.entering, "the probe missed the entering frame").toBe(true);
    expect(frame.pointerEvents).toBe("auto");
  });
});
