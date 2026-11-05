/**
 * ORACLE: tier 1 (APG) — one menu family behind three hosts (backlog row 435).
 *
 * DropdownMenu, ContextMenu and Menubar are three hosts of one menu: items, checkable items, submenus, roving focus
 * and typeahead are the same behaviour, so this spec iterates the three hosts over one body of assertions. A fix to an
 * item then fails (or passes) in all three at once, and a host that drifts shows up as one red row.
 *
 * Rule source: APG Menu and Menubar pattern, "Keyboard Interaction" (`content/patterns/menubar/menu-and-menubar-
 * pattern.html`, pinned commit `7e4034b262bc0d25332e330d8a582aaf34113829` of `w3c/aria-practices`, see
 * `playwright/oracle/reference/README.md`): Down/Up Arrow move to the next/previous item (wrapping), Home/End to the
 * first/last, a printable character moves to the next item whose label starts with it, Enter activates, Escape closes
 * and returns focus to the element the menu was opened from; Space on a `menuitemcheckbox` changes the state "without
 * closing the menu". Where this spec asserts something APG does not say (selection model, hover intent) the test
 * title says it is this crate's contract.
 *
 * WHAT IS THE SAME in all three hosts (the iterated tests below):
 *   - the open menu is a named `role="menu"` of `menuitem` / `menuitemcheckbox` / `menuitemradio` items; a disabled item
 *     is `aria-disabled` and `data-disabled`;
 *   - Down/Up/Home/End rove through the enabled items and wrap, a disabled item is skipped;
 *   - typeahead lands on an enabled item by the start of its label and never on a disabled one;
 *   - Enter and Space select the focused plain item: the menu closes, the selection is reported once, focus returns to
 *     the trigger; Escape closes and returns focus too; a click selects the same way, a click on a disabled item is
 *     inert; a press outside closes;
 *   - a checkbox item toggles on click, Space and Enter and keeps the menu open (Base UI's `closeOnClick = false`); a
 *     radio item moves the group's one check; `close_on_select: true` closes the whole menu tree; a disabled
 *     checkable item cannot be toggled; typeahead reaches a checkable item;
 *   - a touch tap on an item selects it;
 *   - (DropdownMenu and ContextMenu) a submenu opens on ArrowRight / Enter / Space / click / hover intent and puts
 *     focus on its first item, ArrowLeft and Escape close just the submenu and return focus to its trigger, an item of
 *     the submenu closes the whole tree, typeahead inside it is scoped to its own items.
 *
 * INTENTIONAL DIFFERENCES (each one asserted in the "differences" blocks at the end, so they cannot drift unseen):
 *   - Selection commit (`SelectOn`, primitives/src/menu_item.rs): DropdownMenu commits on `click` (a press that ends off
 *     the item selects nothing), ContextMenu on release (`pointerup` for a mouse, the `click` after the release for
 *     touch and pen: the long press that opened the menu and lifts over an item must not select it), Menubar on
 *     `pointerdown` for a mouse (it closes and hands focus back to the trigger inside that same press, backlog row
 *     140) and on the release for touch and pen, like ContextMenu (a touch that turns into a scroll never selects,
 *     backlog row 489; `menubar.spec.ts` "An item press" asserts the scroll, the tap and the mouse press).
 *   - ContextMenu: no trigger button and no keyboard opening (the trigger is `tabindex="-1"`); the menu is placed at the
 *     pointer point (a long press opens it for touch and pen); its scroll lock is the only one of the three.
 *   - Menubar: the trigger row is a roving `menubar` of `menuitem` triggers; ArrowLeft/ArrowRight, Home and End on an
 *     open menu move to the neighbouring, first or last menu and open it, hovering a trigger while one menu is open
 *     switches menus; no submenus.
 *   - DropdownMenu: the trigger is a toggle button; ArrowDown/ArrowUp on the closed trigger open the menu on the
 *     first/last item.
 *
 * LOCATORS: a control is named by role and position inside the demo frame (`demo-frame.ts`), never by copy. The popup
 * is `[role="menu"][popover]` (the Menubar's `role="menu"` wrapper has no `popover`); a submenu is the last such
 * element, a descendant of the root menu. Item facts are read from the DOM (the enabled items in tree order are the
 * roving order), so the demos can change their copy and their item count.
 */

