import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * Contrast and distinctness of the HIGHLIGHTED, HOVERED and SELECTED rows on the surfaces of the dark ladder
 * (backlog row 111, phase C, option C: page #0a0a0a, card and popover #171717, accent / muted / secondary
 * #262626). axe reads the settled state, so it cannot see a hovered menu row, a keyboard-highlighted option or
 * a pressed page number; this spec measures them from COMPUTED colours, in light and dark.
 *
 * Option C ties dark muted, secondary and accent. An element whose rest fill is `--dx-muted` and whose hover or
 * selected fill is `--dx-accent` (or a popover row highlighted with `--dx-accent` on `--dx-popover`) must still
 * move, and the failure mode is silent: the state paints the same colour as its surroundings (1.00:1). For each
 * probe below the spec asserts, in both modes:
 *
 *   1. every text node inside the probed element (item label, shortcut, muted description) against the
 *      element's own effective background >= 4.5:1 (WCAG AA, normal text), in the probed state. An ICON-ONLY
 *      probe (`icon: true`, e.g. the calendar's prev/next buttons) has no text, so its svg glyphs are measured
 *      instead: the painted colour (computed `stroke`, else `fill`, else `color`) against the same background
 *      >= 3:1 (WCAG 1.4.11 non-text contrast), and the "no text" complaint is not raised;
 *   2. the probed state's background differs from its SURFACE (the panel or card it sits on) by at least
 *      DISTINCT_HOVER per channel, the same step `theme-preset-contrast.spec.ts` demands of a hover;
 *   3. where the probe has a rest state, the probed state's background differs from its own REST background
 *      by at least DISTINCT_HOVER (this is the muted-rest / accent-hover tie).
 *
 * Method, as in `theme-preset-contrast.spec.ts`: transitions and animations are switched off so
 * `getComputedStyle` reads the settled value, and each colour is composited layer by layer from <html> down to
 * the element through a 1x1 canvas (a `color-mix(..., transparent)` fill is translucent).
 *
 * Run against a dev server or an SSG build (functional, not timing: debug builds are fine):
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8083 npx playwright test --config=baseline.local.config.ts ./dark-surface-contrast.spec.ts
 */

const AA = 4.5;
/** WCAG 1.4.11 non-text contrast: the floor for a glyph that is the only content of a control. */
const NON_TEXT = 3;
/** Smallest per-channel difference (0-255) that counts as "a different background"; same step as the preset sweep. */
const DISTINCT_HOVER = 6;
const NAV_TIMEOUT = 20 * 60 * 1000;

type Mode = "light" | "dark";
type RGB = [number, number, number];

type Probe = {
  /** Label used in failure messages. */
  name: string;
  /** The element measured, in the state `enter` leaves it in. */
  target: (page: Page) => Locator;
  /** The surface the target sits on (a menu panel, a card, a popup); its effective background is the comparison. */
  surface: (page: Page) => Locator;
  /** Moves the page into the probed state (hover the row, press ArrowDown, ...). */
  enter: (page: Page, target: Locator) => Promise<void>;
  /** Puts the target back at rest WITHOUT closing the surface; omitted when the target has no rest state to compare. */
  rest?: (page: Page, target: Locator) => Promise<void>;
  /**
   * Modes in which the target must differ from its SURFACE (check 2). Default: both. A state that is Nova's own
   * faint light-mode step (an outline page number on a white page, `muted/50` on white) names its dark-only
   * reason here; phase C moves no light colour, so light has nothing new to guard.
   */
  distinctFromSurfaceIn?: Mode[];
  /**
   * The target's content is an icon (an svg), not text: its glyphs are checked at NON_TEXT (3:1) against the
   * target's background, and having no text node is expected. Default false (text at AA, and text is required).
   */
  icon?: boolean;
};

type Scene = {
  /** Opens the surface under test and returns once it is visible. */
  open: (page: Page) => Promise<void>;
  probes: Probe[];
};

type Measure = {
  bg: RGB;
  surface: RGB;
  /** Every text node's colour against `bg`, with a label. */
  texts: { label: string; ratio: number; fg: RGB }[];
  /** Every svg glyph's painted colour against `bg`, with a label. */
  icons: { label: string; ratio: number; fg: RGB }[];
};

const rgbText = (c: RGB) => `rgb(${c.join(",")})`;
const maxChannelDiff = (a: RGB, b: RGB) => Math.max(...a.map((v, i) => Math.abs(v - b[i])));

/** Reads the target's background (composited), its surface's, and every text node's contrast, in one evaluate. */
async function measure(target: Locator, surface: Locator): Promise<Measure> {
  const surfaceHandle = await surface.first().elementHandle();
  if (!surfaceHandle) throw new Error("surface matched nothing");
  return target.first().evaluate((el, surfaceEl) => {
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
    // Effective background: composite every ancestor's background-color from <html> down to `node`.
    const effective = (node: Element): RGB => {
      const chain: Element[] = [];
      for (let n: Element | null = node; n; n = n.parentElement) chain.unshift(n);
      let bg: RGB = [255, 255, 255];
      for (const n of chain) bg = over(bg, getComputedStyle(n).backgroundColor);
      return bg;
    };
    const bg = effective(el);
    const texts: { label: string; ratio: number; fg: RGB }[] = [];
    for (const node of [el, ...el.querySelectorAll("*")]) {
      if (node instanceof SVGElement) continue;
      const own = [...node.childNodes]
        .filter((c) => c.nodeType === Node.TEXT_NODE)
        .map((c) => c.textContent?.trim() ?? "")
        .join(" ")
        .trim();
      if (!own) continue;
      const fg = over(bg, getComputedStyle(node).color);
      texts.push({ label: `"${own.slice(0, 24)}" <${node.tagName.toLowerCase()}.${[...node.classList][0] ?? ""}>`, ratio: ratio(fg, bg), fg });
    }
    // Icons: the painted colour of an svg's shapes (stroke icons paint with `stroke`, filled ones with `fill`).
    const icons: { label: string; ratio: number; fg: RGB }[] = [];
    for (const svg of el.querySelectorAll("svg")) {
      const shapes = [...svg.querySelectorAll("path, circle, rect, line, polyline, polygon, ellipse")];
      for (const shape of shapes.length ? shapes : [svg]) {
        const cs = getComputedStyle(shape);
        const paint = cs.stroke !== "none" && cs.strokeWidth !== "0px" ? cs.stroke : cs.fill !== "none" ? cs.fill : cs.color;
        const fg = over(bg, paint);
        icons.push({ label: `svg <${shape.tagName.toLowerCase()}> in <${el.tagName.toLowerCase()}.${[...el.classList][0] ?? ""}>`, ratio: ratio(fg, bg), fg });
      }
    }
    return { bg, surface: effective(surfaceEl as Element), texts, icons };
  }, surfaceHandle);
}

const hover = (page: Page, target: Locator) => target.hover().then(() => page.waitForTimeout(50));
const away = (page: Page) => page.mouse.move(1, 1).then(() => page.waitForTimeout(50));
const gotoComponent = (page: Page, name: string) => gotoHydrated(page, `${BASE_URL}/component/?name=${name}&`, { timeout: NAV_TIMEOUT });

const preview = (page: Page) => page.locator("#component-preview-frame").first();

const SCENES: Record<string, Scene> = {
  "Dropdown menu": {
    open: async (page) => {
      await gotoComponent(page, "dropdown_menu");
      await page.getByRole("button", { name: "Open Menu" }).click();
      await expect(page.getByRole("menu")).toBeVisible();
    },
    probes: [
      {
        name: "dropdown item hovered",
        target: (page) => page.getByRole("menuitem", { name: "Edit" }),
        surface: (page) => page.getByRole("menu"),
        enter: hover,
        rest: async (page) => away(page),
      },
    ],
  },
  Select: {
    open: async (page) => {
      await gotoComponent(page, "select");
      await page.getByRole("combobox").filter({ hasText: /Select an option|Apple|Banana/ }).click();
      await expect(page.getByRole("listbox")).toHaveAttribute("data-state", "open");
    },
    probes: [
      {
        name: "select option hovered",
        target: (page) => page.getByRole("listbox").getByRole("option", { name: "banana" }),
        surface: (page) => page.getByRole("listbox"),
        enter: hover,
        rest: async (page) => away(page),
      },
    ],
  },
  Combobox: {
    open: async (page) => {
      await gotoComponent(page, "combobox");
      const input = page.getByRole("combobox", { name: "Select framework" });
      await input.focus();
      await page.keyboard.press("ArrowDown");
      await expect(page.locator("[role='listbox'][data-state='open']")).toBeVisible();
    },
    probes: [
      {
        name: "combobox option hovered",
        target: (page) => page.locator("[role='listbox'][data-state='open']").getByRole("option", { name: "SvelteKit" }),
        surface: (page) => page.locator("[role='listbox'][data-state='open']"),
        enter: hover,
        rest: async (page) => away(page),
      },
    ],
  },
  Command: {
    open: async (page) => {
      await gotoComponent(page, "command");
      await page.getByRole("button", { name: "Open Command Palette" }).click();
      await expect(page.getByRole("dialog")).toBeVisible();
    },
    probes: [
      {
        name: "command option hovered",
        target: (page) => page.getByRole("listbox", { name: "Commands" }).getByRole("option").nth(2),
        surface: (page) => page.getByRole("dialog"),
        enter: hover,
        rest: async (page) => away(page),
      },
    ],
  },
  "Navigation menu": {
    open: async (page) => {
      await gotoComponent(page, "navigation_menu");
      const trigger = page.getByRole("navigation", { name: "Component navigation menu" }).getByRole("button", { name: "Getting started" });
      await trigger.hover();
      await expect(trigger).toHaveAttribute("aria-expanded", "true");
    },
    probes: [
      {
        // The open trigger sits on the bar, not on the popover: it must read as open against it.
        name: "navigation trigger while its panel is open",
        distinctFromSurfaceIn: ["dark"], // light: Nova's `data-open:bg-muted/50` on white is a 5/255 step
        target: (page) =>
          page.getByRole("navigation", { name: "Component navigation menu" }).getByRole("button", { name: "Getting started" }),
        surface: (page) => page.getByRole("navigation", { name: "Component navigation menu" }),
        enter: async () => {},
      },
    ],
  },
  Calendar: {
    open: async (page) => {
      // The calendar demo opens on May 2026; pin "today" inside it so the today-cell probe has a target.
      await page.clock.setFixedTime(new Date("2026-05-15T12:00:00Z"));
      await gotoComponent(page, "calendar");
      await expect(preview(page)).toBeVisible({ timeout: 30_000 });
    },
    probes: [
      {
        name: "calendar next-month button hovered",
        target: (page) => preview(page).locator(".dx-calendar-nav-next").first(),
        surface: (page) => preview(page).locator(".dx-calendar").first(),
        enter: hover,
        rest: async (page) => away(page),
        icon: true, // the chevron is the whole button: no text, so its glyph is held to 3:1 (WCAG 1.4.11)
      },
      {
        // Today's cell is the muted fill and its hover is the accent fill: the dark tie option C creates.
        name: "calendar today cell hovered",
        target: (page) => preview(page).locator('.dx-calendar-grid-cell[data-month="current"][data-today="true"]').first(),
        surface: (page) => preview(page).locator(".dx-calendar").first(),
        enter: hover,
        rest: async (page) => away(page),
      },
      {
        name: "calendar plain day hovered",
        target: (page) =>
          preview(page).locator('.dx-calendar-grid-cell[data-month="current"]:not([data-today="true"]):not([data-selected="true"]):not([data-disabled="true"])').nth(3),
        surface: (page) => preview(page).locator(".dx-calendar").first(),
        enter: hover,
        rest: async (page) => away(page),
      },
    ],
  },
  Pagination: {
    open: async (page) => {
      await gotoComponent(page, "pagination");
      await expect(preview(page).locator('a[aria-current="page"]')).toBeVisible();
    },
    probes: [
      {
        name: "pagination current page",
        distinctFromSurfaceIn: ["dark"], // light: Nova's outline page number is `bg-background` on a white page
        target: (page) => preview(page).locator('a[aria-current="page"]'),
        surface: (page) => preview(page).locator("nav").first(),
        enter: async () => {},
      },
      {
        name: "pagination other page hovered",
        target: (page) => preview(page).getByRole("link", { name: "3", exact: true }),
        surface: (page) => preview(page).locator("nav").first(),
        enter: hover,
        rest: async (page) => away(page),
      },
    ],
  },
};

for (const mode of ["light", "dark"] as Mode[]) {
  test.describe(`${mode} mode`, () => {
    test.use({ colorScheme: mode });

    for (const [sceneName, scene] of Object.entries(SCENES)) {
      test(`${sceneName}: highlighted, hovered and selected states read and differ`, async ({ page }) => {
        await page.context().addCookies([{ name: "dx_theme", value: mode, url: BASE_URL }]);
        await page.emulateMedia({ colorScheme: mode, reducedMotion: "reduce" });
        await scene.open(page);
        await expect(page.locator("html")).toHaveAttribute("data-theme", mode);
        // shadcn's `.dark` is written together with the attribute (theme.rs `apply_mode`).
        expect(await page.evaluate(() => document.documentElement.classList.contains("dark"))).toBe(mode === "dark");
        await page.addStyleTag({ content: "*,*::before,*::after{transition:none!important;animation:none!important}" });

        const problems: string[] = [];
        for (const probe of scene.probes) {
          const target = probe.target(page);
          await expect(target.first(), `${probe.name}: target is present`).toBeVisible();
          const surface = probe.surface(page);

          let restBg: RGB | null = null;
          if (probe.rest) {
            await probe.rest(page, target);
            restBg = (await measure(target, surface)).bg;
          }
          await probe.enter(page, target);
          const m = await measure(target, surface);

          console.log(
            `[dark-surface] ${mode} ${probe.name}: bg ${rgbText(m.bg)} on ${rgbText(m.surface)}` +
              (restBg ? `, rest ${rgbText(restBg)}` : "") +
              `, text ${m.texts.map((t) => t.ratio.toFixed(2)).join("/")}` +
              (probe.icon ? `, icon ${m.icons.map((i) => i.ratio.toFixed(2)).join("/")}` : ""),
          );

          if (probe.icon) {
            if (m.icons.length === 0) problems.push(`${probe.name}: icon probe, but no svg found inside the probed element`);
            for (const i of m.icons.filter((i) => i.ratio < NON_TEXT)) {
              problems.push(`${probe.name}: ${i.label} is ${i.ratio.toFixed(2)}:1 (< ${NON_TEXT}, non-text) -- glyph ${rgbText(i.fg)} on ${rgbText(m.bg)}`);
            }
          } else if (m.texts.length === 0) {
            problems.push(`${probe.name}: no text found inside the probed element`);
          }
          for (const t of m.texts.filter((t) => t.ratio < AA)) {
            problems.push(`${probe.name}: ${t.label} is ${t.ratio.toFixed(2)}:1 (< ${AA}) -- text ${rgbText(t.fg)} on ${rgbText(m.bg)}`);
          }
          if ((probe.distinctFromSurfaceIn ?? ["light", "dark"]).includes(mode) && maxChannelDiff(m.bg, m.surface) < DISTINCT_HOVER) {
            problems.push(`${probe.name}: its background ${rgbText(m.bg)} is indistinguishable from its surface ${rgbText(m.surface)} (< ${DISTINCT_HOVER}/255 per channel)`);
          }
          if (restBg && maxChannelDiff(m.bg, restBg) < DISTINCT_HOVER) {
            problems.push(`${probe.name}: the state ${rgbText(m.bg)} is indistinguishable from rest ${rgbText(restBg)} (< ${DISTINCT_HOVER}/255 per channel)`);
          }
        }
        expect(problems).toEqual([]);
      });
    }
  });
}
