import { test, expect } from "./fixtures";
import * as fs from "fs";
import * as path from "path";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * The keyboard-focus ring is grey (shadcn's neutral `--ring`) until an accent is picked. With no accent it is
 * shadcn's EXACT value, light 2.3-2.6:1 on white (under 1.4.11's 3:1, shadcn's own); in contrast mode
 * (`<html data-dx-contrast="wcag">`, or `prefers-contrast: more`) its solid edge reads >= 3:1 (WCAG 1.4.11,
 * non-text contrast) against the page, card and popover surfaces, in the default theme and under every
 * theme-picker preset, in both modes. With an accent picked the ring takes the accent's hue and chroma
 * (row 385), lightness clamped per mode, and reads >= 3:1 in BOTH contrast settings.
 *
 * Construction under test (`preview/assets/dx-components-theme.css`, `--dx-ring-source` / `--dx-ring-color`,
 * the "Contrast mode" block): the source is `--dx-accent-ring` when an accent is active, else the user's or
 * preset's `--ring`, else shadcn neutral's (light oklch(0.708 0 0) = 2.6:1 on white, dark oklch(0.556 0 0));
 * in contrast mode the LIGHT arm's OKLCH lightness is clamped (hue and chroma kept), so a base colour's tint,
 * and a pasted theme's coloured ring, still come through. The accent ring's own clamp (light L <= 0.62, dark
 * L >= 0.6) is the `:root[data-theme-accent]` rule. The switch itself, and the other calibrated roles, are in
 * contrast-mode.spec.ts.
 *
 * Method: focus the docs Input, read the first colour of its focused `box-shadow` (the 1px solid edge of
 * `--dx-ring-field`), resolve it through a canvas, and compare with the three surface tokens resolved the
 * same way. Presets are applied as the picker does (`data-theme-base` / `data-theme-accent` on <html>),
 * the mode through the one writer `window.__dxApplyMode`. Names are READ FROM `theme-presets.css`.
 */

const NON_TEXT = 3;
/** Neutral ring chroma must stay below this (OKLCH C): the default is grey, not a saturated blue such as #2b7fff (C ~0.2). */
const NEUTRAL_CHROMA = 0.03;
/** Hue tolerance in degrees. A ring's chroma is 0.01-0.04, so 8-bit rounding alone is a few degrees. */
const HUE_TOLERANCE = 20;
/** An accent ring's hue against the accent's `--primary` hue. Gamut mapping of the clamped value moves lime and
 * yellow's by 4 and 9 degrees; every other accent is within 3. */
const ACCENT_HUE_TOLERANCE = 10;
/** An accent whose primary is at least this chromatic has a hue worth tracking (OKLCH C). */
const CHROMATIC = 0.05;
/** The accent clamp's bounds, as written in the theme (`--dx-accent-ring-l-max` / `-l-min`), with 8-bit rounding slack. */
const ACCENT_L_MAX = 0.62 + 0.012;
const ACCENT_L_MIN = 0.6 - 0.012;

const PRESETS_CSS = fs.readFileSync(path.join(__dirname, "..", "preview", "assets", "theme-presets.css"), "utf8");
const names = (axis: "base" | "accent") => [
  ...new Set([...PRESETS_CSS.matchAll(new RegExp(`\\[data-theme-${axis}="([a-z]+)"\\] \\{`, "g"))].map((m) => m[1])),
];
const BASES = names("base");
const ACCENTS = names("accent");

type Row = {
  mode: "light" | "dark";
  base: string;
  accent: string;
  /** Lowest contrast of the ring edge over background / card / popover. */
  ratio: number;
  worstSurface: string;
  /** The ring edge in OKLCH. */
  ring: { l: number; c: number; h: number };
  /** The `--ring` custom property as the page declares it, in OKLCH (null when none is set). */
  declared: { l: number; c: number; h: number } | null;
  /** The ring edge's 8-bit sRGB, and the sRGB of the source shadcn would draw (the declared `--ring`, else neutral's). */
  edgeRgb: number[];
  sourceRgb: number[];
  /** The page's `--dx-primary` (the accent's fill when an accent is picked), in OKLCH. */
  primary: { l: number; c: number; h: number };
};

test.describe("focus ring contrast", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/input/`);
    const input = page.getByRole("textbox", { name: "Enter your name" });
    await expect(input).toBeVisible();
    // Focus by keyboard: Tab from the top of the document until the Input has it.
    await input.focus();
    await page.keyboard.press("Shift+Tab");
    await page.keyboard.press("Tab");
    await expect(input).toBeFocused();
    await page.addStyleTag({ content: "*,*::before,*::after{transition:none!important;animation:none!important}" });
  });

  /** Applies each combination in turn and measures the focused Input's ring edge. `contrast` is the
   * `data-dx-contrast` value (null: none); the contrast-mode tests below use `wcag`, the default ones null. */
  const measure = (page: import("@playwright/test").Page, mode: "light" | "dark", override?: string, contrast: string | null = "wcag") =>
    page.evaluate(
      ({ mode, bases, accents, override, contrast }) => {
        (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(mode);
        (window as unknown as { __dxTheme: (k: string, v: string | null) => void }).__dxTheme("dx-contrast", contrast);
        const html = document.documentElement;
        if (override) html.style.setProperty("--ring", override);
        else html.style.removeProperty("--ring");
        const input = document.activeElement as HTMLElement;
        const cv = document.createElement("canvas");
        cv.width = cv.height = 1;
        const cx = cv.getContext("2d", { willReadFrequently: true })!;
        const rgbOf = (css: string): [number, number, number] => {
          cx.clearRect(0, 0, 1, 1);
          cx.fillStyle = "#fff";
          cx.fillRect(0, 0, 1, 1);
          cx.fillStyle = css;
          cx.fillRect(0, 0, 1, 1);
          const d = cx.getImageData(0, 0, 1, 1).data;
          return [d[0], d[1], d[2]];
        };
        const lum = ([r, g, b]: number[]) => {
          const f = (v: number) => ((v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
          return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
        };
        const ratio = (a: number[], b: number[]) => {
          const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
          return (hi + 0.05) / (lo + 0.05);
        };
        const toOklch = ([r, g, b]: number[]) => {
          const lin = (v: number) => ((v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
          const [R, G, B] = [lin(r), lin(g), lin(b)];
          const l = Math.cbrt(0.4122214708 * R + 0.5363325363 * G + 0.0514459929 * B);
          const m = Math.cbrt(0.2119034982 * R + 0.6806995451 * G + 0.1073969566 * B);
          const s = Math.cbrt(0.0883024619 * R + 0.2817188376 * G + 0.6299787005 * B);
          const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
          const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
          const bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
          return { l: L, c: Math.hypot(a, bb), h: ((Math.atan2(bb, a) * 180) / Math.PI + 360) % 360 };
        };
        const surface = (token: string) => {
          const probe = document.createElement("i");
          probe.style.backgroundColor = `var(${token})`;
          document.body.append(probe);
          const c = rgbOf(getComputedStyle(probe).backgroundColor);
          probe.remove();
          return c;
        };
        const out: Row[] = [];
        for (const base of ["none", ...bases]) {
          for (const accent of ["none", ...accents]) {
            if (base === "none") html.removeAttribute("data-theme-base");
            else html.setAttribute("data-theme-base", base);
            if (accent === "none") html.removeAttribute("data-theme-accent");
            else html.setAttribute("data-theme-accent", accent);
            const shadow = getComputedStyle(input).boxShadow;
            if (shadow === "none") throw new Error(`no focus ring at base=${base} accent=${accent}`);
            // The first colour of the computed `box-shadow` is the 1px solid edge of `--dx-ring-field`.
            const edge = rgbOf(shadow.slice(0, shadow.indexOf(" 0px")));
            let worst = Infinity;
            let worstSurface = "";
            for (const token of ["--dx-background", "--dx-card", "--dx-popover"]) {
              const r = ratio(edge, surface(token));
              if (r < worst) [worst, worstSurface] = [r, token];
            }
            const raw = getComputedStyle(html).getPropertyValue("--ring");
            // Chromium serialises the declared colour as lab(); resolve it through the same canvas.
            const declared = raw.trim() ? toOklch(rgbOf(raw.trim())) : null;
            const sourceRgb = rgbOf(raw.trim() || (mode === "light" ? "oklch(0.708 0 0)" : "oklch(0.556 0 0)"));
            out.push({
              edgeRgb: edge,
              sourceRgb,
              primary: toOklch(surface("--dx-primary")),
              mode,
              base,
              accent,
              ratio: worst,
              worstSurface,
              ring: toOklch(edge),
              declared,
            });
          }
        }
        html.style.removeProperty("--ring");
        return out;
      },
      { mode, bases: BASES, accents: ACCENTS, override, contrast },
    );

  const hueGap = (a: number, b: number) => Math.min(Math.abs(a - b), 360 - Math.abs(a - b));
  const label = (r: Row) => `${r.mode} base=${r.base} accent=${r.accent}`;

  test("the sweep covers every base and accent the stylesheet defines", () => {
    expect(BASES).toHaveLength(9);
    expect(ACCENTS).toHaveLength(18);
  });

  for (const mode of ["light", "dark"] as const) {
    test(`${mode}, contrast mode: ring edge >= 3:1 on background, card and popover, default and every preset`, async ({ page }) => {
      const rows = await measure(page, mode);
      const worst = rows.reduce((a, b) => (b.ratio < a.ratio ? b : a));
      console.log(`[ring ${mode}] worst ${worst.ratio.toFixed(3)}:1 on ${worst.worstSurface} at ${label(worst)}`);
      const perBase = new Map<string, number>();
      for (const r of rows) perBase.set(r.base, Math.min(perBase.get(r.base) ?? Infinity, r.ratio));
      console.log(`[ring ${mode}] per base: ` + [...perBase].map(([b, v]) => `${b} ${v.toFixed(2)}`).join(", "));
      const bad = rows.filter((r) => r.ratio < NON_TEXT);
      expect(
        bad.map((r) => `${label(r)}: ${r.ratio.toFixed(2)}:1 on ${r.worstSurface}`).slice(0, 12),
        `${bad.length}/${rows.length} combinations under ${NON_TEXT}:1`,
      ).toEqual([]);
    });

    for (const contrast of [null, "wcag"] as const) {
      test(`${mode}, ${contrast ?? "default"}: with no preset and no accent the ring is neutral grey`, async ({ page }) => {
        const rows = (await measure(page, mode, undefined, contrast)).filter((r) => r.base === "none" && r.accent === "none");
        expect(rows.length).toBeGreaterThan(0);
        for (const r of rows) {
          expect(r.ring.c, `${label(r)}: chroma ${r.ring.c.toFixed(3)} (blue #2b7fff is ~0.2)`).toBeLessThan(NEUTRAL_CHROMA);
        }
      });
    }

    test(`${mode}, default: the ring is shadcn's exact value (the preset's own --ring, neutral's when none), under 3:1 in light`, async ({ page }) => {
      const rows = (await measure(page, mode, undefined, null)).filter((r) => r.accent === "none");
      // No preset: shadcn neutral's own ring, light oklch(0.708 0 0), dark oklch(0.556 0 0) (8-bit rounding: +-0.01).
      const none = rows.find((r) => r.base === "none" && r.accent === "none")!;
      expect(Math.abs(none.ring.l - (mode === "light" ? 0.708 : 0.556)), `${label(none)}: ring L ${none.ring.l.toFixed(3)}`).toBeLessThan(0.012);
      // Every base with no accent: the edge IS the declared `--ring`, not a clamped copy of it, to the byte.
      for (const r of rows.filter((r) => r.declared)) {
        expect(Math.abs(r.ring.l - r.declared!.l), `${label(r)}: ring L ${r.ring.l.toFixed(3)} vs --ring L ${r.declared!.l.toFixed(3)}`).toBeLessThan(0.012);
      }
      for (const r of rows) expect(r.edgeRgb, `${label(r)}: ring edge is the unclamped source`).toEqual(r.sourceRgb);
      const w = rows.reduce((a, b) => (b.ratio < a.ratio ? b : a));
      console.log(`[ring ${mode} default] worst ${w.ratio.toFixed(3)}:1 on ${w.worstSurface} at ${label(w)} (shadcn's own)`);
      if (mode === "light") {
        // shadcn's own grey ring is 2.3-2.6:1 on a white surface; that is the documented, allowed default.
        expect(w.ratio, "shadcn's light ring is under 3:1").toBeLessThan(NON_TEXT);
        expect(w.ratio, "and not weaker than shadcn's own 2.3:1 floor").toBeGreaterThanOrEqual(2.3);
      }
    });

    test(`${mode}: with no accent the ring keeps its base colour's hue`, async ({ page }) => {
      const rows = (await measure(page, mode)).filter((r) => r.accent === "none" && r.declared && r.declared.c >= 0.01);
      expect(rows.length).toBeGreaterThan(0);
      const bad = rows.filter((r) => hueGap(r.ring.h, r.declared!.h) > HUE_TOLERANCE);
      expect(
        bad.map((r) => `${label(r)}: ring hue ${r.ring.h.toFixed(0)} vs --ring hue ${r.declared!.h}`).slice(0, 12),
        `${bad.length}/${rows.length} combinations whose ring hue left the base's --ring hue`,
      ).toEqual([]);
      // The one named case: stone is a warm base (hue ~56) and keeps its warm ring when no accent is picked.
      const stone = rows.find((r) => r.base === "stone")!;
      expect(hueGap(stone.ring.h, 56), `stone ring hue ${stone.ring.h.toFixed(0)}`).toBeLessThan(HUE_TOLERANCE);
    });

    for (const contrast of [null, "shadcn", "wcag"] as const) {
      test(`${mode}, ${contrast ?? "default"}: every accent x base ring edge >= 3:1 on background, card and popover (row 385)`, async ({ page }) => {
        const rows = (await measure(page, mode, undefined, contrast)).filter((r) => r.accent !== "none");
        expect(rows).toHaveLength(BASES.length * ACCENTS.length + ACCENTS.length);
        const worst = rows.reduce((a, b) => (b.ratio < a.ratio ? b : a));
        console.log(`[accent ring ${mode} ${contrast ?? "default"}] worst ${worst.ratio.toFixed(3)}:1 on ${worst.worstSurface} at ${label(worst)}`);
        const bad = rows.filter((r) => r.ratio < NON_TEXT);
        expect(
          bad.map((r) => `${label(r)}: ${r.ratio.toFixed(2)}:1 on ${r.worstSurface}`).slice(0, 12),
          `${bad.length}/${rows.length} accent rings under ${NON_TEXT}:1`,
        ).toEqual([]);
      });
    }

    test(`${mode}: the accent ring tracks the accent's hue and chroma, and moves lightness only into its bound (row 385)`, async ({ page }) => {
      const rows = (await measure(page, mode)).filter((r) => r.accent !== "none");
      const chromatic = rows.filter((r) => r.primary.c >= CHROMATIC);
      expect(chromatic.length).toBeGreaterThanOrEqual(ACCENTS.length * BASES.length);
      const wrongHue = chromatic.filter((r) => hueGap(r.ring.h, r.primary.h) > ACCENT_HUE_TOLERANCE);
      expect(
        wrongHue.map((r) => `${label(r)}: ring hue ${r.ring.h.toFixed(0)} vs --primary hue ${r.primary.h.toFixed(0)}`).slice(0, 12),
        `${wrongHue.length}/${chromatic.length} accent rings left the accent's hue`,
      ).toEqual([]);
      // Chroma survives the clamp (gamut mapping may trim a little): never grey.
      const grey = chromatic.filter((r) => r.ring.c < r.primary.c * 0.6);
      expect(grey.map((r) => `${label(r)}: ring chroma ${r.ring.c.toFixed(3)} vs ${r.primary.c.toFixed(3)}`).slice(0, 12)).toEqual([]);
      // Lightness: light is at most the ceiling, dark at least the floor; an accent already inside moves not at all.
      for (const r of rows) {
        if (mode === "light") {
          expect(r.ring.l, `${label(r)}: light ring L ${r.ring.l.toFixed(3)} over the ceiling`).toBeLessThanOrEqual(Math.max(ACCENT_L_MAX, r.primary.l + 0.012));
          if (r.primary.l <= ACCENT_L_MAX) expect(Math.abs(r.ring.l - r.primary.l), `${label(r)}: an in-bound accent is passed through`).toBeLessThan(0.012);
        } else {
          expect(r.ring.l, `${label(r)}: dark ring L ${r.ring.l.toFixed(3)} under the floor`).toBeGreaterThanOrEqual(Math.min(ACCENT_L_MIN, r.primary.l - 0.012));
          if (r.primary.l >= ACCENT_L_MIN) expect(Math.abs(r.ring.l - r.primary.l), `${label(r)}: an in-bound accent is passed through`).toBeLessThan(0.012);
        }
      }
      // The named cases: blue under a warm base is blue, lime keeps its green and the docs accent its blue.
      const stoneBlue = rows.find((r) => r.base === "stone" && r.accent === "blue")!;
      expect(hueGap(stoneBlue.ring.h, 265), `stone + blue ring hue ${stoneBlue.ring.h.toFixed(0)}`).toBeLessThan(ACCENT_HUE_TOLERANCE);
      const docs = rows.find((r) => r.base === "none" && r.accent === "docs")!;
      expect(hueGap(docs.ring.h, 260), `docs ring hue ${docs.ring.h.toFixed(0)}`).toBeLessThan(ACCENT_HUE_TOLERANCE);
    });

    test(`${mode}: a page-level --ring never beats a picked accent (row 385)`, async ({ page }) => {
      const rows = await measure(page, mode, "oklch(0.7 0.2 30)");
      const red = rows.find((r) => r.base === "none" && r.accent === "blue")!;
      expect(hueGap(red.ring.h, red.primary.h), `blue accent over --ring orange: ring hue ${red.ring.h.toFixed(0)}`).toBeLessThan(ACCENT_HUE_TOLERANCE);
      // With no accent the host's --ring is still the source.
      const host = rows.find((r) => r.base === "none" && r.accent === "none")!;
      expect(hueGap(host.ring.h, 30), `no accent, --ring orange: ring hue ${host.ring.h.toFixed(0)}`).toBeLessThan(HUE_TOLERANCE);
    });

    test(`${mode}: a page-level --ring is honoured; only lightness is clamped, hue is kept`, async ({ page }) => {
      // A pasted shadcn theme's coloured ring: orange-red, and a pale green that the light clamp must darken.
      for (const [override, hue] of [
        ["oklch(0.7 0.2 30)", 30],
        ["oklch(0.9 0.08 150)", 150],
      ] as const) {
        const rows = (await measure(page, mode, override)).filter((r) => r.base === "none" && r.accent === "none");
        const r = rows[0];
        expect(hueGap(r.ring.h, hue), `${mode} --ring: ${override} -> ring hue ${r.ring.h.toFixed(0)}`).toBeLessThan(HUE_TOLERANCE);
        expect(r.ring.c, `${mode} --ring: ${override} keeps its chroma (${r.ring.c.toFixed(3)})`).toBeGreaterThan(0.04);
        expect(r.ratio, `${mode} --ring: ${override} -> ${r.ratio.toFixed(2)}:1 on ${r.worstSurface}`).toBeGreaterThanOrEqual(NON_TEXT);
      }
      // A ring that already reads is passed through, not recoloured toward grey.
      const deep = (await measure(page, mode, "oklch(0.45 0.15 264)")).find((r) => r.base === "none" && r.accent === "none")!;
      expect(deep.ring.l, "a deep ring keeps its lightness").toBeLessThan(0.5);
    });
  }

  test("light: `prefers-contrast: more` calibrates the ring by itself; `data-dx-contrast=\"shadcn\"` opts out", async ({ page }) => {
    await page.emulateMedia({ contrast: "more" });
    const calibrated = (await measure(page, "light", undefined, null)).find((r) => r.base === "none" && r.accent === "none")!;
    expect(calibrated.ratio, `media query on: ${calibrated.ratio.toFixed(2)}:1`).toBeGreaterThanOrEqual(NON_TEXT);
    const optedOut = (await measure(page, "light", undefined, "shadcn")).find((r) => r.base === "none" && r.accent === "none")!;
    expect(optedOut.ratio, `shadcn opt-out: ${optedOut.ratio.toFixed(2)}:1`).toBeLessThan(NON_TEXT);
    await page.emulateMedia({ contrast: "no-preference" });
    const off = (await measure(page, "light", undefined, null)).find((r) => r.base === "none" && r.accent === "none")!;
    expect(off.ratio).toBeLessThan(NON_TEXT);
  });
});
