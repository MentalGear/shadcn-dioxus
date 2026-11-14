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
 * does not animate out of `display: none` plays the exit and snaps the entry. The entry is instead a `@keyframes`
 * animation bound to the open state, which starts when the element starts being rendered in every engine
 * (`dx-modal-in` and `dx-scrim-in` for the modal dialog family, `--dx-floating-enter` for popovers and menus).
 *
 * Two claims per overlay kind, opened by real input:
 *   - behaviour: the opacity (the scrim's alpha) of the frame sequence starts below rest, never falls and ends at
 *     rest (the sequence is asserted, no duration is bounded, so the cases carry no timing tag); under `prefers-reduced-motion: reduce` it never shows an intermediate value;
 *   - mechanism, standing in for the engines this suite cannot run: no style sheet carries an `@starting-style`
 *     rule that matches the element, and the element has a keyframe animation (a `CSSAnimation` with a name) while it is
 *     open, not only a transition.
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
  anims: { name: string | null; pseudo: string | null }[];
};

type Kind = {
  name: string;
  url: string;
  open: (page: Page) => Promise<void>;
  /** Whether the element is a modal `<dialog>` with a scrim. */
  scrim: boolean;
  /** The keyframes the entry names. */
  keyframes: RegExp;
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
    keyframes: /^dx-modal-in$/,
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
    const el = document.querySelector<HTMLElement>('dialog[open][data-open], [data-dx-surface="floating"][data-open]');
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
        pseudo: (a.effect as KeyframeEffect).pseudoElement ?? null,
      }));
    return { o: Number(getComputedStyle(el).opacity), bd, anims } as Frame;
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

    test("is a keyframe animation, not a `@starting-style` transition", async ({ page }) => {
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
      expect(named(null, kind.keyframes), `no running keyframe animation on the element: ${JSON.stringify(frames[0]?.anims)}`).toBe(true);
      if (kind.scrim) {
        expect(named("::backdrop", /^dx-scrim-in$/), "no running `dx-scrim-in` animation on the ::backdrop").toBe(true);
      }
    });
  });
}
