import { test, expect } from "./fixtures";
import { type Locator, type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";

/**
 * Toast exit animation, the visible cap and the two entry cases (backlog row 290, Sonner parity).
 * One source of direction: the toaster's side (`--toast-edge` on the container). A brand-new toast
 * enters from, and a closed one leaves toward, the toaster's own edge; a queued toast that becomes
 * visible at the far end of the EXPANDED pile slides in from the opposite side.
 */
const R = "Stacked toast notifications";
const TOP = "Top toast notifications";

const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => gotoHydrated(page, `${BASE_URL}/component/toast/`);

type Sample = { y: number; o: number; connected: boolean; removed: string | null; inert: boolean };

/** Records `translate` y (px) and opacity of `el` once per frame until it leaves the document. */
async function startSampler(el: Locator) {
  await el.evaluate((node) => {
    const out: Sample[] = [];
    (window as unknown as { __dxS: Sample[] }).__dxS = out;
    const y = () => {
      const t = getComputedStyle(node).translate;
      if (!t || t === "none") return 0;
      const v = t.split(" ")[1] ?? "0px";
      return v.endsWith("%") ? (parseFloat(v) * (node as HTMLElement).offsetHeight) / 100 : parseFloat(v);
    };
    const tick = () => {
      const el = node as HTMLElement;
      out.push({
        y: y(),
        o: parseFloat(getComputedStyle(el).opacity),
        connected: el.isConnected,
        removed: el.getAttribute("data-removed"),
        inert: el.hasAttribute("inert"),
      });
      if (out.length < 600 && el.isConnected) requestAnimationFrame(tick);
    };
    tick();
  });
}
const samples = (page: Page) => page.evaluate(() => (window as unknown as { __dxS: Sample[] }).__dxS);

/** The translate y of every toast the moment it is inserted (its first animation frame). */
async function watchInserted(page: Page, name: string) {
  await region(page, name).evaluate((el) => {
    const out: number[] = [];
    (window as unknown as { __dxIn: number[] }).__dxIn = out;
    new MutationObserver((muts) => {
      for (const m of muts)
        for (const n of Array.from(m.addedNodes)) {
          const t = n instanceof Element ? (n.matches('[role="alertdialog"]') ? n : n.querySelector('[role="alertdialog"]')) : null;
          if (!t) continue;
          const v = (getComputedStyle(t).translate.split(" ")[1] ?? "0px");
          out.push(v.endsWith("%") ? (parseFloat(v) * (t as HTMLElement).offsetHeight) / 100 : parseFloat(v));
        }
    }).observe(el, { childList: true, subtree: true });
  });
}
const inserted = (page: Page) => page.evaluate(() => (window as unknown as { __dxIn: number[] }).__dxIn);

async function addStack(page: Page, n: number) {
  const add = page.getByRole("button", { name: "Add to stack" });
  for (let i = 0; i < n; i++) await add.click();
  await expect(toasts(page, R)).toHaveCount(n);
}

test.describe("toast: enter, exit and visible cap (row 290)", () => {
  test("a brand-new toast enters from the toaster's edge: from below at the bottom", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2); // collapsed
    await watchInserted(page, R);
    await addStack(page, 1);
    const [y] = await inserted(page);
    expect(y, "starts below its resting place").toBeGreaterThan(8);
    await expect.poll(async () => toasts(page, R).first().evaluate((el) => getComputedStyle(el).translate)).toMatch(/^(none|0px|0px 0px)$/);
  });

  test("a brand-new toast enters from the toaster's edge: from above at the top", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await watchInserted(page, TOP);
    await page.getByRole("button", { name: "Show top toast" }).click();
    await expect(toasts(page, TOP)).toHaveCount(1);
    const [y] = await inserted(page);
    expect(y, "starts above its resting place").toBeLessThan(-8);
  });

  for (const side of ["bottom", "top"] as const) {
    test(`closing plays an exit toward the edge, inert, then unmounts (${side})`, async ({ page }) => {
      await open(page);
      await page.mouse.move(2, 2);
      const name = side === "bottom" ? R : TOP;
      if (side === "bottom") await addStack(page, 1);
      else await page.getByRole("button", { name: "Show top toast" }).click();
      const toast = toasts(page, name).first();
      await expect(toast).toBeVisible();
      await startSampler(toast);
      await toast.getByRole("button", { name: "close" }).click();
      await expect(toasts(page, name)).toHaveCount(0);
      const s = await samples(page);
      const out = s.filter((x) => x.removed === "true" && x.connected);
      expect(out.length, "still mounted while it animates out").toBeGreaterThan(2);
      expect(out.every((x) => x.inert), "inert while leaving").toBe(true);
      const end = out[out.length - 1];
      expect(end.o, "fades out").toBeLessThan(out[0].o + 0.01);
      expect(Math.min(...out.map((x) => x.o)), "ends transparent").toBeLessThan(0.5);
      const far = side === "bottom" ? Math.max(...out.map((x) => x.y)) : Math.min(...out.map((x) => x.y));
      expect(side === "bottom" ? far : -far, "slides toward the toaster's edge").toBeGreaterThan(8);
    });
  }

  test("at most three show; the fourth waits hidden, on hover too", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await addStack(page, 4);
    const t = toasts(page, R);
    // DOM order is newest first: the last one is the oldest, past the cap.
    for (let i = 0; i < 3; i++) await expect(t.nth(i)).toHaveAttribute("data-visible", "true");
    await expect(t.nth(3)).toHaveAttribute("data-visible", "false");
    await expect(t.nth(3)).toHaveCSS("opacity", "0");
    const box = await measureAtRest(t.first());
    await page.mouse.move(box.x + 20, box.y + box.height / 2);
    await expect(region(page, R)).toHaveCSS("display", "block");
    await page.waitForTimeout(500);
    await expect(t.nth(3), "still hidden while the pile is expanded").toHaveCSS("opacity", "0");
    await expect(t.nth(3)).toHaveCSS("pointer-events", "none");
  });

  test("collapsed: closing one fades the queued toast in", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await addStack(page, 4);
    const queued = toasts(page, R).filter({ hasText: "Notification #1" });
    await expect(queued).toHaveAttribute("data-visible", "false");
    // A dispatched click: no hover or focus, so the pile stays collapsed.
    await toasts(page, R).first().getByRole("button", { name: "close" }).dispatchEvent("click");
    await expect(toasts(page, R)).toHaveCount(3);
    await expect(queued).toHaveAttribute("data-visible", "true");
  });

  test("expanded (hover held): the queued toast slides in from above with a fade", async ({ page }) => {
    await open(page);
    await page.mouse.move(2, 2);
    await addStack(page, 4);
    const t = toasts(page, R);
    const queued = t.filter({ hasText: "Notification #1" });
    await expect(queued).toHaveAttribute("data-visible", "false");

    // Hold the pointer over the pile (expanded).
    const box = await measureAtRest(t.first());
    await page.mouse.move(box.x + 20, box.y + box.height / 2);
    await expect.poll(() => region(page, R).evaluate((el) => el.matches(":hover"))).toBe(true);
    await page.waitForTimeout(400); // the restack settles; the queued toast rests shifted away from the edge

    await startSampler(queued);
    await t.first().getByRole("button", { name: "close" }).click();
    await expect(t).toHaveCount(3);
    await expect(t.nth(2)).toHaveAttribute("data-visible", "true");
    await page.waitForTimeout(500);
    expect(await region(page, R).evaluate((el) => el.matches(":hover")), "hover held throughout").toBe(true);

    const s = await samples(page);
    const rising = s.filter((x) => x.o > 0.05 && x.o < 0.95);
    expect(rising.length, "caught the transition mid-way").toBeGreaterThan(0);
    expect(rising.every((x) => x.y < -0.5), "while it fades in it is still above its slot (bottom toaster)").toBe(true);
    const last = s[s.length - 1];
    expect(last.o).toBeGreaterThan(0.99);
    expect(Math.abs(last.y)).toBeLessThan(0.5);
  });
});
