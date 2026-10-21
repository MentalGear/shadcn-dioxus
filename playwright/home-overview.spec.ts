/**
 * HOME / OVERVIEW PAGE polish (backlog rows 297, 298, 300), all on `/`.
 *
 *   297  the Sidebar card's "Collapse" control offers Offcanvas and Icon, not "None" (the component keeps
 *        `SidebarCollapsible::None`; its own variant demos still show it).
 *   298  the two-column component card has a vertical rule between the description column and the example column:
 *        the theme's `--dx-border` in both modes, full height of the header/content row and not the footer, on the
 *        content column's LEADING edge (so it flips for RTL). Stacked (a narrow gallery or a phone) there is no
 *        column to divide and no rule.
 *   300  the hero title "shadcn-dioxus" is bold like the navbar brand, the subtitle stays light.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8090 npx playwright test --config=ssg.local.config.ts home-overview.spec.ts
 */
import { test, expect } from "./fixtures";
import { baseUrlOr } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";

const BASE = baseUrlOr("http://127.0.0.1:8080");

type Mode = "light" | "dark";
const MODES: Mode[] = ["light", "dark"];

async function applyMode(page: import("@playwright/test").Page, mode: Mode) {
  await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
}

test.describe("sidebar card controls (row 297)", () => {
  test("Collapse offers Offcanvas and Icon, and no None", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await gotoHydrated(page, `${BASE}/`);
    const card = page.locator(".dx-component-card", { has: page.getByText("Collapse", { exact: true }) });
    await card.first().scrollIntoViewIfNeeded();
    await expect(card.getByRole("button", { name: "Offcanvas", exact: true })).toHaveCount(1);
    await expect(card.getByRole("button", { name: "Icon", exact: true })).toHaveCount(1);
    await expect(page.getByRole("button", { name: "None", exact: true })).toHaveCount(0);
  });

  test("the remaining choices still drive the live sidebar", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await gotoHydrated(page, `${BASE}/`);
    const sidebar = page.locator('[data-slot="sidebar"]:not([data-mobile="true"])').first();
    await page.getByRole("button", { name: "Icon", exact: true }).click();
    await expect(sidebar).toHaveAttribute("data-collapsible", "icon");
    await page.getByRole("button", { name: "Offcanvas", exact: true }).click();
    await expect(sidebar).not.toHaveAttribute("data-collapsible", "icon");
  });
});

test.describe("component card divider (row 298)", () => {
  async function firstBody(page: import("@playwright/test").Page) {
    await page.evaluate(() => document.documentElement.setAttribute("data-cv-settle", ""));
    const body = page.locator(".dx-component-card-body").first();
    await body.scrollIntoViewIfNeeded();
    return body;
  }

  for (const mode of MODES) {
    test(`two columns (1800 px, ${mode}): a --dx-border rule on the content column, header/content rows only`, async ({ page }) => {
      await page.setViewportSize({ width: 1800, height: 1000 });
      await gotoHydrated(page, `${BASE}/`);
      await applyMode(page, mode);
      const body = await firstBody(page);
      const r = await readAtRest(body, (el) => {
        const content = el.querySelector<HTMLElement>(':scope > [data-slot="card-content"]')!;
        const header = el.querySelector<HTMLElement>(':scope > [data-slot="card-header"]')!;
        const footer = el.querySelector<HTMLElement>(':scope > [data-slot="card-footer"]')!;
        const cs = getComputedStyle(content);
        const probe = document.createElement("div");
        probe.style.cssText = "border: 1px solid var(--dx-border); position: absolute; visibility: hidden;";
        el.appendChild(probe);
        const token = getComputedStyle(probe).borderTopColor;
        probe.remove();
        const c = content.getBoundingClientRect();
        const h = header.getBoundingClientRect();
        const f = footer.getBoundingClientRect();
        return {
          width: cs.borderInlineStartWidth,
          style: cs.borderInlineStartStyle,
          color: cs.borderInlineStartColor,
          token,
          endWidth: cs.borderInlineEndWidth,
          sideBySide: c.left > h.left + 1 && Math.abs(c.top - h.top) < 2,
          contentBottom: c.bottom,
          footerTop: f.top,
          contentHeight: c.height,
          headerHeight: h.height,
        };
      });
      expect(r.sideBySide, "this width must lay the card out in two columns").toBe(true);
      expect(r.width).toBe("1px");
      expect(r.style).toBe("solid");
      expect(r.color).toBe(r.token);
      expect(r.endWidth).toBe("0px");
      // Excludes the footer: the rule's column ends above it. It spans the whole header/content row.
      expect(r.contentBottom).toBeLessThanOrEqual(r.footerTop);
      expect(r.contentHeight).toBeGreaterThanOrEqual(r.headerHeight - 1);
    });
  }

  test("the rule sits on the other side in RTL", async ({ page }) => {
    await page.setViewportSize({ width: 1800, height: 1000 });
    await gotoHydrated(page, `${BASE}/`);
    const body = await firstBody(page);
    const r = await readAtRest(body, (el) => {
      el.setAttribute("dir", "rtl");
      const cs = getComputedStyle(el.querySelector('[data-slot="card-content"]')!);
      return { left: cs.borderLeftWidth, right: cs.borderRightWidth };
    });
    expect(r).toEqual({ left: "0px", right: "1px" });
  });

  for (const width of [390, 1100]) {
    test(`stacked (${width} px): no rule`, async ({ page }) => {
      await page.setViewportSize({ width, height: 900 });
      await gotoHydrated(page, `${BASE}/`);
      const body = await firstBody(page);
      const r = await readAtRest(body, (el) => {
        const content = el.querySelector<HTMLElement>(':scope > [data-slot="card-content"]')!;
        const header = el.querySelector<HTMLElement>(':scope > [data-slot="card-header"]')!;
        const cs = getComputedStyle(content);
        return {
          start: cs.borderInlineStartWidth,
          end: cs.borderInlineEndWidth,
          top: cs.borderTopWidth,
          stacked: content.getBoundingClientRect().top >= header.getBoundingClientRect().bottom - 1,
        };
      });
      expect(r.stacked, "this width must stack the card").toBe(true);
      expect(r).toMatchObject({ start: "0px", end: "0px", top: "0px" });
    });
  }
});

test.describe("hero title (row 300)", () => {
  for (const width of [1280, 390]) {
    test(`"shadcn-dioxus" is bold like the navbar brand, the subtitle stays light (${width} px)`, async ({ page }) => {
      await page.setViewportSize({ width, height: 900 });
      await gotoHydrated(page, `${BASE}/`);
      const weight = (sel: string) => page.locator(sel).first().evaluate((n) => getComputedStyle(n).fontWeight);
      expect(await weight(".dx-hero-title")).toBe("700");
      expect(await weight(".dx-navbar-brand")).toBe("700");
      expect(await weight(".dx-hero-subtitle")).toBe("300");
    });
  }
});
