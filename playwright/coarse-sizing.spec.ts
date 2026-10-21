import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";

/**
 * Backlog rows 349 and 350: touch sizing is one switch, `data-dx-coarse="auto|on|off"` on `<html>` or any
 * container (`dx-components-theme.css`, "Coarse sizing"), and the theme picker's "Touch sizing" control and the
 * docs' side-by-side section drive it.
 *
 * The probes are real `dx-` controls (an input, a slider thumb, a menu row) inside containers this spec
 * writes, so the result does not depend on the demo copy; the docs section is checked separately. The matrix
 * runs twice, once on a fine pointer (no media match) and once on an emulated coarse one (`hasTouch` +
 * `isMobile`), and `on` / `off` must give the same answer on both: forcing must not depend on the device.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8106 npx playwright test --config=ssg.local.config.ts coarse-sizing.spec.ts
 */

const DOCS = `${BASE_URL}/docs`;

type Sizes = { fontSize: string; thumb: string; menuRow: string; actions: string };

/** Fine = Nova's dense sizes, coarse = the touch step. Menu row min-height is `0px` (no floor) when fine. */
const FINE: Sizes = { fontSize: "14px", thumb: "12px", menuRow: "0px", actions: "0" };
const COARSE: Sizes = { fontSize: "16px", thumb: "20px", menuRow: "44px", actions: "1" };

/**
 * Wraps a probe set in `<div data-dx-coarse=mode>` (nested inside `outer` when given) and returns what its
 * controls compute. `html` is the attribute on `<html>` (or null for none).
 */
async function sizes(page: Page, opts: { html: string | null; mode: string | null; outer?: string }): Promise<Sizes> {
  return page.evaluate(({ html, mode, outer }) => {
    const root = document.documentElement;
    if (html === null) root.removeAttribute("data-dx-coarse");
    else root.setAttribute("data-dx-coarse", html);
    document.getElementById("coarse-probe")?.remove();
    const wrap = document.createElement("div");
    wrap.id = "coarse-probe";
    let inner = wrap;
    if (outer) {
      const o = document.createElement("div");
      o.setAttribute("data-dx-coarse", outer);
      wrap.append(o);
      inner = o;
    }
    const host = document.createElement("div");
    if (mode !== null) host.setAttribute("data-dx-coarse", mode);
    host.innerHTML =
      '<input class="dx-input" aria-label="probe">' +
      '<div class="dx-slider"><div class="dx-slider-track"><div class="dx-slider-thumb"></div></div></div>' +
      '<div class="dx-command-item"></div>' +
      '<div class="dx-code-block"><div class="dx-code-block-actions"></div></div>';
    inner.append(host);
    document.body.append(wrap);
    const q = (s: string) => getComputedStyle(host.querySelector(s)!);
    return {
      fontSize: q(".dx-input").fontSize,
      thumb: q(".dx-slider-thumb").width,
      menuRow: q(".dx-command-item").minHeight,
      actions: q(".dx-code-block-actions").opacity,
    };
  }, opts);
}

const MATRIX: { name: string; html: string | null; mode: string | null; outer?: string }[] = [
  { name: "container on", html: null, mode: "on" },
  { name: "container off", html: null, mode: "off" },
  { name: "html on", html: "on", mode: null },
  { name: "html off", html: "off", mode: null },
  { name: "container off inside html on", html: "on", mode: "off" },
  { name: "container on inside html off", html: "off", mode: "on" },
  { name: "auto inside an off container", html: null, mode: "auto", outer: "off" },
  { name: "auto inside an on container", html: null, mode: "auto", outer: "on" },
  { name: "no attribute", html: null, mode: null },
];

