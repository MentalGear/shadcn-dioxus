/**
 * An anchored panel is never painted outside the top layer, and keeps its open height through its whole close
 * (backlog row 492).
 *
 * A popover the browser closes natively (Escape, a light dismiss, a press on the trigger) leaves the top layer at once.
 * A component sheet's `display` would keep such a panel painting through the exit animation as an ordinary positioned
 * box, placed against another containing block and clipped by its ancestors, so
 * `primitives/src/top_layer/anchor.css` hides it for every component sheet.
 *
 *   - ColorPicker: the close is recorded on every animation frame from the close request for 500 ms,
 *     above and below its trigger, at a desktop and a phone viewport, for each way of closing. Every frame is either
 *     not rendered or the open panel: in the top layer, its height within 1 px of the open height.
 *   - Every other anchored overlay that closes natively: no frame after Escape or an outside press paints a panel
 *     that is not in the top layer.
 */
import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { awaitAnchoredPanelSettled } from "./animations";

const PAGE_URL = `${BASE_URL}/component/color_picker/`;
const NAV_TIMEOUT = 20 * 60 * 1000;

type Frame = {
  t: number;
  h: number;
  top: number;
  maxHeight: string;
  transform: string;
  translate: string;
  opacity: string;
  side: string | null;
  anims: string;
  open: boolean;
  display: string;
};

/** Starts the frame recording: one frame per animation tick for 500 ms, longer than the close animation. */
async function startRecording(page: Page) {
  await page.evaluate(() => {
    const panel = document.querySelector<HTMLElement>("[data-dx-anchored][popover]")!;
    const frames: Frame[] = [];
    const t0 = performance.now();
    const w = window as unknown as { __frames: Frame[]; __recording: boolean };
    w.__frames = frames;
    w.__recording = true;
    const tick = () => {
      const open = panel.isConnected && panel.matches(":popover-open");
      const r = panel.getBoundingClientRect();
      const cs = getComputedStyle(panel);
      frames.push({
        t: Math.round(performance.now() - t0),
        h: Math.round(r.height * 10) / 10,
        top: Math.round(r.top * 10) / 10,
        maxHeight: cs.maxHeight,
        transform: cs.transform,
        translate: cs.translate,
        opacity: cs.opacity,
        side: panel.getAttribute("data-placed-side"),
        anims: panel.getAnimations().map((a) => (a as CSSAnimation).animationName ?? "?").join(","),
        open,
        display: panel.isConnected ? cs.display : "none",
      });
      if (performance.now() - t0 < 500) requestAnimationFrame(tick);
      else w.__recording = false;
    };
    requestAnimationFrame(tick);
  });
}

async function stopRecording(page: Page): Promise<Frame[]> {
  await page.waitForFunction(() => !(window as unknown as { __recording: boolean }).__recording, null, { timeout: 5000 });
  return page.evaluate(() => (window as unknown as { __frames: Frame[] }).__frames);
}

const CLOSERS = {
  Escape: (page: Page) => page.keyboard.press("Escape"),
  "outside press": (page: Page) => page.mouse.click(2, 2),
  "trigger press": (page: Page) => page.getByRole("button", { name: /Color picker/i }).first().click(),
} as const;

const VIEWPORTS = {
  desktop: { width: 1280, height: 800 },
  phone: { width: 390, height: 844 },
} as const;

