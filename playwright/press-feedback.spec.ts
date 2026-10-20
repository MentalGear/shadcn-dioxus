import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * Backlog row 205 (the owner's touch finding), part 1 and 2:
 *
 *  1. No sticky hover. A touch browser keeps `:hover` on the element it just tapped; every hover rule now
 *     lives inside `@media (hover: hover) and (pointer: fine)` (`scripts/check-hover-media.sh`), so after a
 *     tap a control looks exactly like it did at rest.
 *  2. Press feedback, a switchable system utility (`dx-components-theme.css`, "Press feedback"): a pressed
 *     `dx-` control takes `scale: var(--dx-press-scale)` and a `--dx-press-fill` wash through ONE
 *     zero-specificity rule; `<html data-dx-press-feedback="off">` (the theme picker's "Press feedback"
 *     switch, persisted in a cookie like the other preferences) removes it; the transition exists only
 *     under `prefers-reduced-motion: no-preference`.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8106 npx playwright test --config=ssg.local.config.ts press-feedback.spec.ts
 */

const BUTTON_PAGE = `${BASE_URL}/component/button/`;

/** A `dx-` control with no component sheet: only the theme's own rules apply to it. */
async function addProbe(page: Page): Promise<Locator> {
  await page.evaluate(() => {
    const b = document.createElement("button");
    b.id = "press-probe";
    b.className = "dx-press-probe";
    b.type = "button";
    b.textContent = "probe";
    b.style.cssText = "position:fixed;left:24px;bottom:24px;width:96px;height:48px;z-index:99999";
    document.body.append(b);
  });
  return page.locator("#press-probe");
}

async function center(el: Locator) {
  const box = (await el.boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

/**
 * Holds a finger down on `el` and returns the release function. `Input.dispatchTouchEvent` alone never
 * raises `:active` in headless Chromium (no tap-down gesture is derived from it); the emulated-touch
 * mouse event does, and still reports `pointerType: "touch"`.
 */
async function press(page: Page, el: Locator) {
  const client = await page.context().newCDPSession(page);
  const c = await center(el);
  const [x, y] = [Math.round(c.x), Math.round(c.y)]; // the protocol takes integers
  await client.send("Input.emulateTouchFromMouseEvent", { type: "mousePressed", x, y, button: "left", clickCount: 1 });
  const down = Date.now();
  return async () => {
    // A real touch lasts 40-100ms. Headless Chromium can leave `:active` set when the up lands within the
    // frame of the down (now that the press applies at once, a poll can be satisfied that fast).
    await page.waitForTimeout(Math.max(0, 60 - (Date.now() - down)));
    await client.send("Input.emulateTouchFromMouseEvent", { type: "mouseReleased", x, y, button: "left", clickCount: 1 });
    // No `client.detach()` here: detaching straight after the up event drops it and leaves `:active` set
    // (measured: 1 in 3 releases stuck). The session dies with the test's context.
  };
}

const style = (el: Locator, prop: string) =>
  el.evaluate((node, p) => getComputedStyle(node).getPropertyValue(p), prop);

test.describe("touch", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, colorScheme: "light" });
  test.skip(({ browserName }) => browserName !== "chromium", "CDP touch input and mobile emulation are Chromium-only");

  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, BUTTON_PAGE);
  });

  test("the emulated device cannot hover", async ({ page }) => {
    expect(await page.evaluate(() => matchMedia("(hover: hover) and (pointer: fine)").matches)).toBe(false);
  });

  test("after a tap there is no hover fill: the control looks like it did at rest", async ({ page }) => {
    // The outline Button has a hover fill (`--dx-muted`); a tap leaves `:hover` on it in a touch browser.
    const button = page.locator('.dx-button[data-style="outline"]:not([disabled])').first();
    await button.scrollIntoViewIfNeeded();
    const rest = await style(button, "background-color");
    await button.tap();
    // Past the colour transition and the press release.
    await page.waitForTimeout(500);
    expect(await style(button, "background-color")).toBe(rest);
    // The tap did leave :hover behind (that is the thing the media query neutralises); skip the claim if the
    // engine ever stops doing that, the assertion above is the contract.
    const stuck = await button.evaluate((el) => el.matches(":hover"));
    test.info().annotations.push({ type: "sticky-hover-emulated", description: String(stuck) });
  });

  test("during the press the control scales and takes the wash; on release it is back", async ({ page }) => {
    const button = page.locator('.dx-button[data-style="secondary"]:not([disabled])').first();
    await button.scrollIntoViewIfNeeded();
    expect(await style(button, "scale")).toBe("none");
    const release = await press(page, button);
    await expect.poll(() => style(button, "scale")).toBe("0.98");
    expect(await style(button, "background-image")).toContain("linear-gradient");
    await release();
    await expect.poll(() => style(button, "scale")).toBe("none");
  });

  // Backlog row 295. A tap on a phone is 40-100ms. These pin the two reasons it used to show nothing.

  test("the press lands instantly even where a component has its own transition (a tap is 40-100ms)", async ({ page }) => {
    const button = page.locator('.dx-button[data-style="secondary"]:not([disabled])').first();
    await button.scrollIntoViewIfNeeded();
    // Button owns `transition: all 150ms`, which a zero-specificity theme rule cannot override.
    expect(await style(button, "transition-duration")).toBe("0.15s");
    const release = await press(page, button);
    await expect.poll(() => style(button, "transition-duration")).toBe("0s");
    // At the first frame of :active the scale is already the full press scale, not a 150ms ramp toward it.
    expect(await style(button, "scale")).toBe("0.98");
    await release();
    await expect.poll(() => style(button, "transition-duration")).toBe("0.15s");
  });

  test("a short press leaves a visible tail: the scale eases back over frames, not in one", async ({ page }) => {
    const button = page.locator('.dx-button[data-style="secondary"]:not([disabled])').first();
    await button.scrollIntoViewIfNeeded();
    await button.evaluate((el) => {
      const w = window as unknown as { __s: number[]; __on: boolean };
      w.__s = [];
      w.__on = true;
      (function f() {
        if (!w.__on) return;
        const v = getComputedStyle(el).scale;
        w.__s.push(v === "none" ? 1 : parseFloat(v));
        requestAnimationFrame(f);
      })();
    });
    const release = await press(page, button);
    await release(); // holds the touch to a realistic 60ms
    await page.waitForTimeout(400);
    const frames = await page.evaluate(() => {
      const w = window as unknown as { __s: number[]; __on: boolean };
      w.__on = false;
      return w.__s;
    });
    const full = frames.indexOf(Math.min(...frames));
    expect(Math.min(...frames), "the press reached the full scale even though the touch was short").toBeCloseTo(0.98, 3);
    const tail = frames.slice(full).filter((v) => v > 0.98 && v < 0.999);
    expect(tail.length, `frames between pressed and rest after release: ${frames.join(",")}`).toBeGreaterThanOrEqual(2);
  });

  // The wash used to be a zero-specificity `background-image`, wiped by any component that writes the `background`
  // shorthand: 23 control families (Tabs, Menubar / Navbar / Navigation Menu triggers, Toolbar, ...) showed only a
  // sub-pixel scale. These are the ones the audit found; the gate-free guard is that each paints the wash.
  for (const [route, selector] of [
    ["tabs", ".dx-tabs-trigger"],
    ["menubar", ".dx-menubar-trigger"],
    ["toolbar", ".dx-toolbar-button"],
    ["navigation_menu", ".dx-navigation-menu-trigger"],
    ["carousel", ".dx-carousel-next"],
  ] as const) {
    test(`the wash paints on ${selector} although its sheet writes a background shorthand`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/${route}/`);
      const control = page.locator(`${selector}:not([disabled])`).first();
      await control.scrollIntoViewIfNeeded();
      expect(await style(control, "background-image")).toBe("none");
      const release = await press(page, control);
      await expect.poll(() => style(control, "background-image")).toContain("linear-gradient");
      expect(await style(control, "scale")).toBe("0.98");
      await release();
      await expect.poll(() => style(control, "background-image")).toBe("none");
    });
  }

  test("the switch removes it: with data-dx-press-feedback=off a press changes nothing", async ({ page }) => {
    await page.evaluate(() => document.documentElement.setAttribute("data-dx-press-feedback", "off"));
    const button = page.locator('.dx-button[data-style="secondary"]:not([disabled])').first();
    await button.scrollIntoViewIfNeeded();
    const release = await press(page, button);
    // Long enough for :active to have landed if it were going to apply scale.
    await page.waitForTimeout(400);
    expect(await style(button, "scale")).toBe("none");
    expect(await style(button, "background-image")).toBe("none");
    await release();
  });

  test("a tap does not hover-open a navbar or navigation menu by hover intent", async ({ page }) => {
    // The compat mouse events a tap produces must not start the 150 ms open-intent timer.
    await gotoHydrated(page, `${BASE_URL}/component/navigation_menu/`);
    const trigger = page
      .getByRole("navigation", { name: "Component navigation menu" })
      .getByRole("button", { name: "Getting started" });
    await expect(trigger).toBeVisible({ timeout: 60_000 });
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await trigger.tap();
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
    // Still open (not toggled shut by a stray hover open + click) a while later.
    await page.waitForTimeout(600);
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
  });
});

test.describe("transition and reduced motion", () => {
  test.use({ colorScheme: "light" });

  test("the press transition exists with no-preference and is gone with reduce", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await gotoHydrated(page, BUTTON_PAGE);
    const probe = await addProbe(page);
    expect(await style(probe, "transition-property")).toBe("scale");
    expect(await style(probe, "transition-duration")).toBe("0.2s");

    await page.emulateMedia({ reducedMotion: "reduce" });
    expect(await style(probe, "transition-property")).toBe("all");
    // The theme's reduced-motion layer pins durations to `--dx-motion-duration-reduced` (0.01ms): instant.
    expect(parseFloat(await style(probe, "transition-duration"))).toBeLessThan(0.001);
  });

  test("zero specificity: a stylesheet's own transition wins over the theme's", async ({ page }) => {
    await gotoHydrated(page, BUTTON_PAGE);
    const probe = await addProbe(page);
    await page.addStyleTag({ content: "#press-probe { transition: opacity 3s; }" });
    expect(await style(probe, "transition-property")).toBe("opacity");
  });
});

test.describe("theme picker switch", () => {
  test.use({ colorScheme: "light", viewport: { width: 1280, height: 800 } });

  test("Press feedback toggles the attribute, survives a reload before first paint, and Reset restores it", async ({ page }) => {
    await gotoHydrated(page, BUTTON_PAGE);
    const html = page.locator("html");
    await expect(html).not.toHaveAttribute("data-dx-press-feedback", /.+/);

    await page.getByRole("button", { name: "Theme", exact: true }).click();
    const dialog = page.locator(".dx-theme-picker");
    const toggle = dialog.getByRole("switch", { name: /Press feedback/ });
    await expect(toggle).toBeVisible();
    await expect(toggle).toHaveAttribute("aria-checked", "true");

    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-checked", "false");
    await expect(html).toHaveAttribute("data-dx-press-feedback", "off");
    expect((await page.context().cookies()).some((c) => c.name === "dx_dx_press-feedback" && c.value === "off")).toBe(true);

    // The pre-paint snippet applies it from the cookie: present at DOMContentLoaded, before hydration.
    await page.reload({ waitUntil: "domcontentloaded" });
    expect(await page.evaluate(() => document.documentElement.getAttribute("data-dx-press-feedback"))).toBe("off");
    await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });

    await page.getByRole("button", { name: "Theme", exact: true }).click();
    const again = page.locator(".dx-theme-picker").getByRole("switch", { name: /Press feedback/ });
    await expect(again).toHaveAttribute("aria-checked", "false");
    await page.locator(".dx-theme-picker").getByRole("button", { name: "Reset" }).click();
    await expect(again).toHaveAttribute("aria-checked", "true");
    await expect(html).not.toHaveAttribute("data-dx-press-feedback", /.+/);
  });
});
