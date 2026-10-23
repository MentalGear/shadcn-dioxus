import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

// The docs Button page paints with shadcn's base-nova text colours (docs-site lane, backlog row 231). Expected
// values are what https://ui.shadcn.com/docs/components/base/button computes with no preset (its `:root` and
// `.dark` blocks: --foreground oklch(0.145) = #0a0a0a / #fafafa, --primary-foreground #fafafa / #171717,
// --secondary-foreground #171717 / #fafafa, --primary oklch(0.205) = #171717 / #e5e5e5, --secondary #f5f5f5 / #262626,
// --border #e5e5e5), read in Chromium from a local replica of that page. Light ink was pure #000 / #000 primary until
// the owner moved the theme's defaults to shadcn's near-black (2026-10-23, backlog row 373). The Destructive Button is the one deliberate difference (row 134:
// a solid fill, because shadcn's 10% tint fails 4.5:1 for text): its fill is the destructive role and its ink
// stays light; it is asserted separately so a change is a decision, not an accident.
// Run against the SSG lane.

type Rgb = string;

/** Any CSS colour -> "#rrggbb", through a canvas (lab(), oklab() and color-mix() all come out as sRGB). */
async function hex(page: Page, css: string): Promise<Rgb> {
  return page.evaluate((c) => {
    const cx = document.createElement("canvas").getContext("2d")!;
    cx.clearRect(0, 0, 1, 1);
    cx.fillStyle = c;
    cx.fillRect(0, 0, 1, 1);
    const d = cx.getImageData(0, 0, 1, 1).data;
    if (d[3] === 0) return "transparent";
    return "#" + [d[0], d[1], d[2]].map((n) => n.toString(16).padStart(2, "0")).join("");
  }, css);
}

const VARIANT_FRAME = "#component-preview-frame-all_variants";

async function paint(page: Page, label: string) {
  const el = page.locator(`${VARIANT_FRAME} .dx-button`, { hasText: new RegExp(`^${label}$`) }).first();
  const raw = await el.evaluate((n) => {
    const cs = getComputedStyle(n);
    return { bg: cs.backgroundColor, color: cs.color, radius: cs.borderTopLeftRadius, height: cs.height };
  });
  return { bg: await hex(page, raw.bg), color: await hex(page, raw.color), radius: raw.radius, height: raw.height };
}

const EXPECTED = {
  light: {
    Primary: { bg: "#171717", color: "#fafafa" },
    Secondary: { bg: "#f5f5f5", color: "#171717" },
    Outline: { bg: "#ffffff", color: "#0a0a0a" },
    Ghost: { bg: "transparent", color: "#0a0a0a" },
    Link: { bg: "transparent", color: "#171717" }, // `--dx-primary-ink`: the link ink is the primary, clamped to L <= 0.5
  },
  dark: {
    Primary: { bg: "#e5e5e5", color: "#171717" },
    Secondary: { bg: "#262626", color: "#fafafa" },
    // Outline's dark fill is `input/30` over the page and differs by a rounding step; its ink is what matters.
    Outline: { bg: null, color: "#fafafa" },
    Ghost: { bg: "transparent", color: "#fafafa" },
    Link: { bg: "transparent", color: "#e5e5e5" },
  },
} as const;

for (const mode of ["light", "dark"] as const) {
  test.describe(`${mode}: the Button variants read as shadcn's`, () => {
    test.use({ colorScheme: mode });

    test(`text and fill colours, ${mode}`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/button/`);
      await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
      for (const [label, want] of Object.entries(EXPECTED[mode])) {
        const got = await paint(page, label);
        expect.soft(got.color, `${label} text`).toBe(want.color);
        if (want.bg !== null) expect.soft(got.bg, `${label} fill`).toBe(want.bg);
        // Geometry was already shadcn's: radius 10px, default height 32px.
        expect.soft(got.radius, `${label} radius`).toBe("10px");
        expect.soft(got.height, `${label} height`).toBe("32px");
      }
    });

    test(`the solid Destructive Button is kept (row 134), ${mode}`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/button/`);
      await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
      const got = await paint(page, "Destructive");
      // Light: the destructive role itself (shadcn's oklch(0.577 0.245 27.325) = #e7000b); dark: 60% of it over the page. Either way a FILL
      // (not shadcn's transparent tint) carrying light text.
      expect(got.bg).not.toBe("transparent");
      if (mode === "light") expect(got.bg).toBe("#e7000b");
      expect(got.color).not.toBe(got.bg);
    });
  });
}

test("the first demo is shadcn's button-demo: one outline Button and an icon-only Submit button", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button/`);
  const frame = page.locator("#component-preview-frame");
  const buttons = frame.locator(".dx-button");
  await expect(buttons).toHaveCount(2);
  await expect(buttons.nth(0)).toHaveText("Button");
  await expect(buttons.nth(0)).toHaveAttribute("data-style", "outline");
  await expect(buttons.nth(1)).toHaveAttribute("aria-label", "Submit");
  await expect(buttons.nth(1)).toHaveAttribute("data-size", "icon");
});

test("the demo frame is shadcn's preview frame: rounded-2xl, no border, 40px padding, 288px tall, soft surface fill", async ({
  page,
}) => {
  await gotoHydrated(page, `${BASE_URL}/component/button/`);
  const frame = page.locator("#component-preview-frame");
  await expect(frame).toHaveCSS("border-top-left-radius", "16px");
  await expect(frame).toHaveCSS("padding-top", "40px");
  // Owner request (docs-css lane): no border, a soft fill between the page and the card token
  // (`--dx-docs-preview-surface`), so the frame is no longer transparent.
  await expect(frame).toHaveCSS("border-top-width", "0px");
  await expect(frame).not.toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
  await expect(frame).toHaveCSS("background-image", "none");
  expect((await frame.boundingBox())!.height).toBe(288);
});
