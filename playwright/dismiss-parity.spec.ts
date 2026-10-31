/**
 * One dismiss policy for every overlay (backlog row 487): Escape, a press outside, a touch tap versus a touch scroll,
 * focus leaving, focus return, and nested layers.
 *
 * The table below is the policy of each surface (`primitives/src/dismiss.rs`, "Policy per surface"). Every row runs the
 * same cases, so a rule that holds for one surface and not for another is a red test, not a convention:
 *
 *   - Escape closes it, and focus is back on its trigger (the trigger of a hover surface keeps the focus it had);
 *   - a mouse press outside closes it;
 *   - a touch that starts outside and scrolls the page leaves it open, a tap outside closes it;
 *   - focus moving to a control outside closes it, where the surface's policy says so;
 *   - nested: a Select, a Popover and a Tooltip inside a Dialog take the first Escape, the Dialog the second.
 *
 * Limits: Chromium only. The touch cases dispatch CDP touch events, which reproduce the browser's decision to take a
 * gesture for a scroll (`pointercancel`), not a particular device's scroll slop.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { fixedViewportBox } from "./viewport-box";

const NAV_TIMEOUT = 20 * 60 * 1000;
const FIXTURE = `${BASE_URL}/component/top_layer/`;
const NAV_MENU_PAGE = `${BASE_URL}/component/navigation_menu/`;

type Surface = {
  name: string;
  url: string;
  /** The element a user returns to: focus is here after Escape. */
  trigger: string;
  /** The panel; "closed" means it is hidden or gone. */
  content: string;
  open: (page: Page) => Promise<void>;
  /** A mouse press outside closes it. */
  pressOutside: boolean;
  /** A touch scroll that starts outside leaves it open and a tap outside closes it (a modal page is locked: skipped). */
  touch: boolean;
  /** Focus moving to a control outside closes it. */
  focusOutside: boolean;
  /** A touch tap closes it, a mouse press closes it, but it is a modal whose scroll is locked. */
  modal?: boolean;
};

const clickTrigger = (id: string) => (page: Page) => page.locator(`#${id}`).click();

