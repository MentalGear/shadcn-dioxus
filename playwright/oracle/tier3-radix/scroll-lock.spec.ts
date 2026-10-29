/**
 * ORACLE (tier 3 -- Radix-parity, labelled opinion; see
 * dev-docs/conformance-harness.md "Tier 3 -- Radix-parity" and
 * playwright/oracle/tier3-radix/README.md's rule-source policy).
 *
 * Rule: while a modal surface is open, the page behind it must not scroll.
 * No W3C standard specifies this -- it is a UX convention several mature
 * libraries implement. Base UI's `useScrollLock` (`@base-ui/utils`) sets
 * `overflow: hidden` on the element that scrolls the viewport and keeps a
 * classic scrollbar's width with `scrollbar-gutter: stable`; Radix's
 * `react-remove-scroll` intercepts wheel/touch/key events instead. This
 * repo follows Base UI's method, implemented at `primitives/src/scroll_lock.rs`
 * (see that file's header for the adaptation notes, the nesting-aware counter
 * and the release timing).
 *
 * WHO LOCKS (owner decision, backlog row 479): Dialog, AlertDialog
 * (and so Sheet and Drawer), a Popover with `is_modal: true`, and ContextMenu.
 * An anchored overlay -- DropdownMenu, Menubar, Select, a default Popover --
 * leaves the page scrollable and stays open while the page scrolls; "DropdownMenu
 * leaves the page scrollable" below pins that for the dropdown.
 *
 * Fixture: the nested-dialog case uses the dialog demo's "Open Nested Dialog"
 * button (`preview/src/components/dialog/variants/main/mod.rs`,
 * dev-docs/plan.md Phase 3.2).
 *
 * Select (dev-docs/backlog.md row 9): `Select`'s open listbox is wired to
 * this same scroll lock -- see "Select locks and releases page scroll"
 * below, opt-out via a `scroll_lock` prop.
 *
 * The `scroll_lock=false` opt-out itself has a fixture and a case too
 * (dev-docs/backlog.md row 72) -- see "Select scroll_lock=false opt-out
 * releases page scroll" below, against a labelled instance on the
 * `top_layer` fixture page
 * (`preview/src/components/top_layer/component.rs`).
 *
 * `assertScrollIsLocked` checks the page against a mouse wheel and (in the
 * touch block) a touch pan. It does not assert that a scripted
 * `window.scrollTo()` call is blocked: under `overflow: hidden` a script can
 * still move the page, and the dialog's own `showModal()` focus scroll relies
 * on that being restored (see "Dialog does not leave the page permanently
 * scrolled"). iOS momentum scroll is a device check, not a Playwright one.
 *
 * LAYOUT-SHIFT ASSERTIONS (below, `assertNoHorizontalShift`): a lock that hid
 * `<html>`'s overflow without reserving a classic (non-overlay) scrollbar's
 * space would shift everything horizontally by the scrollbar's width the
 * instant the lock engages (15px under a forced real scrollbar, see
 * `playwright/xvfb.local.config.ts`). That shift is invisible under this repo's
 * default headless Chromium, which renders 0-width overlay scrollbars even for a
 * genuinely scrollable page -- so `assertNoHorizontalShift` is trivially
 * satisfied there and only bites for real when run under `xvfb.local.config.ts`.
 * Both runs matter: headless proves the lock doesn't *introduce* a shift on
 * overlay-scrollbar platforms (a gutter reserved at lock time would), and the
 * Xvfb run proves it leaves classic-scrollbar platforms unshifted.
 */

import { test, expect } from "../../fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";
import { measureAtRest, readStatic, staticBox } from "../../animations";
import { demoRole } from "../../demo-frame";
import { pageMetrics } from "../../viewport-box";

const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app

// dev-docs/backlog.md row 109: this suite clicks/scrolls triggers right
// after navigating, on the SSG lane -- `gotoHydrated` so those interactions
// can't land before hydration attaches listeners.
const goto = (page: Page, name: string, extra = "") =>
  gotoHydrated(page, `${BASE_URL}/component/${name}/${extra ? `?${extra}` : ""}`, {
    timeout: NAV_TIMEOUT,
  });

/** Sanity check: the route must actually have more content than fits in the
 * viewport, or "scrollY didn't move" would be true regardless of any lock. */
async function assertPageIsScrollable(page: Page) {
  const pm = await pageMetrics(page);
  expect(
    pm.scrollHeight,
    "fixture route must be taller than the viewport for this test to mean anything",
  ).toBeGreaterThan(pm.innerHeight);
}

