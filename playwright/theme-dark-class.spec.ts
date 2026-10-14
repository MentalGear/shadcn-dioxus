import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * shadcn's `.dark` class as the second spelling of `data-theme="dark"` (backlog row 158 (a)).
 *
 * The construction under test:
 *   - CSS (`dx-components-theme.css`): `html.dark` is in the SAME selector list as
 *     `html[data-theme="dark"]` (the `--dark`/`--light` space toggle and `color-scheme`), so a host
 *     that sets only `class="dark"` gets dark. The explicit attribute wins a disagreement:
 *     `class="dark"` + `data-theme="light"` is LIGHT (the rules are disjoint, no source-order luck).
 *   - JS (`theme.rs` PREPAINT_JS `apply_mode`, also `window.__dxApplyMode`): the one writer of
 *     `data-theme` and the `dark` class. It writes them together, and resolves "system" (no
 *     `data-theme`) from `prefers-color-scheme`, following it live.
 *
 * Part 1 loads the theme stylesheet (read from disk, like theme-preset.spec.ts reads the presets) into a
 * bare host page with NO script at all, so it proves the CSS alone. Part 2 is the real site, with a pasted
 * shadcn theme (a `:root` block plus a `.dark` block) and the three modes: light, dark, system.
 *
 * Colours are compared as painted pixels' computed rgb, via `--dx-background` (shadcn's `--background`
 * first, then our ramp).
 */

const THEME_CSS = fs.readFileSync(
  path.join(__dirname, "..", "preview", "assets", "dx-components-theme.css"),
  "utf8",
);

const HOST = "http://host.dx-theme.test";

/** The host page: an `<html>` with exactly the given attributes, the theme sheet, optionally a pasted shadcn theme. */
async function openHost(page: Page, htmlAttrs: string, pasted = "") {
  await page.route(`${HOST}/**`, (route) =>
    route.fulfill({
      contentType: "text/html",
      body: `<!doctype html><html ${htmlAttrs}><head><meta charset="utf-8"><style>${THEME_CSS}</style><style>${pasted}</style></head>
<body><div id="probe" style="background: var(--dx-background); width: 10px; height: 10px"></div></body></html>`,
    }),
  );
  await page.goto(`${HOST}/`);
}

const probe = (page: Page) =>
  page.evaluate(() => {
    const root = getComputedStyle(document.documentElement);
    return {
      bg: getComputedStyle(document.getElementById("probe")!).backgroundColor,
      scheme: root.colorScheme,
    };
  });

const DARK_BG = "rgb(10, 10, 10)"; // --primary-color-2 dark (#0a0a0a)
const LIGHT_BG = "rgb(255, 255, 255)";

test.describe("host page, theme sheet only (no script)", () => {
  test.use({ colorScheme: "light" }); // OS light: any dark below comes from the class/attribute alone

  test('`<html class="dark">` with no data-theme renders dark', async ({ page }) => {
    await openHost(page, 'class="dark"');
    expect(await probe(page)).toEqual({ bg: DARK_BG, scheme: "dark" });
  });

  test("`.dark` among other classes still counts, a look-alike class does not", async ({ page }) => {
    await openHost(page, 'class="antialiased dark font-sans"');
    expect((await probe(page)).bg).toBe(DARK_BG);
    for (const cls of ["darkish", "not-dark", "dark-mode"]) {
      await openHost(page, `class="${cls}"`);
      expect(await probe(page), cls).toEqual({ bg: LIGHT_BG, scheme: "light dark" });
    }
  });

  test('`class="dark"` + data-theme="light": the explicit attribute wins, so it is light', async ({ page }) => {
    await openHost(page, 'class="dark" data-theme="light"');
    expect(await probe(page)).toEqual({ bg: LIGHT_BG, scheme: "light" });
  });

  test('`class="dark"` + data-theme="dark" is dark; data-theme="dark" alone is unchanged', async ({ page }) => {
    await openHost(page, 'class="dark" data-theme="dark"');
    expect(await probe(page)).toEqual({ bg: DARK_BG, scheme: "dark" });
    await openHost(page, 'data-theme="dark"');
    expect(await probe(page)).toEqual({ bg: DARK_BG, scheme: "dark" });
  });

  test("neither: prefers-color-scheme decides (unchanged)", async ({ page }) => {
    await openHost(page, "");
    expect((await probe(page)).bg).toBe(LIGHT_BG);
    await page.emulateMedia({ colorScheme: "dark" });
    expect((await probe(page)).bg).toBe(DARK_BG);
  });

  test("a light OS with `.dark` is dark, and a dark OS with data-theme=light is light", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await openHost(page, 'data-theme="light"');
    expect((await probe(page)).bg).toBe(LIGHT_BG);
    await openHost(page, 'class="dark"');
    expect((await probe(page)).bg).toBe(DARK_BG);
  });

  test("a pasted shadcn theme (:root + .dark) paints its own light and dark values", async ({ page }) => {
    const pasted = ":root { --background: rgb(250, 0, 0); } .dark { --background: rgb(0, 0, 250); }";
    await openHost(page, "", pasted);
    expect((await probe(page)).bg).toBe("rgb(250, 0, 0)");
    await openHost(page, 'class="dark"', pasted);
    expect((await probe(page)).bg).toBe("rgb(0, 0, 250)");
  });
});

