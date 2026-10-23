import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * The theme's own default fallbacks (what a page gets with NO preset and no `--foreground` etc. supplied) are
 * shadcn Neutral's, both modes (owner decision 2026-10-23, backlog row 373, proposals P7 and P8 of the theme
 * contrast audit). Computed in the browser, resolved to sRGB through a canvas so `oklch()` expectations and the
 * painted token compare in one space.
 *
 *  - every role named below resolves to shadcn's value (1/255 of rounding allowed);
 *  - a user-supplied shadcn variable (`--foreground`, `--sidebar-primary`, ...) still wins over every default;
 *  - the upstream numbered ramp and `--focused-border-color` are gone (nothing resolves them any more).
 *
 * Not in scope here (kept on purpose, asserted elsewhere): `--dx-muted-foreground`, the chart colours, the docs
 * blue, dark `--dx-destructive` / `--dx-sidebar-primary` / `--dx-sidebar-primary-foreground`, the calibrated
 * "Contrast mode" values (`contrast-mode.spec.ts`).
 */

type Pair = readonly [light: string, dark: string];
const N = (l: number) => `oklch(${l} 0 0)`;
const SAME = (css: string): Pair => [css, css];

// token -> [light, dark] as shadcn Neutral spells them (apps/v4 globals.css `:root` / `.dark`).
const EXPECTED: Record<string, Pair> = {
  "--dx-foreground": [N(0.145), N(0.985)],
  "--dx-card-foreground": [N(0.145), N(0.985)],
  "--dx-popover-foreground": [N(0.145), N(0.985)],
  "--dx-primary": [N(0.205), N(0.922)],
  "--dx-accent": [N(0.97), N(0.269)],
  "--dx-accent-foreground": [N(0.205), N(0.985)],
  "--dx-destructive-foreground": SAME(N(0.985)),
  "--dx-destructive": ["oklch(0.577 0.245 27.325)", "#a22e2e"], // dark kept: a FILL under near-white text
  "--dx-sidebar-background": [N(0.985), N(0.205)],
  "--dx-sidebar-foreground": [N(0.145), N(0.985)],
  "--dx-sidebar-primary": [N(0.205), "#fafafa"], // dark kept (shadcn's is a blue chip)
  "--dx-sidebar-primary-foreground": [N(0.985), "#000"], // dark kept
  "--dx-sidebar-accent": [N(0.97), N(0.269)],
  "--dx-sidebar-accent-foreground": [N(0.205), N(0.985)],
  "--dx-sidebar-border": [N(0.922), "oklch(1 0 0 / 10%)"],
  "--dx-sidebar-ring": [N(0.708), N(0.556)],
};

// `--dx-<role>` -> the bare shadcn variable a host or preset supplies.
const BARE: Record<string, string> = {
  "--dx-foreground": "--foreground",
  "--dx-card-foreground": "--card-foreground",
  "--dx-popover-foreground": "--popover-foreground",
  "--dx-primary": "--primary",
  "--dx-accent": "--accent",
  "--dx-accent-foreground": "--accent-foreground",
  "--dx-destructive-foreground": "--destructive-foreground",
  "--dx-destructive": "--destructive",
  "--dx-border-strong": "--border-strong",
  "--dx-sidebar-background": "--sidebar",
  "--dx-sidebar-foreground": "--sidebar-foreground",
  "--dx-sidebar-primary": "--sidebar-primary",
  "--dx-sidebar-primary-foreground": "--sidebar-primary-foreground",
  "--dx-sidebar-accent": "--sidebar-accent",
  "--dx-sidebar-accent-foreground": "--sidebar-accent-foreground",
  "--dx-sidebar-border": "--sidebar-border",
  "--dx-sidebar-ring": "--sidebar-ring",
};

const REMOVED = [
  "--primary-color",
  "--primary-color-1",
  "--primary-color-2",
  "--primary-color-3",
  "--primary-color-4",
  "--primary-color-5",
  "--primary-color-6",
  "--primary-color-7",
  "--secondary-color",
  "--secondary-color-1",
  "--secondary-color-2",
  "--secondary-color-3",
  "--secondary-color-4",
  "--secondary-color-5",
  "--secondary-color-6",
  "--primary-success-color",
  "--secondary-success-color",
  "--primary-warning-color",
  "--secondary-warning-color",
  "--primary-error-color",
  "--secondary-error-color",
  "--contrast-error-color",
  "--primary-info-color",
  "--secondary-info-color",
  "--focused-border-color",
  "--highlight-color-main",
  "--highlight-color-secondary",
];

for (const mode of ["light", "dark"] as const) {
  test.describe(`${mode}: no-preset defaults are shadcn Neutral`, () => {
    test.use({ colorScheme: mode });

    test.beforeEach(async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/button/`);
      await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
      expect(await page.evaluate(() => document.documentElement.getAttribute("data-theme-base"))).toBeNull();
      expect(await page.evaluate(() => document.documentElement.getAttribute("data-theme-accent"))).toBeNull();
    });

    test("each defaulted role resolves to shadcn's value", async ({ page }) => {
      const idx = mode === "light" ? 0 : 1;
      const got = await page.evaluate(
        ({ entries }) => {
          const cx = document.createElement("canvas").getContext("2d")!;
          const toRgba = (css: string) => {
            cx.clearRect(0, 0, 1, 1);
            cx.fillStyle = "#000"; // reset: an unparsable value would otherwise keep the previous fill
            cx.fillStyle = css;
            cx.fillRect(0, 0, 1, 1);
            return Array.from(cx.getImageData(0, 0, 1, 1).data);
          };
          const resolve = (value: string) => {
            const probe = document.createElement("i");
            probe.style.color = value;
            document.body.append(probe);
            const c = getComputedStyle(probe).color;
            probe.remove();
            return c;
          };
          return entries.map(([token, want]) => ({
            token,
            have: toRgba(resolve(`var(${token})`)),
            want: toRgba(want),
          }));
        },
        { entries: Object.entries(EXPECTED).map(([t, pair]) => [t, pair[idx]] as [string, string]) },
      );
      for (const { token, have, want } of got) {
        for (let i = 0; i < 4; i++) {
          expect(Math.abs(have[i] - want[i]), `${mode} ${token} channel ${i}: have ${have} want ${want}`).toBeLessThanOrEqual(1);
        }
      }
    });

    test("a user-supplied shadcn variable wins over every default", async ({ page }) => {
      const got = await page.evaluate((bare) => {
        const root = document.documentElement;
        const resolve = (token: string) => {
          const probe = document.createElement("i");
          probe.style.color = `var(${token})`;
          document.body.append(probe);
          const c = getComputedStyle(probe).color;
          probe.remove();
          return c;
        };
        const out: Record<string, string> = {};
        for (const [token, name] of Object.entries(bare)) {
          root.style.setProperty(name, "rgb(1, 2, 3)");
          out[token] = resolve(token);
          root.style.removeProperty(name);
        }
        return out;
      }, BARE);
      for (const [token, c] of Object.entries(got)) expect(c, `${mode} ${token} under its bare variable`).toBe("rgb(1, 2, 3)");
    });

    test("the upstream ramp and the two dead highlight tokens are gone", async ({ page }) => {
      const left = await page.evaluate(
        (names) => names.filter((n) => getComputedStyle(document.documentElement).getPropertyValue(n).trim() !== ""),
        REMOVED,
      );
      expect(left).toEqual([]);
    });
  });
}
