import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// Row 239: `all: unset` (0,1,0) outranked every zero-specificity theme baseline on Switch, the Slider thumb and both
// ColorPicker thumbs. The `all:` resets are gone (scripts/check-theme-baseline.sh bans them) and spelled out; this
// spec pins (a) the theme baselines now reaching those four controls and (b) that the visual computed style is
// exactly what `all: unset` produced (captured on the pre-change build, light scheme).
const GOTO = { timeout: 20 * 60 * 1000 };

// Computed values of the pre-change build (light). `rect` is the border box, rounded to 2 decimals.
//
// Re-baselined 2026-10-23: `cursor` is `default` (was `pointer`): the interactive-element cursor is the theme's one rule now
// (`data-dx-interactive-element-cursor`, cursor-option.spec.ts), and shadcn v4's arrow is its default; nothing else moved.
//
// Row 331 verdict (2026-10-21): KEPT. Re-checked against today's build: all four records still equal the
// pre-change values, so no re-baseline was needed. What only this snapshot catches: a probe that appended
// `cursor: default`, `text-indent`, `appearance: auto`, `transition-property: none`, `letter-spacing`, `border-width`
// and `text-shadow`/`word-spacing` declarations (appended to the shipped sheet) to the four controls went red
// here (4 of 4) while this file's per-control tests, `switch.spec`, `slider.spec`, `slider-thumb-clip.spec`,
// `color-picker*.spec` and `overlay-switch.spec` stayed green. The gates `check-theme-baseline.sh` (bans `all:`,
// checks the two theme rules) and `check-popup-root-reset.sh` (checks the popup-root rule) scan sources for those
// shapes and read no computed value, and `popup-style-isolation` compares a popup with its own clone, so a
// sheet-level change reaches both sides: by their construction, not by a run on the probe.
const BEFORE: Record<string, Record<string, string>> = {
  "switch": {
    "display": "flex",
    "position": "relative",
    "width": "32px",
    "height": "18.3906px",
    "margin-top": "0px",
    "margin-left": "0px",
    "padding-top": "0px",
    "padding-left": "0px",
    "border-top-width": "0px",
    "border-top-style": "none",
    "border-top-left-radius": "9999px",
    "background-image": "none",
    "color": "rgb(0, 0, 0)",
    "font-size": "16px",
    "font-weight": "400",
    "line-height": "normal",
    "letter-spacing": "normal",
    "text-align": "start",
    "text-transform": "none",
    "text-indent": "0px",
    "text-shadow": "none",
    "word-spacing": "0px",
    "cursor": "default",
    "outline-style": "none",
    "appearance": "none",
    "transform": "none",
    "transition-property": "background-color, box-shadow",
    "align-items": "center",
    "background-color": "rgb(229, 229, 229)",
    "rect": "32x18.39"
  },
  "slider-thumb": {
    "display": "block",
    "position": "absolute",
    "width": "12px",
    "height": "12px",
    "margin-top": "0px",
    "margin-left": "0px",
    "padding-top": "0px",
    "padding-left": "0px",
    "border-top-width": "1px",
    "border-top-style": "solid",
    "border-top-left-radius": "50%",
    "background-image": "none",
    "color": "rgb(0, 0, 0)",
    "font-size": "16px",
    "font-weight": "400",
    "line-height": "normal",
    "letter-spacing": "normal",
    "text-align": "start",
    "text-transform": "none",
    "text-indent": "0px",
    "text-shadow": "none",
    "word-spacing": "0px",
    "cursor": "default",
    "outline-style": "none",
    "appearance": "none",
    "transform": "matrix(1, 0, 0, 1, -6, -6)",
    "transition-property": "box-shadow",
    "align-items": "normal",
    "background-color": "rgb(255, 255, 255)",
    "border-top-color": "rgb(43, 127, 255)",
    "rect": "12x12"
  },
  "cp-slider-thumb": {
    "display": "block",
    "position": "absolute",
    "width": "16px",
    "height": "16px",
    "margin-top": "0px",
    "margin-left": "0px",
    "padding-top": "0px",
    "padding-left": "0px",
    "border-top-width": "2px",
    "border-top-style": "solid",
    "border-top-left-radius": "50%",
    "background-image": "none",
    "color": "rgb(0, 0, 0)",
    "font-size": "16px",
    "font-weight": "400",
    "line-height": "normal",
    "letter-spacing": "normal",
    "text-align": "start",
    "text-transform": "none",
    "text-indent": "0px",
    "text-shadow": "none",
    "word-spacing": "0px",
    "cursor": "default",
    "outline-style": "none",
    "appearance": "none",
    "transform": "matrix(1, 0, 0, 1, -10, -10)",
    "transition-property": "transform, box-shadow, border-color",
    "align-items": "normal",
    "border-top-color": "rgb(0, 0, 0)",
    "rect": "20x20"
  },
  "cp-area-thumb": {
    "display": "block",
    "position": "absolute",
    "width": "16px",
    "height": "16px",
    "margin-top": "0px",
    "margin-left": "0px",
    "padding-top": "0px",
    "padding-left": "0px",
    "border-top-width": "2px",
    "border-top-style": "solid",
    "border-top-left-radius": "50%",
    "background-image": "none",
    "color": "rgb(0, 0, 0)",
    "font-size": "16px",
    "font-weight": "400",
    "line-height": "normal",
    "letter-spacing": "normal",
    "text-align": "start",
    "text-transform": "none",
    "text-indent": "0px",
    "text-shadow": "none",
    "word-spacing": "0px",
    "cursor": "default",
    "outline-style": "none",
    "appearance": "none",
    "transform": "matrix(1, 0, 0, 1, -10, -10)",
    "transition-property": "transform, box-shadow, border-color",
    "align-items": "normal",
    "border-top-color": "rgb(0, 0, 0)",
    "rect": "20x20"
  }
};

