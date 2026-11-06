/**
 * Row 491: the Drawer's scrollable-content demo. The body is the scroll container while the header and
 * footer stay put and the page behind does not scroll; with the on-screen keyboard emulated (the
 * visualViewport stub of `keyboard-inset.spec.ts`, see its header for why), a field in the scrolling
 * body is brought into the visible band.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { awaitAnimationsSettled, measureAtRest } from "./animations";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const PHONE = { width: 390, height: 844 };
const KEYBOARD = 336;

/** A stub of `window.visualViewport` whose height is the layout viewport minus the keyboard. */
async function stubVisualViewport(page: Page) {
  await page.addInitScript((layout) => {
    const state = { kb: 0 };
    const stub: any = new EventTarget();
    Object.defineProperties(stub, {
      width: { get: () => layout.width },
      // The keyboard is the part of the layout viewport the visual one lost.
      height: { get: () => layout.height - state.kb },
      offsetTop: { get: () => 0 },
      offsetLeft: { get: () => 0 },
      pageTop: { get: () => 0 },
      pageLeft: { get: () => 0 },
      scale: { get: () => 1 },
    });
    Object.defineProperty(window, "visualViewport", { value: stub, configurable: true });
    (window as any).__dxKb = {
      set(kb: number) {
        state.kb = kb;
        stub.dispatchEvent(new Event("resize"));
        stub.dispatchEvent(new Event("scroll"));
      },
    };
  }, PHONE);
}

for (const mode of ["light", "dark"] as const) {
  test.describe(`drawer/scrollable_content, ${mode}`, () => {
    test.use({ colorScheme: mode, viewport: PHONE });

    async function openDrawer(page: Page) {
      await stubVisualViewport(page);
      await gotoHydrated(page, `${BASE_URL}/component/drawer/`, GOTO_OPTS);
      await page.evaluate((m) => (window as any).__dxApplyMode(m), mode);
      const root = page.locator("#component-preview-frame-scrollable_content").first();
      await root.scrollIntoViewIfNeeded();
      const trigger = root.getByRole("button", { name: "Open scrollable drawer" });
      await expect(trigger).toBeVisible({ timeout: 60_000 });
      await trigger.click();
      const dialog = page.getByRole("dialog", { name: "Release notes" });
      await expect(dialog).toBeVisible();
      await awaitAnimationsSettled(dialog);
      return dialog;
    }

    test("the body is the scroll container; header and footer stay put; the page does not scroll", async ({ page }) => {
      const dialog = await openDrawer(page);
      const body = dialog.getByRole("region", { name: "Release notes list" });
      const title = dialog.getByRole("heading", { name: "Release notes" });
      const done = dialog.getByRole("button", { name: "Done reading" });
      const m = await body.evaluate((el) => ({ sh: el.scrollHeight, ch: el.clientHeight, oy: getComputedStyle(el).overflowY }));
      expect(m.sh).toBeGreaterThan(m.ch);
      expect(m.oy).toBe("auto");
      // The panel itself is not the scroller.
      expect(await dialog.evaluate((el) => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(1);
      const top = async (l: typeof title) => Math.round((await measureAtRest(l)).y);
      const [t0, d0] = [await top(title), await top(done)];
      const scrollY0 = await page.evaluate(() => window.scrollY);
      await body.hover();
      await page.mouse.wheel(0, 600);
      await expect.poll(() => body.evaluate((el) => el.scrollTop)).toBeGreaterThan(0);
      expect(await top(title)).toBe(t0);
      expect(await top(done)).toBe(d0);
      await expect(done).toBeInViewport();
      expect(await page.evaluate(() => window.scrollY)).toBe(scrollY0);
      await expectNoAxeViolations(page, `drawer/scrollable_content phone ${mode}`);
      if (mode === "light") await page.screenshot({ path: test.info().outputPath("drawer-scrollable-phone.png") });
      await done.click();
      await expect(dialog).toBeHidden();
    });

    test("with the keyboard up, the feedback field in the scrolling body is inside the visible band", async ({ page }) => {
      const dialog = await openDrawer(page);
      const body = dialog.getByRole("region", { name: "Release notes list" });
      const field = dialog.getByRole("textbox", { name: "Feedback" });
      await page.evaluate((kb) => (window as any).__dxKb.set(kb), KEYBOARD);
      await expect(dialog).toHaveAttribute("data-dx-keyboard", "open");
      expect(await dialog.evaluate((el) => el.style.getPropertyValue("--dx-keyboard-inset"))).toBe(`${KEYBOARD}px`);
      // `preventScroll`: the browser's own focus scroll would pass without the helper.
      await field.evaluate((el) => (el as HTMLElement).focus({ preventScroll: true }));
      const bandBottom = PHONE.height - KEYBOARD;
      await expect
        .poll(async () => {
          const f = await measureAtRest(field);
          const b = await measureAtRest(body);
          return f.y >= Math.max(0, b.y) - 0.5 && f.y + f.height <= Math.min(bandBottom, b.y + b.height) + 0.5;
        })
        .toBe(true);
      // The field was scrolled to, so the body scrolled and the footer is still above the keyboard.
      expect(await body.evaluate((el) => el.scrollTop)).toBeGreaterThan(0);
      const footer = await measureAtRest(dialog.getByRole("button", { name: "Done reading" }));
      expect(footer.y + footer.height).toBeLessThanOrEqual(bandBottom + 1);
      await page.evaluate(() => (window as any).__dxKb.set(0));
      await expect(dialog).not.toHaveAttribute("data-dx-keyboard", /.*/);
    });
  });
}
