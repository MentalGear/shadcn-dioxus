import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * Menu item parts, hoisted (backlog row 155 follow-up g2).
 *
 * The separator, shortcut, destructive and inset CSS used to be repeated in the Dropdown, Context and Menubar
 * sheets; it is now one `data-dx-menu-*` section of the theme. This spec proves the hoist is equivalent: the
 * computed style of a menu item, its separator, its shortcut and a destructive item is read from each of the
 * three menus and (a) has the same absolute values the three old sheets declared, (b) is identical across the
 * three hosts, so a part can no longer drift between them. It also checks the opt-in attributes the wrappers write.
 */

const block = (name: string, variant: string) => `${BASE_URL}/component/block/?name=${name}&variant=${variant}&`;

type Host = {
  label: string;
  open: (page: Page, scene: "parts" | "destructive") => Promise<void>;
  menu: (page: Page) => Locator;
  plainItem: string;
  shortcutItem: { name: string; shortcut: string };
  destructiveName: string;
};

const HOSTS: Host[] = [
  {
    label: "DropdownMenu",
    open: async (page, scene) => {
      await gotoHydrated(page, block("dropdown_menu", scene === "parts" ? "shortcuts" : "destructive"));
      await page.getByRole("button", { name: scene === "parts" ? "Shortcuts menu" : "Danger zone" }).click();
      await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
      await page.mouse.move(1, 1);
    },
    menu: (page) => page.getByRole("menu"),
    plainItem: "Profile",
    shortcutItem: { name: "Billing", shortcut: "⌘B" },
    destructiveName: "Delete project",
  },
  {
    label: "ContextMenu",
    open: async (page, scene) => {
      await gotoHydrated(page, block("context_menu", scene === "parts" ? "groups" : "destructive"));
      await page
        .getByRole("button", { name: scene === "parts" ? "Right click for groups" : "Right click for destructive" })
        .click({ button: "right" });
      await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
      await page.mouse.move(1, 1);
    },
    menu: (page) => page.getByRole("menu"),
    plainItem: "New File",
    shortcutItem: { name: "Open File", shortcut: "⌘O" },
    destructiveName: "Delete note",
  },
  {
    label: "Menubar",
    open: async (page, scene) => {
      await gotoHydrated(page, block("menubar", "shortcuts"));
      await page.getByRole("menuitem", { name: scene === "parts" ? "Document" : "Selection" }).click();
      await expect(page.locator(".dx-menubar-content")).toHaveAttribute("data-state", "open");
      await page.mouse.move(1, 1);
    },
    menu: (page) => page.locator(".dx-menubar-content"),
    plainItem: "New Tab",
    shortcutItem: { name: "New Window", shortcut: "⌘N" },
    destructiveName: "Delete Selection",
  },
];

async function resolveColor(page: Page, css: string): Promise<string> {
  return page.evaluate((value) => {
    const probe = document.createElement("div");
    probe.style.color = value;
    document.body.appendChild(probe);
    const resolved = getComputedStyle(probe).color;
    probe.remove();
    return resolved;
  }, css);
}

const pick = (el: Element, props: string[]) => {
  const s = getComputedStyle(el);
  const out: Record<string, string> = {};
  for (const p of props) out[p] = s.getPropertyValue(p);
  return out;
};

const ITEM_PROPS = ["display", "position", "padding-top", "padding-bottom", "border-top-left-radius", "font-size", "cursor", "column-gap", "overflow-wrap", "outline-style", "color"];
const SEPARATOR_PROPS = ["height", "margin-top", "margin-bottom", "background-color"];
const SHORTCUT_PROPS = ["font-size", "letter-spacing", "color"];

async function readParts(page: Page, host: Host) {
  await host.open(page, "parts");
  const menu = host.menu(page);
  const item = await menu.getByRole("menuitem", { name: host.plainItem }).first().evaluate(pick, ITEM_PROPS);
  const sep = await menu.getByRole("separator").first().evaluate(pick, SEPARATOR_PROPS);
  const shortcut = await menu.locator("span", { hasText: host.shortcutItem.shortcut }).evaluate(pick, SHORTCUT_PROPS);
  return { item, sep, shortcut };
}