import { test, expect } from "../../fixtures";
import { type Locator, type Page } from "@playwright/test";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";
import { demoFrame, demoRole } from "../../demo-frame";
import { awaitAnimationsSettled, measureAtRest } from "../../animations";

const ITEM = ':is([role="menuitem"], [role="menuitemcheckbox"], [role="menuitemradio"])';
const ENABLED = `${ITEM}:not([aria-disabled="true"])`;
const DISABLED = `${ITEM}[aria-disabled="true"]`;

/** How a host commits a mouse press on a plain item: `click`, on release, or on the press itself (touch and pen: on the tap, in all three). */
type Commit = "click" | "release" | "press";

interface Host {
  /** The describe title. */
  name: string;
  /** The component's route name. */
  route: string;
  commit: Commit;
  submenus: boolean;
  /** A variant whose menu holds a `close_on_select` checkable item, and how to reach it once the menu is open. */
  closing: { variant: string; reach: (page: Page) => Promise<Locator> };
}

/** The open root menu. */
const rootMenu = (page: Page) => page.locator('[role="menu"][popover]').first();
/** The innermost open menu: the submenu while one is open. */
const innermostMenu = (page: Page) => page.locator('[role="menu"][popover]').last();
const anyMenu = (page: Page) => page.locator('[role="menu"][popover]');

const trigger = (host: Host, page: Page, variant = "main"): Locator =>
  host.route === "menubar" ? demoRole(page, "menuitem", 0, variant) : demoRole(page, "button", 0, variant);

async function open(host: Host, page: Page, variant = "main"): Promise<void> {
  const t = trigger(host, page, variant);
  if (host.route === "context_menu") await t.click({ button: "right" });
  else await t.click();
  const menu = rootMenu(page);
  await expect(menu).toHaveAttribute("data-state", "open");
  await awaitAnimationsSettled(menu);
}

const goto = (page: Page, host: Host) =>
  gotoHydrated(page, `${BASE_URL}/component/${host.route}/`, { timeout: 20 * 60 * 1000 });

/** The text the item shows, trimmed. */
async function label(item: Locator): Promise<string> {
  return ((await item.textContent()) ?? "").trim();
}

/** ArrowDown until `target` holds focus (bounded), so a test reaches an item whatever the demo's order. */
async function arrowTo(page: Page, target: Locator): Promise<void> {
  for (let i = 0; i < 16; i++) {
    await page.keyboard.press("ArrowDown");
    if (await target.evaluate((el) => el === document.activeElement)) return;
  }
  await expect(target, "reached by ArrowDown").toBeFocused();
}

const HOSTS: Host[] = [
  {
    name: "DropdownMenu",
    route: "dropdown_menu",
    commit: "click",
    submenus: true,
    closing: {
      variant: "checkboxes",
      // The "More options" submenu's last item closes the whole tree.
      reach: async (page) => {
        await rootMenu(page).locator(`${ITEM}[aria-haspopup]`).click();
        const sub = innermostMenu(page);
        await expect(sub).toHaveAttribute("data-state", "open");
        await awaitAnimationsSettled(sub);
        return sub.locator('[role="menuitemcheckbox"]').last();
      },
    },
  },
  {
    name: "ContextMenu",
    route: "context_menu",
    commit: "release",
    submenus: true,
    closing: {
      variant: "checkboxes",
      reach: async (page) => {
        await rootMenu(page).locator(`${ITEM}[aria-haspopup]`).click();
        const sub = innermostMenu(page);
        await expect(sub).toHaveAttribute("data-state", "open");
        await awaitAnimationsSettled(sub);
        return sub.locator('[role="menuitemcheckbox"]').last();
      },
    },
  },
  {
    name: "Menubar",
    route: "menubar",
    commit: "press",
    submenus: false,
    closing: {
      // The "Options" menu: Create, Launch, Pinned (a checkbox item with close_on_select).
      variant: "rtl",
      reach: async (page) => rootMenu(page).locator('[role="menuitemcheckbox"]').first(),
    },
  },
];