const html = (page: Page) =>
  page.evaluate(() => ({
    theme: document.documentElement.getAttribute("data-theme"),
    dark: document.documentElement.classList.contains("dark"),
  }));

/** A pasted shadcn theme, injected after every site sheet, and transitions off so reads are not mid-fade. */
const PASTED_RED_BLUE =
  ":root { --background: rgb(250, 0, 0); } .dark { --background: rgb(0, 0, 250); } *, *::before, *::after { transition: none !important; }";

const navbarBg = (page: Page) =>
  page.evaluate(() => getComputedStyle(document.querySelector(".dx-preview-navbar")!).backgroundColor);

async function openSite(page: Page, cookie?: "dark" | "light") {
  if (cookie) await page.context().addCookies([{ name: "dx_theme", value: cookie, url: BASE_URL }]);
  await gotoHydrated(page, `${BASE_URL}/component/button/`);
  await page.addStyleTag({ content: PASTED_RED_BLUE });
}

test.describe("the site, with a pasted shadcn theme", () => {
  test.use({ colorScheme: "light" });

  test("cookie dark: data-theme and .dark are written together, the .dark block paints", async ({ page }) => {
    await openSite(page, "dark");
    expect(await html(page)).toEqual({ theme: "dark", dark: true });
    await expect.poll(() => navbarBg(page)).toBe("rgb(0, 0, 250)");
  });

  test("cookie light: no .dark, the :root block paints, even under a dark OS", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await openSite(page, "light");
    expect(await html(page)).toEqual({ theme: "light", dark: false });
    await expect.poll(() => navbarBg(page)).toBe("rgb(250, 0, 0)");
  });

  test("system mode: .dark follows prefers-color-scheme, live, with no data-theme", async ({ page }) => {
    await openSite(page);
    expect(await html(page)).toEqual({ theme: null, dark: false });
    await expect.poll(() => navbarBg(page)).toBe("rgb(250, 0, 0)");

    await page.emulateMedia({ colorScheme: "dark" });
    await expect.poll(async () => (await html(page)).dark, { message: "page-lifetime change listener" }).toBe(true);
    expect((await html(page)).theme).toBeNull();
    await expect.poll(() => navbarBg(page)).toBe("rgb(0, 0, 250)");

    await page.emulateMedia({ colorScheme: "light" });
    await expect.poll(async () => (await html(page)).dark).toBe(false);
    await expect.poll(() => navbarBg(page)).toBe("rgb(250, 0, 0)");
  });

  test("system mode under a dark OS is dark from the first paint's script", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await openSite(page);
    expect(await html(page)).toEqual({ theme: null, dark: true });
    await expect.poll(() => navbarBg(page)).toBe("rgb(0, 0, 250)");
  });

  test("an explicit mode ignores later OS changes; clearing it returns to system", async ({ page }) => {
    await openSite(page, "light");
    await page.emulateMedia({ colorScheme: "dark" });
    // Give a (wrongly) live listener a chance to fire; the explicit mode must not move.
    await page.waitForTimeout(150);
    expect(await html(page)).toEqual({ theme: "light", dark: false });

    await page.evaluate(() => (window as unknown as { __dxTheme: (k: string, v: null) => void }).__dxTheme("theme", null));
    expect(await html(page)).toEqual({ theme: null, dark: true });
  });

  test("the toggle and a reload keep the pair consistent", async ({ page }) => {
    await openSite(page, "light");
    await page.getByRole("button", { name: "Enable dark mode" }).click();
    await expect.poll(() => html(page)).toEqual({ theme: "dark", dark: true });
    await page.reload();
    await page.waitForFunction(() => document.documentElement.dataset.hydrated === "true");
    expect(await html(page)).toEqual({ theme: "dark", dark: true });
    await page.getByRole("button", { name: "Enable light mode" }).click();
    await expect.poll(() => html(page)).toEqual({ theme: "light", dark: false });
  });

  test("a junk dx_theme cookie is system, not dark", async ({ page }) => {
    await page.context().addCookies([{ name: "dx_theme", value: "purple", url: BASE_URL }]);
    await gotoHydrated(page, `${BASE_URL}/component/button/`);
    expect(await html(page)).toEqual({ theme: null, dark: false });
  });
});