for (const device of [false, true]) {
  test.describe(device ? "coarse pointer (touch emulation)" : "fine pointer (desktop)", () => {
    test.use(
      device
        ? { viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, colorScheme: "light" }
        : { viewport: { width: 1280, height: 800 }, colorScheme: "light" },
    );
    test.skip(({ browserName }) => device && browserName !== "chromium", "mobile emulation is Chromium-only");

    test("the pointer media query is what the context says", async ({ page }) => {
      await gotoHydrated(page, DOCS);
      expect(await page.evaluate(() => matchMedia("(pointer: coarse)").matches)).toBe(device);
    });

    for (const c of MATRIX) {
      test(`${c.name}`, async ({ page }) => {
        await gotoHydrated(page, DOCS);
        const got = await sizes(page, c);
        // What the case should resolve to, innermost attribute first: `on` / `off` force, `auto` is the media
        // query again (even inside a forced ancestor), no attribute follows the ancestors.
        const chain = [c.mode, c.outer, c.html].filter((v): v is string => v !== null && v !== undefined);
        const first = chain.find((v) => v === "on" || v === "off" || v === "auto");
        const coarse = first === "on" ? true : first === "off" ? false : device;
        expect(got).toEqual(coarse ? COARSE : FINE);
      });
    }

    test("an invalid value behaves as auto", async ({ page }) => {
      await gotoHydrated(page, DOCS);
      expect(await sizes(page, { html: null, mode: "sideways" })).toEqual(device ? COARSE : FINE);
    });
  });
}

test.describe("docs section and theme picker", () => {
  test.use({ colorScheme: "light", viewport: { width: 1280, height: 800 } });

  test("the docs render a fine and a coarse container side by side", async ({ page }) => {
    await gotoHydrated(page, DOCS);
    const off = page.locator('[data-coarse-pane="off"]');
    const on = page.locator('[data-coarse-pane="on"]');
    await off.scrollIntoViewIfNeeded();
    await expect(on).toBeVisible();
    const fs = (p: typeof off) => p.locator(".dx-input").evaluate((e) => getComputedStyle(e).fontSize);
    expect(await fs(off)).toBe("14px");
    expect(await fs(on)).toBe("16px");
    const thumb = async (p: typeof off) => (await measureAtRest(p.locator(".dx-slider-thumb"))).width;
    expect(await thumb(off)).toBeCloseTo(12, 0);
    expect(await thumb(on)).toBeCloseTo(20, 0);
    const row = async (p: typeof off) => (await measureAtRest(p.locator(".dx-command-item").first())).height;
    expect(await row(off)).toBeLessThan(34);
    expect(await row(on)).toBeGreaterThanOrEqual(43.5);
  });

  test("Touch sizing in the theme picker writes data-dx-coarse, persists before first paint, and Reset clears it", async ({
    page,
  }) => {
    await gotoHydrated(page, DOCS);
    const html = page.locator("html");
    await expect(html).not.toHaveAttribute("data-dx-coarse", /.+/);

    await page.getByRole("button", { name: "Theme", exact: true }).click();
    const picker = page.locator(".dx-theme-picker");
    const group = picker.locator('[aria-labelledby^="dx-theme-coarse-label"]');
    const chip = (label: string) => group.locator("button", { hasText: new RegExp(`^${label}$`) });
    await expect(chip("Auto")).toBeVisible();

    await chip("On").click();
    await expect(html).toHaveAttribute("data-dx-coarse", "on");
    expect((await page.context().cookies()).some((c) => c.name === "dx_dx_coarse" && c.value === "on")).toBe(true);
    // The page really switched: an `auto` container follows `html`'s forced value only when it has none, so
    // read a control outside any container.
    expect(await page.locator(".dx-input").first().evaluate((e) => getComputedStyle(e).fontSize)).toBe("16px");

    await chip("Off").click();
    await expect(html).toHaveAttribute("data-dx-coarse", "off");

    // Written by the pre-paint snippet from the cookie: present at DOMContentLoaded, before hydration.
    await page.reload({ waitUntil: "domcontentloaded" });
    expect(await page.evaluate(() => document.documentElement.getAttribute("data-dx-coarse"))).toBe("off");
    await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });

    await page.getByRole("button", { name: "Theme", exact: true }).click();
    await expect(chip("Off")).toHaveAttribute("aria-pressed", "true");
    await chip("Auto").click();
    await expect(html).not.toHaveAttribute("data-dx-coarse", /.+/);

    await chip("On").click();
    await expect(html).toHaveAttribute("data-dx-coarse", "on");
    await picker.getByRole("button", { name: "Reset" }).click();
    await expect(html).not.toHaveAttribute("data-dx-coarse", /.+/);
  });

  test("the snippet rejects a value that is not auto, on or off", async ({ page }) => {
    await gotoHydrated(page, DOCS);
    const got = await page.evaluate(() => {
      const w = window as unknown as { __dxTheme: (k: string, v: string | null) => void };
      w.__dxTheme("dx-coarse", "maybe");
      return document.documentElement.getAttribute("data-dx-coarse");
    });
    expect(got).toBeNull();
  });
});