/**
 * Attempts to scroll the page the way a person actually would -- a mouse
 * wheel over the page -- and asserts `window.scrollY` did not move.
 *
 * Deliberately does NOT also try `window.scrollTo()`: the lock (`overflow:
 * hidden` on the viewport scroller; see scroll_lock.rs's module doc) stops
 * wheel-, touch-, keyboard- and scrollbar-initiated scrolling, but a direct,
 * scripted `scrollTo()`/`scrollTop =` call is honored by every browser
 * regardless of overflow. On this page, with the lock held,
 * `page.mouse.wheel()` leaves `scrollY` alone, while `window.scrollTo()` still
 * moves it, so asserting against a scripted `scrollTo()` here would fail a
 * correct implementation.
 */
async function assertScrollIsLocked(page: Page) {
  const before = await page.evaluate(() => window.scrollY);
  await page.mouse.move(400, 300);
  await page.mouse.wheel(0, 800);
  const after = await page.evaluate(() => window.scrollY);
  expect(
    after,
    "page must not scroll via mouse wheel while a modal surface is locking it",
  ).toBe(before);
}

/**
 * Asserts the page can scroll again via mouse wheel (the inverse of the
 * above).
 *
 * Retries the wheel attempt rather than making just one: the unlock
 * releases through a `document::eval` round-trip queued from the
 * closing component's unmount, which lands well after Playwright observes
 * the closed element gone from the accessibility tree (empirically, over a
 * second later is not unusual) -- not an implementation defect, just two
 * different signals settling at very different times. Each attempt resets
 * scroll position with a direct `scrollTo()` first; per the note on
 * `assertScrollIsLocked`, that call proves nothing about the lock either
 * way, it's just attempt setup.
 */
async function assertScrollIsUnlocked(page: Page) {
  const deadline = Date.now() + 8000;
  let scrollY = 0;
  do {
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.mouse.move(400, 300);
    await page.mouse.wheel(0, 800);
    await page.waitForTimeout(200);
    scrollY = await page.evaluate(() => window.scrollY);
  } while (scrollY === 0 && Date.now() < deadline);
  expect(
    scrollY,
    "page must scroll via mouse wheel again once nothing is locking it",
  ).toBeGreaterThan(0);
}

/**
 * Returns the current right edge of a `position: fixed; right: 0` marker,
 * injecting it once per page if it isn't already there.
 *
 * `position: fixed` elements are positioned against the initial containing
 * block, whose size tracks the true viewport net of whatever scrollbar the
 * platform is actually rendering right now -- unlike normal-flow content,
 * which a `padding-right` compensation recipe can reach but this can't (that
 * recipe does not cover this element shape; see
 * `dev-docs/archive/research/phase4-spike-findings.md`, Round 2 Construction A). A stand-in for a
 * realistic right-aligned navbar action.
 */
async function rightEdgeMarkerRight(page: Page): Promise<number> {
  await page.evaluate(() => {
    let marker = document.getElementById("scroll-lock-shift-probe");
    if (!marker) {
      marker = document.createElement("div");
      marker.id = "scroll-lock-shift-probe";
      marker.style.cssText =
        "position:fixed; top:0; right:0; width:4px; height:4px; pointer-events:none; z-index:2147483647;";
      document.body.appendChild(marker);
    }
  });
  return readStatic(page.locator("#scroll-lock-shift-probe"), (marker) => marker.getBoundingClientRect().right);
}

/**
 * Asserts the right-anchored marker (see `rightEdgeMarkerRight`) has not
 * moved from `expectedRight` -- i.e. engaging or releasing the scroll lock
 * must not shift page layout horizontally. Only bites for real on a platform
 * with a non-zero scrollbar gap; see the module header's LAYOUT-SHIFT note.
 */
async function assertNoHorizontalShift(page: Page, expectedRight: number, when: string) {
  expect(await rightEdgeMarkerRight(page), `layout must not shift horizontally ${when}`).toBe(
    expectedRight,
  );
}

