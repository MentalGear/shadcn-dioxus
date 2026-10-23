import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// Row 243: a Button hover fill must be shadcn base-nova's, by construction. shadcn writes the secondary hover as
// `color-mix(in oklch, var(--secondary), var(--foreground) 5%)`; the theme's `--dx-hover-mix-space` carries that
// space and every "step the fill toward the ink" hover reads it. The expected colours are computed IN THE PAGE from
// the resolved tokens with shadcn's formula (a probe element), never typed in, so a token or recipe change cannot
// leave the spec stale; two literal anchors (#e5e5e5 / #2f2f2f, base-nova's measured values) pin the formula.
const GOTO = { timeout: 20 * 60 * 1000 };

async function open(page: Page, scheme: "light" | "dark") {
  await page.emulateMedia({ colorScheme: scheme });
  await gotoHydrated(page, `${BASE_URL}/component/button/`, GOTO);
  await page.waitForLoadState("networkidle");
}

// Any CSS colour (the computed value may be an `oklch(...)` string) to 8-bit sRGB "r,g,b", via a canvas pixel: the
// same quantisation a screenshot shows, and what "#e5e5e5" means.
const toRgb = (page: Page, color: string) =>
  page.evaluate((c) => {
    const ctx = document.createElement("canvas").getContext("2d", { colorSpace: "srgb" })!;
    ctx.clearRect(0, 0, 1, 1);
    ctx.fillStyle = c;
    ctx.fillRect(0, 0, 1, 1);
    return Array.from(ctx.getImageData(0, 0, 1, 1).data.slice(0, 3)).join(",");
  }, color);

// The fill the button computes while hovered, as sRGB.
const hoverFill = async (page: Page, btn: Locator) => {
  await btn.scrollIntoViewIfNeeded();
  await btn.hover();
  return toRgb(page, await btn.evaluate((e) => getComputedStyle(e).backgroundColor));
};

// style -> shadcn's hover formula over the page's resolved tokens (null: not derived from the step recipe).
const SHADCN_HOVER: Record<string, string | null> = {
  secondary: "color-mix(in oklch, var(--dx-secondary), var(--dx-foreground) 5%)",
  destructive: "color-mix(in oklch, var(--dx-destructive-fill), var(--dx-foreground) 8%)",
  primary: "color-mix(in oklab, var(--dx-primary) 80%, transparent)",
};

// A probe element resolves `var(--dx-*)` against the page's tokens (a canvas cannot).
const probe = (page: Page, expr: string) =>
  page.evaluate((c) => {
    const p = document.createElement("span");
    document.body.appendChild(p);
    p.style.backgroundColor = c;
    const v = getComputedStyle(p).backgroundColor;
    p.remove();
    return v;
  }, expr);

for (const scheme of ["light", "dark"] as const) {
  test(`Button hover fills follow shadcn's formula (${scheme})`, async ({ page }) => {
    await open(page, scheme);
    for (const style of Object.keys(SHADCN_HOVER)) {
      const btn = page.locator(`.dx-button[data-style="${style}"]:not(:disabled)`).first();
      await expect(btn).toBeVisible();
      const expected = await toRgb(page, await probe(page, SHADCN_HOVER[style]!));
      await expect.poll(() => hoverFill(page, btn), { message: `${style} hover` }).toBe(expected);
    }
  });
}

// Measured with shadcn's docs ink `--foreground #000`, light #e5e5e5; the theme's ink is shadcn's near-black
// oklch(0.145) since row 373 (owner, 2026-10-23), which steps #f5f5f5 5% toward #0a0a0a and lands on #e7e7e7. Dark
// is unchanged (the dark ink was already #fafafa).
test("the secondary hover is #e7e7e7 (light, near-black ink) and base-nova's measured #2f2f2f (dark)", async ({ page }) => {
  for (const [scheme, rgb] of [["light", "231,231,231"], ["dark", "47,47,47"]] as const) {
    await open(page, scheme);
    const btn = page.locator('.dx-button[data-style="secondary"]:not(:disabled)').first();
    await expect.poll(() => hoverFill(page, btn)).toBe(rgb);
  }
});

test("hover recipe reads the theme's one interpolation space", async ({ page }) => {
  await open(page, "light");
  expect(await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue("--dx-hover-mix-space").trim())).toBe("oklch");
});
