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
  return async () => {
    await client.send("Input.emulateTouchFromMouseEvent", { type: "mouseReleased", x, y, button: "left", clickCount: 1 });
    await client.detach();
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
    expect(await style(probe, "transition-duration")).toBe("0.1s");

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
