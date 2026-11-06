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
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole } from "./demo-frame";
import { fixedViewportBox } from "./viewport-box";

const NAV_TIMEOUT = 20 * 60 * 1000;
const FIXTURE = `${BASE_URL}/component/top_layer/`;

type Surface = {
  name: string;
  url: string;
  /** The element a user returns to: focus is here after Escape. */
  trigger: (page: Page) => Locator;
  /** The panel; "closed" means it is hidden or gone. */
  content: (page: Page) => Locator;
  open: (page: Page) => Promise<void>;
  /** A mouse press outside closes it. */
  pressOutside: boolean;
  /** A touch scroll that starts outside leaves it open and a tap outside closes it (a modal page is locked: skipped). */
  touch: boolean;
  /** Focus moving to a control outside closes it. */
  focusOutside: boolean;
  /** Escape hands focus back to the trigger. */
  focusReturn: boolean;
  /** A modal: the press outside lands on its backdrop. */
  modal?: boolean;
};

const byId = (id: string) => (page: Page) => page.locator(`#${id}`);
const clickId = (id: string) => (page: Page) => page.locator(`#${id}`).click();

const SURFACES: Surface[] = [
  {
    name: "Tooltip",
    url: FIXTURE,
    trigger: byId("clip-tooltip-trigger"),
    content: byId("clip-tooltip-content"),
    open: (page) => page.locator("#clip-tooltip-trigger").focus(),
    pressOutside: true,
    touch: false,
    focusOutside: true,
    focusReturn: true,
  },
  {
    name: "HoverCard",
    url: FIXTURE,
    trigger: byId("clip-hovercard-trigger"),
    content: byId("clip-hovercard-content"),
    open: (page) => page.locator("#clip-hovercard-trigger").focus(),
    pressOutside: true,
    touch: false,
    focusOutside: true,
    focusReturn: true,
  },
  {
    name: "Popover",
    url: FIXTURE,
    trigger: byId("clip-popover-trigger"),
    content: byId("clip-popover-content"),
    open: clickId("clip-popover-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: false,
    focusReturn: true,
  },
  {
    name: "Select",
    url: FIXTURE,
    trigger: byId("clip-select-trigger"),
    content: byId("clip-select-content"),
    open: clickId("clip-select-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
    focusReturn: true,
  },
  {
    name: "Combobox",
    url: FIXTURE,
    trigger: byId("clip-combobox-trigger"),
    content: byId("clip-combobox-content"),
    open: async (page) => {
      await page.locator("#clip-combobox-trigger").click();
      await page.keyboard.press("ArrowDown");
    },
    // A press outside the input, the list and the clear button closes the list (Base UI's Combobox): a mouse
    // press at once, a touch or pen press as a tap. The next describe blocks cover a mouse press that cannot blur
    // the input and the clear button.
    pressOutside: true,
    touch: true,
    focusOutside: true,
    focusReturn: true,
  },
  {
    name: "Navbar",
    url: FIXTURE,
    trigger: byId("clip-navbar-trigger"),
    content: byId("clip-navbar-content"),
    // Opened from the keyboard: the focus that goes into the content is what Escape hands back.
    open: async (page) => {
      await page.locator("#clip-navbar-trigger").focus();
      await page.keyboard.press("Enter");
    },
    pressOutside: true,
    touch: true,
    focusOutside: false,
    focusReturn: true,
  },
  {
    name: "NavigationMenu (hover activation)",
    url: FIXTURE,
    trigger: byId("clip-navigation-menu-trigger"),
    content: byId("clip-navigation-menu-content"),
    open: clickId("clip-navigation-menu-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
    focusReturn: true,
  },
  {
    name: "NavigationMenu (click activation)",
    url: `${BASE_URL}/component/navigation_menu/`,
    trigger: (page) => page.getByRole("button", { name: "Guides" }),
    content: (page) => page.locator("[data-dx-anchored='navigation-menu']", { hasText: "Installation" }),
    open: (page) => page.getByRole("button", { name: "Guides" }).click(),
    pressOutside: true,
    touch: true,
    focusOutside: true,
    focusReturn: true,
  },
  {
    name: "Dialog",
    url: `${BASE_URL}/component/dialog/`,
    trigger: (page) => demoRole(page, "button"),
    content: (page) => page.getByRole("dialog"),
    open: (page) => demoRole(page, "button").click(),
    pressOutside: true,
    touch: false,
    focusOutside: false,
    focusReturn: true,
    modal: true,
  },
  {
    name: "AlertDialog",
    url: `${BASE_URL}/component/alert_dialog/`,
    trigger: (page) => demoRole(page, "button"),
    content: (page) => page.getByRole("alertdialog"),
    open: (page) => demoRole(page, "button").click(),
    // An alert dialog never light-dismisses.
    pressOutside: false,
    touch: false,
    focusOutside: false,
    focusReturn: true,
    modal: true,
  },
  {
    name: "DropdownMenu",
    url: FIXTURE,
    trigger: byId("clip-dropdown-menu-trigger"),
    content: byId("clip-dropdown-menu-content"),
    open: clickId("clip-dropdown-menu-trigger"),
    pressOutside: true,
    touch: true,
    // The menu hosts share one focus-out policy (`Policy::MENU`): the fixture gives the root its own `id`, which the
    // dismiss layer must still find.
    focusOutside: true,
    focusReturn: true,
  },
  {
    name: "Menubar",
    url: FIXTURE,
    trigger: byId("clip-menubar-trigger"),
    content: byId("clip-menubar-content"),
    open: clickId("clip-menubar-trigger"),
    pressOutside: true,
    touch: true,
    focusOutside: true,
    focusReturn: true,
  },
];

async function prepare(page: Page, surface: Surface) {
  await gotoHydrated(page, surface.url, { timeout: NAV_TIMEOUT });
  if (!surface.modal) {
    // A tall page, so a touch drag outside the panel really scrolls it.
    await page.evaluate(() => {
      document.body.style.paddingTop = "1500px";
      document.body.style.paddingBottom = "3000px";
    });
    await surface.trigger(page).first().evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
  }
  await surface.open(page);
  await expect(surface.content(page).first()).toBeVisible();
}

/** A point in the viewport that is on nothing interactive and outside the surface and its trigger. */
async function quietPoint(page: Page, surface: Surface): Promise<{ x: number; y: number }> {
  const box = await fixedViewportBox(page);
  if (surface.modal) {
    // the backdrop: the corner of the viewport, outside the centred dialog
    return { x: box.x + 5, y: box.y + 5 };
  }
  const content = await surface.content(page).first().elementHandle();
  const trigger = await surface.trigger(page).first().elementHandle();
  const point = await page.evaluate(
    ({ keep, vw, vh }) => {
      const interactive = "a, button, input, select, textarea, dialog, [popover], [role=button], [role=menuitem], [role=option]";
      for (const x of [Math.floor(vw * 0.9), Math.floor(vw * 0.6), 200, 40, 6]) {
        for (let y = 100; y < vh - 20; y += 23) {
          const el = document.elementFromPoint(x, y);
          if (!el || keep.some((k) => k && (k.contains(el) || el.contains(k)))) continue;
          if (el.closest(interactive)) continue;
          return { x, y };
        }
      }
      return null;
    },
    { keep: [content, trigger], vw: box.width, vh: box.y + box.height },
  );
  expect(point, "a quiet point outside the surface").not.toBeNull();
  return point as { x: number; y: number };
}

/** A button the page did not have, placed at a point so a press there is a press on a focusable control. */
async function addOutsideButton(page: Page, at: { x: number; y: number }) {
  await page.evaluate(({ x, y }) => {
    const b = document.createElement("button");
    b.id = "dismiss-spec-outside";
    b.textContent = "outside";
    b.style.cssText = `position:fixed;left:${x}px;top:${y}px;width:60px;height:24px;z-index:2147483000`;
    document.body.append(b);
  }, at);
}

const focusId = (page: Page) => page.evaluate(() => document.activeElement?.id ?? "");

const settle = (page: Page) =>
  page.evaluate(() => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(r, 60)))));

