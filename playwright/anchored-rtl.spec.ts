/**
 * Anchored overlays under `dir="rtl"` (backlog row 488): the panel opens inside the viewport, beside its
 * anchor, and a panel aligned to the inline start hangs from the anchor's right edge. Checked on the engine's own
 * path and on the script path (`stripAnchorSupportsBlock`), for the overlays that open below a trigger and for
 * the Select's item-aligned mode, which places itself in viewport pixels.
 *
 * The page direction is set on `<html>` after hydration, the way the page-level direction toggle does, so the
 * panel inherits it from its trigger's ancestors.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { stripAnchorSupportsBlock } from "./no-anchor-engine";

const FIXTURE = `${BASE_URL}/component/top_layer/`;
const NAV_TIMEOUT = 20 * 60 * 1000;

type Overlay = {
  name: string;
  trigger: string;
  content: string;
  open: (page: Page) => Promise<void>;
  /** The panel's right edge sits at the anchor's right edge (inline start in RTL). */
  startAligned: boolean;
};

const click = (id: string) => (page: Page) => page.locator(`#${id}`).click();

const OVERLAYS: Overlay[] = [
  { name: "Popover", trigger: "clip-popover-trigger", content: "clip-popover-content", open: click("clip-popover-trigger"), startAligned: false },
  { name: "DropdownMenu", trigger: "clip-dropdown-menu-trigger", content: "clip-dropdown-menu-content", open: click("clip-dropdown-menu-trigger"), startAligned: false },
  { name: "Select (popper)", trigger: "clip-select-trigger", content: "clip-select-content", open: click("clip-select-trigger"), startAligned: true },
  {
    name: "Combobox",
    trigger: "clip-combobox-trigger",
    content: "clip-combobox-content",
    open: async (page) => {
      await page.locator("#clip-combobox-trigger").click();
      await page.keyboard.press("ArrowDown");
    },
    startAligned: true,
  },
];

const rect = (locator: Locator) =>
  locator.first().evaluate((el) => {
    const r = el.getBoundingClientRect();
    return { l: r.left, r: r.right, t: r.top, b: r.bottom, w: r.width, h: r.height };
  });

async function rtl(page: Page) {
  await page.evaluate(() => {
    document.documentElement.dir = "rtl";
    document.body.style.paddingTop = "600px";
    document.body.style.paddingBottom = "1500px";
  });
}

for (const scriptPath of [false, true]) {
  test.describe(`RTL anchored placement, ${scriptPath ? "script path" : "engine path"}`, () => {
    for (const overlay of OVERLAYS) {
      test(`${overlay.name}: opens inside the viewport beside its trigger`, async ({ page }) => {
        await gotoHydrated(page, FIXTURE, { timeout: NAV_TIMEOUT });
        if (scriptPath) await stripAnchorSupportsBlock(page);
        await rtl(page);
        await page.locator(`#${overlay.trigger}`).evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
        await overlay.open(page);
        const content = page.locator(`#${overlay.content}`);
        await expect(content).toBeVisible();
        await page.waitForTimeout(400);
        const [c, t, vp] = await Promise.all([
          rect(content),
          rect(page.locator(`#${overlay.trigger}`)),
          page.evaluate(() => ({ w: document.documentElement.clientWidth, h: window.innerHeight })),
        ]);
        const where = JSON.stringify({ c, t, vp });
        expect(c.w, `the panel has a width ${where}`).toBeGreaterThan(20);
        expect(c.l, `inside the viewport on the left ${where}`).toBeGreaterThanOrEqual(-1);
        expect(c.r, `inside the viewport on the right ${where}`).toBeLessThanOrEqual(vp.w + 1);
        expect(c.t, `inside the viewport at the top ${where}`).toBeGreaterThanOrEqual(-1);
        expect(c.b, `inside the viewport at the bottom ${where}`).toBeLessThanOrEqual(vp.h + 1);
        expect(c.l < t.r && c.r > t.l, `the panel overlaps its trigger's columns ${where}`).toBe(true);
        if (overlay.startAligned) {
          expect(Math.abs(c.r - t.r), `the panel's right edge is the trigger's ${where}`).toBeLessThanOrEqual(2);
        }
      });
    }
  });
}

test("Select (item-aligned): opens inside the viewport over its trigger in RTL", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/select/`, { timeout: NAV_TIMEOUT });
  await page.evaluate(() => (document.documentElement.dir = "rtl"));
  const frame = page.locator("#component-preview-frame-aligned").first();
  await frame.scrollIntoViewIfNeeded();
  const trigger = frame.getByRole("combobox").first();
  await trigger.click();
  const list = page.locator("[role='listbox'][data-state='open']").first();
  await expect(list).toBeVisible();
  await page.waitForTimeout(400);
  const [c, t, vp] = await Promise.all([
    rect(list),
    rect(trigger),
    page.evaluate(() => ({ w: document.documentElement.clientWidth, h: window.innerHeight })),
  ]);
  const where = JSON.stringify({ c, t, vp });
  expect(c.l, where).toBeGreaterThanOrEqual(-1);
  expect(c.r, where).toBeLessThanOrEqual(vp.w + 1);
  expect(c.t, where).toBeGreaterThanOrEqual(-1);
  expect(c.b, where).toBeLessThanOrEqual(vp.h + 1);
  expect(c.l < t.r && c.r > t.l, `overlaps the trigger's columns ${where}`).toBe(true);
});