for (const host of HOSTS) {
  test.describe(`${host.label}: hoisted menu item parts`, () => {
    test("the wrappers opt into the theme's data-dx-menu-* parts", async ({ page }) => {
      await host.open(page, "parts");
      const menu = host.menu(page);
      await expect(menu.getByRole("separator").first()).toHaveAttribute("data-dx-menu-separator", "true");
      await expect(menu.getByRole("menuitem", { name: host.plainItem }).first()).toHaveAttribute("data-dx-menu-item", "true");
      const hint = menu.locator("span", { hasText: host.shortcutItem.shortcut });
      await expect(hint).toHaveAttribute("data-dx-menu-shortcut", "true");
    });

    test("item, separator and shortcut computed style match the old per-sheet values", async ({ page }) => {
      await host.open(page, "parts");
      const menu = host.menu(page);
      const muted = await resolveColor(page, "var(--dx-muted-foreground)");
      const border = await resolveColor(page, "var(--dx-border)");
      const foreground = await resolveColor(page, "var(--dx-foreground)");

      const item = await menu.getByRole("menuitem", { name: host.plainItem }).first().evaluate(pick, ITEM_PROPS);
      expect(item["display"]).toBe("flex");
      expect(item["position"]).toBe("relative");
      expect(item["font-size"], "text-sm").toBe("14px");
      // The interactive-element cursor is the theme's one rule now (cursor-option.spec.ts): shadcn v4's arrow by default.
      expect(item["cursor"]).toBe("default");
      expect(item["color"], "the ordinary ink").toBe(foreground);

      const sep = await menu.getByRole("separator").first().evaluate(pick, SEPARATOR_PROPS);
      expect(sep["height"], "h-px").toBe("1px");
      expect(sep["margin-top"], "my-1").toBe("4px");
      expect(sep["margin-bottom"]).toBe("4px");
      expect(sep["background-color"], "border colour").toBe(border);

      const shortcut = await menu
        .locator("span", { hasText: host.shortcutItem.shortcut })
        .evaluate(pick, SHORTCUT_PROPS);
      expect(shortcut["font-size"], "text-xs").toBe("12px");
      expect(shortcut["letter-spacing"], "tracking-widest: 0.1em of 12px").toBe("1.2px");
      expect(shortcut["color"], "muted at rest").toBe(muted);

    });

    test("a destructive item reads the clamped destructive ink; inset moves the leading padding", async ({ page }) => {
      await host.open(page, "destructive");
      const menu = host.menu(page);
      const destructive = await resolveColor(page, "var(--dx-destructive-text)");
      const item = menu.getByRole("menuitem", { name: host.destructiveName });
      expect(await item.evaluate((el) => getComputedStyle(el).color)).toBe(destructive);

      // `data-inset` on a plain item: the theme's 2rem inset, except Menubar's own indicator slot.
      const pad = await menu.getByRole("menuitem").first().evaluate((el) => {
        const before = parseFloat(getComputedStyle(el).paddingLeft);
        el.setAttribute("data-inset", "true");
        const after = parseFloat(getComputedStyle(el).paddingLeft);
        el.removeAttribute("data-inset");
        return { before, after };
      });
      if (host.label === "Menubar") {
        const slot = await menu.evaluate((el) => {
          const probe = document.createElement("div");
          probe.style.paddingLeft = "calc(var(--dx-menu-item-indicator-inset) + var(--dx-menu-item-indicator-size) + var(--dx-menu-item-indicator-gap))";
          el.appendChild(probe);
          const v = parseFloat(getComputedStyle(probe).paddingLeft);
          probe.remove();
          return v;
        });
        expect(pad.after, "Menubar's inset is its indicator slot").toBeCloseTo(slot, 0);
      } else {
        expect(pad.after, "pl-8").toBe(32);
      }
    });
  });
}

test("the three menus paint the same item, separator and shortcut", async ({ page }) => {
  const [first, ...rest] = HOSTS;
  const reference = await readParts(page, first);
  for (const host of rest) expect(await readParts(page, host), `${host.label} equals ${first.label}`).toEqual(reference);
});
