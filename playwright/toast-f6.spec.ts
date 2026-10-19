import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";

/** The toast docs page mounts one provider per demo, each with its own named region. */
const region = (page: Page, name: string) => page.getByRole("region", { name });
const open = (page: Page) => page.goto(`${BASE_URL}/component/?name=toast&`);

const SWIPE = "Swipe toast notifications";
const TOP = "Top toast notifications"; // mounted after SWIPE

test.describe("toast: one F6 handler per page (row 262)", () => {
  test("F6 focuses the region that holds toasts, not the last-mounted provider", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show swipeable toast" }).click();
    await expect(region(page, SWIPE).getByRole("alertdialog")).toHaveCount(1);
    await page.locator("body").click({ position: { x: 5, y: 5 } });
    await page.keyboard.press("F6");
    await expect(region(page, SWIPE)).toBeFocused();
  });

  test("F6 does nothing while no region holds a toast", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show swipeable toast" }).click();
    const t = region(page, SWIPE).getByRole("alertdialog");
    await expect(t).toHaveCount(1);
    await t.getByRole("button").first().click(); // close
    await expect(t).toHaveCount(0);
    await page.locator("body").click({ position: { x: 5, y: 5 } });
    await page.keyboard.press("F6");
    await expect(region(page, SWIPE)).not.toBeFocused();
    await expect(region(page, TOP)).not.toBeFocused();
  });

  test("with toasts in two regions F6 starts at the most recent and cycles, wrapping", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show swipeable toast" }).click();
    await expect(region(page, SWIPE).getByRole("alertdialog")).toHaveCount(1);
    await page.getByRole("button", { name: "Show top toast" }).click();
    await expect(region(page, TOP).getByRole("alertdialog")).toHaveCount(1);
    await page.locator("body").click({ position: { x: 5, y: 5 } });

    await page.keyboard.press("F6");
    await expect(region(page, TOP)).toBeFocused();
    await page.keyboard.press("F6");
    await expect(region(page, SWIPE)).toBeFocused();
    await page.keyboard.press("F6");
    await expect(region(page, TOP)).toBeFocused();
  });

  test("the page holds exactly one F6 handler, and no per-provider keydown listener from toast", async ({ page }) => {
    await open(page);
    await page.waitForTimeout(800);
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Debugger.enable"); // handler object ids are only handed out with the debugger on
    const doc = await cdp.send("Runtime.evaluate", { expression: "document" });
    const { listeners } = await cdp.send("DOMDebugger.getEventListeners", { objectId: doc.result.objectId! });
    let f6 = 0;
    for (const l of listeners.filter((x) => x.type === "keydown")) {
      const src = await cdp.send("Runtime.callFunctionOn", {
        objectId: l.handler!.objectId!,
        functionDeclaration: "function(){ return Function.prototype.toString.call(this); }",
        returnByValue: true,
      });
      if (String(src.result.value).includes("'F6'")) f6++;
    }
    // Eight providers are mounted on this page: a per-provider handler would make this 8.
    expect(f6).toBe(1);
  });
});