for (const host of HOSTS) {
  test.describe(`menu family parity — ${host.name}`, () => {
    test.beforeEach(async ({ page }) => {
      await goto(page, host);
    });

    // ---- anatomy ----

    test("the open menu is a named role=menu of items; a disabled item is aria-disabled and data-disabled", async ({ page }) => {
      await open(host, page);
      const menu = rootMenu(page);
      const named =
        (await menu.getAttribute("aria-labelledby")) !== null || (await menu.getAttribute("aria-label")) !== null;
      expect(named, "APG: a menu has an accessible name").toBe(true);
      expect(await menu.locator(ITEM).count(), "the menu has items").toBeGreaterThanOrEqual(3);
      const disabled = menu.locator(DISABLED);
      expect(await disabled.count(), "the demo has a disabled item").toBeGreaterThanOrEqual(1);
      await expect(disabled.first()).toHaveAttribute("data-disabled", "true");
    });

    // ---- keyboard ----

    test("Down, Up, Home and End rove through the enabled items and wrap; a disabled item is skipped", async ({ page }) => {
      await open(host, page);
      const enabled = rootMenu(page).locator(ENABLED);
      const n = await enabled.count();
      expect(n).toBeGreaterThanOrEqual(2);

      await page.keyboard.press("ArrowDown");
      await expect(enabled.nth(0)).toBeFocused();
      for (let i = 1; i < n; i++) {
        await page.keyboard.press("ArrowDown");
        await expect(enabled.nth(i), `ArrowDown #${i + 1}`).toBeFocused();
      }
      await page.keyboard.press("ArrowDown");
      await expect(enabled.nth(0), "ArrowDown wraps to the first").toBeFocused();
      await page.keyboard.press("ArrowUp");
      await expect(enabled.nth(n - 1), "ArrowUp wraps to the last").toBeFocused();
      if (host.route !== "menubar") {
        // In a Menubar, Home and End move along the row of menus instead (see the Menubar differences block).
        await page.keyboard.press("Home");
        await expect(enabled.nth(0)).toBeFocused();
        await page.keyboard.press("End");
        await expect(enabled.nth(n - 1)).toBeFocused();
      }
      await expect(rootMenu(page).locator(`${DISABLED}:focus`), "a disabled item never holds focus").toHaveCount(0);
    });

    test("typeahead lands on an enabled item by the start of its label, never on a disabled one", async ({ page }) => {
      await open(host, page);
      const menu = rootMenu(page);
      const enabled = menu.locator(ENABLED);
      await arrowTo(page, enabled.last());

      const first = enabled.first();
      const letter = (await label(first))[0].toLowerCase();
      await page.keyboard.press(letter);
      await expect(first, `typing "${letter}" moves to the first item (wrapping)`).toBeFocused();

      // The typeahead buffer clears after TYPEAHEAD_RESET_MS (1 s); the disabled item's first letter then matches nothing.
      // timing-bound-ok: nothing may happen in this window, so a loaded machine only lengthens it
      await page.waitForTimeout(1200);
      const disabledLetter = (await label(menu.locator(DISABLED).first()))[0].toLowerCase();
      await page.keyboard.press(disabledLetter);
      await expect(first, "typeahead does not move onto a disabled item").toBeFocused();
    });

    for (const [key, nth] of [
      ["Enter", 0],
      ["Space", 1],
    ] as const) {
      test(`${key} selects the focused plain item: the menu closes, the selection is reported, focus returns to the trigger`, async ({ page }) => {
        await open(host, page);
        const enabled = rootMenu(page).locator(ENABLED);
        for (let i = 0; i <= nth; i++) await page.keyboard.press("ArrowDown");
        const item = enabled.nth(nth);
        await expect(item).toBeFocused();
        const text = await label(item);
        await page.keyboard.press(key);
        await expect(anyMenu(page), "the menu closes").toHaveCount(0);
        await expect(demoFrame(page), "the selection is reported").toContainText(new RegExp(`Selected: ${text}`, "i"));
        await expect(trigger(host, page), "focus returns to the trigger").toBeFocused();
      });
    }

    test("Escape closes the menu and returns focus to the trigger", async ({ page }) => {
      await open(host, page);
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("Escape");
      await expect(anyMenu(page)).toHaveCount(0);
      await expect(trigger(host, page), "focus returns to the trigger").toBeFocused();
    });

    test("Tab leaves the menu and closes it", async ({ page }) => {
      await open(host, page);
      await page.keyboard.press("ArrowDown");
      await page.keyboard.press("Tab");
      await expect(anyMenu(page)).toHaveCount(0);
    });

    // ---- pointer ----

    test("a click selects the item: the menu closes, the selection is reported, focus returns to the trigger", async ({ page }) => {
      await open(host, page);
      const item = rootMenu(page).locator(ENABLED).nth(1);
      const text = await label(item);
      await item.click();
      await expect(anyMenu(page)).toHaveCount(0);
      await expect(demoFrame(page)).toContainText(new RegExp(`Selected: ${text}`, "i"));
      await expect(trigger(host, page), "a mouse press hands focus back to the trigger").toBeFocused();
    });

    test("a click on a disabled item does nothing: the menu stays open and nothing is reported", async ({ page }) => {
      await open(host, page);
      // `force`: Playwright otherwise waits for an aria-disabled element to become actionable.
      await rootMenu(page).locator(DISABLED).first().click({ force: true });
      await page.waitForTimeout(200);
      await expect(rootMenu(page)).toHaveAttribute("data-state", "open");
      await expect(demoFrame(page)).not.toContainText("Selected:");
    });

    test(`a press outside the menu closes it`, async ({ page }) => {
      await open(host, page);
      await page.locator("body").click({ position: { x: 0, y: 0 } });
      await expect(anyMenu(page)).toHaveCount(0);
    });

    test(`the pointer commits on ${host.commit === "click" ? "click" : host.commit === "release" ? "release" : "press"} (SelectOn)`, async ({ page }) => {
      await open(host, page);
      const menu = rootMenu(page);
      const item = menu.locator(ENABLED).nth(1);
      const text = await label(item);
      const box = await item.boundingBox();
      if (!box) throw new Error("the item has no box");
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      await page.mouse.down();
      if (host.commit === "press") {
        await expect(anyMenu(page), "pressing selects at once").toHaveCount(0);
        await expect(demoFrame(page)).toContainText(new RegExp(`Selected: ${text}`, "i"));
        await page.mouse.up();
        return;
      }
      await page.waitForTimeout(150);
      await expect(menu, "pressing alone does not select").toHaveAttribute("data-state", "open");
      await expect(demoFrame(page)).not.toContainText("Selected:");
      // Release off the item: nothing is selected.
      await page.mouse.move(2, 2);
      await page.mouse.up();
      await page.waitForTimeout(150);
      await expect(menu, "a press that ends off the item selects nothing").toHaveAttribute("data-state", "open");
      await expect(demoFrame(page)).not.toContainText("Selected:");
      // Press and release on the item: selected.
      await item.click();
      await expect(anyMenu(page)).toHaveCount(0);
      await expect(demoFrame(page)).toContainText(new RegExp(`Selected: ${text}`, "i"));
    });

    // ---- submenus (DropdownMenu and ContextMenu) ----

    if (host.submenus) {
      const subTrigger = (page: Page) => rootMenu(page).locator(`${ITEM}[aria-haspopup]`).first();
      const subItems = (page: Page) => innermostMenu(page).locator(ITEM);

      test("a submenu: ArrowRight opens it on its first item, ArrowLeft closes it and returns focus to the sub-trigger", async ({ page }) => {
        await open(host, page);
        const st = subTrigger(page);
        await expect(st).toHaveAttribute("aria-expanded", "false");
        await arrowTo(page, st);
        await page.keyboard.press("ArrowRight");
        await expect(anyMenu(page)).toHaveCount(2);
        await expect(st).toHaveAttribute("aria-expanded", "true");
        await expect(subItems(page).first(), "focus lands on the submenu's first item").toBeFocused();

        await page.keyboard.press("ArrowLeft");
        await expect(anyMenu(page)).toHaveCount(1);
        await expect(st).toBeFocused();
        await expect(st).toHaveAttribute("aria-expanded", "false");
        await expect(rootMenu(page), "the root menu stays open").toHaveAttribute("data-state", "open");
      });

      for (const key of ["Enter", "Space"]) {
        test(`${key} on the sub-trigger opens the submenu; Escape closes just the submenu`, async ({ page }) => {
          await open(host, page);
          const st = subTrigger(page);
          await arrowTo(page, st);
          await page.keyboard.press(key);
          await expect(anyMenu(page)).toHaveCount(2);
          await expect(subItems(page).first()).toBeFocused();
          await page.keyboard.press("Escape");
          await expect(anyMenu(page), "Escape closes only the menu that holds focus").toHaveCount(1);
          await expect(st).toBeFocused();
          await expect(rootMenu(page)).toHaveAttribute("data-state", "open");
        });
      }

      test("an item of the submenu closes the whole tree and reports its value", async ({ page }) => {
        await open(host, page);
        await arrowTo(page, subTrigger(page));
        await page.keyboard.press("ArrowRight");
        await expect(anyMenu(page)).toHaveCount(2);
        await page.keyboard.press("ArrowDown");
        const item = subItems(page).nth(1);
        await expect(item).toBeFocused();
        const text = await label(item);
        await page.keyboard.press("Enter");
        await expect(anyMenu(page), "the whole tree closes").toHaveCount(0);
        await expect(demoFrame(page)).toContainText(new RegExp(`Selected: ${text}`, "i"));
        await expect(trigger(host, page)).toBeFocused();
      });

      test("a click on the sub-trigger opens the submenu on its first item; a click on a submenu item closes the tree", async ({ page }) => {
        await open(host, page);
        await subTrigger(page).click();
        await expect(anyMenu(page)).toHaveCount(2);
        await expect(subItems(page).first(), "a click moves focus into the submenu").toBeFocused();
        const item = subItems(page).nth(1);
        const text = await label(item);
        await awaitAnimationsSettled(innermostMenu(page));
        await item.click();
        await expect(anyMenu(page)).toHaveCount(0);
        await expect(demoFrame(page)).toContainText(new RegExp(`Selected: ${text}`, "i"));
      });

      test("hovering the sub-trigger opens the submenu without moving focus; leaving both closes it (this crate's hover-intent contract)", async ({ page }) => {
        await open(host, page);
        const st = subTrigger(page);
        await awaitAnimationsSettled(st);
        await st.hover();
        await expect(st, "opens after the hover-intent delay").toHaveAttribute("aria-expanded", "true");
        await expect(anyMenu(page)).toHaveCount(2);
        await expect(subItems(page).first(), "a hover open does not steal focus").not.toBeFocused();
        // Moving to a plain root item leaves both halves; the submenu closes after the grace delay.
        await rootMenu(page).locator(ENABLED).first().hover();
        await expect(st).toHaveAttribute("aria-expanded", "false");
        await expect(anyMenu(page)).toHaveCount(1);
      });

      test("typeahead inside the submenu searches only its own items", async ({ page }) => {
        await open(host, page);
        await arrowTo(page, subTrigger(page));
        await page.keyboard.press("ArrowRight");
        await expect(anyMenu(page)).toHaveCount(2);
        const items = subItems(page);
        await expect(items.first()).toBeFocused();
        const second = items.nth(1);
        await page.keyboard.press((await label(second))[0].toLowerCase());
        await expect(second).toBeFocused();
        await expect(anyMenu(page), "typing never closes or leaves the submenu").toHaveCount(2);
      });
    }

    // ---- checkable items ----

    test("checkbox items: aria-checked is always present; a click toggles, Space and Enter toggle in place, the menu stays open; a disabled one is inert", async ({ page }) => {
      await open(host, page, "checkboxes");
      const menu = rootMenu(page);
      const boxes = menu.locator('[role="menuitemcheckbox"]');
      const count = await boxes.count();
      expect(count).toBeGreaterThanOrEqual(2);
      for (let i = 0; i < count; i++) {
        const checked = await boxes.nth(i).getAttribute("aria-checked");
        expect(checked, `checkbox ${i} always carries aria-checked`).toMatch(/^(true|false)$/);
        await expect(boxes.nth(i)).toHaveAttribute("data-state", checked === "true" ? "checked" : "unchecked");
      }

      const enabledBoxes = menu.locator('[role="menuitemcheckbox"]:not([aria-disabled="true"])');
      const target = enabledBoxes.first();
      const before = await target.getAttribute("aria-checked");
      await target.click();
      await expect(target).toHaveAttribute("aria-checked", before === "true" ? "false" : "true");
      await expect(menu, "a toggle keeps the menu open").toHaveAttribute("data-state", "open");

      // Keyboard: focus the second enabled checkbox, Space then Enter toggle it in place.
      const other = enabledBoxes.nth(1);
      await arrowTo(page, other);
      const start = await other.getAttribute("aria-checked");
      await page.keyboard.press("Space");
      await expect(other).toHaveAttribute("aria-checked", start === "true" ? "false" : "true");
      await expect(other, "focus stays on the toggled item").toBeFocused();
      await page.keyboard.press("Enter");
      await expect(other).toHaveAttribute("aria-checked", start ?? "false");
      await expect(menu).toHaveAttribute("data-state", "open");

      const disabledBox = menu.locator('[role="menuitemcheckbox"][aria-disabled="true"]');
      if ((await disabledBox.count()) > 0) {
        const was = await disabledBox.first().getAttribute("aria-checked");
        await disabledBox.first().click({ force: true });
        await expect(disabledBox.first(), "a disabled checkbox cannot be toggled").toHaveAttribute("aria-checked", was ?? "false");
      }
    });

    test("typeahead reaches a checkbox item through its text_value", async ({ page }) => {
      await open(host, page, "checkboxes");
      const menu = rootMenu(page);
      const enabled = menu.locator(ENABLED);
      await arrowTo(page, enabled.last());
      const firstBox = menu.locator('[role="menuitemcheckbox"]:not([aria-disabled="true"])').first();
      await page.keyboard.press((await label(firstBox))[0].toLowerCase());
      await expect(firstBox).toBeFocused();
    });

    test("radio items: one item is checked, a click moves the check, Space and Enter choose, the menu stays open", async ({ page }) => {
      await open(host, page, "radio_group");
      const menu = rootMenu(page);
      const group = menu.locator('[role="group"]').first();
      await expect(group).toBeVisible();
      const radios = group.locator('[role="menuitemradio"]');
      const checkedCount = () => group.locator('[role="menuitemradio"][aria-checked="true"]').count();
      expect(await checkedCount(), "exactly one radio is checked").toBe(1);

      // The first enabled radio that is not checked, by position, so the locator keeps naming it once it is.
      const firstUnchecked = async () => {
        const states = await radios.evaluateAll((els) =>
          els.map((el) => ({ checked: el.getAttribute("aria-checked") === "true", disabled: el.getAttribute("aria-disabled") === "true" })),
        );
        const at = states.findIndex((s) => !s.checked && !s.disabled);
        expect(at, "an enabled unchecked radio exists").toBeGreaterThanOrEqual(0);
        return radios.nth(at);
      };

      const clicked = await firstUnchecked();
      await clicked.click();
      await expect(clicked).toHaveAttribute("aria-checked", "true");
      expect(await checkedCount(), "still exactly one checked").toBe(1);
      await expect(menu, "choosing keeps the menu open").toHaveAttribute("data-state", "open");

      // Keyboard: Space / Enter on another radio moves the check to it.
      const byKey = await firstUnchecked();
      await arrowTo(page, byKey);
      await page.keyboard.press("Space");
      await expect(byKey).toHaveAttribute("aria-checked", "true");
      expect(await checkedCount()).toBe(1);
      const byEnter = await firstUnchecked();
      await arrowTo(page, byEnter);
      await page.keyboard.press("Enter");
      await expect(byEnter).toHaveAttribute("aria-checked", "true");
      expect(await checkedCount()).toBe(1);
      expect(await radios.count()).toBeGreaterThanOrEqual(2);
    });

    test("close_on_select: true on a checkable item closes the whole menu tree", async ({ page }) => {
      await open(host, page, host.closing.variant);
      const item = await host.closing.reach(page);
      await expect(item).toBeVisible();
      await item.click();
      await expect(anyMenu(page), "the whole tree closes").toHaveCount(0);
    });
  });
}

