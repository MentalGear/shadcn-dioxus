import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { hoverSettled } from "./animations";

// Row 260 (row 243): every hover / pressed fill that steps a colour toward the ink reads the theme's
// `--dx-hover-mix-space` (shadcn's `in oklch`), so each one must equal the formula evaluated IN THE PAGE from the
// resolved tokens. Like button-hover.spec.ts, nothing is typed in: a probe element resolves the expected colour,
// and a canvas quantises both sides to the 8-bit sRGB a screenshot shows. Light and dark.
const GOTO = { timeout: 20 * 60 * 1000 };

async function open(page: Page, name: string, scheme: "light" | "dark") {
  await page.emulateMedia({ colorScheme: scheme });
  await gotoHydrated(page, `${BASE_URL}/component/${name}/${scheme === "dark" ? "?dark_mode=true" : ""}`, GOTO);
  await page.waitForLoadState("networkidle");
}

const toRgb = (page: Page, color: string) =>
  page.evaluate((c) => {
    const ctx = document.createElement("canvas").getContext("2d", { colorSpace: "srgb" })!;
    ctx.clearRect(0, 0, 1, 1);
    ctx.fillStyle = c;
    ctx.fillRect(0, 0, 1, 1);
    return Array.from(ctx.getImageData(0, 0, 1, 1).data.slice(0, 3)).join(",");
  }, color);

// `expr` resolved as a background colour by a probe child of `host` (so a component-scoped custom property works).
const resolveIn = (host: Locator, expr: string) =>
  host.evaluate((el, c) => {
    const p = document.createElement("span");
    el.appendChild(p);
    p.style.backgroundColor = c;
    const v = getComputedStyle(p).backgroundColor;
    p.remove();
    return v;
  }, expr);

const SPACE = "var(--dx-hover-mix-space)";
const step = (fill: string, pct: number) => `color-mix(in ${SPACE}, ${fill}, var(--dx-foreground) ${pct}%)`;

async function expectFill(page: Page, el: Locator, read: () => Promise<string>, formula: string, label: string) {
  const expected = await toRgb(page, await resolveIn(el, formula));
  await expect.poll(async () => toRgb(page, await read()), { message: label }).toBe(expected);
}

const bg = (el: Locator) => () => el.evaluate((e) => getComputedStyle(e).backgroundColor);

for (const scheme of ["light", "dark"] as const) {
  test(`Toggle pressed fill is accent stepped 15% toward the ink (${scheme})`, async ({ page }) => {
    await open(page, "toggle", scheme);
    const t = page.locator("#component-preview-frame").getByRole("button", { name: "Toggle bookmark", exact: true });
    await t.click();
    await expect(t).toHaveAttribute("data-state", "on");
    await page.mouse.move(0, 0);
    await expectFill(page, t, bg(t), step("var(--dx-accent)", 15), "toggle on");
  });

  test(`ToggleGroup pressed item fill is accent stepped 15% toward the ink (${scheme})`, async ({ page }) => {
    await open(page, "toggle_group", scheme);
    const item = page.locator("#component-preview-frame .dx-toggle-group-item").first();
    await item.click();
    await expect(item).toHaveAttribute("data-state", "on");
    await page.mouse.move(0, 0);
    await expectFill(page, item, bg(item), step("var(--dx-accent)", 15), "toggle_group on");
  });

  test(`AlertDialog destructive action hover is the fill stepped 8% toward the ink (${scheme})`, async ({ page }) => {
    await open(page, "alert_dialog", scheme);
    await page.getByRole("button", { name: "Show Alert Dialog", exact: true }).click();
    const action = page.locator(".dx-alert-dialog-action").first();
    await expect(action).toBeVisible();
    await action.hover();
    await expectFill(page, action, bg(action), step("var(--dx-destructive-fill)", 8), "alert-dialog action hover");
  });

  test(`Calendar today-cell hover is accent-selected stepped 6% toward the ink (${scheme})`, async ({ page }) => {
    // The range demo fixes its view to May 2026; pin "today" inside it (and out of the 5..12 range) so a today cell exists.
    await page.clock.setFixedTime(new Date("2026-05-15T12:00:00Z"));
    await page.emulateMedia({ colorScheme: scheme });
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=calendar&variant=range&${scheme === "dark" ? "dark_mode=true" : ""}`, GOTO);
    await page.waitForLoadState("networkidle");
    const cell = page
      .locator('.dx-calendar-grid-cell[data-month="current"][data-today="true"]:not([data-selected="true"],[data-disabled="true"])')
      .first();
    await cell.scrollIntoViewIfNeeded();
    await expect(cell).toBeVisible();
    await cell.hover();
    await expectFill(page, cell, bg(cell), step("var(--dx-accent-selected)", 6), "calendar today hover");
  });

  test(`NavigationMenu featured card hover surface is muted stepped 8% toward the ink (${scheme})`, async ({ page }) => {
    await open(page, "navigation_menu", scheme);
    const nav = page.getByRole("navigation", { name: "Component navigation menu" });
    await nav.getByRole("button", { name: "Getting started" }).hover();
    const card = page.locator(".dx-navigation-menu-featured").first();
    await expect(card).toBeVisible();
    await hoverSettled(card);
    const surface = () => resolveIn(card, "var(--dx-navigation-menu-featured-surface)");
    await expectFill(page, card, surface, step("var(--dx-muted)", 8), "featured hover");
  });

  test(`Bubble button hover steps secondary and muted 5% toward the ink (${scheme})`, async ({ page }) => {
    await open(page, "bubble", scheme);
    for (const variant of ["secondary", "muted"]) {
      const content = await page.evaluateHandle((v) => {
        const wrap = document.createElement("div");
        wrap.className = "dx-bubble";
        wrap.dataset.variant = v;
        wrap.style.cssText = "position:fixed;top:40px;left:40px;z-index:99999";
        wrap.innerHTML = '<button class="dx-bubble-content" type="button">probe</button>';
        document.body.appendChild(wrap);
        return wrap.firstElementChild!;
      }, variant);
      const btn = page.locator(".dx-bubble-content", { hasText: "probe" });
      await btn.hover();
      await expectFill(page, btn, bg(btn), step(`var(--dx-${variant})`, 5), `bubble ${variant}`);
      await content.dispose();
      await btn.evaluate((e) => e.parentElement!.remove());
    }
  });
}

test("every changed site reads the theme token, not a literal space", async ({ page }) => {
  await open(page, "toggle", "light");
  expect(await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue("--dx-hover-mix-space").trim())).toBe("oklch");
});