test.describe("Dialog locks and releases page scroll", () => {
  test("scroll is locked while open and restored after Escape", async ({ page }) => {
    await goto(page, "dialog");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    await demoRole(page, "button").click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await assertNoHorizontalShift(page, rightBefore, "while the dialog is open");

    await assertScrollIsLocked(page);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);

    await assertScrollIsUnlocked(page);
    await assertNoHorizontalShift(page, rightBefore, "after the dialog closes and scroll unlocks");
  });

  test("nested dialog: closing the inner one leaves the page locked; closing the outer one restores scroll", async ({
    page,
  }) => {
    await goto(page, "dialog");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    // Open the outer dialog.
    await demoRole(page, "button").click();
    const outer = page.getByRole("dialog").first();
    await expect(outer).toBeVisible();
    await assertScrollIsLocked(page);
    await assertNoHorizontalShift(page, rightBefore, "with the outer dialog open");

    // Open the nested dialog from inside the outer one's content.
    await page.getByRole("dialog").first().getByRole("button").first().click();
    await expect(page.getByRole("dialog")).toHaveCount(2);
    await assertScrollIsLocked(page);
    await assertNoHorizontalShift(page, rightBefore, "with both dialogs open");

    // Close the inner dialog only -- the outer one is still open, so the
    // page must still be locked. This is the refcount case: a naive
    // "unlock whenever any modal closes" implementation fails here.
    await page.getByRole("dialog").last().getByRole("button").first().click();
    await expect(page.getByRole("dialog")).toHaveCount(1);
    await assertScrollIsLocked(page);
    await assertNoHorizontalShift(page, rightBefore, "with only the outer dialog open again");

    // Close the outer dialog -- nothing is locking anymore. Closed via its
    // own Close button rather than Escape: this test's subject is the
    // scroll-lock refcount, not the (separately covered, and here
    // momentarily still settling right after the inner dialog's unmount)
    // Escape-priority stack in `use_global_escape_listener` (lib.rs).
    await page.getByRole("dialog").locator('[data-slot="dialog-close"]').click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await assertScrollIsUnlocked(page);
    await assertNoHorizontalShift(page, rightBefore, "after both dialogs close and scroll unlocks");
  });
});

test.describe("AlertDialog locks and releases page scroll", () => {
  test("scroll is locked while open and restored after Escape", async ({ page }) => {
    await goto(page, "alert_dialog");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    await demoRole(page, "button").click();
    await expect(page.getByRole("alertdialog")).toBeVisible();
    await assertNoHorizontalShift(page, rightBefore, "while the alert dialog is open");

    await assertScrollIsLocked(page);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("alertdialog")).toHaveCount(0);

    await assertScrollIsUnlocked(page);
    await assertNoHorizontalShift(page, rightBefore, "after the alert dialog closes and scroll unlocks");
  });
});

// Popover is NON-MODAL by default (dev-docs/backlog.md row 169): only `is_modal: true` (the
// `modal` demo) takes the scroll lock. `popover-non-modal.spec.ts` covers the non-modal
// default in depth; the second describe below pins the contrast on this file's own helpers.
test.describe("Popover (is_modal: true) locks and releases page scroll", () => {
  test("scroll is locked while open and restored after Escape", async ({
    page,
  }) => {
    await goto(page, "popover", "variant=modal&");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    await demoRole(page, "button", 0, "modal").click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await assertNoHorizontalShift(page, rightBefore, "while the popover is open");

    await assertScrollIsLocked(page);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);

    await assertScrollIsUnlocked(page);
    await assertNoHorizontalShift(page, rightBefore, "after the popover closes and scroll unlocks");
  });
});

test.describe("Popover (non-modal default) never takes the scroll lock", () => {
  test("page scrolls by wheel while open, no lock marker, nothing to undo after Escape", async ({
    page,
  }) => {
    await goto(page, "popover");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    await demoRole(page, "button").click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await assertNoHorizontalShift(page, rightBefore, "while the non-modal popover is open");

    await assertScrollIsUnlocked(page);
    expect(
      await page.evaluate(() => (window as unknown as { __dxScrollLocked?: boolean }).__dxScrollLocked === true),
      "a non-modal popover must not engage the scroll lock",
    ).toBe(false);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await assertScrollIsUnlocked(page);
    await assertNoHorizontalShift(page, rightBefore, "after the non-modal popover closes");
  });
});

/**
 * A touch pan of the page: a real one-finger drag through CDP (`Input.dispatchTouchEvent`) starting at (x, y)
 * and moving the finger up by `distance` px, which scrolls the page down when nothing locks it. Needs a
 * `hasTouch` context.
 */
async function touchScrollPage(page: Page, x: number, y: number, distance: number) {
  const cdp = await page.context().newCDPSession(page);
  const send = (type: string, points: { x: number; y: number }[]) =>
    cdp.send("Input.dispatchTouchEvent", { type, touchPoints: points.map((p, i) => ({ ...p, id: i + 1 })) });
  await send("touchStart", [{ x, y }]);
  for (let step = 1; step <= 10; step++) {
    await send("touchMove", [{ x, y: y - (distance * step) / 10 }]);
    await page.waitForTimeout(16);
  }
  await send("touchEnd", []);
  await cdp.detach();
  await page.waitForTimeout(300);
}