const SURFACES: Surface[] = [
  {
    name: "Tooltip",
    url: FIXTURE,
    trigger: "#clip-tooltip-trigger",
    content: "#clip-tooltip-content",
    open: (page) => page.locator("#clip-tooltip-trigger").focus(),
    pressOutside: true,
    touch: false,
    focusOutside: true,
  },
  {
    name: "HoverCard",
    url: FIXTURE,
    trigger: "#clip-hovercard-trigger",
    content: "#clip-hovercard-content",
    open: (page) => page.locator("#clip-hovercard-trigger").focus(),
    pressOutside: true,
    touch: false,
    focusOutside: true,
  },
  {
    name: "Popover",
    url: FIXTURE,
    trigger: "#clip-popover-trigger",
    content: "#clip-popover-content",
    open: clickTrigger("clip-popover-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: false,
  },
  {
    name: "Select",
    url: FIXTURE,
    trigger: "#clip-select-trigger",
    content: "#clip-select-content",
    open: clickTrigger("clip-select-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
  },
  {
    name: "Combobox",
    url: FIXTURE,
    trigger: "#clip-combobox-trigger",
    content: "#clip-combobox-content",
    open: async (page) => {
      await page.locator("#clip-combobox-trigger").click();
      await page.keyboard.press("ArrowDown");
    },
    pressOutside: true,
    touch: true,
    focusOutside: true,
  },
  {
    name: "Navbar",
    url: FIXTURE,
    trigger: "#clip-navbar-trigger",
    content: "#clip-navbar-content",
    open: clickTrigger("clip-navbar-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
  },
  {
    name: "NavigationMenu (hover activation)",
    url: FIXTURE,
    trigger: "#clip-navigation-menu-trigger",
    content: "#clip-navigation-menu-content",
    open: clickTrigger("clip-navigation-menu-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
  },
  {
    name: "NavigationMenu (click activation)",
    url: NAV_MENU_PAGE,
    trigger: "button:has-text('Guides')",
    content: "[data-dx-anchored='navigation-menu']:has-text('Installation')",
    open: (page) => page.getByRole("button", { name: "Guides" }).click(),
    pressOutside: true,
    touch: true,
    focusOutside: true,
  },
  {
    name: "Dialog",
    url: FIXTURE,
    trigger: "#dialog-clip-trigger",
    content: "#dialog-clip-content",
    open: clickTrigger("dialog-clip-trigger"),
    pressOutside: true,
    touch: false,
    focusOutside: false,
    modal: true,
  },
  {
    name: "AlertDialog",
    url: FIXTURE,
    trigger: "#dialog-inert-trigger",
    content: "#dialog-inert-content",
    open: clickTrigger("dialog-inert-trigger"),
    // An alert dialog never light-dismisses.
    pressOutside: false,
    touch: false,
    focusOutside: false,
    modal: true,
  },
  {
    name: "DropdownMenu",
    url: FIXTURE,
    trigger: "#clip-dropdown-menu-trigger",
    content: "#clip-dropdown-menu-content",
    open: clickTrigger("clip-dropdown-menu-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
  },
  {
    name: "Menubar",
    url: FIXTURE,
    trigger: "#clip-menubar-trigger",
    content: "#clip-menubar-content",
    open: clickTrigger("clip-menubar-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
  },
];

async function prepare(page: Page, surface: Surface) {
  await gotoHydrated(page, surface.url, { timeout: NAV_TIMEOUT });
  // A tall page, so a touch drag outside the panel really scrolls it.
  await page.evaluate(() => {
    document.body.style.paddingTop = "1500px";
    document.body.style.paddingBottom = "3000px";
  });
  await page.locator(surface.trigger).first().evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
  await surface.open(page);
  await expect(page.locator(surface.content).first()).toBeVisible();
}

/** A point in the viewport that is not inside the surface, its trigger or any interactive element. */
async function quietPoint(page: Page, surface: Surface): Promise<{ x: number; y: number }> {
  const box = await fixedViewportBox(page);
  const point = await page.evaluate(
    ({ content, trigger, bottom }) => {
      const keep = [document.querySelector(content), document.querySelector(trigger)];
      for (const x of [6, 14, 24, 40, 60]) {
        for (let y = 90; y < bottom - 20; y += 23) {
          const el = document.elementFromPoint(x, y);
          if (!el) continue;
          if (keep.some((k) => k && (k.contains(el) || el.contains(k)))) continue;
          if (el.closest("a, button, input, select, textarea, [tabindex], dialog, [popover]")) continue;
          return { x, y };
        }
      }
      return null;
    },
    { content: surface.content, trigger: surface.trigger, bottom: box.y + box.height },
  );
  expect(point, "a quiet point outside the surface").not.toBeNull();
  return point as { x: number; y: number };
}

const focusId = (page: Page) => page.evaluate(() => document.activeElement?.id ?? "");
const triggerId = async (page: Page, surface: Surface) =>
  page.locator(surface.trigger).first().evaluate((el) => (el.id ? el.id : (el as HTMLElement).innerText));

const settle = (page: Page) =>
  page.evaluate(() => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(r, 60)))));

for (const surface of SURFACES) {
  test.describe(surface.name, () => {
    test("Escape closes it and focus is back on the trigger", async ({ page }) => {
      await prepare(page, surface);
      await page.keyboard.press("Escape");
      await expect(page.locator(surface.content).first()).toBeHidden();
      const trigger = page.locator(surface.trigger).first();
      await expect(trigger, "focus returns to the trigger").toBeFocused();
    });

    test(`a mouse press outside ${surface.pressOutside ? "closes it" : "leaves it open"}`, async ({ page }) => {
      await prepare(page, surface);
      const { x, y } = await quietPoint(page, surface);
      await page.mouse.click(x, y);
      if (surface.pressOutside) {
        await expect(page.locator(surface.content).first()).toBeHidden();
      } else {
        await settle(page);
        await expect(page.locator(surface.content).first()).toBeVisible();
      }
    });

    if (surface.focusOutside) {
      test("focus moving to a control outside closes it", async ({ page }) => {
        await prepare(page, surface);
        await page.evaluate(() => {
          const b = document.createElement("button");
          b.id = "dismiss-spec-outside";
          b.textContent = "outside";
          b.style.position = "fixed";
          b.style.left = "2px";
          b.style.top = "2px";
          document.body.append(b);
          b.focus();
        });
        await expect(page.locator(surface.content).first()).toBeHidden();
        expect(await focusId(page), "focus stays where the user put it").toBe("dismiss-spec-outside");
      });
    }

    test("a mouse press outside leaves focus where the user put it", async ({ page }) => {
      test.skip(!surface.pressOutside, "the surface does not close on an outside press");
      test.skip(!!surface.modal, "a modal returns focus to its opener");
      await prepare(page, surface);
      await page.evaluate(() => {
        const b = document.createElement("button");
        b.id = "dismiss-spec-outside";
        b.textContent = "outside";
        b.style.position = "fixed";
        b.style.left = "2px";
        b.style.top = "2px";
        document.body.append(b);
      });
      await page.locator("#dismiss-spec-outside").click();
      await expect(page.locator(surface.content).first()).toBeHidden();
      expect(await focusId(page)).toBe("dismiss-spec-outside");
    });
  });
}

test.describe("touch: a scroll that starts outside is not a dismissal, a tap is", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  for (const surface of SURFACES.filter((s) => s.touch)) {
    test(`${surface.name}: a touch scroll leaves it open, a tap outside closes it`, async ({ page }) => {
      await prepare(page, surface);
      const content = page.locator(surface.content).first();
      const outside = await quietPoint(page, surface);
      const cdp = await page.context().newCDPSession(page);
      const touch = (type: string, y?: number) =>
        cdp.send("Input.dispatchTouchEvent", {
          type,
          touchPoints: y === undefined ? [] : [{ x: outside.x, y }],
        });
      const scrolledBefore = await page.evaluate(() => window.scrollY);
      // a finger that starts outside and drags: the browser takes the gesture to scroll and cancels the pointer
      await touch("touchStart", outside.y);
      for (let i = 1; i <= 8; i++) {
        await touch("touchMove", outside.y - i * 20);
        await page.waitForTimeout(16);
      }
      await touch("touchEnd");
      await settle(page);
      expect(await page.evaluate(() => window.scrollY), "the drag scrolled the page").not.toBe(scrolledBefore);
      await expect(content, "a touch scroll that began outside must not close it").toBeVisible();
      // a press that lifts in place is a tap
      const tap = await quietPoint(page, surface);
      outside.x = tap.x;
      await touch("touchStart", tap.y);
      await touch("touchEnd");
      await expect(content, "a tap outside closes it").toBeHidden();
    });
  }
});

test.describe("nested layers: only the topmost closes", () => {
  const prepareNested = async (page: Page) => {
    await gotoHydrated(page, FIXTURE, { timeout: NAV_TIMEOUT });
    await page.locator("#nested-dialog-trigger").click();
    await expect(page.locator("#nested-dialog-content")).toBeVisible();
  };

  test("a Select inside a Dialog: the first Escape closes the list, the second the Dialog", async ({ page }) => {
    await prepareNested(page);
    await page.locator("#nested-select-trigger").click();
    await expect(page.locator("#nested-select-content")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#nested-select-content")).toBeHidden();
    await expect(page.locator("#nested-dialog-content"), "the dialog stays open").toBeVisible();
    await expect(page.locator("#nested-select-trigger"), "focus is back on the select trigger").toBeFocused();
    await page.keyboard.press("Escape");
    await expect(page.locator("#nested-dialog-content")).toBeHidden();
  });

  test("a Popover inside a Dialog: the first Escape closes the popover, the second the Dialog", async ({ page }) => {
    await prepareNested(page);
    await page.locator("#nested-popover-trigger").click();
    await expect(page.locator("#nested-popover-content")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#nested-popover-content")).toBeHidden();
    await expect(page.locator("#nested-dialog-content"), "the dialog stays open").toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#nested-dialog-content")).toBeHidden();
  });

  test("a Tooltip inside a Dialog: the first Escape closes the tooltip, the second the Dialog", async ({ page }) => {
    await prepareNested(page);
    await page.locator("#nested-tooltip-trigger").focus();
    await expect(page.locator("#nested-tooltip-content")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#nested-tooltip-content")).toBeHidden();
    await expect(page.locator("#nested-dialog-content"), "the dialog stays open").toBeVisible();
    await expect(page.locator("#nested-tooltip-trigger"), "focus stays on the trigger").toBeFocused();
    await page.keyboard.press("Escape");
    await expect(page.locator("#nested-dialog-content")).toBeHidden();
  });

  test("a Select inside a Dialog: a press on the dialog's own text closes the list and keeps the dialog", async ({ page }) => {
    await prepareNested(page);
    await page.locator("#nested-select-trigger").click();
    await expect(page.locator("#nested-select-content")).toBeVisible();
    await page.locator("#nested-dialog-content h2, #nested-dialog-content [role=heading]").first().click();
    await expect(page.locator("#nested-select-content")).toBeHidden();
    await expect(page.locator("#nested-dialog-content")).toBeVisible();
  });
});