// ---- touch: a tap on an item selects it, in all three hosts (a Menubar item no longer commits on the touch's pointerdown) ----

test.describe("menu family parity — touch", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, colorScheme: "light" });

  for (const host of HOSTS) {
    test(`${host.name}: a tap on an item selects it and closes the menu`, async ({ page }) => {
      await goto(page, host);
      const t = trigger(host, page);
      if (host.route === "context_menu") {
        // A touch long press opens the context menu: a touch `pointerdown` held past the long-press delay.
        await t.evaluate((el) => {
          const r = el.getBoundingClientRect();
          el.dispatchEvent(
            new PointerEvent("pointerdown", {
              pointerId: 7777,
              pointerType: "touch",
              isPrimary: true,
              clientX: r.x + r.width / 2,
              clientY: r.y + r.height / 2,
              button: 0,
              buttons: 1,
              bubbles: true,
              cancelable: true,
            }),
          );
        });
      } else {
        await t.tap();
      }
      const menu = rootMenu(page);
      await expect(menu).toHaveAttribute("data-state", "open");
      await awaitAnimationsSettled(menu);
      // Outlive the 700 ms window in which a long-press-opened ContextMenu ignores the re-dispatched press.
      await page.waitForTimeout(900);
      const item = menu.locator(ENABLED).nth(1);
      const text = await label(item);
      await item.tap();
      await expect(anyMenu(page)).toHaveCount(0);
      await expect(demoFrame(page)).toContainText(new RegExp(`Selected: ${text}`, "i"));
    });
  }
});

