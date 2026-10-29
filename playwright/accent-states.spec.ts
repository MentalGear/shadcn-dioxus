import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoFrame, demoRole, demoRoles, popupItem } from "./demo-frame";

/**
 * The picked accent on active and selected states (backlog row 389). With `<html data-theme-accent>` set, the
 * theme's slot family replaces what those states paint; with none set every slot is the colour its state paints
 * without an accent, to the byte.
 *
 * Construction under test (`preview/assets/dx-components-theme.css`, the `:root[data-theme-accent]` rule):
 *   --dx-primary-mark   the accent as a non-text mark (dot, drop line, range and progress fill): `--dx-primary`
 *                       with OKLCH L at most 0.58 in light, at least 0.62 in dark; no accent: `--dx-primary`.
 *   --dx-active-mark    the same for the Tabs bar; no accent: `--dx-foreground`.
 *   --dx-active-ink     the accent as text (an active label) = `--dx-primary-ink`; no accent: `--dx-foreground`.
 *   --dx-accent-tint    a highlighted / open / current SURFACE carrying the accent's hue (L 0.97 light, 0.269 dark,
 *                       chroma x 0.35); no accent: `--dx-accent`.
 *   --dx-accent-tint-selected  the selected step (dark: tint + 15% ink; light: the tint); no accent:
 *                       `--dx-accent-selected`.
 *   --dx-sidebar-accent-tint   the sidebar's twin; no accent: `--dx-sidebar-accent`.
 *   --dx-tint-step-ink  the ink the pressed / selected step mixes toward, hue missing so the mix keeps the
 *                       accent's hue; no accent: `--dx-foreground`.
 *
 * Two halves. (1) The TOKEN sweep, every base x accent x mode x contrast setting, measures contrast on the
 * surfaces the marks and tints sit on (marks >= 3:1 on page, card, popover and the muted track; ink >= 4.5:1;
 * foreground on a tint >= 7:1), that the hue tracks the accent, and that with no accent every slot equals its
 * plain source. (2) The WIRING tests open each touched component and compare its computed colour with the slot (and
 * with the plain expression when no accent is picked): the proof that the component reads the slot. "Equal" is the painted sRGB colour: `--muted` is written as hex and `--accent` as oklch.
 */

const NON_TEXT = 3;
const TEXT = 4.5;
const TINT_TEXT = 7;
const CHROMATIC = 0.05;
/** The mark keeps the accent's hue to within gamut mapping; a pale tint (L 0.97, an sRGB clip of blue and indigo) and a
 * dark-yellow ink (L <= 0.5, the `--dx-primary-ink` clamp) drift further. */
const HUE_TOLERANCE = 12;
const HUE_TOLERANCE_WIDE = 25;
const MARK_L_MAX = 0.58 + 0.012;
const MARK_L_MIN = 0.62 - 0.012;

const PRESETS_CSS = fs.readFileSync(path.join(__dirname, "..", "preview", "assets", "theme-presets.css"), "utf8");
const names = (axis: "base" | "accent") => [
  ...new Set([...PRESETS_CSS.matchAll(new RegExp(`\\[data-theme-${axis}="([a-z]+)"\\] \\{`, "g"))].map((m) => m[1])),
];
const BASES = names("base");
const ACCENTS = names("accent");

