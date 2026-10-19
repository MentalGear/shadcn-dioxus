import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * Menu item parts -- the class guard for backlog row 155 group 2.
 *
 * `Separator`, `Group`, `Shortcut`, `variant="destructive"` and `inset` are built once in
 * `primitives/src/menu_item.rs` and wired into DropdownMenu, ContextMenu and Menubar, so one spec file holds all
 * three hosts to the same contract. Each host has one scene per part (the demos mirror shadcn's
 * `*-shortcuts` / `*-groups` / `*-destructive` / `menubar-demo` examples):
 *
 *   - structure: `role="separator"` (horizontal) and `role="group"` render, the separator is not a roving-focus
 *     target (ArrowDown walks past it), a destructive item is still a plain `menuitem`;
 *   - geometry (layout offsets, so the pop-in `scale` animation cannot skew them): the separator is 1px tall and
 *     spans the panel edge to edge (undoing the surface's padding); the shortcut sits at the inline end of its row;
 *   - destructive colour, in light and dark: the item text and icon read `--dx-destructive-text` (never the
 *     fill-grade `--dx-destructive`, scripts/check-destructive-text.sh), and stay >= 4.5:1 against the
 *     destructive-tinted highlight they sit on, both keyboard-focused and hovered;
 *   - inset: an `inset` item or label reserves the leading slot (Menubar's real demo; Dropdown and Context menus
 *     through the attribute, since shadcn's own examples only use `inset` in the menubar).
 *
 * Functional, not timing: debug SSG builds are fine.
 */

const AA = 4.5;
type Mode = "light" | "dark";
type RGB = [number, number, number];

const block = (name: string, variant: string) => `${BASE_URL}/component/block/?name=${name}&variant=${variant}&`;

type Host = {
  label: string;
  name: string;
  /** The scene with separators, groups and shortcuts. */
  parts: string;
  /** The scene with a destructive item. */
  destructive: string;
  open: (page: Page, scene: "parts" | "destructive") => Promise<void>;
  /** The item labels in roving-focus order for the `parts` scene (disabled items already left out). */
  order: string[];
  /** A destructive item's accessible name in the `destructive` scene. */
  destructiveName: string;
  /** An item with a shortcut hint in the `parts` scene. */
  shortcutItem: { name: string; shortcut: string };
  /** The panel root. */
  menu: (page: Page) => Locator;
};

const HOSTS: Host[] = [
  {
    label: "DropdownMenu",
    name: "dropdown_menu",
    parts: "shortcuts",
    destructive: "destructive",
    open: async (page, scene) => {
      await gotoHydrated(page, block("dropdown_menu", scene === "parts" ? "shortcuts" : "destructive"));
      await page.getByRole("button", { name: scene === "parts" ? "Shortcuts menu" : "Danger zone" }).click();
      await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
    },
    order: ["Profile", "Billing", "Settings", "Log out"],
    destructiveName: "Delete project",
    shortcutItem: { name: "Billing", shortcut: "⌘B" },
    menu: (page) => page.getByRole("menu"),
  },
  {
    label: "ContextMenu",
    name: "context_menu",
    parts: "groups",
    destructive: "destructive",
    open: async (page, scene) => {
      await gotoHydrated(page, block("context_menu", scene === "parts" ? "groups" : "destructive"));
      await page
        .getByRole("button", { name: scene === "parts" ? "Right click for groups" : "Right click for destructive" })
        .click({ button: "right" });
      await expect(page.getByRole("menu")).toHaveAttribute("data-state", "open");
      // The pointer is left over the new menu, which would hover its first row; park it.
      await page.mouse.move(1, 1);
    },
    order: ["New File", "Open File", "Save File", "Undo", "Redo", "Cut", "Copy", "Paste", "Delete selection"],
    destructiveName: "Delete note",
    shortcutItem: { name: "Open File", shortcut: "⌘O" },
    menu: (page) => page.getByRole("menu"),
  },
  {
    label: "Menubar",
    name: "menubar",
    parts: "shortcuts",
    destructive: "shortcuts",
    open: async (page, scene) => {
      await gotoHydrated(page, block("menubar", "shortcuts"));
      // `destructive` lives in the second menu ("Selection"), `parts` in the first ("Document").
      await page.getByRole("menuitem", { name: scene === "parts" ? "Document" : "Selection" }).click();
      await expect(page.locator(".dx-menubar-content")).toHaveAttribute("data-state", "open");
      await page.mouse.move(1, 1);
    },
    order: ["New Tab", "New Window", "Print"],
    destructiveName: "Delete Selection",
    shortcutItem: { name: "New Window", shortcut: "⌘N" },
    // `.dx-menubar-menu` wrappers are role="menu" too (APG puts it on the item group); the panel is the content.
    menu: (page) => page.locator(".dx-menubar-content"),
  },
];

for (const host of HOSTS) {
  test.describe(`${host.label}: separator, group, shortcut`, () => {
    test("renders a horizontal separator and groups; ArrowDown walks past the separator", async ({ page }) => {
      await host.open(page, "parts");
      const menu = host.menu(page);

      const separators = menu.getByRole("separator");
      expect(await separators.count(), "at least one separator between the groups").toBeGreaterThanOrEqual(1);
      await expect(separators.first()).toHaveAttribute("aria-orientation", "horizontal");
      await expect(separators.first(), "a separator is not a roving-focus target").not.toHaveAttribute("tabindex", /.*/);
      expect(await menu.getByRole("group").count(), "the items sit in groups").toBeGreaterThanOrEqual(1);

      // Roving focus visits only the items, in order, and wraps.
      await page.keyboard.press("ArrowDown");
      for (const name of host.order) {
        await expect(menu.getByRole("menuitem", { name }).first(), `ArrowDown reaches ${name}`).toBeFocused();
        await page.keyboard.press("ArrowDown");
      }
      await expect(menu.getByRole("menuitem", { name: host.order[0] }).first(), "wraps past the last item").toBeFocused();
    });

    test("the separator is a 1px hairline spanning the panel edge to edge", async ({ page }) => {
      await host.open(page, "parts");
      const geometry = await host
        .menu(page)
        .getByRole("separator")
        .first()
        .evaluate((sep) => {
          const panel = sep.closest("[role='menu']") as HTMLElement;
          const style = getComputedStyle(sep);
          return {
            height: (sep as HTMLElement).offsetHeight,
            left: (sep as HTMLElement).offsetLeft,
            width: (sep as HTMLElement).offsetWidth,
            panelInner: panel.clientWidth,
            background: style.backgroundColor,
          };
        });
      expect(geometry.height, "h-px").toBe(1);
      expect(geometry.left, "starts at the panel's padding edge (-mx-1 undoes p-1)").toBe(0);
      expect(geometry.width, "ends at the opposite padding edge").toBe(geometry.panelInner);
      expect(geometry.background, "painted, not transparent").not.toBe("rgba(0, 0, 0, 0)");
    });

    test("a shortcut hint sits at the inline end of its row, muted, and takes the item's ink when highlighted", async ({ page }) => {
      await host.open(page, "parts");
      const menu = host.menu(page);
      const item = menu.getByRole("menuitem", { name: host.shortcutItem.name }).first();
      const shortcut = item.locator("span", { hasText: host.shortcutItem.shortcut });
      await expect(shortcut).toHaveCount(1);

      const read = () =>
        item.evaluate((el, text) => {
          const hint = [...el.querySelectorAll("span")].find((s) => s.textContent === text) as HTMLElement;
          const style = getComputedStyle(el);
          return {
            // Layout offsets of the hint inside the (position: relative) item row.
            endGap: (el as HTMLElement).clientWidth - (hint.offsetLeft + hint.offsetWidth),
            paddingEnd: parseFloat(style.paddingRight),
            hintColor: getComputedStyle(hint).color,
            itemColor: style.color,
          };
        }, host.shortcutItem.shortcut);

      const rest = await read();
      expect(Math.abs(rest.endGap - rest.paddingEnd), "pushed to the inline end (ml-auto), inside the item's padding").toBeLessThanOrEqual(1);
      expect(rest.hintColor, "muted at rest").not.toBe(rest.itemColor);

      // Keyboard-highlight it: the theme paints `:focus-visible`, which a scripted `.focus()` does not match.
      for (let i = 0; i < 12 && !(await item.evaluate((el) => el === document.activeElement)); i++) await page.keyboard.press("ArrowDown");
      await expect(item).toBeFocused();
      const focused = await read();
      expect(focused.hintColor, "highlighted: the hint takes the item's ink").toBe(focused.itemColor);
    });
  });
}

/** Resolves a CSS colour to the rgb triple the browser paints (a probe element, so `var()` and `oklch()` resolve). */
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

type Contrast = { text: number; icon: number | null; bg: RGB; fg: RGB };

/** The destructive item's label and icon against its effective (composited) background, in the current state. */
async function contrastOf(item: Locator): Promise<Contrast> {
  return item.evaluate((el) => {
    const cv = document.createElement("canvas");
    cv.width = cv.height = 1;
    const cx = cv.getContext("2d", { willReadFrequently: true })!;
    const over = (under: RGB, css: string): RGB => {
      cx.clearRect(0, 0, 1, 1);
      cx.fillStyle = `rgb(${under[0]} ${under[1]} ${under[2]})`;
      cx.fillRect(0, 0, 1, 1);
      cx.fillStyle = css;
      cx.fillRect(0, 0, 1, 1);
      const d = cx.getImageData(0, 0, 1, 1).data;
      return [d[0], d[1], d[2]];
    };
    const lum = ([r, g, b]: RGB) => {
      const f = (v: number) => {
        const c = v / 255;
        return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
      };
      return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
    };
    const ratio = (a: RGB, b: RGB) => {
      const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
      return (hi + 0.05) / (lo + 0.05);
    };
    const chain: Element[] = [];
    for (let n: Element | null = el; n; n = n.parentElement) chain.unshift(n);
    let bg: RGB = [255, 255, 255];
    for (const n of chain) bg = over(bg, getComputedStyle(n).backgroundColor);
    const fg = over(bg, getComputedStyle(el).color);
    const shape = el.querySelector("svg path, svg circle, svg line, svg polyline, svg rect");
    let icon: number | null = null;
    if (shape) {
      const cs = getComputedStyle(shape);
      const paint = cs.stroke !== "none" ? cs.stroke : cs.fill;
      icon = ratio(over(bg, paint), bg);
    }
    return { text: ratio(fg, bg), icon, bg, fg };
  });
}

for (const mode of ["light", "dark"] as Mode[]) {
  for (const host of HOSTS) {
    test.describe(`${host.label}: destructive variant (${mode})`, () => {
      test.use({ colorScheme: mode });

      test("is a plain menuitem in the destructive ink; the highlight is a tint that keeps the text >= 4.5:1", async ({ page }) => {
        await page.context().addCookies([{ name: "dx_theme", value: mode, url: BASE_URL }]);
        await page.emulateMedia({ colorScheme: mode, reducedMotion: "reduce" });
        await host.open(page, "destructive");
        await expect(page.locator("html")).toHaveAttribute("data-theme", mode);
        await page.addStyleTag({ content: "*,*::before,*::after{transition:none!important;animation:none!important}" });

        const menu = host.menu(page);
        const item = menu.getByRole("menuitem", { name: host.destructiveName });
        const other = menu.getByRole("menuitem").first();
        await expect(item, "a destructive item keeps role=menuitem").toHaveCount(1);
        await expect(item).toHaveAttribute("data-variant", "destructive");
        await expect(other, "an ordinary item says so").toHaveAttribute("data-variant", "default");

        const destructiveText = await resolveColor(page, "var(--dx-destructive-text)");
        const foreground = await resolveColor(page, "var(--dx-foreground)");
        expect(await item.evaluate((el) => getComputedStyle(el).color), "text is the clamped --dx-destructive-text").toBe(destructiveText);
        expect(destructiveText, "and not the ordinary ink").not.toBe(foreground);
        const iconColor = await item.evaluate((el) => {
          const svg = el.querySelector("svg");
          return svg ? getComputedStyle(svg).color : null;
        });
        if (iconColor !== null) expect(iconColor, "the icon wears the destructive ink").toBe(destructiveText);

        const restBg = (await contrastOf(item)).bg;

        // Keyboard highlight.
        await page.keyboard.press("ArrowDown");
        const stops = (await menu.getByRole("menuitem").count()) - 1;
        for (let i = 0; i < stops; i++) await page.keyboard.press("ArrowDown");
        await expect(item, "ArrowDown reaches the destructive item").toBeFocused();
        const focused = await contrastOf(item);
        console.log(`[menu-parts] ${mode} ${host.label} focused: bg rgb(${focused.bg}) rest rgb(${restBg}) text ${focused.text.toFixed(2)} icon ${focused.icon?.toFixed(2)}`);
        expect(focused.bg, "the highlight is a tint, not the surface").not.toEqual(restBg);
        expect(focused.text, "focused label contrast").toBeGreaterThanOrEqual(AA);
        if (focused.icon !== null) expect(focused.icon, "focused icon contrast").toBeGreaterThanOrEqual(3);

        // The ordinary item keeps the accent highlight, not the destructive tint.
        const first = menu.getByRole("menuitem").first();
        await page.keyboard.press("ArrowDown"); // the destructive item is last: wraps to the first
        await expect(first).toBeFocused();
        const accent = await resolveColor(page, "var(--dx-accent)");
        expect((await contrastOf(first)).bg.map(String).join(","), "an ordinary highlight is the accent fill").toBe(
          accent.match(/\d+/g)!.slice(0, 3).join(","),
        );

        // Pointer highlight.
        await page.mouse.move(1, 1);
        await item.hover();
        const hovered = await contrastOf(item);
        console.log(`[menu-parts] ${mode} ${host.label} hovered: bg rgb(${hovered.bg}) text ${hovered.text.toFixed(2)} icon ${hovered.icon?.toFixed(2)}`);
        expect(hovered.bg, "hover paints the tint too").not.toEqual(restBg);
        expect(hovered.text, "hovered label contrast").toBeGreaterThanOrEqual(AA);
      });
    });
  }
}

test.describe("inset reserves the leading slot", () => {
  test("Menubar: an inset item and the Selection menu's labels line up with the checkable slot", async ({ page }) => {
    await gotoHydrated(page, block("menubar", "shortcuts"));
    await page.getByRole("menuitem", { name: "Selection" }).click();
    const menu = page.locator(".dx-menubar-content");
    const inset = menu.getByRole("menuitem", { name: "Select All" });
    const plain = menu.getByRole("menuitem", { name: "Delete Selection" });
    await expect(inset).toHaveAttribute("data-inset", "true");
    await expect(plain, "an item that did not ask for it carries no data-inset").not.toHaveAttribute("data-inset", /.*/);

    const pad = (l: Locator) => l.evaluate((el) => parseFloat(getComputedStyle(el).paddingLeft));
    const slot = await page.evaluate(() => {
      const probe = document.createElement("div");
      probe.style.paddingLeft = "calc(var(--dx-menu-item-indicator-inset) + var(--dx-menu-item-indicator-size) + var(--dx-menu-item-indicator-gap))";
      document.body.appendChild(probe);
      const v = parseFloat(getComputedStyle(probe).paddingLeft);
      probe.remove();
      return v;
    });
    expect(await pad(plain), "a plain item has the ordinary 6px padding").toBeLessThan(slot);
    // The panel sets the menubar's 6px indicator inset, so measure the slot from the open panel, not <body>.
    const panelSlot = await menu.evaluate((panel) => {
      const probe = document.createElement("div");
      probe.style.paddingLeft = "calc(var(--dx-menu-item-indicator-inset) + var(--dx-menu-item-indicator-size) + var(--dx-menu-item-indicator-gap))";
      panel.appendChild(probe);
      const v = parseFloat(getComputedStyle(probe).paddingLeft);
      probe.remove();
      return v;
    });
    expect(slot, "sanity: the slot is positive").toBeGreaterThan(0);
    expect(await pad(inset), "inset = the leading indicator slot").toBeCloseTo(panelSlot, 0);
  });

  for (const host of HOSTS.filter((h) => h.label !== "Menubar")) {
    test(`${host.label}: data-inset="true" on an item or label moves its text to the 2rem inset`, async ({ page }) => {
      await host.open(page, "parts");
      const menu = host.menu(page);
      const item = menu.getByRole("menuitem").first();
      const label = menu.locator("[data-dx-menu-label]").first();
      const before = await item.evaluate((el) => parseFloat(getComputedStyle(el).paddingLeft));
      const labelBefore = await label.evaluate((el) => parseFloat(getComputedStyle(el).paddingLeft));
      // The Rust side of `inset` is the primitive's `data-inset` (cargo test: menu_item::tests) and the Menubar demo;
      // shadcn's Dropdown and Context examples do not use it, so this drives the same attribute by hand.
      await item.evaluate((el) => el.setAttribute("data-inset", "true"));
      await label.evaluate((el) => el.setAttribute("data-inset", "true"));
      const rem = await page.evaluate(() => parseFloat(getComputedStyle(document.documentElement).fontSize));
      expect(before, "ordinary padding is below the inset").toBeLessThan(2 * rem);
      expect(await item.evaluate((el) => parseFloat(getComputedStyle(el).paddingLeft)), "item: pl-8").toBeCloseTo(2 * rem, 0);
      expect(labelBefore).toBeLessThan(2 * rem);
      expect(await label.evaluate((el) => parseFloat(getComputedStyle(el).paddingLeft)), "label: pl-8").toBeCloseTo(2 * rem, 0);
    });
  }
});
