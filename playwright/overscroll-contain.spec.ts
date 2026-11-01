/**
 * A scrollable list inside an anchored overlay does not chain its scroll to the page once the list
 * reaches its end, and the page stays scrollable from outside the list (backlog row 477: no page lock,
 * the panel stays open). The construction is `overscroll-behavior: contain`
 * on the anchored panel (`primitives/src/top_layer/anchor.css`, every `[data-dx-anchored]` overlay) and,
 * for the Select's item-aligned list, which carries no `data-dx-anchored`, on that list's own stylesheet
 * rule (`preview/src/components/select/style.css`).
 *
 * Per surface (Select, Combobox, DropdownMenu): open it with the viewport short enough that its list is
 * taller than its room, so it scrolls; wheel over the list until it reaches its end; keep wheeling and the
 * page's `scrollY` does not change; then wheel over the page outside the panel: the page scrolls and the
 * panel stays open. The scrolling element's computed `overscroll-behavior-y` is `contain`.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { awaitAnimationsSettled } from "./animations";
import { pageMetrics } from "./viewport-box";

const NAV_TIMEOUT = 20 * 60 * 1000;

type Surface = {
  name: string;
  url: string;
  /** Short enough that the list is taller than the room on the side it opens to, so it scrolls in its own box. */
  height: number;
  trigger: (page: Page) => Locator;
  panel: (page: Page) => Locator;
};

const SURFACES: Surface[] = [
  {
    name: "Select",
    url: `${BASE_URL}/component/select/`,
    height: 360,
    trigger: (page) => page.getByRole("combobox", { name: "Time zone select" }),
    panel: (page) => page.getByRole("listbox", { name: "Time zone options" }),
  },
  {
    name: "Combobox",
    url: `${BASE_URL}/component/combobox/`,
    height: 360,
    trigger: (page) => page.getByRole("combobox", { name: "Select framework" }),
    panel: (page) => page.locator("[role='listbox'][data-state='open']"),
  },
  {
    name: "DropdownMenu",
    url: `${BASE_URL}/component/dropdown_menu/`,
    height: 240,
    trigger: (page) => page.getByRole("button", { name: "Open Menu" }),
    panel: (page) => page.getByRole("menu"),
  },
];

/** Gives the page room to scroll in both directions, so the wheel over the page always can. */
async function padPage(page: Page) {
  await page.evaluate(() => {
    document.body.style.paddingTop = "3000px";
    document.body.style.paddingBottom = "3000px";
  });
}

/** Resolves once the panel's scroll position and the page's `scrollY` stop changing (smooth wheel scrolls). */
async function settle(panel: Locator) {
  await panel.evaluate(
    (el) =>
      new Promise<void>((resolve) => {
        let prev = "";
        let still = 0;
        const frame = () => {
          const now = `${el.scrollTop}|${window.scrollY}`;
          still = now === prev ? still + 1 : 0;
          prev = now;
          if (still >= 6) resolve();
          else requestAnimationFrame(frame);
        };
        requestAnimationFrame(frame);
      }),
  );
}

/** The element that scrolls the list: the panel itself, or the first scrollable descendant of it. */
async function scrollerOf(panel: Locator) {
  return panel.evaluate((el) => {
    const scrolls = (e: Element) => {
      const cs = getComputedStyle(e);
      return /auto|scroll/.test(cs.overflowY) && e.scrollHeight > e.clientHeight + 1;
    };
    const found = scrolls(el) ? el : [...el.querySelectorAll("*")].find(scrolls) ?? null;
    if (!found) return null;
    return {
      isPanel: found === el,
      overscrollY: getComputedStyle(found).overscrollBehaviorY,
      scrollTop: found.scrollTop,
      atEnd: found.scrollTop + found.clientHeight >= found.scrollHeight - 1,
    };
  });
}

/**
 * A point on the page that is outside the panel and the trigger, and whose wheel would scroll the page
 * itself: no scrollable ancestor other than the document, no overlay in the way.
 */