const lockMarker = (page: Page) =>
  page.evaluate(() => (window as unknown as { __dxScrollLocked?: boolean }).__dxScrollLocked === true);

test.describe("DropdownMenu leaves the page scrollable", () => {
  test("a wheel scroll moves the page while the menu is open, the menu stays open, no lock is taken", async ({
    page,
  }) => {
    await goto(page, "dropdown_menu");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    const trigger = demoRole(page, "button");
    await trigger.click();
    await expect(trigger).toHaveAttribute("data-state", "open");
    await assertNoHorizontalShift(page, rightBefore, "while the dropdown menu is open");

    await assertScrollIsUnlocked(page);
    await expect(trigger, "scrolling the page leaves an anchored menu open").toHaveAttribute("data-state", "open");
    expect(await lockMarker(page), "a dropdown menu must not take the scroll lock").toBe(false);

    await page.keyboard.press("Escape");
    await expect(trigger).toHaveAttribute("data-state", "closed");
    await assertNoHorizontalShift(page, rightBefore, "after the dropdown menu closes");
  });

  test.describe("on a touch device", () => {
    test.use({ hasTouch: true });

    test("a touch pan moves the page while the menu is open and the menu stays open", async ({ page }) => {
      await goto(page, "dropdown_menu");
      await assertPageIsScrollable(page);

      const trigger = demoRole(page, "button");
      await trigger.tap();
      await expect(trigger).toHaveAttribute("data-state", "open");

      await page.evaluate(() => window.scrollTo(0, 0));
      // Pan from a point of the main column that is not under the menu (the sidebar scrolls by itself).
      await touchScrollPage(page, 1000, 650, 300);
      expect(await page.evaluate(() => window.scrollY), "the touch pan moved the page").toBeGreaterThan(0);
      await expect(trigger, "scrolling the page leaves an anchored menu open").toHaveAttribute("data-state", "open");
      expect(await lockMarker(page), "a dropdown menu must not take the scroll lock").toBe(false);
    });
  });
});

test.describe("ContextMenu locks and releases page scroll", () => {
  test("scroll is locked while open (modal by default) and restored after Escape", async ({
    page,
  }) => {
    await goto(page, "context_menu");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    await demoRole(page, "button").click({ button: "right" });
    await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
    await assertNoHorizontalShift(page, rightBefore, "while the context menu is open");

    await assertScrollIsLocked(page);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("menu")).toHaveCount(0);

    await assertScrollIsUnlocked(page);
    await assertNoHorizontalShift(page, rightBefore, "after the context menu closes and scroll unlocks");
  });

  test("the page keeps its scroll position through the lock", async ({ page }) => {
    await goto(page, "context_menu");
    await assertPageIsScrollable(page);
    const trigger = demoRole(page, "button");
    // Scroll the page so the trigger is still in view, then press at its centre with the mouse (a
    // `locator.click()` would scroll the page itself first).
    const top = await readStatic(trigger, (el) => el.getBoundingClientRect().top + window.scrollY);
    const scrolled = Math.max(60, Math.round(top) - 200);
    await page.evaluate((y) => window.scrollTo(0, y), scrolled);
    await expect.poll(() => page.evaluate(() => window.scrollY)).toBe(scrolled);
    const box = await staticBox(trigger);
    await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2, { button: "right" });
    await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
    await assertScrollIsLocked(page);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("menu")).toHaveCount(0);
    await expect.poll(() => lockMarker(page)).toBe(false);
    expect(await page.evaluate(() => window.scrollY), "restored to the position it had").toBe(scrolled);
  });

  test.describe("on a touch device", () => {
    test.use({ hasTouch: true });

    test("a touch pan does not move the page while the menu is open, and does after it closes", async ({ page }) => {
      await goto(page, "context_menu");
      await assertPageIsScrollable(page);
      await page.evaluate(() => window.scrollTo(0, 0));

      await demoRole(page, "button").click({ button: "right" });
      await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
      await touchScrollPage(page, 1000, 650, 300);
      expect(await page.evaluate(() => window.scrollY), "the touch pan must not move the locked page").toBe(0);

      await page.keyboard.press("Escape");
      await expect(page.getByRole("menu")).toHaveCount(0);
      await expect.poll(() => lockMarker(page)).toBe(false);
      await touchScrollPage(page, 1000, 650, 300);
      expect(await page.evaluate(() => window.scrollY), "the touch pan moves the unlocked page").toBeGreaterThan(0);
    });
  });
});