for (const [vpName, viewport] of Object.entries(VIEWPORTS)) {
  for (const placement of ["below", "above"] as const) {
    // One load per viewport and placement; the three ways of closing run one after the other on it, each reopening the panel.
    test(`ColorPicker ${placement} its trigger (${vpName}): the panel keeps its open height through the close (every closer)`, async ({
      page,
    }) => {
      await page.setViewportSize(viewport);
      await gotoHydrated(page, PAGE_URL, { timeout: NAV_TIMEOUT });
      const trigger = page.getByRole("button", { name: /Color picker/i }).first();
      await page.evaluate(() => {
        document.body.style.paddingTop = "3000px";
        document.body.style.paddingBottom = "3000px";
      });
      await trigger.evaluate((el, p) => {
        el.scrollIntoView({ block: p === "above" ? "end" : "start" });
        window.scrollBy(0, p === "above" ? 24 : -24);
      }, placement);
      for (const [closerName, close] of Object.entries(CLOSERS)) {
        await test.step(closerName, async () => {
          await trigger.click();
          const panel = page.locator("[data-dx-anchored][popover]").first();
          await expect(panel).toBeVisible();
          await awaitAnchoredPanelSettled(panel);
          const openFrame = await panel.evaluate((el) => ({
            h: el.getBoundingClientRect().height,
            top: el.getBoundingClientRect().top,
          }));
          const triggerTop = await trigger.evaluate((el) => el.getBoundingClientRect().top);
          expect(openFrame.top < triggerTop ? "above" : "below", `${closerName}: the side the panel opened on`).toBe(placement);

          await startRecording(page);
          await close(page);
          const frames = await stopRecording(page);
          if (process.env.RECORD_CLOSE) console.log(JSON.stringify({ vpName, placement, closerName, open: openFrame, frames }));

          expect(frames.length, `${closerName}: frames recorded`).toBeGreaterThan(2);
          const rendered = frames.filter((f) => f.display !== "none");
          expect(
            rendered.filter((f) => !f.open),
            `${closerName}: frames that paint the panel outside the top layer`,
          ).toEqual([]);
          const clipped = rendered.filter((f) => Math.abs(f.h - openFrame.h) > 1);
          expect(clipped, `${closerName}: frames whose height differs from the open height ${openFrame.h}px`).toEqual([]);
        });
      }
    });
  }
}

/** The other anchored overlays that close natively, on the shared fixture page and the date picker's own. */
const FIXTURE = `${BASE_URL}/component/top_layer/`;
const OVERLAYS: { name: string; url: string; open: (page: Page) => Promise<void> }[] = [
  { name: "Popover", url: FIXTURE, open: (page) => page.locator("#clip-popover-trigger").click() },
  { name: "DropdownMenu", url: FIXTURE, open: (page) => page.locator("#clip-dropdown-menu-trigger").click() },
  { name: "Menubar", url: FIXTURE, open: (page) => page.locator("#clip-menubar-trigger").click() },
  {
    name: "Combobox",
    url: FIXTURE,
    open: async (page) => {
      await page.locator("#clip-combobox-trigger").click();
      await page.keyboard.press("ArrowDown");
    },
  },
  {
    name: "Tooltip",
    url: FIXTURE,
    // The tooltip opens on focus; a trigger that kept focus through the first close is blurred so the second open is a focus event.
    open: async (page) => {
      const trigger = page.locator("#clip-tooltip-trigger");
      await trigger.blur();
      await trigger.focus();
    },
  },
  { name: "Navbar", url: FIXTURE, open: (page) => page.locator("#clip-navbar-trigger").click() },
  { name: "NavigationMenu", url: FIXTURE, open: (page) => page.locator("#clip-navigation-menu-trigger").click() },
  {
    name: "DatePicker",
    url: `${BASE_URL}/component/date_picker/`,
    open: (page) => page.getByRole("button", { name: "Show Calendar" }).first().click(),
  },
];

for (const overlay of OVERLAYS) {
  // One load per overlay; both ways of closing run on it, each reopening the panel.
  test(`${overlay.name}: no frame of a native close paints the panel outside the top layer (Escape, outside press)`, async ({
    page,
  }) => {
    await gotoHydrated(page, overlay.url, { timeout: NAV_TIMEOUT });
    for (const closerName of ["Escape", "outside press"] as const) {
      await test.step(closerName, async () => {
        await overlay.open(page);
        const panel = page.locator("[data-dx-anchored][popover]:popover-open").first();
        await expect(panel).toBeVisible();
        await awaitAnchoredPanelSettled(panel);
        await page.evaluate(() => {
          const w = window as unknown as { __stray: string[]; __sampling: boolean };
          w.__stray = [];
          w.__sampling = true;
          const t0 = performance.now();
          const tick = () => {
            for (const el of document.querySelectorAll<HTMLElement>("[data-dx-anchored][popover]")) {
              if (!el.matches(":popover-open") && getComputedStyle(el).display !== "none") {
                w.__stray.push(`${el.id || el.className} at ${Math.round(performance.now() - t0)} ms`);
              }
            }
            if (performance.now() - t0 < 600) requestAnimationFrame(tick);
            else w.__sampling = false;
          };
          requestAnimationFrame(tick);
        });
        await (closerName === "Escape" ? page.keyboard.press("Escape") : page.mouse.click(2, 2));
        await page.waitForFunction(() => !(window as unknown as { __sampling: boolean }).__sampling);
        const stray = await page.evaluate(() => (window as unknown as { __stray: string[] }).__stray);
        expect(stray, `${closerName}: panels painted outside the top layer`).toEqual([]);
      });
    }
  });
}
