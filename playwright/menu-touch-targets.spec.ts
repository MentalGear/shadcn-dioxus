import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";

/**
 * Backlog row 291: menus and option lists have a touch size step, set ONCE in the theme.
 *
 * Nova's menu row is ~28px, too small for a finger. `dx-components-theme.css` declares four slots
 * (`--dx-menu-item-min-height`, `--dx-menu-item-font-size`, `--dx-menu-trigger-min-height`,
 * `--dx-menu-trigger-padding`) and replaces them under `(pointer: coarse)` (row 349 moved them behind the
 * `data-dx-coarse` switch, `coarse-sizing.spec.ts`); every menu / listbox / palette sheet reads the slots, so none carries a `@media` copy.
 *
 * Real touch emulation (`hasTouch` + `isMobile`) matches that query in Chromium; a plain desktop context does
 * not, and must keep today's dense rows.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8106 npx playwright test --config=ssg.local.config.ts menu-touch-targets.spec.ts
 */

const block = (name: string, variant: string) => `${BASE_URL}/component/block/?name=${name}&variant=${variant}&`;
const TARGET = 44;

type Row = { label: string; open: (page: Page) => Promise<void>; rows: (page: Page) => Locator };

const HOSTS: Row[] = [
  {
    label: "DropdownMenu items",
    open: async (page) => {
      await gotoHydrated(page, block("dropdown_menu", "shortcuts"));
      await page.getByRole("button", { name: "Shortcuts menu" }).click();
      await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
    },
    rows: (page) => page.getByRole("menu").getByRole("menuitem"),
  },
  {
    label: "ContextMenu items",
    open: async (page) => {
      await gotoHydrated(page, block("context_menu", "groups"));
      await page.getByRole("button", { name: "Right click for groups" }).click({ button: "right" });
      await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
    },
    rows: (page) => page.getByRole("menu").getByRole("menuitem"),
  },
  {
    label: "Menubar items",
    open: async (page) => {
      await gotoHydrated(page, block("menubar", "shortcuts"));
      await page.getByRole("menuitem", { name: "Document" }).click();
      await expect(page.locator(".dx-menubar-content")).toHaveAttribute("data-state", "open");
    },
    rows: (page) => page.locator(".dx-menubar-content").getByRole("menuitem"),
  },
  {
    label: "Menubar triggers",
    open: async (page) => {
      await gotoHydrated(page, block("menubar", "shortcuts"));
      await expect(page.locator(".dx-menubar-trigger").first()).toBeVisible();
    },
    rows: (page) => page.locator(".dx-menubar-trigger"),
  },
  {
    label: "Command items",
    open: async (page) => {
      await gotoHydrated(page, block("command", "inline"));
      await expect(page.locator(".dx-command-item").first()).toBeVisible();
    },
    rows: (page) => page.locator(".dx-command-item"),
  },
  {
    label: "Select options",
    open: async (page) => {
      await gotoHydrated(page, block("select", "main"));
      await page.getByRole("combobox").first().click();
      await expect(page.getByRole("listbox")).toHaveAttribute("data-state", "open");
    },
    rows: (page) => page.getByRole("listbox").getByRole("option"),
  },
  {
    label: "Combobox options",
    open: async (page) => {
      await gotoHydrated(page, block("combobox", "main"));
      await page.getByRole("combobox", { name: "Select framework" }).click();
      await expect(page.locator("[role='listbox'][data-state='open']")).toBeVisible();
    },
    rows: (page) => page.locator("[role='listbox'][data-state='open']").getByRole("option"),
  },
  {
    label: "NavigationMenu triggers",
    open: async (page) => {
      await gotoHydrated(page, block("navigation_menu", "main"));
      await expect(page.locator(".dx-navigation-menu-trigger").first()).toBeVisible();
    },
    rows: (page) => page.locator(".dx-navigation-menu-trigger"),
  },
];

async function heights(rows: Locator): Promise<number[]> {
  const n = await rows.count();
  expect(n, "the host renders rows").toBeGreaterThan(1);
  const out: number[] = [];
  for (let i = 0; i < n; i++) out.push((await measureAtRest(rows.nth(i))).height);
  return out;
}

test.describe("touch (pointer: coarse)", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, colorScheme: "light" });
  test.skip(({ browserName }) => browserName !== "chromium", "mobile emulation is Chromium-only");

  test("the emulated device matches the theme's coarse query", async ({ page }) => {
    await gotoHydrated(page, block("menubar", "shortcuts"));
    expect(await page.evaluate(() => matchMedia("(pointer: coarse)").matches)).toBe(true);
  });

  for (const h of HOSTS) {
    test(`${h.label} are at least ${TARGET}px tall`, async ({ page }) => {
      await h.open(page);
      for (const height of await heights(h.rows(page))) expect(height).toBeGreaterThanOrEqual(TARGET - 0.5);
    });
  }

  for (const h of HOSTS.filter((x) => /^(DropdownMenu|Select|Combobox)/.test(x.label))) {
    test(`${h.label}: the text steps up to the base size`, async ({ page }) => {
      await h.open(page);
      const size = await h.rows(page).first().evaluate((el) => getComputedStyle(el).fontSize);
      expect(size).toBe("16px");
    });
  }
});

test.describe("fine pointer (desktop)", () => {
  test.use({ viewport: { width: 1280, height: 800 }, colorScheme: "light" });

  test("the coarse query does not match, and the rows keep Nova's dense size", async ({ page }) => {
    await HOSTS[0].open(page);
    expect(await page.evaluate(() => matchMedia("(pointer: coarse)").matches)).toBe(false);
    for (const height of await heights(HOSTS[0].rows(page))) {
      expect(height).toBeGreaterThan(24);
      expect(height).toBeLessThan(34);
    }
  });

  for (const h of HOSTS.filter((x) => /^(Select|Combobox) options$/.test(x.label))) {
    test(`${h.label} keep Nova's dense size and 14px text`, async ({ page }) => {
      await h.open(page);
      for (const height of await heights(h.rows(page))) {
        expect(height).toBeGreaterThan(24);
        expect(height).toBeLessThan(34);
      }
      expect(await h.rows(page).first().evaluate((el) => getComputedStyle(el).fontSize)).toBe("14px");
    });
  }
});

test.describe("one token set, no per-sheet copies", () => {
  test.use({ viewport: { width: 1280, height: 800 }, colorScheme: "light" });

  test("retuning the slot moves every sheet that reads it", async ({ page }) => {
    await HOSTS[0].open(page);
    await page.evaluate(() => document.documentElement.style.setProperty("--dx-menu-item-min-height", "60px"));
    for (const height of await heights(HOSTS[0].rows(page))) expect(height).toBeGreaterThanOrEqual(59.5);
    // Command, Select and Combobox read the same slot.
    for (const h of HOSTS.filter((x) => /^(Command items|Select options|Combobox options)$/.test(x.label))) {
      await h.open(page);
      await page.evaluate(() => document.documentElement.style.setProperty("--dx-menu-item-min-height", "60px"));
      for (const height of await heights(h.rows(page))) expect(height, h.label).toBeGreaterThanOrEqual(59.5);
    }
  });
});