test.describe("Dialog locks on a touch device and gives the page's own styles back", () => {
  test.describe("touch", () => {
    test.use({ hasTouch: true });

    test("a touch pan does not move the page while the dialog is open", async ({ page }) => {
      await goto(page, "dialog");
      await assertPageIsScrollable(page);
      await page.evaluate(() => window.scrollTo(0, 0));

      await demoRole(page, "button").tap();
      await expect(page.getByRole("dialog")).toBeVisible();
      await touchScrollPage(page, 1000, 650, 300);
      expect(await page.evaluate(() => window.scrollY), "the touch pan must not move the locked page").toBe(0);
    });
  });

  test("the viewport scroller's inline styles are the same before and after a lock cycle", async ({ page }) => {
    await goto(page, "dialog");
    await assertPageIsScrollable(page);
    const styles = () =>
      page.evaluate(() => ({
        // An emptied `style` attribute reads as "" where an untouched one is null: the same styles.
        html: document.documentElement.getAttribute("style") || "",
        body: document.body.getAttribute("style") || "",
      }));
    const before = await styles();
    expect(before.html + before.body, "no lock style before the lock").not.toContain("overflow");

    await demoRole(page, "button").click();
    await expect(page.getByRole("dialog")).toBeVisible();
    const during = await styles();
    expect(during.html + during.body, "the lock hides overflow inline").toContain("overflow");

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect.poll(() => lockMarker(page)).toBe(false);
    expect(await styles(), "no inline style is left behind by the lock").toEqual(before);
  });
});

/**
 * dev-docs/backlog.md row 9: `Select`'s open listbox locks page scroll too,
 * via the same `ScrollLockGuard`/`use_scroll_lock` mechanism as every other
 * `test.describe` block above -- see this file's header doc for how it's
 * wired (opt-out `scroll_lock` prop, default `true`, on `Select`/
 * `SelectMulti`, mounted from `SelectListRendered`, `primitives/src/select/
 * components/list.rs`).
 */
test.describe("Select locks and releases page scroll", () => {
  test("scroll is locked while the listbox is open (default scroll_lock=true) and restored after Escape", async ({
    page,
  }) => {
    await goto(page, "select");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    // Same role/locator convention as select.spec.ts's `singleSelectTrigger`
    // -- dev-docs/backlog.md row 8: the trigger has role="combobox".
    const trigger = demoRole(page, "combobox");
    await trigger.click();
    const listbox = page.getByRole("listbox");
    await expect(listbox).toHaveAttribute("data-state", "open");
    await assertNoHorizontalShift(page, rightBefore, "while the select listbox is open");

    // A wheel attempt while the listbox is open must not scroll the page,
    // same contract as every other overlay above.
    await assertScrollIsLocked(page);

    await page.keyboard.press("Escape");
    await expect(listbox).toHaveCount(0);

    await assertScrollIsUnlocked(page);
    await assertNoHorizontalShift(page, rightBefore, "after the select listbox closes and scroll unlocks");
  });

  /**
   * dev-docs/backlog.md rows 9/72: the `scroll_lock=false` opt-out itself has its own
   * fixture: `preview/src/components/top_layer/component.rs`'s "Select scroll_lock
   * opt-out" section holds a labelled, distinct `Select` instance with
   * `scroll_lock: false` explicitly set -- the `top_layer` fixture page is used
   * here (not `select`'s own gallery page, which the default-`true` case above
   * already uses) because it is already taller than the viewport (every other
   * overlay section on it), satisfying `assertPageIsScrollable` for free, and it
   * gives this instance its own trigger id with no risk of colliding with the
   * default-Select case's own role/text-based locator above.
   */
  test("scroll_lock=false leaves the page scrollable while the listbox is open", async ({ page }) => {
    await goto(page, "top_layer");
    await assertPageIsScrollable(page);
    const rightBefore = await rightEdgeMarkerRight(page);

    // `.evaluate((el) => el.click())`, not Playwright's own `.click()` --
    // same technique `top-layer.spec.ts` uses for its own
    // fixed/off-screen triggers (e.g. `#stack-popover-trigger`,
    // `#edge-bottom-popover-trigger`): this fixture's "Select
    // scroll_lock opt-out" section is the LAST thing on the page, so a real
    // `.click()`'s own actionability auto-scroll (`scrollIntoViewIfNeeded`)
    // lands the page at its maximum scrollY, leaving no further room for the
    // wheel-scroll assertion below to move `window.scrollY` at all,
    // regardless of whether the lock was actually suppressed (a
    // test-methodology hazard; see `dev-docs/dx-serve-hot-reload.md` for the
    // general class).
    // Clicking via raw JS bypasses that auto-scroll entirely, leaving the
    // page at its initial scrollY (0) with the whole page below still free
    // to scroll into.
    const trigger = page.locator("#scroll-lock-select-trigger");
    await trigger.evaluate((el) => (el as HTMLElement).click());
    const listbox = page.locator("#scroll-lock-select-content");
    await expect(listbox).toHaveAttribute("data-state", "open");
    await assertNoHorizontalShift(page, rightBefore, "while the scroll_lock=false listbox is open");

    // The opt-out itself: reuses `assertScrollIsUnlocked` as-is (it resets
    // scrollY to 0 and wheel-scrolls on each attempt, exactly the shape
    // this case needs, and already retries against the same settling delay
    // this assertion would otherwise have to duplicate: a
    // bespoke single-shot `wheel()` -> immediate
    // `scrollY` read here is genuinely racy: `window.scrollY` does not
    // always update synchronously the instant `page.mouse.wheel()`
    // resolves, occasionally landing a couple hundred ms later even
    // outside this sandbox's own CPU contention -- the same class of
    // settling delay `assertScrollIsUnlocked`'s own doc and Rule 8's
    // `waitForTimeout` in `top-layer.spec.ts` already document elsewhere).
    // Unlike every *other* call to this helper in this file (which follows
    // a real lock releasing), here nothing was ever locking in the first
    // place -- proving the identical thing: a wheel scroll succeeds while
    // this listbox is open, `scroll_lock: false` suppresses the lock
    // rather than merely being accepted and ignored.
    await assertScrollIsUnlocked(page);

    await page.keyboard.press("Escape");
    await expect(listbox).toHaveCount(0);
  });
});