async function pagePoint(page: Page, panel: Locator, trigger: Locator) {
  const [p, t] = await Promise.all([panel.boundingBox(), trigger.boundingBox()]);
  const m = await pageMetrics(page);
  const outside = await page.evaluate(
    ({ p, t, width, height }) => {
      const within = (r: { x: number; y: number; width: number; height: number } | null, x: number, y: number) =>
        !!r && x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height;
      const scrollsOwn = (e: Element) => {
        const cs = getComputedStyle(e);
        return /auto|scroll/.test(cs.overflowY) && e.scrollHeight > e.clientHeight + 1;
      };
      for (let y = 8; y < height - 8; y += 16) {
        for (let x = 8; x < width - 8; x += 16) {
          if (within(p, x, y) || within(t, x, y)) continue;
          const hit = document.elementFromPoint(x, y);
          if (!hit || hit.closest("[data-dx-anchored], [popover], dialog, [role='listbox'], [role='menu']")) continue;
          let chain = false;
          for (let e: Element | null = hit; e && e !== document.documentElement; e = e.parentElement) {
            if (scrollsOwn(e)) {
              chain = true;
              break;
            }
          }
          if (!chain) return { x, y };
        }
      }
      return null;
    },
    {
      p: p ? { x: p.x, y: p.y, width: p.width, height: p.height } : null,
      t: t ? { x: t.x, y: t.y, width: t.width, height: t.height } : null,
      width: m.innerWidth,
      height: m.innerHeight,
    },
  );
  expect(outside, "a point outside the panel where the wheel scrolls the page").not.toBeNull();
  return outside!;
}

for (const surface of SURFACES) {
  test(`${surface.name}: a wheel past the list's end does not scroll the page, and the page scrolls from outside`, async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1024, height: surface.height });
    await gotoHydrated(page, surface.url, { timeout: NAV_TIMEOUT });
    await padPage(page);

    const trigger = surface.trigger(page);
    const panel = surface.panel(page);
    await trigger.evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
    await trigger.click();
    await expect(panel).toHaveAttribute("data-state", "open");
    await awaitAnimationsSettled(panel);

    // The list scrolls, and it is the element whose overscroll is contained.
    const scroller = await scrollerOf(panel);
    expect(scroller, "the open list scrolls inside its own box").not.toBeNull();
    expect(scroller!.overscrollY, "the scrolling element contains its overscroll").toBe("contain");

    // Wheel over the list, a notch at a time, until it reaches its end.
    const box = await panel.boundingBox();
    expect(box).not.toBeNull();
    await page.mouse.move(box!.x + box!.width / 2, box!.y + box!.height / 2);
    await page.mouse.wheel(0, 120);
    await settle(panel);
    expect((await scrollerOf(panel))!.scrollTop, "the first wheel scrolls the list").toBeGreaterThan(0);
    let atEnd = (await scrollerOf(panel))!.atEnd;
    for (let notch = 0; notch < 80 && !atEnd; notch++) {
      await page.mouse.wheel(0, 120);
      await settle(panel);
      atEnd = (await scrollerOf(panel))!.atEnd;
    }
    expect(atEnd, "the list reached its end").toBe(true);

    // Keep wheeling over the list at its end: the page does not move.
    const atEndY = await page.evaluate(() => window.scrollY);
    for (let notch = 0; notch < 4; notch++) {
      await page.mouse.wheel(0, 400);
      await settle(panel);
    }
    expect(await page.evaluate(() => window.scrollY), "the page's scrollY after wheeling past the list's end").toBe(atEndY);

    // Outside the panel the page scrolls, and the panel stays open.
    const point = await pagePoint(page, panel, trigger);
    await page.mouse.move(point.x, point.y);
    await page.mouse.wheel(0, 300);
    await settle(panel);
    expect(await page.evaluate(() => window.scrollY), "the page scrolls from outside the panel").toBeGreaterThan(atEndY);
    await expect(panel).toHaveAttribute("data-state", "open");
  });
}