for (const surface of SURFACES) {
  test.describe(surface.name, () => {
    test(`Escape closes it${surface.focusReturn ? " and focus is back on the trigger" : ""}`, async ({ page }) => {
      await prepare(page, surface);
      await page.keyboard.press("Escape");
      await expect(surface.content(page).first()).toBeHidden();
      if (surface.focusReturn) {
        await expect(surface.trigger(page).first(), "focus returns to the trigger").toBeFocused();
      }
    });

    test(`a mouse press outside ${surface.pressOutside ? "closes it" : "leaves it open"}`, async ({ page }) => {
      await prepare(page, surface);
      const { x, y } = await quietPoint(page, surface);
      await page.mouse.click(x, y);
      if (surface.pressOutside) {
        await expect(surface.content(page).first()).toBeHidden();
      } else {
        await settle(page);
        await expect(surface.content(page).first()).toBeVisible();
      }
    });

    if (surface.focusOutside) {
      test("focus moving to a control outside closes it and stays there", async ({ page }) => {
        await prepare(page, surface);
        await addOutsideButton(page, await quietPoint(page, surface));
        await page.locator("#dismiss-spec-outside").focus();
        await expect(surface.content(page).first()).toBeHidden();
        expect(await focusId(page), "focus stays where the user put it").toBe("dismiss-spec-outside");
      });
    }

    if (surface.pressOutside && !surface.modal) {
      test("a mouse press on a control outside closes it and focus stays on that control", async ({ page }) => {
        await prepare(page, surface);
        await addOutsideButton(page, await quietPoint(page, surface));
        await page.locator("#dismiss-spec-outside").click();
        await expect(surface.content(page).first()).toBeHidden();
        expect(await focusId(page)).toBe("dismiss-spec-outside");
      });
    }
  });
}