// ---- touch: the press alone selects nothing (all three hosts) ----

test.describe("menu family parity — touch press", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, colorScheme: "light" });

  for (const host of HOSTS) {
    test(`${host.name}: a touch pointerdown on an item, with no release, selects nothing`, async ({ page }) => {
      await goto(page, host);
      // Opened by mouse (a context menu by a right click): this case is about the item press, not the opening gesture.
      await open(host, page);
      const menu = rootMenu(page);
      await page.waitForTimeout(900);
      const item = menu.locator(ENABLED).nth(1);
      await item.evaluate((el) => {
        const r = el.getBoundingClientRect();
        el.dispatchEvent(
          new PointerEvent("pointerdown", {
            pointerId: 7778,
            pointerType: "touch",
            isPrimary: true,
            clientX: r.x + r.width / 2,
            clientY: r.y + r.height / 2,
            button: 0,
            buttons: 1,
            bubbles: true,
            cancelable: true,
          }),
        );
      });
      await page.waitForTimeout(300);
      await expect(menu, "a touch press without a release leaves the menu open").toHaveAttribute("data-state", "open");
      await expect(demoFrame(page)).not.toContainText("Selected:");
    });
  }
});

// ---- the intentional differences, asserted ----

test.describe("menu family differences — DropdownMenu", () => {
  const host = HOSTS[0];
  test.beforeEach(async ({ page }) => {
    await goto(page, host);
  });

  test("the trigger toggles; ArrowDown on the closed trigger opens the menu on its first item, ArrowUp on the last", async ({ page }) => {
    const t = trigger(host, page);
    await t.click();
    await expect(rootMenu(page)).toHaveAttribute("data-state", "open");
    await t.click();
    await expect(anyMenu(page), "a second click closes").toHaveCount(0);

    await t.focus();
    await page.keyboard.press("ArrowDown");
    const menu = rootMenu(page);
    await expect(menu).toHaveAttribute("data-state", "open");
    await expect(menu.locator(ENABLED).first()).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(anyMenu(page)).toHaveCount(0);

    await t.focus();
    await page.keyboard.press("ArrowUp");
    await expect(rootMenu(page).locator(ENABLED).last()).toBeFocused();
  });
});

