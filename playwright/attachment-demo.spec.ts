import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// The main Attachment demo is shadcn's `attachment-demo`: an image group, an uploading file with a Spinner and a
// finished code file. (It was a plain list of three file chips with no image: backlog row 230.)
const URL = `${BASE_URL}/component/attachment/`;
const GOTO = { timeout: 20 * 60 * 1000 };
const main = (page: Page) => page.locator("#component-preview-frame").first();
const variant = (page: Page, name: string) => page.locator(`#component-preview-frame-${name}`).first();

async function open(page: Page) {
  await gotoHydrated(page, URL, GOTO);
  await expect(main(page).locator('[data-slot="attachment"]').first()).toBeVisible({ timeout: 60_000 });
}

test("the main demo starts with an image: an image group, an uploading file and a code file", async ({ page }) => {
  await open(page);
  const demo = main(page);
  const cards = demo.locator('[data-slot="attachment"]');
  await expect(cards).toHaveCount(5);

  // The first thing shown is the image group: three vertical cards whose media is a real, loaded, bundled <img>.
  const group = demo.locator('[data-slot="attachment-group"]');
  await expect(group.locator('[data-slot="attachment"][data-orientation="vertical"]')).toHaveCount(3);
  const images = group.locator('[data-slot="attachment-media"][data-variant="image"] img');
  await expect(images).toHaveCount(3);
  for (let i = 0; i < 3; i++) {
    await expect(images.nth(i)).toBeVisible();
    // Bundled with asset!, never hot-linked, and decoded (a broken image has naturalWidth 0).
    expect(await images.nth(i).getAttribute("src")).toMatch(/^\/(?!\/).*\.svg$|^\/assets\//);
    await expect.poll(() => images.nth(i).evaluate((el: HTMLImageElement) => el.naturalWidth)).toBeGreaterThan(0);
  }
  const firstTop = (await cards.first().boundingBox())!.y;
  const uploadTop = (await demo.locator('[data-state="uploading"]').boundingBox())!.y;
  expect(firstTop).toBeLessThan(uploadTop);

  const uploading = demo.locator('[data-slot="attachment"][data-state="uploading"]');
  await expect(uploading).toHaveCount(1);
  await expect(uploading).toContainText("Uploading · 64%");
  await expect(uploading.getByRole("status")).toBeVisible(); // the Spinner as the media

  const code = demo.locator('[data-slot="attachment"]', { hasText: "message-renderer.rs" });
  await expect(code).toHaveAttribute("data-state", "done");

  await demo.getByRole("button", { name: "Cancel upload" }).click();
  await expect(uploading).toHaveCount(0);
  await demo.getByRole("button", { name: "Remove message-renderer.rs" }).click();
  await expect(cards).toHaveCount(3);
});

test("the group is a ScrollArea strip: it scrolls on its own and fades its edges, with snap points", async ({ page }) => {
  await open(page);
  // The `group` variant holds more chips than fit, so it is the one that overflows.
  const strip = variant(page, "group").getByRole("group", { name: "Attached files" });
  await expect(strip).toHaveAttribute("data-scroll-direction", "horizontal");
  await expect(strip).toHaveAttribute("data-scroll-fade", "x");
  await expect(strip).toHaveClass(/dx-scroll-fade-x/);
  const s = await strip.evaluate((el: HTMLElement) => {
    const c = getComputedStyle(el);
    return { overflowX: c.overflowX, snap: c.scrollSnapType, scrollable: el.scrollWidth > el.clientWidth + 10, mask: c.maskImage };
  });
  expect(s.overflowX).toBe("auto");
  expect(s.snap).toContain("mandatory");
  expect(s.scrollable).toBe(true);
  expect(s.mask).toContain("linear-gradient");

  // Scrolling moves it, and the start fade appears (the fade depths are scroll-driven registered properties).
  const startFade = () =>
    strip.evaluate((el: HTMLElement) => {
      const prev = el.style.marginInlineStart;
      el.style.marginInlineStart = "var(--dx-scroll-fade-s)";
      const v = parseFloat(getComputedStyle(el).marginInlineStart);
      el.style.marginInlineStart = prev;
      return v;
    });
  expect(await startFade()).toBe(0);
  await strip.evaluate((el) => (el.scrollLeft = 150));
  await expect.poll(startFade).toBeGreaterThan(0);
  // Snapping settles on a chip's start edge.
  await expect
    .poll(() =>
      strip.evaluate((el: HTMLElement) => {
        const left = el.getBoundingClientRect().left + parseFloat(getComputedStyle(el).scrollPaddingInlineStart || "0");
        return Math.min(...Array.from(el.children).map((c) => Math.abs(c.getBoundingClientRect().left - left)));
      }),
    )
    .toBeLessThan(2);
});

test("main demo has no automatically detectable a11y issues", async ({ page }) => {
  await open(page);
  await expectNoAxeViolations(page, "attachment/main: loaded", { include: "#component-preview-frame" });
});