async function open(page: Page, name: string) {
  await gotoHydrated(page, `${BASE_URL}/component/${name}/`, GOTO);
  await page.waitForLoadState("networkidle");
}

async function pick(page: Page, which: string) {
  if (which === "switch") {
    await open(page, "switch");
    return page.locator("#component-preview-frame").getByRole("switch").first();
  }
  if (which === "slider-thumb") {
    await open(page, "slider");
    return page.locator(".dx-slider-thumb").first();
  }
  await open(page, "color_picker");
  await page.getByRole("button", { name: /Color picker/i }).first().click();
  await expect(page.getByRole("dialog")).toBeVisible();
  return page.locator(which === "cp-slider-thumb" ? ".dx-color-picker-slider-thumb" : ".dx-color-picker-area-thumb").first();
}

for (const which of Object.keys(BEFORE)) {
  test.describe(which, () => {
    test("theme baselines reach it (touch-action, user-select) and no `all` reset remains", async ({ page }) => {
      const el = await pick(page, which);
      await expect(el).toBeVisible();
      const cs = await el.evaluate((e) => {
        const s = getComputedStyle(e);
        return { touch: s.touchAction, us: s.userSelect, webkitUs: s.getPropertyValue("-webkit-user-select") };
      });
      expect(cs.touch).toBe("manipulation");
      expect(cs.us).toBe("none");
      expect(cs.webkitUs).toBe("none");
    });

    test("visual computed style equals the pre-change `all: unset` build", async ({ page }) => {
      const el = await pick(page, which);
      await expect(el).toBeVisible();
      const props = Object.keys(BEFORE[which]).filter((p) => p !== "rect");
      const got = await el.evaluate((e, ps) => {
        const s = getComputedStyle(e);
        const o: Record<string, string> = {};
        for (const p of ps) o[p] = s.getPropertyValue(p);
        const r = e.getBoundingClientRect();
        // Rounded to 2 decimals (row 258): a thumb at a fractional position under `translate(-10px, -10px)` gives
        // 19.99999237060547 on some loads. The computed width/height above stay exact.
        const r2 = (n: number) => Math.round(n * 100) / 100;
        o.rect = `${r2(r.width)}x${r2(r.height)}`;
        return o;
      }, props);
      // The slider thumb's border is the focus-ring colour token, which the golden recorded as the old default
      // blue (rgb(43, 127, 255)); it is now shadcn's grey, so the border is compared to the token itself and
      // every other property to the golden (`all: unset` equivalence is about the rest).
      const expected: Record<string, string> = { ...BEFORE[which] };
      if (which === "slider-thumb") {
        expected["border-top-color"] = await page.evaluate(() => {
          const probe = document.createElement("i");
          probe.style.color = "var(--dx-ring-color)";
          document.body.append(probe);
          const c = getComputedStyle(probe).color;
          probe.remove();
          return c;
        });
      }
      expect(got).toEqual(expected);
    });
  });
}

test("Switch gets the theme's press feedback while pressed", async ({ page }) => {
  const sw = await pick(page, "switch");
  await expect(sw).toBeVisible();
  await page.emulateMedia({ reducedMotion: "reduce" });
  expect(await sw.evaluate((e) => getComputedStyle(e).scale)).toBe("none");
  const box = (await sw.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  try {
    await expect.poll(() => sw.evaluate((e) => getComputedStyle(e).scale)).not.toBe("none");
    expect(await sw.evaluate((e) => getComputedStyle(e).backgroundImage)).toContain("linear-gradient");
  } finally {
    await page.mouse.up();
  }
});