type Rgb = [number, number, number];
const lum = ([r, g, b]: Rgb) => {
  const f = (v: number) => ((v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
};
const ratio = (a: Rgb, b: Rgb) => {
  const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};
const toOklch = ([r, g, b]: Rgb) => {
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
const hueGap = (a: number, b: number) => Math.min(Math.abs(a - b), 360 - Math.abs(a - b));

/** Every token the sweep reads, as `key: custom property`. */
const TOKENS = {
  bg: "--dx-background",
  card: "--dx-card",
  popover: "--dx-popover",
  muted: "--dx-muted",
  sidebar: "--dx-sidebar",
  fg: "--dx-foreground",
  accentFg: "--dx-accent-foreground",
  sidebarAccentFg: "--dx-sidebar-accent-foreground",
  mutedFg: "--dx-muted-foreground",
  primary: "--dx-primary",
  primaryMark: "--dx-primary-mark",
  activeMark: "--dx-active-mark",
  activeInk: "--dx-active-ink",
  primaryInk: "--dx-primary-ink",
  tint: "--dx-accent-tint",
  tintSel: "--dx-accent-tint-selected",
  sidebarTint: "--dx-sidebar-accent-tint",
  accent: "--dx-accent",
  accentSel: "--dx-accent-selected",
  sidebarAccent: "--dx-sidebar-accent",
  drop: "--dx-drop-indicator",
  tableSel: "--dx-table-selected-background",
  ringColor: "--dx-ring-color",
  ringInk: "--dx-ring-ink",
  accentRingInk: "--dx-accent-ring-ink",
  stepInk: "--dx-tint-step-ink",
} as const;
type Key = keyof typeof TOKENS;

type Row = {
  mode: "light" | "dark";
  contrast: string;
  base: string;
  accent: string;
  rgb: Record<Key, Rgb>;
  /** Chromium's serialisation of the computed colour, for the "byte for byte" identity checks. */
  str: Record<Key, string>;
};
const label = (r: Row) => `${r.mode} ${r.contrast} base=${r.base} accent=${r.accent}`;

/** Installs `window.__acc`: colour probes and the theme switches, used by every test below. */
function installHelpers() {
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
  /** The computed value of `prop` for a throwaway element declaring `prop: expr`. */
  const probe = (expr: string, prop = "background-color") => {
    const i = document.createElement("i");
    i.style.setProperty(prop, expr);
    document.body.append(i);
    const v = getComputedStyle(i).getPropertyValue(prop);
    i.remove();
    return v;
  };
  const html = document.documentElement;
  const w = window as unknown as {
    __dxApplyMode: (m: string) => void;
    __dxTheme: (k: string, v: string | null) => void;
    __acc: unknown;
  };
  w.__acc = {
    rgbOf,
    probe,
    tokenRgb: (token: string) => rgbOf(probe(`var(${token})`)),
    tokenStr: (token: string) => probe(`var(${token})`),
    set(mode: string, base: string, accent: string, contrast: string | null = null) {
      w.__dxApplyMode(mode);
      w.__dxTheme("dx-contrast", contrast);
      if (base === "none") html.removeAttribute("data-theme-base");
      else html.setAttribute("data-theme-base", base);
      if (accent === "none") html.removeAttribute("data-theme-accent");
      else html.setAttribute("data-theme-accent", accent);
    },
  };
  const style = document.createElement("style");
  style.textContent = "*,*::before,*::after{transition:none!important;animation:none!important}";
  document.head.append(style);
}

const install = (page: Page) => page.evaluate(`(${installHelpers.toString()})()`);

test.describe("accent on active and selected states (row 389)", () => {
  test("the sweep covers every base and accent the stylesheet defines", () => {
    expect(BASES).toHaveLength(9);
    expect(ACCENTS).toHaveLength(18);
  });

  test.describe("token sweep", () => {
    test.beforeEach(async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/input/`);
      await install(page);
    });

    const sweep = (page: Page, mode: "light" | "dark", contrast: string | null) =>
      page.evaluate(
        ({ mode, bases, accents, contrast, tokens }) => {
          const A = (window as unknown as { __acc: { set: Function; tokenRgb: (t: string) => number[]; tokenStr: (t: string) => string } }).__acc;
          const out: unknown[] = [];
          for (const base of ["none", ...bases]) {
            for (const accent of ["none", ...accents]) {
              A.set(mode, base, accent, contrast);
              const rgb: Record<string, number[]> = {};
              const str: Record<string, string> = {};
              for (const [k, t] of Object.entries(tokens)) {
                rgb[k] = A.tokenRgb(t);
                str[k] = A.tokenStr(t);
              }
              out.push({ mode, contrast: contrast ?? "default", base, accent, rgb, str });
            }
          }
          return out;
        },
        { mode, bases: BASES, accents: ACCENTS, contrast, tokens: TOKENS },
      ) as Promise<Row[]>;

    for (const mode of ["light", "dark"] as const) {
      for (const contrast of [null, "wcag"] as const) {
        const title = `${mode}, ${contrast ?? "default"}`;

        test(`${title}: with no accent every slot is its plain source, byte for byte`, async ({ page }) => {
          const rows = (await sweep(page, mode, contrast)).filter((r) => r.accent === "none");
          expect(rows).toHaveLength(BASES.length + 1);
          const pairs: [Key, Key][] = [
            ["primaryMark", "primary"],
            ["activeMark", "fg"],
            ["activeInk", "fg"],
            ["tint", "accent"],
            ["tintSel", "accentSel"],
            ["sidebarTint", "sidebarAccent"],
            ["drop", "primary"],
            ["tableSel", "muted"],
            ["stepInk", "fg"],
          ];
          const bad: string[] = [];
          for (const r of rows) {
            for (const [slot, src] of pairs) {
              // `--muted` and `--accent` are one colour written in two notations (hex / oklch): compare what is painted.
              const same = slot === "tableSel" ? r.rgb[slot].join() === r.rgb[src].join() : r.str[slot] === r.str[src];
              if (!same) bad.push(`${label(r)}: ${slot} ${r.str[slot]} != ${src} ${r.str[src]}`);
            }
          }
          expect(bad.slice(0, 10), `${bad.length} slots differ from their plain source with no accent`).toEqual([]);
          // `--dx-accent-ring-ink` is declared only with an accent: undeclared, the Verified badge keeps its white.
          for (const r of rows) expect(r.str.accentRingInk, `${label(r)}: accent ring ink is undeclared`).toBe("rgba(0, 0, 0, 0)");
        });

        test(`${title}: the default (no preset, no accent) values are shadcn's own`, async ({ page }) => {
          const r = (await sweep(page, mode, contrast)).find((x) => x.base === "none" && x.accent === "none")!;
          const near = (a: Rgb, b: Rgb) => a.every((v, i) => Math.abs(v - b[i]) <= 1);
          const L = mode === "light";
          expect(near(r.rgb.activeMark, L ? [10, 10, 10] : [250, 250, 250]), `tabs bar ${r.rgb.activeMark}`).toBe(true);
          expect(near(r.rgb.primaryMark, L ? [23, 23, 23] : [229, 229, 229]), `dot, range, fill ${r.rgb.primaryMark}`).toBe(true);
          expect(near(r.rgb.tint, L ? [245, 245, 245] : [38, 38, 38]), `tint ${r.rgb.tint}`).toBe(true);
          expect(near(r.rgb.tableSel, L ? [245, 245, 245] : [38, 38, 38]), `table selected row ${r.rgb.tableSel}`).toBe(true);
          expect(near(r.rgb.tintSel, L ? [245, 245, 245] : [65, 65, 65]), `tint selected ${r.rgb.tintSel}`).toBe(true);
        });

        test(`${title}: every accent x base: marks >= 3:1 on page, card, popover and the muted track (and lightness moves only into its bound)`, async ({ page }) => {
          const rows = (await sweep(page, mode, contrast)).filter((r) => r.accent !== "none");
          expect(rows).toHaveLength((BASES.length + 1) * ACCENTS.length);
          let worst = { v: Infinity, at: "" };
          const bad: string[] = [];
          for (const r of rows) {
            for (const s of ["bg", "card", "popover", "muted"] as const) {
              const v = ratio(r.rgb.primaryMark, r.rgb[s]);
              if (v < worst.v) worst = { v, at: `${label(r)} on ${s}` };
              if (v < NON_TEXT) bad.push(`${label(r)}: mark ${v.toFixed(2)}:1 on ${s}`);
            }
            expect(r.rgb.activeMark, `${label(r)}: the Tabs bar mark is the same mark`).toEqual(r.rgb.primaryMark);
            expect(r.rgb.drop, `${label(r)}: the drop indicator is the same mark`).toEqual(r.rgb.primaryMark);
            const mark = toOklch(r.rgb.primaryMark);
            const primary = toOklch(r.rgb.primary);
            if (mode === "light") {
              expect(mark.l, `${label(r)}: light mark L ${mark.l.toFixed(3)} over the ceiling`).toBeLessThanOrEqual(Math.max(MARK_L_MAX, primary.l + 0.012));
            } else {
              expect(mark.l, `${label(r)}: dark mark L ${mark.l.toFixed(3)} under the floor`).toBeGreaterThanOrEqual(Math.min(MARK_L_MIN, primary.l - 0.012));
            }
          }
          console.log(`[accent mark ${title}] worst ${worst.v.toFixed(3)}:1 at ${worst.at}`);
          expect(bad.slice(0, 12), `${bad.length}/${rows.length * 4} marks under ${NON_TEXT}:1`).toEqual([]);
        });

        test(`${title}: every accent x base: ink >= 4.5:1 on page, card and popover; foreground on a tint >= 7:1`, async ({ page }) => {
          const rows = (await sweep(page, mode, contrast)).filter((r) => r.accent !== "none");
          const worst = { ink: Infinity, inkMuted: Infinity, tint: Infinity, tintSel: Infinity, sidebar: Infinity, mutedOnTint: Infinity, inkOnTint: Infinity, at: {} as Record<string, string> };
          const upd = (k: "ink" | "inkMuted" | "tint" | "tintSel" | "sidebar" | "mutedOnTint" | "inkOnTint", v: number, r: Row) => {
            if (v < worst[k]) {
              worst[k] = v;
              worst.at[k] = label(r);
            }
          };
          const bad: string[] = [];
          for (const r of rows) {
            for (const s of ["bg", "card", "popover"] as const) {
              const v = ratio(r.rgb.activeInk, r.rgb[s]);
              upd("ink", v, r);
              if (v < TEXT) bad.push(`${label(r)}: ink ${v.toFixed(2)}:1 on ${s}`);
            }
            upd("inkMuted", ratio(r.rgb.activeInk, r.rgb.muted), r);
            for (const f of ["fg", "accentFg"] as const) {
              const a = ratio(r.rgb[f], r.rgb.tint);
              const b = ratio(r.rgb[f], r.rgb.tintSel);
              upd("tint", a, r);
              upd("tintSel", b, r);
              if (a < TINT_TEXT) bad.push(`${label(r)}: ${f} on tint ${a.toFixed(2)}:1`);
              if (b < TINT_TEXT) bad.push(`${label(r)}: ${f} on tint-selected ${b.toFixed(2)}:1`);
            }
            for (const f of ["sidebarAccentFg"] as const) {
              const a = ratio(r.rgb[f], r.rgb.sidebarTint);
              upd("sidebar", a, r);
              if (a < TINT_TEXT) bad.push(`${label(r)}: sidebar accent foreground on the sidebar tint ${a.toFixed(2)}:1`);
            }
            // Secondary text on the range band of an adjacent month: no worse than on the muted fill it replaces.
            const mutedOnTint = ratio(r.rgb.mutedFg, r.rgb.tint);
            upd("mutedOnTint", mutedOnTint, r);
            const mutedOnMuted = ratio(r.rgb.mutedFg, r.rgb.muted);
            // (4.5:1 where that pair reaches it; otherwise within 0.3 of what shadcn's own muted foreground gives there.)
            if (mutedOnTint < (mutedOnMuted >= TEXT ? TEXT : mutedOnMuted - 0.3)) bad.push(`${label(r)}: muted foreground on the tint ${mutedOnTint.toFixed(2)}:1, on the muted fill it replaces ${mutedOnMuted.toFixed(2)}:1`);
            // An icon in the ink on a highlighted row.
            const inkIcon = ratio(r.rgb.activeInk, r.rgb.tint);
            upd("inkOnTint", inkIcon, r);
            if (inkIcon < NON_TEXT) bad.push(`${label(r)}: ink icon on tint ${inkIcon.toFixed(2)}:1`);
          }
          console.log(
            `[accent text ${title}] ink ${worst.ink.toFixed(2)} (${worst.at.ink}); ink on muted ${worst.inkMuted.toFixed(2)}; fg on tint ${worst.tint.toFixed(2)} (${worst.at.tint}); on tint-selected ${worst.tintSel.toFixed(2)} (${worst.at.tintSel}); sidebar ${worst.sidebar.toFixed(2)}; muted fg on tint ${worst.mutedOnTint.toFixed(2)} (${worst.at.mutedOnTint}); ink icon on tint ${worst.inkOnTint.toFixed(2)}`,
          );
          expect(bad.slice(0, 12), `${bad.length} text pairs under their bound`).toEqual([]);
        });

        test(`${title}: the selected Tag's ink >= 4.5:1 on its fill, the Verified icon >= 3:1, under every base x accent`, async ({ page }) => {
          const rows = await sweep(page, mode, contrast);
          let worstTag = Infinity;
          let worstIcon = Infinity;
          const bad: string[] = [];
          for (const r of rows) {
            const tag = ratio(r.rgb.ringInk, r.rgb.ringColor);
            worstTag = Math.min(worstTag, tag);
            if (tag < TEXT) bad.push(`${label(r)}: tag ink ${tag.toFixed(2)}:1`);
            if (r.accent !== "none") {
              const icon = ratio(r.rgb.accentRingInk, r.rgb.ringColor);
              worstIcon = Math.min(worstIcon, icon);
              if (icon < NON_TEXT) bad.push(`${label(r)}: verified icon ${icon.toFixed(2)}:1`);
            }
          }
          console.log(`[accent tag ${title}] tag ink worst ${worstTag.toFixed(2)}:1, verified icon worst ${worstIcon.toFixed(2)}:1`);
          expect(bad.slice(0, 12)).toEqual([]);
        });
      }

      test(`${mode}: the mark, ink and tint track the accent's hue; the tint steps from the popover as the neutral accent does`, async ({ page }) => {
        const all = await sweep(page, mode, null);
        const rows = all.filter((r) => r.accent !== "none");
        const chromatic = rows.filter((r) => toOklch(r.rgb.primary).c >= CHROMATIC);
        expect(chromatic.length).toBeGreaterThanOrEqual(ACCENTS.length * BASES.length);
        const bad: string[] = [];
        for (const r of chromatic) {
          const p = toOklch(r.rgb.primary);
          for (const k of ["primaryMark", "activeInk", "tint", "tintSel"] as const) {
            const o = toOklch(r.rgb[k]);
            if (hueGap(o.h, p.h) > (k === "primaryMark" ? HUE_TOLERANCE : HUE_TOLERANCE_WIDE)) bad.push(`${label(r)}: ${k} hue ${o.h.toFixed(0)} vs primary ${p.h.toFixed(0)}`);
          }
          const mark = toOklch(r.rgb.primaryMark);
          if (mark.c < p.c * 0.5) bad.push(`${label(r)}: mark chroma ${mark.c.toFixed(3)} vs ${p.c.toFixed(3)}`);
          const tint = toOklch(r.rgb.tint);
          if (tint.c < 0.008) bad.push(`${label(r)}: tint chroma ${tint.c.toFixed(3)} is not visible`);
        }
        expect(bad.slice(0, 12), `${bad.length}/${chromatic.length} accents left their hue`).toEqual([]);
        // The step from the popover (the highlight of a menu row) is at least what the neutral accent gives
        // (light 1.09, dark 1.19 on the white / #171717 popover), less 3%.
        const neutral = all.find((r) => r.base === "none" && r.accent === "none")!;
        const floor = ratio(neutral.rgb.accent, neutral.rgb.popover) * 0.97;
        let worst = Infinity;
        const weak: string[] = [];
        for (const r of rows.filter((x) => x.base === "none")) {
          const v = ratio(r.rgb.tint, r.rgb.popover);
          worst = Math.min(worst, v);
          if (v < floor) weak.push(`${label(r)}: tint vs popover ${v.toFixed(3)} < ${floor.toFixed(3)}`);
        }
        console.log(`[accent tint ${mode}] weakest step from the popover ${worst.toFixed(3)} (neutral ${(floor / 0.97).toFixed(3)})`);
        expect(weak.slice(0, 12)).toEqual([]);
        // Named cases: blue is blue, lime keeps its green.
        const blue = rows.find((r) => r.base === "none" && r.accent === "blue")!;
        expect(hueGap(toOklch(blue.rgb.tint).h, 262), "blue tint hue").toBeLessThan(HUE_TOLERANCE_WIDE);
        const lime = rows.find((r) => r.base === "none" && r.accent === "lime")!;
        expect(hueGap(toOklch(lime.rgb.primaryMark).h, 130), "lime mark hue").toBeLessThan(25);
      });
    }

    test("the slots take the accent's mark, a page-level --primary alone does not", async ({ page }) => {
      // A host that pastes shadcn's `--primary` without a picked accent keeps its plain look: the slots are untouched.
      const r = await page.evaluate(() => {
        const A = (window as unknown as { __acc: { set: Function; tokenStr: (t: string) => string } }).__acc;
        A.set("light", "none", "none");
        document.documentElement.style.setProperty("--primary", "oklch(0.9 0.2 100)");
        const out = { mark: A.tokenStr("--dx-primary-mark"), primary: A.tokenStr("--dx-primary"), tint: A.tokenStr("--dx-accent-tint"), accent: A.tokenStr("--dx-accent") };
        document.documentElement.style.removeProperty("--primary");
        return out;
      });
      expect(r.mark).toBe(r.primary);
      expect(r.tint).toBe(r.accent);
    });
  });

  /**
   * Wiring: a component's computed colour is the slot's value. `plain` is the expression the state paints without an accent;
   * `next` the slot expression. With no accent the element must equal `plain`; with an accent, `next`, and (a
   * chromatic accent, a state that changes) not `plain`.
   */
  type State = {
    label: string;
    /** JS expression evaluated in the page that returns the element (or null). */
    find: string;
    prop: string;
    pseudo?: string;
    plain: string;
    next: string;
    /** The state never changes colour by itself (it reads a slot that already follows the accent). */
    unchanged?: boolean;
  };
  type Check = { label: string; combo: string; got: string; want: string; wantPlain: string | null; plain: string };

  const COMBOS: [string, string][] = [
    ["none", "none"],
    ["none", "blue"],
    ["none", "lime"],
    ["none", "yellow"],
    ["stone", "violet"],
  ];

  async function checkStates(page: Page, states: State[]) {
    await install(page);
    const out = (await page.evaluate(
      ({ states, combos }) => {
        const A = (window as unknown as { __acc: { set: Function; probe: (e: string, p?: string) => string; rgbOf: (c: string) => number[] } }).__acc;
        const res: unknown[] = [];
        for (const mode of ["light", "dark"]) {
          for (const [base, accent] of combos) {
            A.set(mode, base, accent);
            for (const s of states) {
              const el = new Function(`return (${s.find})`)() as Element | null;
              if (!el) {
                res.push({ label: s.label, combo: `${mode} ${base}/${accent}`, got: "NO ELEMENT", want: "", wantPlain: null, plain: "" });
                continue;
              }
              const got = getComputedStyle(el, s.pseudo ?? null).getPropertyValue(s.prop);
              res.push({
                label: s.label,
                combo: `${mode} ${base}/${accent}`,
                got,
                want: A.probe(s.next, s.prop),
                wantPlain: accent === "none" ? A.probe(s.plain, s.prop) : null,
                plainSame:
                  accent === "none" &&
                  (got === A.probe(s.plain, s.prop) ||
                    (/color/.test(s.prop) && s.prop !== "accent-color" && A.rgbOf(got).join() === A.rgbOf(A.probe(s.plain, s.prop)).join())),
                plain: A.probe(s.plain, s.prop),
                unchanged: !!s.unchanged,
                accent,
              });
            }
          }
        }
        return res;
      },
      { states, combos: COMBOS },
    )) as (Check & { unchanged: boolean; accent: string })[];
    const bad: string[] = [];
    for (const c of out) {
      if (c.got === "NO ELEMENT") bad.push(`${c.label} ${c.combo}: element not found`);
      else if (c.got !== c.want) bad.push(`${c.label} ${c.combo}: painted ${c.got}, slot says ${c.want}`);
      else if (c.accent === "none" && !c.plainSame) bad.push(`${c.label} ${c.combo}: no accent paints ${c.got}, the plain expression gives ${c.wantPlain}`);
      else if (c.accent !== "none" && !c.unchanged && c.accent !== "none" && c.got === c.plain) bad.push(`${c.label} ${c.combo}: does not follow the accent (still ${c.plain})`);
    }
    expect(bad.slice(0, 12), `${bad.length}/${out.length} wiring checks failed`).toEqual([]);
  }

  test("Tabs: the line bar (horizontal and vertical) takes the mark, the default active label the ink", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tabs/`);
    await expect(demoRoles(page, "tab").first()).toBeVisible();
    const trig = (cond: string) =>
      `[...document.querySelectorAll('.dx-tabs-trigger[data-state="active"]')].find((e) => { const t = e.closest('.dx-tabs'); return ${cond}; })`;
    await checkStates(page, [
      {
        label: "line bar (docs DEMO/CODE tabs)",
        find: trig(`t.dataset.variant === 'line' && t.dataset.orientation !== 'vertical'`),
        prop: "background-color",
        pseudo: "::after",
        plain: "var(--dx-foreground)",
        next: "var(--dx-active-mark)",
      },
      {
        label: "vertical line bar",
        find: trig(`t.dataset.variant === 'line' && t.dataset.orientation === 'vertical'`),
        prop: "background-color",
        pseudo: "::after",
        plain: "var(--dx-foreground)",
        next: "var(--dx-active-mark)",
      },
      {
        label: "default active label",
        find: trig(`t.dataset.variant !== 'line'`),
        prop: "color",
        plain: "var(--dx-foreground)",
        next: "var(--dx-active-ink)",
      },
      {
        label: "line active label stays foreground",
        find: trig(`t.dataset.variant === 'line' && t.dataset.orientation !== 'vertical'`),
        prop: "color",
        plain: "var(--dx-foreground)",
        next: "var(--dx-foreground)",
        unchanged: true,
      },
    ]);
  });

  test("Navigation Menu: the current link (data-active) paints the selected tint, top-level and in a panel", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/navigation_menu/`);
    const trigger = demoRole(page, "button", 0);
    await expect(trigger).toBeVisible();
    await trigger.click();
    const link = popupItem(page, "menu", "link", 1).or(demoFrame(page).locator(".dx-navigation-menu-content .dx-navigation-menu-link").nth(1));
    await expect(link.first()).toBeVisible();
    await page.mouse.move(0, 0);
    await page.evaluate(() => {
      document.querySelectorAll(".dx-navigation-menu-link").forEach((a, i) => {
        if (i === 1) a.setAttribute("data-active", "true");
      });
    });
    await checkStates(page, [
      {
        label: "panel link data-active",
        find: `document.querySelector('.dx-navigation-menu-content .dx-navigation-menu-link[data-active="true"]')`,
        prop: "background-color",
        // A state with no rule of its own: with no accent it is the neutral selected fill (`--dx-accent-selected`).
        plain: "var(--dx-accent-selected)",
        next: "var(--dx-accent-tint-selected)",
      },
    ]);
  });

  test("Sidebar: the active row and the active sub-row paint the sidebar tint (the docs nav is the library's Sidebar)", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button/`);
    await expect(page.locator('.dx-sidebar-menu-button[data-active="true"]').first()).toBeVisible();
    await checkStates(page, [
      {
        label: "docs nav active row",
        find: `document.querySelector('.dx-sidebar-menu-button[data-active="true"]')`,
        prop: "background-color",
        plain: "var(--dx-sidebar-accent)",
        next: "var(--dx-sidebar-accent-tint)",
      },
    ]);
    // The sidebar block demo's nested menu: mark its first sub-row current (the demo itself has none).
    await gotoHydrated(page, `${BASE_URL}/component/block/sidebar/main/`);
    const sub = page.locator(".dx-sidebar-menu-sub-button").first();
    await expect(sub).toBeAttached();
    await sub.evaluate((el) => el.setAttribute("data-active", "true"));
    await checkStates(page, [
      {
        label: "active sub-row",
        find: `document.querySelector('.dx-sidebar-menu-sub-button[data-active="true"]')`,
        prop: "background-color",
        plain: "var(--dx-sidebar-accent)",
        next: "var(--dx-sidebar-accent-tint)",
      },
    ]);
  });

  test("Toggle, Toggle Group and Toolbar: pressed paints the tint", async ({ page }) => {
    const pressed = "color-mix(in var(--dx-hover-mix-space), var(--dx-accent-tint), var(--dx-tint-step-ink) 15%)";
    const pressedPlain = "color-mix(in var(--dx-hover-mix-space), var(--dx-accent), var(--dx-foreground) 15%)";
    await gotoHydrated(page, `${BASE_URL}/component/toggle/`);
    await demoFrame(page).locator('.dx-toggle[data-state="off"]').first().click();
    await page.mouse.move(0, 0);
    await expect(demoFrame(page).locator('.dx-toggle[data-state="on"]').first()).toBeVisible();
    await checkStates(page, [{ label: "toggle on", find: `document.querySelector('#component-preview-frame .dx-toggle[data-state="on"]')`, prop: "background-color", plain: pressedPlain, next: pressed }]);

    await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
    await demoFrame(page).locator('.dx-toggle-group-item[data-state="off"]').first().click();
    await page.mouse.move(0, 0);
    await expect(demoFrame(page).locator('.dx-toggle-group-item[data-state="on"]').first()).toBeVisible();
    await checkStates(page, [
      { label: "toggle group item on", find: `document.querySelector('#component-preview-frame .dx-toggle-group-item[data-state="on"]')`, prop: "background-color", plain: pressedPlain, next: pressed },
    ]);

    await gotoHydrated(page, `${BASE_URL}/component/toolbar/`);
    const pressedButton = demoFrame(page).locator('.dx-toolbar-button[data-state="on"], .dx-toolbar-button[aria-pressed="true"]');
    if ((await pressedButton.count()) === 0) await demoFrame(page).locator(".dx-toolbar-button[aria-pressed], .dx-toolbar-button[data-state]").first().click();
    await page.mouse.move(0, 0);
    await expect(pressedButton.first()).toBeVisible();
    await checkStates(page, [
      {
        label: "toolbar pressed",
        find: `document.querySelector('#component-preview-frame .dx-toolbar-button[data-state="on"], #component-preview-frame .dx-toolbar-button[aria-pressed="true"]')`,
        prop: "background-color",
        plain: "var(--dx-accent-selected)",
        next: "var(--dx-accent-tint-selected)",
      },
    ]);
  });

  test("Tag Group: the selected tag's ink is the theme's ring ink, its fill the ring colour", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tag_group/`);
    const tag = demoFrame(page).locator(".dx-tag-group-tag").first();
    await tag.click();
    await page.mouse.move(0, 0);
    await expect(demoFrame(page).locator('.dx-tag-group-tag[data-selected="true"]').first()).toBeVisible();
    const sel = `document.querySelector('.dx-tag-group-tag[data-selected="true"]')`;
    await checkStates(page, [
      { label: "selected tag fill", find: sel, prop: "background-color", plain: "var(--dx-ring-color)", next: "var(--dx-ring-color)", unchanged: true },
      {
        label: "selected tag ink",
        find: sel,
        prop: "color",
        plain: "oklch(from var(--dx-ring-color) clamp(0.05, calc((0.575 - l) * 1000 + 0.05), 0.99) min(c, 0.014) h)",
        next: "var(--dx-ring-ink)",
        unchanged: true,
      },
    ]);
  });

  test("Carousel, Slider, Progress: the active dot, the range and the fill paint the mark", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/carousel/`);
    await expect(page.locator(".dx-carousel-indicator[aria-selected='true'], .dx-carousel-picker-dot[data-active='true']").first()).toBeAttached();
    await checkStates(page, [
      {
        label: "carousel active dot",
        find: `document.querySelector('.dx-carousel-indicator[aria-selected="true"], .dx-carousel-picker-dot[data-active="true"]')`,
        prop: "background-color",
        plain: "var(--dx-primary)",
        next: "var(--dx-primary-mark)",
      },
    ]);
    await gotoHydrated(page, `${BASE_URL}/component/slider/`);
    await expect(demoFrame(page).locator(".dx-slider-range").first()).toBeAttached();
    await checkStates(page, [
      { label: "slider range", find: `document.querySelector('.dx-slider-range')`, prop: "background-color", plain: "var(--dx-primary)", next: "var(--dx-primary-mark)" },
    ]);
    await gotoHydrated(page, `${BASE_URL}/component/progress/`);
    await expect(demoFrame(page).locator(".dx-progress-indicator").first()).toBeAttached();
    await checkStates(page, [
      { label: "progress fill", find: `document.querySelector('.dx-progress-indicator')`, prop: "background-color", plain: "var(--dx-primary)", next: "var(--dx-primary-mark)" },
    ]);
  });

  test("Menu, Select, Combobox, Command: the highlighted row paints the tint, the check the ink", async ({ page }) => {
    // Dropdown menu: hover the first item of the open menu.
    await gotoHydrated(page, `${BASE_URL}/component/dropdown_menu/`);
    await demoRole(page, "button", 0).click();
    const item = popupItem(page, "menu", "menuitem", 0);
    await expect(item).toBeVisible();
    await item.hover();
    await checkStates(page, [
      {
        label: "dropdown menu highlighted item",
        find: `document.querySelector('[role="menu"] [data-dx-menu-item]:hover')`,
        prop: "background-color",
        plain: "var(--dx-accent)",
        next: "var(--dx-accent-tint)",
      },
    ]);
    await page.keyboard.press("Escape");

    await gotoHydrated(page, `${BASE_URL}/component/select/`);
    await demoRole(page, "combobox", 0).click();
    await popupItem(page, "listbox", "option", 0).click();
    await demoRole(page, "combobox", 0).click();
    const option = popupItem(page, "listbox", "option", 1);
    await expect(option).toBeVisible();
    await option.hover();
    await checkStates(page, [
      {
        label: "select highlighted option",
        find: `document.querySelector('.dx-select-option:hover')`,
        prop: "background-color",
        plain: "var(--dx-accent)",
        next: "var(--dx-accent-tint)",
      },
      {
        label: "select check",
        find: `document.querySelector('.dx-select-option-indicator')`,
        prop: "color",
        plain: "var(--dx-foreground)",
        next: "var(--dx-active-ink)",
      },
    ]);
    await page.keyboard.press("Escape");

    await gotoHydrated(page, `${BASE_URL}/component/command/`);
    await demoFrame(page, "inline").locator(".dx-command-item").nth(1).hover();
    await expect(demoFrame(page, "inline").locator('.dx-command-item[data-highlighted="true"]').first()).toBeVisible();
    await checkStates(page, [
      {
        label: "command highlighted item",
        find: `document.querySelector('#component-preview-frame-inline .dx-command-item[data-highlighted="true"]')`,
        prop: "background-color",
        plain: "var(--dx-accent)",
        next: "var(--dx-accent-tint)",
      },
    ]);
  });

  test("Combobox: the highlighted option paints the tint, the check the ink (the muted foreground with no accent)", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/combobox/`);
    await demoRole(page, "combobox", 0).click();
    await popupItem(page, "listbox", "option", 0).click();
    await demoRole(page, "combobox", 0).click();
    await expect(popupItem(page, "listbox", "option", 0)).toBeVisible();
    await page.keyboard.press("ArrowDown");
    await checkStates(page, [
      {
        label: "combobox highlighted option",
        find: `document.querySelector('.dx-combobox-option[data-highlighted="true"]')`,
        prop: "background-color",
        plain: "var(--dx-accent)",
        next: "var(--dx-accent-tint)",
      },
      {
        label: "combobox check",
        find: `document.querySelector('.dx-combobox-option-indicator')`,
        prop: "color",
        plain: "var(--dx-muted-foreground)",
        next: "var(--dx-accent-ink, var(--dx-muted-foreground))",
      },
    ]);
  });

  test("Menubar: the open trigger paints the tint", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/menubar/`);
    await demoFrame(page).locator(".dx-menubar-trigger").first().click();
    await page.mouse.move(0, 0);
    await expect(page.locator('.dx-menubar-menu[data-state="open"] .dx-menubar-trigger').first()).toBeVisible();
    await checkStates(page, [
      {
        label: "menubar open trigger",
        find: `document.querySelector('.dx-menubar-menu[data-state="open"] .dx-menubar-trigger')`,
        prop: "background-color",
        plain: "var(--dx-muted)",
        next: "var(--dx-accent-tint)",
      },
    ]);
  });

  test("Calendar: the range band paints the tint", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/calendar/`);
    const frame = demoFrame(page, "range");
    const days = frame.locator('.dx-calendar-grid-cell[data-month="current"]');
    await expect(days.first()).toBeVisible();
    await days.nth(4).click();
    await days.nth(11).click();
    await page.mouse.move(0, 0);
    await expect(frame.locator('.dx-calendar-grid-cell[data-selection-between="true"]').first()).toBeVisible();
    await checkStates(page, [
      {
        label: "range band",
        find: `document.querySelector('.dx-calendar-grid-cell[data-month="current"][data-selection-between="true"]')`,
        prop: "background-color",
        plain: "var(--dx-muted)",
        next: "var(--dx-accent-tint)",
      },
      {
        label: "range end cell back",
        find: `document.querySelector('td:has(.dx-calendar-grid-cell[data-selection-end="true"])')`,
        prop: "background-color",
        plain: "var(--dx-muted)",
        next: "var(--dx-accent-tint)",
      },
      {
        label: "selected day stays the primary fill",
        find: `document.querySelector('.dx-calendar-grid-cell[data-month="current"][data-selection-start="true"]')`,
        prop: "background-color",
        plain: "var(--dx-primary)",
        next: "var(--dx-primary)",
        unchanged: true,
      },
    ]);
  });

  test("Table: the selected row takes the theme slot", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/table/`);
    // The demo has no selection: mark a body row selected (the sheet's rule is `[data-state="selected"]`).
    const row = demoFrame(page).locator("tbody tr.dx-table-row").first();
    await expect(row).toBeVisible();
    await row.evaluate((el) => el.setAttribute("data-state", "selected"));
    await page.mouse.move(0, 0);
    await checkStates(page, [
      {
        label: "selected table row",
        find: `document.querySelector('#component-preview-frame tr[data-state="selected"]')`,
        prop: "background-color",
        plain: "var(--dx-muted)",
        next: "var(--dx-accent-tint)",
      },
    ]);
  });

  test("Pagination: the current page takes the ink and the mark border", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/pagination/`);
    await expect(demoFrame(page).locator('.dx-pagination-link[data-active="true"]').first()).toBeVisible();
    await page.mouse.move(0, 0);
    const find = `document.querySelector('.dx-pagination-link[data-active="true"]')`;
    await checkStates(page, [
      { label: "pagination current label", find, prop: "color", plain: "var(--dx-foreground)", next: "var(--dx-active-ink)" },
      {
        label: "pagination current border",
        find,
        prop: "border-top-color",
        plain: "var(--light, var(--dx-border)) var(--dark, var(--dx-input))",
        next: "var(--dx-accent-mark, var(--light, var(--dx-border)) var(--dark, var(--dx-input)))",
      },
    ]);
  });

  test("Docs chrome: sort toggle, charts tab, accent-color and ::selection follow the accent", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/charts/`);
    await expect(page.locator('.dx-charts-tab[aria-current="page"]').first()).toBeVisible();
    await checkStates(page, [
      {
        label: "charts tab current",
        find: `document.querySelector('.dx-charts-tab[aria-current="page"]')`,
        prop: "background-color",
        plain: "var(--dx-muted)",
        next: "var(--dx-accent-tint)",
      },
      {
        label: "root accent-color",
        find: `document.documentElement`,
        prop: "accent-color",
        plain: "auto",
        next: "var(--dx-accent-mark, auto)",
      },
    ]);
    // ::selection is a highlight pseudo: with an accent it is the mark at 30%; with none it is the browser's.
    const sel = await page.evaluate(() => {
      const A = (window as unknown as { __acc: { set: Function; probe: (e: string, p?: string) => string } }).__acc;
      const read = (mode: string, accent: string) => {
        A.set(mode, "none", accent);
        return getComputedStyle(document.body, "::selection").backgroundColor;
      };
      const want = (mode: string) => {
        A.set(mode, "none", "blue");
        return A.probe("color-mix(in oklab, var(--dx-accent-mark) 30%, transparent)");
      };
      return {
        lightBlue: read("light", "blue"),
        lightWant: want("light"),
        darkBlue: read("dark", "blue"),
        darkWant: want("dark"),
        none: read("light", "none"),
      };
    });
    expect(sel.lightBlue).toBe(sel.lightWant);
    expect(sel.darkBlue).toBe(sel.darkWant);
    expect(sel.none, "no accent: the browser's own selection colour").not.toBe(sel.lightBlue);

    await gotoHydrated(page, `${BASE_URL}/component/button/`);
    await install(page);
    const pill = await page.evaluate(() => {
      const A = (window as unknown as { __acc: { set: Function; probe: (e: string, p?: string) => string } }).__acc;
      const el = document.querySelector(
        'html:not([data-nav-sort="category"]) .dx-nav-sort .dx-nav-sort-option[data-nav-sort-option="az"]',
      ) as HTMLElement | null;
      if (!el) return null;
      const out: Record<string, string[]> = {};
      for (const mode of ["light", "dark"]) {
        for (const accent of ["none", "blue"]) {
          A.set(mode, "none", accent);
          out[`${mode} ${accent}`] = [getComputedStyle(el).color, A.probe("var(--dx-foreground)", "color"), A.probe("var(--dx-active-ink)", "color")];
        }
      }
      return out;
    });
    expect(pill, "the docs nav's sort toggle").not.toBeNull();
    for (const [k, [got, fg, ink]] of Object.entries(pill!)) {
      expect(got, `sort toggle pressed label, ${k}`).toBe(k.endsWith("none") ? fg : ink);
    }
  });

  test("Block profile: the Verified icon takes the ring ink under an accent and stays white with none", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/`);
    const badge = page.locator('[style*="--dx-accent-ring-ink"]').first();
    await expect(badge).toBeAttached();
    await checkStates(page, [
      {
        label: "verified badge",
        find: `document.querySelector('[style*="--dx-accent-ring-ink"]')`,
        prop: "color",
        plain: "white",
        next: "var(--dx-accent-ring-ink, white)",
      },
    ]);
  });
});
