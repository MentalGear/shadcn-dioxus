/**
 * Auto-popover opening-gesture oracle (dev-docs/backlog.md row 40).
 *
 * The class under test: a `popover="auto"` surface that becomes
 * `:popover-open` BETWEEN the `pointerdown` and the `pointerup` of the same
 * gesture. WHATWG HTML's light-dismiss algorithm
 * (https://html.spec.whatwg.org/multipage/popover.html#popover-light-dismiss,
 * "To light dismiss open popovers, given a PointerEvent event") is driven by
 * trusted `pointerdown`/`pointerup` events only -- never `click`:
 *
 *   - `pointerdown`: if there is no open auto popover -> return (the
 *     document's "popover pointerdown target" is NOT recorded).
 *   - `pointerup`: ancestor = topmost clicked popover of the target (null for
 *     a trigger that is not inside the popover and not declared via
 *     `popovertarget`); `sameTarget = (ancestor === pointerdown target)`;
 *     the pointerdown target is still its initial `null`, so `sameTarget` is
 *     TRUE -> every open auto popover is hidden.
 *
 * Consequences, per trigger family (primitives/src/):
 *   - open on `click`  (DropdownMenu, Select, non-modal Popover, NavbarTrigger):
 *     the popover shows only after the gesture's `pointerup` has finished -> safe
 *     no matter how long the finger/mouse is held. iOS's lazy click
 *     retargeting (native-dialog.spec.ts Rule 8) is irrelevant here: the
 *     algorithm never reads `click`. So an Auto popover cannot read the
 *     opening tap as outside for these (row 40).
 *   - open on `pointerup` (Menubar): shows after `pointerup` -> safe.
 *   - open on `pointerdown` (nothing does): an Auto popover shown BETWEEN down
 *     and up whenever the hold outlasts Dioxus's render + effect (a few ms) is
 *     hidden by the gesture that opened it, so `NavbarTrigger` (navbar.rs)
 *     opens from `click` and uses `pointerdown` only to snapshot the state the
 *     gesture started from. `navbar.spec.ts` "mobile navigation" cannot see a
 *     regression there: Playwright's `tap()` has zero hold, so `pointerup` is
 *     dispatched before `showPopover()` runs. A real finger holds ~50-150ms.
 *
 * The test holds a real touch for 250ms (CDP `Input.dispatchTouchEvent`,
 * since `page.touchscreen` only exposes an instantaneous `tap`) and asserts
 * that NO native `toggle` -> closed event fires on any `[popover]` during or
 * after the gesture. It records `toggle` events rather than the final
 * visible state on purpose: a compat `mouseenter` after touchend
 * (NavbarNav's `onmouseenter`) can silently re-open a just-light-dismissed
 * Navbar and mask the false dismiss.
 *
 * If the Navbar case is RED, `NavbarTrigger` opens its `auto` popover from
 * `pointerdown`: it must open on `click` (or `NavbarContent` must be `manual` +
 * `use_outside_dismiss`); see dev-docs/backlog.md row 40. The DropdownMenu case
 * is a green control for the click-open family.
 */
import { test, expect } from "../../fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";
import { measureAtRest } from "../../animations";
import { demoRole } from "../../demo-frame";

const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app
const MOBILE_VIEWPORT = { width: 390, height: 844 };
const HOLD_MS = 250;
const SETTLE_MS = 600;

type ToggleLog = { id: string; newState: string }[];

const CASES: {
  name: string;
  url: string;
  trigger: (page: Page) => import("@playwright/test").Locator;
}[] = [
  {
    // NavbarTrigger opens from `click` (`pointerdown` only snapshots state),
    // so the pointerdown/pointerup gap cannot dismiss it.
    name: "NavbarTrigger (opens on click)",
    url: `${BASE_URL}/component/navbar/`,
    trigger: (page) => demoRole(page, "menuitem", 0),
  },
  {
    // click-open control: must stay green regardless of hold length.
    name: "DropdownMenuTrigger (opens on click)",
    url: `${BASE_URL}/component/dropdown_menu/`,
    trigger: (page) => demoRole(page, "button"),
  },
];

for (const kase of CASES) {
  test(`${kase.name}: a held touch (${HOLD_MS}ms) does not light-dismiss the popover it just opened`, async ({
    browser,
  }) => {
    const context = await browser.newContext({ hasTouch: true, viewport: MOBILE_VIEWPORT });
    const page = await context.newPage();
    try {
      await gotoHydrated(page, kase.url, { timeout: NAV_TIMEOUT });
      const trigger = kase.trigger(page).first();
      await trigger.scrollIntoViewIfNeeded();
      const box = await measureAtRest(trigger);
      if (!box) throw new Error("trigger has no bounding box");
      const x = box.x + box.width / 2;
      const y = box.y + box.height / 2;

      // `toggle` does not bubble: capture on `document` sees every
      // `[popover]` element's transitions, including ones mounted later.
      await page.evaluate(() => {
        (window as unknown as { __toggles: ToggleLog }).__toggles = [];
        document.addEventListener(
          "toggle",
          (e) => {
            const t = e.target as HTMLElement;
            (window as unknown as { __toggles: ToggleLog }).__toggles.push({
              id: t.id,
              newState: (e as Event & { newState: string }).newState,
            });
          },
          true,
        );
      });

      const cdp = await context.newCDPSession(page);
      await cdp.send("Input.dispatchTouchEvent", {
        type: "touchStart",
        touchPoints: [{ x, y }],
      });
      await page.waitForTimeout(HOLD_MS);
      await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
      await page.waitForTimeout(SETTLE_MS);

      const toggles = await page.evaluate(
        () => (window as unknown as { __toggles: ToggleLog }).__toggles,
      );
      expect(
        toggles.some((t) => t.newState === "open"),
        `the gesture must have opened a popover at all (else this test is vacuous); toggles=${JSON.stringify(toggles)}`,
      ).toBe(true);
      expect(
        toggles.filter((t) => t.newState === "closed"),
        `no popover may be light-dismissed by the pointerup of the very gesture that opened it; toggles=${JSON.stringify(toggles)}`,
      ).toEqual([]);
    } finally {
      await context.close();
    }
  });
}