test.describe("Dialog does not leave the page permanently scrolled after a lock cycle", () => {
  /**
   * Beyond the wheel-driven blocked/unblocked checks above and the
   * *horizontal* layout-shift checks, this checks the page's own
   * *vertical scroll position*, `window.scrollY`, across a full open-and-
   * close cycle. A modal `<dialog>`'s `showModal()` can move the page by
   * several hundred pixels the instant it runs (its own default
   * scroll-into-view-on-focus behavior, which the lock does not
   * suppress), and a lock/unlock cycle that does not restore
   * `scrollY`/`scrollX` would leave it there: every dialog interaction would
   * permanently scroll the user's page away from where they left it, most
   * visibly stranding this docs site's `position: sticky` top nav
   * off-screen for the rest of the session.
   *
   * `use_early_scroll_capture` in `scroll_lock.rs` covers it (captures/
   * restores scroll position around the lock, from `DialogRoot`/
   * `AlertDialogRoot` specifically so it runs before `DialogContent`'s own
   * `showModal()` call). Run through this suite's own `goto` helper (which
   * waits for `networkidle`; an under-settled page, with its layout still
   * settling mid-interaction, makes the test look red), in default headless
   * Chromium and under `playwright/xvfb.local.config.ts`'s real classic
   * scrollbar, it passes cleanly. It guards a failure mode (a native
   * `<dialog>`'s focus/scroll-into-view behavior
   * stranding the page) is well-documented upstream, and this is the one
   * assertion that catches it.
   */
  test("closing a Dialog restores window.scrollY to where it was before opening", async ({
    page,
  }) => {
    await goto(page, "dialog");
    await assertPageIsScrollable(page);

    const before = await page.evaluate(() => window.scrollY);

    await demoRole(page, "button").click();
    await expect(page.getByRole("dialog")).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);

    // The unlock's `document::eval` round trip settles asynchronously (see
    // `assertScrollIsUnlocked`'s doc above for the same caveat applied to
    // `overflow`) -- poll rather than sampling once immediately.
    const deadline = Date.now() + 8000;
    let after = await page.evaluate(() => window.scrollY);
    while (after !== before && Date.now() < deadline) {
      await page.waitForTimeout(200);
      after = await page.evaluate(() => window.scrollY);
    }

    expect(
      after,
      "the page must be scrolled back to where it was before the dialog opened, not left stranded wherever showModal()'s own focus-scroll behavior happened to leave it",
    ).toBe(before);
  });
});

