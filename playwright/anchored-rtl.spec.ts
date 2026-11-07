/**
 * Anchored overlays under `dir="rtl"` (backlog row 488): the panel opens inside the viewport, beside its
 * anchor, and a panel aligned to the inline start hangs from the anchor's right edge. Checked on the engine's own
 * path and on the script path (`stripAnchorSupportsBlock`), for the overlays that open below a trigger and for
 * the component pages' own RTL demos (Combobox, item-aligned Select, DropdownMenu).
 *
 * On the fixture the direction is flipped after hydration on `<html>` and on every explicit `dir="ltr"` (the demo
 * frame carries one), so the panel inherits it from its trigger's ancestors.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { stripAnchorSupportsBlock } from "./no-anchor-engine";
import { fixedViewportBox } from "./viewport-box";

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

/** `<html dir="rtl">` and every explicit `dir="ltr"` on the way to a trigger (the demo frame carries one). */
const flipDirection = () => {
  document.documentElement.dir = "rtl";
  document.querySelectorAll('[dir="ltr"]').forEach((el) => el.setAttribute("dir", "rtl"));
};

async function rtl(page: Page) {
  await page.evaluate(flipDirection);
  await page.evaluate(() => {
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
          fixedViewportBox(page).then((b) => ({ w: b.x + b.width, h: b.y + b.height })),
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

// The component pages' own RTL demos: a `DirectionProvider` and `dir="rtl"` on the demo root, as the page-level
// direction toggle gives them. A Select takes its direction from that provider (the fixture above has none).
type Demo = { name: string; page: string; frame: string; trigger: (f: Locator) => Locator; anchor?: (f: Locator) => Locator; aligned: boolean };
const DEMOS: Demo[] = [
  { name: "Combobox", page: "combobox", frame: "rtl", trigger: (f: Locator) => f.getByRole("combobox").first(), anchor: (f: Locator) => f.locator("[data-dx-combobox-chips]"), aligned: true },
  { name: "Select (item-aligned)", page: "select", frame: "rtl", trigger: (f: Locator) => f.getByRole("combobox").first(), aligned: false },
  { name: "DropdownMenu", page: "dropdown_menu", frame: "rtl", trigger: (f: Locator) => f.getByRole("button").first(), aligned: false },
];

for (const demo of DEMOS) {
  test(`${demo.name}: the RTL demo opens its panel inside the viewport over or beside its trigger`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/${demo.page}/`, { timeout: NAV_TIMEOUT });
    const frame = page.locator(`#component-preview-frame-${demo.frame}`).first();
    await frame.scrollIntoViewIfNeeded();
    const trigger = demo.trigger(frame);
    const anchor = demo.anchor ? demo.anchor(frame) : trigger;
    await trigger.click();
    const panel = page.locator("[role='listbox']:popover-open, [role='menu']:popover-open").first();
    await expect(panel).toBeVisible();
    await page.waitForTimeout(400);
    const [c, t, vp] = await Promise.all([
      rect(panel),
      rect(anchor),
      fixedViewportBox(page).then((b) => ({ w: b.x + b.width, h: b.y + b.height })),
    ]);
    const where = JSON.stringify({ c, t, vp });
    expect(c.w, `the panel has a width ${where}`).toBeGreaterThan(20);
    expect(c.l, where).toBeGreaterThanOrEqual(-1);
    expect(c.r, where).toBeLessThanOrEqual(vp.w + 1);
    expect(c.t, where).toBeGreaterThanOrEqual(-1);
    expect(c.b, where).toBeLessThanOrEqual(vp.h + 1);
    expect(c.l < t.r && c.r > t.l, `the panel overlaps its trigger's columns ${where}`).toBe(true);
    if (demo.aligned) expect(Math.abs(c.r - t.r), `inline start is the right edge ${where}`).toBeLessThanOrEqual(2);
  });
}
