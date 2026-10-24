import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * Contrast mode (`<html data-dx-contrast="wcag|shadcn">`, `preview/assets/dx-components-theme.css`,
 * "Contrast mode"): the theme's colours are shadcn's exact values by default, and ONE switch turns on a
 * calibrated, WCAG-conforming set. This spec pins both halves and the switch itself:
 *
 *   default (no attribute, no `prefers-contrast`)  shadcn-exact: the clamped roles equal their `-source`
 *                                                   (the preset's own value), so a preset's colour is not moved;
 *   `data-dx-contrast="wcag"`                       every calibrated role meets its threshold in every base,
 *                                                   in both modes, on the REAL controls where one exists;
 *   `prefers-contrast: more`                        turns the calibrated set on by itself,
 *   `data-dx-contrast="shadcn"`                     opts out of that media query;
 *   fixed for everyone, in both modes               the error Toast, `--dx-success`, the docs link blue.
 *
 * Roles (the colour audit's proposals P1, P6, P9; the ring, P3, is in ring-contrast.spec.ts):
 *   P1  `--dx-muted-foreground` >= 4.5:1 on background / card / popover / muted / accent / secondary / sidebar
 *       (light; dark already passes at >= 5.56:1);
 *   P9  the Input / Checkbox / Radio edge and the unchecked Switch track >= 3:1 against background, card and
 *       popover, light and dark (WCAG 1.4.11);
 *   P6  `--dx-sidebar-primary-foreground` >= 4.5:1 on `--dx-sidebar-primary` in dark, every base x accent.
 *
 * Method: the base and accent names are READ FROM `theme-presets.css`; a combination is applied as the picker
 * does (`data-theme-base` / `-accent` on <html>, the mode through `window.__dxApplyMode`, the contrast through
 * `window.__dxTheme('dx-contrast', ...)`); colours are resolved through a 1x1 canvas, a translucent layer
 * composited over what is under it (the field fill and edge over the page in dark).
 */

const AA = 4.5;
const NON_TEXT = 3;

const PRESETS_CSS = fs.readFileSync(path.join(__dirname, "..", "preview", "assets", "theme-presets.css"), "utf8");
const names = (axis: "base" | "accent") => [
  ...new Set([...PRESETS_CSS.matchAll(new RegExp(`\\[data-theme-${axis}="([a-z]+)"\\] \\{`, "g"))].map((m) => m[1])),
];
const BASES = names("base");
const ACCENTS = names("accent");

type RGB = [number, number, number];
type Toolkit = {
  rgbOf: (css: string, under?: RGB) => RGB;
  lum: (c: RGB) => number;
  ratio: (a: RGB, b: RGB) => number;
  token: (name: string, under?: RGB) => RGB;
  alphaOf: (css: string) => number;
  apply: (mode: string, contrast: string | null, base: string, accent: string) => void;
};

/** Installs `window.__cm`, the in-page colour toolkit. Serialised into the page, so it closes over nothing. */
function installToolkit() {
  const cv = document.createElement("canvas");
  cv.width = cv.height = 1;
  const cx = cv.getContext("2d", { willReadFrequently: true })!;
  const rgbOf = (css: string, under: [number, number, number] = [255, 255, 255]): [number, number, number] => {
    cx.clearRect(0, 0, 1, 1);
    cx.fillStyle = `rgb(${under[0]},${under[1]},${under[2]})`;
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
  const token = (name: string, under: [number, number, number] = [255, 255, 255]) => {
    const probe = document.createElement("i");
    probe.style.backgroundColor = `var(${name})`;
    document.body.append(probe);
    const c = rgbOf(getComputedStyle(probe).backgroundColor, under);
    probe.remove();
    return c;
  };
  const alphaOf = (css: string): number => {
    const a = /(?:rgba?|color|oklch|lab|lch|oklab)\(.*?\/\s*([\d.]+)(%?)\s*\)/.exec(css);
    if (!a) return 1;
    return a[2] ? Number(a[1]) / 100 : Number(a[1]);
  };
  const html = document.documentElement;
  const apply = (mode: string, contrast: string | null, base: string, accent: string) => {
    const w = window as unknown as { __dxApplyMode: (m: string) => void; __dxTheme: (k: string, v: string | null) => void };
    w.__dxApplyMode(mode);
    w.__dxTheme("dx-contrast", contrast);
    if (base === "none") html.removeAttribute("data-theme-base");
    else html.setAttribute("data-theme-base", base);
    if (accent === "none") html.removeAttribute("data-theme-accent");
    else html.setAttribute("data-theme-accent", accent);
  };
  (window as unknown as { __cm: Toolkit }).__cm = { rgbOf, lum, ratio, token, alphaOf, apply };
}

async function open(page: Page, route: string) {
  await gotoHydrated(page, `${BASE_URL}${route}`);
  await page.addStyleTag({ content: "*,*::before,*::after{transition:none!important;animation:none!important}" });
  await page.evaluate(installToolkit);
}

const SURFACES = ["--dx-background", "--dx-card", "--dx-popover"];
const MUTED_SURFACES = [...SURFACES, "--dx-muted", "--dx-accent", "--dx-secondary", "--dx-sidebar-background", "--dx-sidebar-accent"];

type Probe = { label: string; ratio: number };
const worst = (rows: Probe[]) => rows.reduce((a, b) => (b.ratio < a.ratio ? b : a));
const fmt = (rows: Probe[]) => rows.map((r) => `${r.label}: ${r.ratio.toFixed(2)}:1`);

test("the sweep covers every base and accent the stylesheet defines", () => {
  expect(BASES).toHaveLength(9);
  expect(ACCENTS).toHaveLength(18);
});

// ---------------------------------------------------------------------------------------------------------
// P1: muted text
// ---------------------------------------------------------------------------------------------------------
test.describe("muted-foreground (P1)", () => {
  test.beforeEach(async ({ page }) => open(page, "/component/button/"));

  /** Light, every base (no accent: an accent never sets muted-foreground), the contrast attribute as given. */
  const sweep = (page: Page, mode: "light" | "dark", contrast: string | null) =>
    page.evaluate(
      ({ mode, contrast, bases, surfaces }) => {
        const cm = (window as unknown as { __cm: Toolkit }).__cm;
        const out: { label: string; ratio: number; value: RGB; declared: RGB | null }[] = [];
        for (const base of ["none", ...bases]) {
          cm.apply(mode, contrast, base, "none");
          const fg = cm.token("--dx-muted-foreground");
          const declaredCss = getComputedStyle(document.documentElement).getPropertyValue("--muted-foreground").trim();
          const declared = declaredCss ? cm.rgbOf(declaredCss) : null;
          for (const s of surfaces) out.push({ label: `${mode} base=${base} on ${s}`, ratio: cm.ratio(fg, cm.token(s)), value: fg, declared });
        }
        return out;
      },
      { mode, contrast, bases: BASES, surfaces: MUTED_SURFACES },
    );

  test("wcag, light: every base reads >= 4.5:1 on background, card, popover, muted, accent, secondary and sidebar", async ({ page }) => {
    const rows = await sweep(page, "light", "wcag");
    const w = worst(rows);
    console.log(`[muted-foreground wcag light] worst ${w.ratio.toFixed(3)}:1 at ${w.label}`);
    const bad = rows.filter((r) => r.ratio < AA);
    expect(fmt(bad).slice(0, 12), `${bad.length}/${rows.length} pairs under ${AA}:1`).toEqual([]);
  });

  test("wcag, dark: >= 4.5:1 everywhere", async ({ page }) => {
    const rows = await sweep(page, "dark", "wcag");
    const bad = rows.filter((r) => r.ratio < AA);
    expect(fmt(bad).slice(0, 12)).toEqual([]);
  });

  test("default, light: shadcn-exact (each preset's own --muted-foreground, no clamp); the gap is documented, not hidden", async ({ page }) => {
    const rows = await sweep(page, "light", null);
    const w = worst(rows);
    console.log(`[muted-foreground default light] worst ${w.ratio.toFixed(3)}:1 at ${w.label} (shadcn's own value)`);
    for (const r of rows.filter((r) => r.declared)) {
      for (let i = 0; i < 3; i++) expect(Math.abs(r.value[i] - r.declared![i]), `${r.label}: ${r.value} vs --muted-foreground ${r.declared}`).toBeLessThanOrEqual(1);
    }
    // The point of the switch: shadcn's own value fails on some surface (olive, 3.86:1), so there is something to calibrate.
    expect(w.ratio, "shadcn's own muted-foreground is < 4.5:1 on a muted surface in some base").toBeLessThan(AA);
  });

  test("the library's own default muted text (#6e6e6e) is the same in both modes of the switch", async ({ page }) => {
    const both = await page.evaluate(() => {
      const cm = (window as unknown as { __cm: Toolkit }).__cm;
      const r = (c: string | null) => {
        cm.apply("light", c, "none", "none");
        return cm.token("--dx-muted-foreground");
      };
      return [r(null), r("shadcn"), r("wcag")];
    });
    expect(both[0]).toEqual(both[1]);
    expect(both[0]).toEqual(both[2]);
  });
});

// ---------------------------------------------------------------------------------------------------------
// P9: input / checkbox / radio edge and the unchecked switch track, on the real controls
// ---------------------------------------------------------------------------------------------------------
const CONTROLS = [
  { name: "input", route: "/component/input/", selector: "input.dx-input:not(:disabled)", pseudo: "", track: false },
  { name: "checkbox", route: "/component/checkbox/", selector: '.dx-checkbox[data-state="unchecked"]:not([data-disabled="true"])', pseudo: "", track: false },
  { name: "radio", route: "/component/radio_group/", selector: ".dx-radio-group-item:not([data-state='checked'])", pseudo: "::before", track: false },
  { name: "switch (unchecked track)", route: "/component/switch/", selector: '.dx-switch[data-state="unchecked"]', pseudo: "", track: true },
] as const;

test.describe("field edges and the unchecked switch track (P9)", () => {
  for (const control of CONTROLS) {
    /** The control's edge (or track) composited over each surface, against that surface. */
    const sweep = (page: Page, mode: "light" | "dark", contrast: string | null) =>
      page.evaluate(
        ({ mode, contrast, bases, surfaces, sel, pseudo, track }) => {
          const cm = (window as unknown as { __cm: Toolkit }).__cm;
          const el = document.querySelector(sel) as HTMLElement | null;
          if (!el) throw new Error(`no element for ${sel}`);
          const out: { label: string; ratio: number }[] = [];
          for (const base of ["none", ...bases]) {
            cm.apply(mode, contrast, base, "none");
            const cs = getComputedStyle(el, pseudo || null);
            const fill = cs.backgroundColor;
            const shadow = cs.boxShadow;
            for (const s of surfaces) {
              const surface = cm.token(s);
              const withFill = cm.rgbOf(fill, surface);
              if (track) {
                out.push({ label: `${mode} base=${base} on ${s}`, ratio: cm.ratio(withFill, surface) });
              } else {
                if (shadow === "none") throw new Error(`no edge on ${sel} at base=${base}`);
                // The first colour of the computed `box-shadow` is the 1px inset edge, painted over the fill.
                const edge = cm.rgbOf(shadow.slice(0, shadow.indexOf(" 0px")), withFill);
                out.push({ label: `${mode} base=${base} on ${s}`, ratio: cm.ratio(edge, surface) });
              }
            }
          }
          return out;
        },
        { mode, contrast, bases: BASES, surfaces: SURFACES, sel: control.selector, pseudo: control.pseudo, track: control.track },
      );

    test.describe(control.name, () => {
      test.beforeEach(async ({ page }) => open(page, control.route));

      for (const mode of ["light", "dark"] as const) {
        test(`wcag, ${mode}: >= 3:1 on background, card and popover, default and every base`, async ({ page }) => {
          const rows = await sweep(page, mode, "wcag");
          const w = worst(rows);
          console.log(`[${control.name} wcag ${mode}] worst ${w.ratio.toFixed(3)}:1 at ${w.label}`);
          const bad = rows.filter((r) => r.ratio < NON_TEXT);
          expect(fmt(bad).slice(0, 12), `${bad.length}/${rows.length} pairs under ${NON_TEXT}:1`).toEqual([]);
        });

        test(`default, ${mode}: shadcn-exact, i.e. the edge is shadcn's own (under 3:1, by design)`, async ({ page }) => {
          const rows = await sweep(page, mode, null);
          const shadcn = await sweep(page, mode, "shadcn");
          const w = worst(rows);
          console.log(`[${control.name} default ${mode}] worst ${w.ratio.toFixed(3)}:1 at ${w.label} (shadcn's own value)`);
          // No attribute and `shadcn` are the same thing without a media query.
          expect(rows.map((r) => r.ratio.toFixed(2))).toEqual(shadcn.map((r) => r.ratio.toFixed(2)));
          // shadcn's edge is the faint #e5e5e5 / white-15% line: far under 3:1. If this ever reaches 3:1 the default drifted.
          expect(w.ratio, "the default edge must stay shadcn's").toBeLessThan(NON_TEXT);
        });
      }
    });
  }
});

// ---------------------------------------------------------------------------------------------------------
// P6: sidebar primary ink in dark
// ---------------------------------------------------------------------------------------------------------
test.describe("sidebar-primary-foreground (P6)", () => {
  test.beforeEach(async ({ page }) => open(page, "/component/sidebar/"));

  const sweep = (page: Page, mode: "light" | "dark", contrast: string | null) =>
    page.evaluate(
      ({ mode, contrast, bases, accents }) => {
        const cm = (window as unknown as { __cm: Toolkit }).__cm;
        const out: { label: string; ratio: number }[] = [];
        for (const base of ["none", ...bases]) {
          for (const accent of ["none", ...accents]) {
            cm.apply(mode, contrast, base, accent);
            const fill = cm.token("--dx-sidebar-primary");
            const ink = cm.token("--dx-sidebar-primary-foreground", fill);
            out.push({ label: `${mode} base=${base} accent=${accent}`, ratio: cm.ratio(ink, fill) });
          }
        }
        return out;
      },
      { mode, contrast, bases: BASES, accents: ACCENTS },
    );

  test("wcag, dark: >= 4.5:1 on --dx-sidebar-primary for every base x accent", async ({ page }) => {
    const rows = await sweep(page, "dark", "wcag");
    const w = worst(rows);
    console.log(`[sidebar-primary-foreground wcag dark] worst ${w.ratio.toFixed(3)}:1 at ${w.label}`);
    const bad = rows.filter((r) => r.ratio < AA);
    expect(fmt(bad).slice(0, 12), `${bad.length}/${rows.length} states under ${AA}:1`).toEqual([]);
  });

  test("default, dark: shadcn-exact (the Blue accent's #eff6ff on #2b7fff stays 3.5:1)", async ({ page }) => {
    const rows = await sweep(page, "dark", null);
    const w = worst(rows);
    console.log(`[sidebar-primary-foreground default dark] worst ${w.ratio.toFixed(3)}:1 at ${w.label}`);
    const blue = rows.find((r) => r.label === "dark base=neutral accent=blue")!;
    expect(blue.ratio).toBeLessThan(AA);
  });
});

// ---------------------------------------------------------------------------------------------------------
// The switch: attribute, media query, opt-out
// ---------------------------------------------------------------------------------------------------------
test.describe("the switch", () => {
  test.beforeEach(async ({ page }) => open(page, "/component/input/"));

  /** The three tokens the switch moves, in light, on the neutral base: input edge, muted text, ring. */
  const read = (page: Page, contrast: string | null) =>
    page.evaluate((contrast) => {
      const cm = (window as unknown as { __cm: Toolkit }).__cm;
      cm.apply("light", contrast, "olive", "none");
      const white: RGB = [255, 255, 255];
      return {
        input: cm.ratio(cm.token("--dx-input"), white),
        ring: cm.ratio(cm.token("--dx-ring-color"), white),
        muted: cm.ratio(cm.token("--dx-muted-foreground"), cm.token("--dx-secondary")),
      };
    }, contrast);

  test("no attribute: shadcn-exact; `wcag` calibrates input edge, ring and muted text (olive base, light)", async ({ page }) => {
    const off = await read(page, null);
    const on = await read(page, "wcag");
    console.log(`[switch] off ${JSON.stringify(off)} on ${JSON.stringify(on)}`);
    expect(off.input).toBeLessThan(NON_TEXT);
    expect(off.ring).toBeLessThan(NON_TEXT);
    expect(off.muted).toBeLessThan(AA);
    expect(on.input).toBeGreaterThanOrEqual(NON_TEXT);
    expect(on.ring).toBeGreaterThanOrEqual(NON_TEXT);
    expect(on.muted).toBeGreaterThanOrEqual(AA);
  });

  test("`prefers-contrast: more` turns the calibrated set on with no attribute", async ({ page }) => {
    await page.emulateMedia({ contrast: "more" });
    const on = await read(page, null);
    expect(on.input).toBeGreaterThanOrEqual(NON_TEXT);
    expect(on.ring).toBeGreaterThanOrEqual(NON_TEXT);
    expect(on.muted).toBeGreaterThanOrEqual(AA);
  });

  test("`data-dx-contrast=\"shadcn\"` opts out of `prefers-contrast: more`", async ({ page }) => {
    await page.emulateMedia({ contrast: "more" });
    const off = await read(page, "shadcn");
    expect(off.input).toBeLessThan(NON_TEXT);
    expect(off.ring).toBeLessThan(NON_TEXT);
    expect(off.muted).toBeLessThan(AA);
    // ... and `wcag` still wins over nothing.
    expect((await read(page, "wcag")).input).toBeGreaterThanOrEqual(NON_TEXT);
  });

  test("`no-preference` and no attribute is the default", async ({ page }) => {
    await page.emulateMedia({ contrast: "no-preference" });
    const off = await read(page, null);
    expect(off.input).toBeLessThan(NON_TEXT);
  });

  test("window.__dxTheme('dx-contrast', v) accepts only wcag and shadcn", async ({ page }) => {
    const attrs = await page.evaluate(() => {
      const w = window as unknown as { __dxTheme: (k: string, v: string | null) => void };
      const html = document.documentElement;
      const get = () => html.getAttribute("data-dx-contrast");
      const seen: (string | null)[] = [];
      for (const v of ["wcag", "bogus", "shadcn", "", null, "WCAG", "wcag "]) {
        w.__dxTheme("dx-contrast", v);
        seen.push(get());
      }
      return seen;
    });
    expect(attrs).toEqual(["wcag", null, "shadcn", null, null, null, null]);
  });

  test("the cookie seeds the attribute before first paint", async ({ page, context }) => {
    await context.addCookies([{ name: "dx_dx_contrast", value: "wcag", url: BASE_URL }]);
    await gotoHydrated(page, `${BASE_URL}/component/input/`);
    expect(await page.evaluate(() => document.documentElement.getAttribute("data-dx-contrast"))).toBe("wcag");
  });
});

// ---------------------------------------------------------------------------------------------------------
// The theme picker's "Contrast" control
// ---------------------------------------------------------------------------------------------------------
test("the theme picker's Contrast control writes data-dx-contrast, persists before first paint, and Reset clears it", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/docs`);
  const html = page.locator("html");
  await expect(html).not.toHaveAttribute("data-dx-contrast", /.+/);

  await page.getByRole("button", { name: "Theme", exact: true }).click();
  const picker = page.locator(".dx-theme-picker");
  const group = picker.locator('[aria-labelledby^="dx-theme-contrast-label"]');
  const chip = (label: string) => group.locator("button", { hasText: new RegExp(`^${label}$`) });
  // "System" (no attribute) is the pressed one to begin with.
  await expect(chip("System")).toHaveAttribute("aria-pressed", "true");
  const inputToken = () =>
    page.evaluate(() => {
      const cm = (window as unknown as { __cm: Toolkit }).__cm;
      return cm.ratio(cm.token("--dx-input"), cm.token("--dx-background"));
    });
  await page.evaluate(installToolkit);
  await page.evaluate(() => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode("light"));
  expect(await inputToken(), "default: shadcn's faint edge").toBeLessThan(NON_TEXT);

  await chip("WCAG").click();
  await expect(html).toHaveAttribute("data-dx-contrast", "wcag");
  expect((await page.context().cookies()).some((c) => c.name === "dx_dx_contrast" && c.value === "wcag")).toBe(true);
  expect(await inputToken(), "WCAG: calibrated edge").toBeGreaterThanOrEqual(NON_TEXT);

  await chip("shadcn").click();
  await expect(html).toHaveAttribute("data-dx-contrast", "shadcn");
  expect(await inputToken()).toBeLessThan(NON_TEXT);

  // Written by the pre-paint snippet from the cookie: present at DOMContentLoaded, before hydration.
  await page.reload({ waitUntil: "domcontentloaded" });
  expect(await page.evaluate(() => document.documentElement.getAttribute("data-dx-contrast"))).toBe("shadcn");
  await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });

  await page.getByRole("button", { name: "Theme", exact: true }).click();
  await expect(chip("shadcn")).toHaveAttribute("aria-pressed", "true");
  await chip("System").click();
  await expect(html).not.toHaveAttribute("data-dx-contrast", /.+/);

  await chip("WCAG").click();
  await expect(html).toHaveAttribute("data-dx-contrast", "wcag");
  await picker.getByRole("button", { name: "Reset" }).click();
  await expect(html).not.toHaveAttribute("data-dx-contrast", /.+/);
});

test("the docs page has a Contrast mode section", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/docs`);
  const section = page.locator("section.dx-docs-section", { has: page.locator("h2#contrast-mode") });
  await expect(section).toBeVisible();
  await expect(section.locator("code", { hasText: "data-dx-contrast" }).first()).toBeVisible();
  // One table row per calibrated role (header excluded).
  await expect(section.locator("tbody tr")).toHaveCount(5);
});

// ---------------------------------------------------------------------------------------------------------
// Fixed for everyone: P2 (error Toast), P4 (success), P5 (docs blue). Same in both settings of the switch.
// ---------------------------------------------------------------------------------------------------------
test.describe("the same in both contrast settings (P2, P4, P5)", () => {
  test.beforeEach(async ({ page }) => open(page, "/component/toast/"));

  for (const contrast of [null, "wcag"] as const) {
    const tag = contrast ?? "default";

    test(`${tag}: the error Toast surface is opaque and its ink reads >= 4.5:1, every base, both modes`, async ({ page }) => {
      const rows = await page.evaluate(
        ({ contrast, bases }) => {
          const cm = (window as unknown as { __cm: Toolkit }).__cm;
          const out: { label: string; ratio: number }[] = [];
          for (const mode of ["light", "dark"]) {
            for (const base of ["none", ...bases]) {
              cm.apply(mode, contrast, base, "none");
              const toast = document.createElement("div");
              toast.className = "dx-toast";
              toast.setAttribute("data-type", "error");
              document.body.append(toast);
              const cs = getComputedStyle(toast);
              // Opaque by construction: an opaque background-color under the fill layer, so the surface cannot
              // depend on what the toast floats over. Measure over two very different backdrops: same answer.
              const bgAlpha = cm.alphaOf(cs.backgroundColor);
              // `linear-gradient(<fill>, <fill>)`: the computed colours contain no ", " outside their own parentheses.
              const inner = cs.backgroundImage.slice("linear-gradient(".length, -1);
              const fill = inner.slice(0, inner.indexOf("), ") + 1);
              if (!cs.backgroundImage.startsWith("linear-gradient(")) throw new Error(`error toast has no fill layer: ${cs.backgroundImage}`);
              const over = (backdrop: [number, number, number]) => cm.rgbOf(fill, cm.rgbOf(cs.backgroundColor, backdrop));
              const onWhite = over([255, 255, 255]);
              const onBlack = over([0, 0, 0]);
              const surface = onWhite;
              const ink = cm.rgbOf(cs.color, surface);
              toast.remove();
              out.push({ label: `${mode} base=${base}`, ratio: cm.ratio(ink, surface) });
              out.push({ label: `${mode} base=${base} background-color alpha ${bgAlpha} (must be 1)`, ratio: bgAlpha === 1 ? 21 : 0 });
              out.push({ label: `${mode} base=${base} surface depends on the backdrop`, ratio: onWhite.join() === onBlack.join() ? 21 : 0 });
            }
          }
          return out;
        },
        { contrast, bases: BASES },
      );
      const w = worst(rows);
      console.log(`[error toast ${tag}] worst ${w.ratio.toFixed(3)}:1 at ${w.label}`);
      const bad = rows.filter((r) => r.ratio < AA);
      expect(fmt(bad).slice(0, 12), `${bad.length}/${rows.length} states under ${AA}:1`).toEqual([]);
    });

    test(`${tag}: --dx-success (the copy-check icon) is >= 3:1 on background, card and popover; light and dark`, async ({ page }) => {
      const rows = await page.evaluate(
        ({ contrast, surfaces }) => {
          const cm = (window as unknown as { __cm: Toolkit }).__cm;
          const out: { label: string; ratio: number }[] = [];
          for (const mode of ["light", "dark"]) {
            cm.apply(mode, contrast, "none", "none");
            const c = cm.token("--dx-success");
            for (const s of surfaces) out.push({ label: `${mode} on ${s}`, ratio: cm.ratio(c, cm.token(s)) });
          }
          return out;
        },
        { contrast, surfaces: SURFACES },
      );
      const bad = rows.filter((r) => r.ratio < NON_TEXT);
      expect(fmt(bad)).toEqual([]);
    });

    test(`${tag}: the docs link blue (--highlight-color-tertiary) reads >= 4.5:1 on background, card, popover and muted`, async ({ page }) => {
      const rows = await page.evaluate(
        ({ contrast, surfaces }) => {
          const cm = (window as unknown as { __cm: Toolkit }).__cm;
          const out: { label: string; ratio: number }[] = [];
          for (const mode of ["light", "dark"]) {
            cm.apply(mode, contrast, "none", "none");
            const probe = document.createElement("i");
            probe.style.color = "var(--highlight-color-tertiary)";
            document.body.append(probe);
            const ink = cm.rgbOf(getComputedStyle(probe).color);
            probe.remove();
            for (const s of surfaces) out.push({ label: `${mode} on ${s}`, ratio: cm.ratio(ink, cm.token(s)) });
          }
          return out;
        },
        { contrast, surfaces: [...SURFACES, "--dx-muted"] },
      );
      const w = worst(rows);
      console.log(`[docs blue ${tag}] worst ${w.ratio.toFixed(3)}:1 at ${w.label}`);
      const bad = rows.filter((r) => r.ratio < AA);
      expect(fmt(bad)).toEqual([]);
    });
  }
});