test.describe("menu family differences — ContextMenu", () => {
  const host = HOSTS[1];
  test.beforeEach(async ({ page }) => {
    await goto(page, host);
  });

  test("a left click does not open it; a right click opens it at the pointer point; the trigger is not in the tab order", async ({ page }) => {
    const t = trigger(host, page);
    await expect(t).toHaveAttribute("tabindex", "-1");
    await t.evaluate((el) => {
      (el as HTMLElement).style.position = "fixed";
      // Clear of the docs site's left sidebar, which would intercept the pointer.
      (el as HTMLElement).style.top = "200px";
      (el as HTMLElement).style.left = "640px";
    });
    await t.click();
    await page.waitForTimeout(300);
    await expect(anyMenu(page), "a primary click is not a context click").toHaveCount(0);

    const rest = await measureAtRest(t);
    const x = rest.x + rest.width / 2;
    const y = rest.y + rest.height / 2;
    await page.mouse.click(x, y, { button: "right" });
    const menu = rootMenu(page);
    await expect(menu).toHaveAttribute("data-state", "open");
    await awaitAnimationsSettled(menu);
    const box = await measureAtRest(menu);
    expect(Math.abs(box.x - x), "the menu's leading edge is at the pointer").toBeLessThan(3);
    expect(Math.abs(box.y - y), "the menu's top edge is at the pointer").toBeLessThan(3);
  });
});