test.describe("scrollbar-gutter reservation survives the lock itself, not just its absence", () => {
  /**
   * NOT the same thing `assertNoHorizontalShift` above checks (a
   * `position: fixed` marker's on-screen edge). This checks
   * `document.documentElement.clientWidth` directly:
   * The lock hides `<html>`'s overflow, which removes a classic
   * scrollbar's width from the page unless a `scrollbar-gutter: stable`
   * keeps it (`scroll_lock.rs`, "How the lock blocks scrolling"): without
   * the gutter `clientWidth` would read 1012px unlocked (1027px viewport
   * minus a real ~15px scrollbar) and jump to the full 1027px the instant a
   * `Dialog` opened and locked scroll (`playwright/xvfb.local.config.ts`, a
   * real classic scrollbar). The lock sets the gutter itself when a classic
   * scrollbar is showing, and the permanent baseline
   * (`ensure_scrollbar_gutter_baseline`) has it in place before any lock.
   */
  test("Dialog: opening a modal does not widen clientWidth by the scrollbar's reserved gutter", async ({
    page,
  }) => {
    await goto(page, "dialog");
    await assertPageIsScrollable(page);

    const before = (await pageMetrics(page)).clientWidth;

    await demoRole(page, "button").click();
    await expect(page.getByRole("dialog")).toBeVisible();

    const duringOpen = (await pageMetrics(page)).clientWidth;

    expect(
      duringOpen,
      "the scrollbar-gutter reservation must still hold clientWidth steady while a modal has scroll locked, not just before any modal has ever opened",
    ).toBe(before);
  });
});

test.describe("Breakpoint flip: exact 1027px viewport with a classic scrollbar", () => {
  /**
   * Ties this suite's generic gutter check above (measured at
   * whatever width `xvfb.local.config.ts`'s default viewport happens to
   * be) directly to the user-facing symptom it prevents:
   * the page jumps and the top nav disappears at around 1027px wide.
   * 1027px is not an arbitrary number -- a (viewport width) of 1027px with a real
   * ~15px classic scrollbar subtracted lands at 1012px, and this repo's own
   * responsive breakpoints react to `document.documentElement.clientWidth`
   * (not `window.innerWidth`, which a scrollbar never affects); a
   * clientWidth that silently jumps from 1012 to 1027 the instant a modal
   * locks scroll is exactly the kind of change that can cross a layout
   * breakpoint sitting anywhere in that 15px band. This test pins the
   * viewport to precisely that width and
   * checks two independent signals at once: `clientWidth` directly (the
   * mechanism), and the sticky top nav's (`nav.dx-preview-navbar`, `position:
   * sticky; top: 0` -- `preview/assets/main.css`) `getBoundingClientRect()`
   * (the symptom the user actually sees -- this element spans the full
   * layout width, so it visibly reflows/shifts if the lock widens
   * `clientWidth`, which reads as "the nav disappeared" when it happens
   * during a modal's open animation).
   *
   * A lock that hides `<html>`'s overflow without keeping the gutter fails both assertions
   * under `xvfb.local.config.ts` (real classic scrollbar): `clientWidth`
   * jumps 1012 -> 1027 on open, and the nav's bounding rect widens/moves to
   * match. Both assertions hold under `oracle.local.config.ts` (headless,
   * 0-width overlay scrollbar -- nothing to catch, proves the lock
   * introduces no shift on that platform either) and `xvfb.local.config.ts`
   * (the real regression case).
   */
  test("Dialog open/close never changes clientWidth or moves the sticky top nav at 1027px", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1027, height: 720 });
    await goto(page, "dialog");
    await assertPageIsScrollable(page);

    const nav = page.locator("nav.dx-preview-navbar");
    const widthBefore = (await pageMetrics(page)).clientWidth;
    const navBefore = await measureAtRest(nav);

    await demoRole(page, "button").click();
    await expect(page.getByRole("dialog")).toBeVisible();

    expect(
      (await pageMetrics(page)).clientWidth,
      "clientWidth must not change the instant the modal locks scroll, at 1027px, where a classic scrollbar's gutter shows",
    ).toBe(widthBefore);
    expect(
      await measureAtRest(nav),
      "the sticky top nav must not reflow/shift while the modal is open",
    ).toEqual(navBefore);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);

    // The unlock's `document::eval` round trip settles asynchronously (see
    // `assertScrollIsUnlocked`'s doc above for the same caveat) -- poll
    // rather than sampling once immediately.
    const deadline = Date.now() + 8000;
    let widthAfter = (await pageMetrics(page)).clientWidth;
    while (widthAfter !== widthBefore && Date.now() < deadline) {
      await page.waitForTimeout(200);
      widthAfter = (await pageMetrics(page)).clientWidth;
    }

    expect(widthAfter, "clientWidth must return to its pre-lock value after the modal closes").toBe(
      widthBefore,
    );
    expect(
      await measureAtRest(nav),
      "the sticky top nav must be back exactly where it was once the modal closes",
    ).toEqual(navBefore);
  });
});

