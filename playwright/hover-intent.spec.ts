import { test, expect } from './fixtures';
import type { Locator, Page } from '@playwright/test';
import { BASE_URL } from './base-url';
import { gotoHydrated } from './hydration';

// A menu opened by pointer hover is a pointer *intent*: when the pointer is no longer over the
// trigger or its panel, the menu closes after a short grace delay (Radix NavigationMenu:
// `onPointerLeave` of the trigger and of the content both start a 150 ms close timer, and
// `onPointerEnter` of either cancels it -- radix-ui/primitives
// packages/react/navigation-menu/src/navigation-menu.tsx). Moving from the trigger into the
// panel, across the gap between them, keeps it open; moving to a sibling trigger switches.
//
// Every hover-opening menu primitive of this repo that has the pattern (the navbar and the
// navigation menu share `primitives/src/hover_intent.rs`) is held to the same four rules below,
// so the class cannot regress for one of them alone. The user report that added this: "the
// nav-bar's hover menu keeps its menu open even though the mouse doesn't hover it anymore".

const GOTO = { timeout: 20 * 60 * 1000 };
/** Longer than the 300 ms close grace plus the exit animation, short enough to prove "promptly". */
const CLOSE_WITHIN_MS = 1500;

type Menu = {
  name: string;
  url: string;
  /** The trigger button the pointer rests on. */
  trigger: (page: Page) => Locator;
  /** The element that carries the open/closed state this menu exposes. */
  isOpen: (page: Page) => Locator;
  /** A link inside the opened panel. */
  panelLink: (page: Page) => Locator;
};

const MENUS: Menu[] = [
  {
    name: 'navbar',
    url: `${BASE_URL}/component/?name=navbar&`,
    trigger: (page) => page.getByRole('menuitem', { name: 'Inputs', exact: true }),
    isOpen: (page) =>
      page
        .getByRole('menu')
        .filter({ has: page.getByRole('menuitem', { name: 'Inputs', exact: true }) })
        .first(),
    panelLink: (page) => page.getByRole('menuitem', { name: 'Calendar', exact: true }),
  },
  {
    name: 'navigation_menu',
    url: `${BASE_URL}/component/?name=navigation_menu&`,
    trigger: (page) =>
      page
        .getByRole('navigation', { name: 'Component navigation menu' })
        .getByRole('button', { name: 'Getting started' }),
    isOpen: (page) =>
      page
        .getByRole('navigation', { name: 'Component navigation menu' })
        .getByRole('button', { name: 'Getting started' }),
    panelLink: (page) =>
      page
        .getByRole('navigation', { name: 'Component navigation menu' })
        .getByRole('link', { name: 'Component Library' }),
  },
];

/** Whether the menu reads as open: `data-state` on a nav wrapper, `aria-expanded` on a trigger. */
async function expectOpen(menu: Locator, open: boolean, timeout?: number) {
  const isButton = (await menu.evaluate((el) => el.tagName)) === 'BUTTON';
  if (isButton) {
    await expect(menu).toHaveAttribute('aria-expanded', open ? 'true' : 'false', { timeout });
  } else {
    await expect(menu).toHaveAttribute('data-state', open ? 'open' : 'closed', { timeout });
  }
}

for (const menu of MENUS) {
  test.describe(`${menu.name}: a hover-opened menu is closed by the pointer leaving (pointer intent)`, () => {
    test.beforeEach(async ({ page }) => {
      await gotoHydrated(page, menu.url, GOTO);
      // Styles + wasm are up once the trigger is actually visible.
      await expect(menu.trigger(page)).toBeVisible({ timeout: 60_000 });
      await expectOpen(menu.isOpen(page), false);
    });

    test('hovering the trigger opens it; moving the pointer far away closes it within ~1s', async ({ page }) => {
      await menu.trigger(page).hover();
      await expectOpen(menu.isOpen(page), true);
      await expect(menu.panelLink(page)).toBeVisible();

      // Far from both the trigger and the panel: the page's own h1 is always present.
      await page.getByRole('heading', { level: 1 }).first().hover();
      await expectOpen(menu.isOpen(page), false, CLOSE_WITHIN_MS);
    });

    test('moving from the trigger into the panel keeps it open (even across the gap)', async ({ page }) => {
      await menu.trigger(page).hover();
      await expectOpen(menu.isOpen(page), true);
      const link = menu.panelLink(page);
      await expect(link).toBeVisible();

      // A real multi-step move: the pointer crosses the gap between the trigger and the panel
      // (outside both for a few frames) and must not close it.
      const from = (await menu.trigger(page).boundingBox())!;
      const to = (await link.boundingBox())!;
      await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
      await page.mouse.move(to.x + to.width / 2, to.y + to.height / 2, { steps: 25 });
      // Rest on the panel for longer than the close grace: a close timer left running from
      // the trip across the gap would have fired by now.
      await page.waitForTimeout(800);
      await expectOpen(menu.isOpen(page), true);
      await expect(link).toBeVisible();
    });

    test('leaving the panel closes it', async ({ page }) => {
      await menu.trigger(page).hover();
      await expectOpen(menu.isOpen(page), true);
      const link = menu.panelLink(page);
      await expect(link).toBeVisible();
      await link.hover();
      await page.waitForTimeout(500);
      await expectOpen(menu.isOpen(page), true);

      await page.getByRole('heading', { level: 1 }).first().hover();
      await expectOpen(menu.isOpen(page), false, CLOSE_WITHIN_MS);
    });

    test('a menu opened by keyboard is closed by a pointer that enters and then leaves it', async ({ page }) => {
      // Radix closes on pointer leave whatever opened the panel; the pointer was never over the
      // menu here, so first entering and then leaving it must not strand it open.
      await menu.trigger(page).focus();
      await page.keyboard.press('ArrowDown');
      await expectOpen(menu.isOpen(page), true);
      await menu.panelLink(page).hover();
      await page.getByRole('heading', { level: 1 }).first().hover();
      await expectOpen(menu.isOpen(page), false, CLOSE_WITHIN_MS);
    });
  });
}