test.describe("menu family differences — Menubar", () => {
  const host = HOSTS[2];
  test.beforeEach(async ({ page }) => {
    await goto(page, host);
  });

  // The trigger row: each `MenubarMenu` wrapper (`role="menu"`, no popover) holds its trigger and, while open, its content.
  const triggers = (page: Page) => demoFrame(page).locator('[role="menubar"] > [role="menu"] > [role="menuitem"]');

  test("ArrowRight on an open menu moves to the neighbouring menu and opens it", async ({ page }) => {
    await open(host, page);
    await expect(rootMenu(page)).toHaveAttribute("data-state", "open");
    await page.keyboard.press("ArrowRight");
    const next = triggers(page).nth(1);
    await expect(next, "focus is on the next trigger").toBeFocused();
    await expect(anyMenu(page), "one menu is open, the neighbour's").toHaveCount(1);
    await expect(next.locator('xpath=following-sibling::*[@role="menu"]')).toHaveAttribute("data-state", "open");
  });

  test("Home and End on an open menu move to the first and last menu of the bar and open it", async ({ page }) => {
    await open(host, page);
    await page.keyboard.press("End");
    const last = triggers(page).last();
    await expect(last, "End moves to the last menu's trigger").toBeFocused();
    await expect(last.locator('xpath=following-sibling::*[@role="menu"]')).toHaveAttribute("data-state", "open");
    await page.keyboard.press("Home");
    await expect(triggers(page).first(), "Home moves back to the first menu's trigger").toBeFocused();
    await expect(anyMenu(page)).toHaveCount(1);
  });

  test("hovering another trigger while one menu is open switches menus; the first menu never opens from a hover", async ({ page }) => {
    const second = triggers(page).nth(1);
    await second.hover();
    await page.waitForTimeout(400);
    await expect(anyMenu(page), "a hover alone opens nothing").toHaveCount(0);
    await open(host, page);
    await second.hover();
    await expect(second, "hover moves the roving focus to the hovered trigger").toBeFocused();
    await expect(anyMenu(page)).toHaveCount(1);
    await expect(second.locator('xpath=following-sibling::*[@role="menu"]')).toHaveAttribute("data-state", "open");
  });
});