test.describe("touch: a scroll that starts outside is not a dismissal, a tap is", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  for (const surface of SURFACES.filter((s) => s.touch)) {
    test(`${surface.name}: a touch scroll leaves it open, a tap outside closes it`, async ({ page }) => {
      await prepare(page, surface);
      const content = surface.content(page).first();
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

test.describe("touch: the backdrop of a modal is a tap target, a drag from it is not", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  const dialog = SURFACES.find((s) => s.name === "Dialog") as Surface;

  const gesture = async (page: Page, steps: { dx: number; dy: number }[]) => {
    const box = await fixedViewportBox(page);
    const start = { x: box.x + 5, y: box.y + 5 };
    const cdp = await page.context().newCDPSession(page);
    const send = (type: string, dx = 0, dy = 0) =>
      cdp.send("Input.dispatchTouchEvent", {
        type,
        touchPoints: type === "touchEnd" ? [] : [{ x: start.x + dx, y: start.y + dy }],
      });
    await send("touchStart");
    for (const step of steps) {
      await send("touchMove", step.dx, step.dy);
      await page.waitForTimeout(16);
    }
    await send("touchEnd");
    await settle(page);
  };

  test("Dialog: a touch drag that starts on the backdrop leaves it open", async ({ page }) => {
    await prepare(page, dialog);
    await gesture(page, Array.from({ length: 8 }, (_, i) => ({ dx: 0, dy: (i + 1) * 10 })));
    await expect(dialog.content(page).first(), "a drag from the backdrop must not close it").toBeVisible();
  });

  test("Dialog: a touch that moves past the tap distance and lifts on the backdrop leaves it open", async ({ page }) => {
    await prepare(page, dialog);
    // 12 px is inside the browser's own click slop, so a `click` would still follow; the tap rule is 10 px
    await gesture(page, [{ dx: 0, dy: 4 }, { dx: 0, dy: 8 }, { dx: 0, dy: 12 }]);
    await expect(dialog.content(page).first(), "a press that moved 12 px is not a tap").toBeVisible();
  });

  test("Dialog: a tap on the backdrop closes it", async ({ page }) => {
    await prepare(page, dialog);
    await gesture(page, []);
    await expect(dialog.content(page).first()).toBeHidden();
  });

  test("Dialog: a tap inside its own box leaves it open", async ({ page }) => {
    await prepare(page, dialog);
    await dialog.content(page).first().tap({ position: { x: 20, y: 20 } });
    await settle(page);
    await expect(dialog.content(page).first()).toBeVisible();
  });
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

// Base UI's Combobox closes on a mouse press outside (`outsidePressEvent: { mouse: 'sloppy', touch: 'intentional' }`)
// and counts the clear button as inside. Red on the build before the Combobox took `Policy::ESCAPE_AND_PRESS`:
// "a mouse press on a control that keeps the input focused closes the list" and "touch: a tap on the clear button
// keeps the list open"; the mouse clear case is parity.
test.describe("Combobox: a press outside closes the list, the clear button is inside", () => {
  const combobox = SURFACES.find((s) => s.name === "Combobox")!;

  test("a mouse press on a control that keeps the input focused closes the list", async ({ page }) => {
    await prepare(page, combobox);
    await addOutsideButton(page, await quietPoint(page, combobox));
    // the press cannot blur the input, so the list closing is the press policy and not the focus arm
    await page.evaluate(() =>
      document.getElementById("dismiss-spec-outside")!.addEventListener("mousedown", (e) => e.preventDefault()),
    );
    await page.locator("#dismiss-spec-outside").click();
    await expect(combobox.content(page).first()).toBeHidden();
  });

  const openWithClear = async (page: Page) => {
    await gotoHydrated(page, `${BASE_URL}/component/combobox/`, { timeout: NAV_TIMEOUT });
    const root = page.locator("#component-preview-frame-multiple_clear").first();
    await root.scrollIntoViewIfNeeded();
    const input = root.getByRole("combobox", { name: "Frameworks with clear" });
    await input.click();
    const list = page.locator("[role='listbox'][data-state='open']");
    await expect(list).toBeVisible();
    return { root, list, clear: root.getByRole("button", { name: "Clear" }) };
  };

  test("a mouse press on the clear button keeps the list open", async ({ page }) => {
    const { root, list, clear } = await openWithClear(page);
    await clear.click();
    await expect(root.locator("[data-dx-combobox-chip]")).toHaveCount(0);
    await settle(page);
    await expect(list).toBeVisible();
  });

  test.describe("touch", () => {
    test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

    test("a tap on the clear button keeps the list open", async ({ page }) => {
      const { root, list, clear } = await openWithClear(page);
      await clear.tap();
      await expect(root.locator("[data-dx-combobox-chip]")).toHaveCount(0);
      await settle(page);
      await expect(list, "the clear button is inside the combobox").toBeVisible();
    });
  });
});