/**
 * Backlog row 375: a lock handed from one surface to another in ONE click. The `top_layer` fixture's
 * "Scroll-lock handoff" section renders two SIBLING themed dialogs (not nested): A's "Continue" closes A and
 * opens B. A keeps its lock through its exit animation (the release waits for the `<dialog>`'s `close`
 * event), so for a moment both are held. If locks were counted per nesting chain, the two siblings would have
 * two counters, and A's last release would run `window.scrollTo(A's saved position)` while B was open, moving
 * the page under B. With one page-wide lock, A's release is not the last one and touches nothing; B's close
 * restores.
 *
 * What the restore does to the page depends on whether anything moved it after A's capture (a `showModal()`
 * focus jump, an app's own scroll): headless Chromium moves nothing here, so A's stray restore lands on the
 * same Y and `scrollY` alone stays flat (one `scrollTo(0, 400)` ~730 ms after B opened, page already at
 * 400, with per-chain counting). The test therefore also spies on `window.scrollTo` and asserts that
 * nothing restores the page while B holds it, which is red under per-chain counting in every run.
 *
 * The page is scrolled to a non-zero Y first, and every click on a page-level trigger goes through
 * `el.click()` so Playwright's actionability auto-scroll never moves the page itself.
 */
test.describe("Scroll lock handed from one dialog to a sibling in one click", () => {
  const scrollY = (page: Page) => page.evaluate(() => window.scrollY);

  test("the page does not move under B when A's lock lets go, and B's close restores it", async ({ page }) => {
    await goto(page, "top_layer");
    await assertPageIsScrollable(page);
    await page.evaluate(() => window.scrollTo(0, 400));
    await expect.poll(() => scrollY(page)).toBe(400);

    await page.locator("#handoff-a-trigger").evaluate((el) => (el as HTMLElement).click());
    const continueButton = page.locator("#handoff-a-continue");
    await expect(continueButton).toBeVisible();
    // Anything the open moved is the open's business (covered above); this test is about the handoff.
    await page.waitForTimeout(500);
    const atOpen = await scrollY(page);
    await page.evaluate(() => {
      const calls: string[] = ((window as any).__handoffScrollTo = []);
      const original = window.scrollTo.bind(window);
      (window as any).scrollTo = (...args: unknown[]) => {
        calls.push(JSON.stringify(args));
        return (original as (...a: unknown[]) => void)(...args);
      };
    });

    await continueButton.evaluate((el) => (el as HTMLElement).click());
    await expect(page.locator("#handoff-b-close")).toBeVisible();
    const samples: number[] = [];
    const start = Date.now();
    for (const at of [0, 100, 400, 800]) {
      await page.waitForTimeout(Math.max(0, at - (Date.now() - start)));
      samples.push(await scrollY(page));
    }
    expect(
      samples,
      `window.scrollY sampled at 0/100/400/800 ms after B opened (A was open at ${atOpen})`,
    ).toEqual([samples[0], samples[0], samples[0], samples[0]]);
    // A's lock lets go at its `close` event, after the exit; its content unmounts ~250 ms later.
    await expect(continueButton).toHaveCount(0);
    await page.waitForTimeout(200);
    expect(await scrollY(page), "the page did not move under B once A was gone").toBe(samples[0]);
    expect(
      await page.evaluate(() => (window as any).__handoffScrollTo),
      "no scroll restore may run while B holds the page (A's release is not the last one)",
    ).toEqual([]);
    expect(await page.evaluate(() => (window as any).__dxScrollLocked), "B still holds the lock").toBe(true);
    await assertScrollIsLocked(page);

    await page.locator("#handoff-b-close").click();
    await expect(page.locator("#handoff-b-close")).toHaveCount(0);
    await expect
      .poll(() => scrollY(page), { message: "B's close restores the position from before A opened" })
      .toBe(400);
    await expect.poll(() => page.evaluate(() => (window as any).__dxScrollLocked)).toBe(false);
  });
});
