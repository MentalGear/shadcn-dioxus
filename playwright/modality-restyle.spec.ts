/**
 * The first key press does not restyle the page (backlog row 490).
 *
 * The focus modality flips on the first key press of a mouse-driven page. The ring tokens it zeroes live on the
 * FOCUSED element (`[data-dx-pointer-focus]`, theme "Focus modality"), not on `:root`, so the flip restyles that one
 * element instead of every rendered element. This is a COUNT, not a duration: Chromium's `UpdateLayoutTree` trace
 * events report how many elements each style recalculation touched, which does not move with machine speed.
 *
 *   - typing the first character in a focused Combobox input, and an ArrowDown from mouse modality, each restyle a
 *     handful of elements (about 20 on a ~5,000 element page), not the ~950 a `:root` token flip touches;
 *   - the ring tokens read on `:root` are the same before and after the flip (nothing is redefined there);
 *   - the pointer-focus marker follows focus and the modality: set on a script-focused button under mouse modality,
 *     gone once a key press makes the modality `keyboard`, and the keyboard-focused button draws its ring.
 */
import type { Page } from "@playwright/test";
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/** Elements restyled by every style recalculation that `action` causes, summed. */
async function restyledElements(page: Page, action: () => Promise<void>): Promise<number> {
  const cdp = await page.context().newCDPSession(page);
  const done = new Promise<{ stream: string }>((resolve) => cdp.once("Tracing.tracingComplete", resolve as never));
  await cdp.send("Tracing.start", { categories: "devtools.timeline", transferMode: "ReturnAsStream" });
  await action();
  await page.waitForTimeout(300);
  await cdp.send("Tracing.end");
  const { stream } = await done;
  let text = "";
  for (;;) {
    const chunk = await cdp.send("IO.read", { handle: stream });
    text += chunk.data;
    if (chunk.eof) break;
  }
  const parsed = JSON.parse(text);
  const events: { name: string; ph: string; args?: { elementCount?: number } }[] = parsed.traceEvents ?? parsed;
  return events
    .filter((e) => e.name === "UpdateLayoutTree" && e.ph === "X")
    .reduce((sum, e) => sum + (e.args?.elementCount ?? 0), 0);
}

async function loadCombobox(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/component/combobox/`);
  const input = page.locator('input[role="combobox"]').first();
  await input.click();
  await expect(page.locator("html")).toHaveAttribute("data-dx-modality", "mouse");
  await page.waitForTimeout(500);
  return input;
}

test.describe("the first key press restyles a handful of elements, not the page", () => {
  test.skip(({ browserName }) => browserName !== "chromium", "style recalculation counts come from Chromium's trace");

  test("typing the first character in the Combobox input", async ({ page }) => {
    await loadCombobox(page);
    const total = await page.evaluate(() => document.querySelectorAll("*").length);
    const restyled = await restyledElements(page, () => page.keyboard.press("a"));
    await expect(page.locator("html")).toHaveAttribute("data-dx-modality", "keyboard");
    expect(restyled, `of ${total} elements`).toBeLessThan(100);
  });

  test("ArrowDown from mouse modality", async ({ page }) => {
    await loadCombobox(page);
    const total = await page.evaluate(() => document.querySelectorAll("*").length);
    const restyled = await restyledElements(page, () => page.keyboard.press("ArrowDown"));
    await expect(page.locator("html")).toHaveAttribute("data-dx-modality", "keyboard");
    expect(restyled, `of ${total} elements`).toBeLessThan(100);
  });
});

test("the flip redefines no ring token on the root", async ({ page }) => {
  await loadCombobox(page);
  const read = () =>
    page.evaluate(() => {
      const cs = getComputedStyle(document.documentElement);
      return ["--dx-ring", "--dx-ring-focus", "--dx-ring-focus-inset"].map((n) => cs.getPropertyValue(n).trim());
    });
  const before = await read();
  expect(before.every((v) => v !== "" && v !== "0 0 #0000" && v !== "0 0 transparent")).toBe(true);
  await page.keyboard.press("a");
  await expect(page.locator("html")).toHaveAttribute("data-dx-modality", "keyboard");
  expect(await read()).toEqual(before);
});

test("the pointer-focus marker follows focus and the modality", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button/`);
  await page.locator("html").evaluate(() => document.documentElement.setAttribute("data-dx-modality", "mouse"));
  const button = page.locator("button.dx-button").first();
  // A script focus under mouse modality (Safari's tapped button, a menu's roving focus): no ring.
  await button.evaluate((el) => (el as HTMLElement).focus());
  await expect(button).toHaveAttribute("data-dx-pointer-focus", "");
  await expect(page.locator("html")).not.toHaveAttribute("data-dx-pointer-focus", /.*/);
  // The first key press makes the modality keyboard: the marker leaves the focused element.
  await page.keyboard.press("Shift");
  await expect(button).toHaveAttribute("data-dx-pointer-focus", "");
  await page.keyboard.press("ArrowRight");
  await expect(page.locator("html")).toHaveAttribute("data-dx-modality", "keyboard");
  await expect(button).not.toHaveAttribute("data-dx-pointer-focus", /.*/);
  // Focus moved under keyboard modality is never marked, and the ring shows.
  await button.evaluate((el) => (el as HTMLElement).blur());
  await button.evaluate((el) => (el as HTMLElement).focus());
  await expect(button).not.toHaveAttribute("data-dx-pointer-focus", /.*/);
  expect(await button.evaluate((el) => getComputedStyle(el).boxShadow)).not.toBe("none");
});
