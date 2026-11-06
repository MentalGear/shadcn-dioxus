import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// Every `PaginationLink`/`PaginationPrevious`/`PaginationNext` renders a real
// `<a href="#">` (`preview/src/components/pagination/component.rs`) --
// clicking one navigates the page (a hash change still counts as a
// navigation) and breaks the rest of the test, so every test below only
// hovers/reads attributes, never clicks. Every locator below is scoped to
// `#component-preview-frame` (the page's own nav/sidebar chrome carries
// dozens of unrelated `<a>` links -- `menubar.spec.ts`'s "keyboard
// navigation" test uses the same scoping for the identical reason).

test("current page is marked aria-current=\"page\"; Previous/Next have accessible names", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/pagination/`, { waitUntil: "networkidle" });
  const preview = page.locator("#component-preview-frame").first();

  const current = preview.locator('a[aria-current="page"]');
  await expect(current).toBeVisible();
  await expect(current).toHaveText("2");

  await expect(preview.getByRole("link", { name: "Go to previous page" })).toBeVisible();
  await expect(preview.getByRole("link", { name: "Go to next page" })).toBeVisible();
  await expect(preview.getByRole("link", { name: "1", exact: true })).toBeVisible();
  await expect(preview.getByRole("link", { name: "3", exact: true })).toBeVisible();
});

// dev-docs/backlog.md row 89: `[data-active]`'s own background
// (`var(--light, var(--primary-color)) var(--dark, var(--primary-color-3))`)
// and `[data-active]:hover`'s (`var(--primary-color-4)`) are
// different token-family members, not a deliberate one-step-lighter pairing
// -- but it does NOT let a user lose the current-page indication on hover:
// `[data-active]`'s own `border` (`var(--primary-color-6)`) is set by
// a rule `:hover` never touches at all, so it persists unconditionally, and
// no non-active link has a border in any state. The exact
// computed values below (both themes) pin it: the
// background does shift by a few RGB points on hover
// the one property no other link's hover ever sets -- never changes.
for (const dark of [false, true]) {
  test(`hovering the current page keeps its border (the actual "you are here" signal) (${dark ? "dark" : "light"} mode)`, async ({ page }) => {
    // dev-docs/backlog.md row 109: this test hovers right after navigating,
    // on the SSG lane -- `gotoHydrated` so that hover can't land before
    // hydration attaches listeners.
    await gotoHydrated(page, `${BASE_URL}/component/pagination/${dark ? "?dark_mode=true" : ""}`);
    const preview = page.locator("#component-preview-frame").first();
    const current = preview.locator('a[aria-current="page"]');
    const other = preview.getByRole("link", { name: "1", exact: true });

    await page.locator("body").hover({ position: { x: 0, y: 0 } });
    // Wait past the CSS transition (--dx-motion-duration-slow, 200ms)
    // before reading computed style -- a mid-transition read returns an
    // interpolated, neither-old-nor-new value.
    await page.waitForTimeout(400);
    const atRest = await current.evaluate((el) => {
      const s = getComputedStyle(el);
      return { bg: s.backgroundColor, border: s.borderColor, width: s.borderTopWidth };
    });

    await current.hover();
    await page.waitForTimeout(400);
    expect(await current.evaluate((el) => el.matches(":hover"))).toBe(true);
    const hovered = await current.evaluate((el) => {
      const s = getComputedStyle(el);
      return { bg: s.backgroundColor, border: s.borderColor, width: s.borderTopWidth };
    });

    // The border -- the one visual cue no other link's hover rule ever
    // sets -- is untouched by hovering the current page.
    expect(hovered.border, JSON.stringify({ atRest, hovered })).toBe(atRest.border);
    expect(hovered.width, JSON.stringify({ atRest, hovered })).toBe(atRest.width);
    expect(hovered.width, "the border must actually be present, not 0px").not.toBe("0px");

    // A non-active link never has a visible border, hovered or not -- so the
    // current page stays visually distinguishable from every other link
    // regardless of which one the pointer happens to rest on.
    await other.hover();
    await page.waitForTimeout(400);
    // Nova's ghost Button carries a 1px TRANSPARENT border (`border border-transparent`), so the
    // width is 1px; what must hold is that the border never becomes visible.
    const otherHoveredBorder = await other.evaluate((el) => getComputedStyle(el).borderTopColor);
    expect(otherHoveredBorder, "a non-active link must not gain a visible border on hover").toBe("rgba(0, 0, 0, 0)");
  });
}

// An unavailable end link (`disabled`) is an anchor without `href`: `aria-disabled`, out of the tab order, and a
// click cannot navigate. The enabled links around it keep their `href`.
test("disabled Previous/Next: aria-disabled, no href, not focusable, click does not navigate", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/pagination/`);
  const preview = page.locator("#component-preview-frame-disabled_ends").first();
  const first = preview.locator('nav[aria-label="First page pagination"]');
  const last = preview.locator('nav[aria-label="Last page pagination"]');

  const previous = first.locator('[data-slot="pagination-link"][data-kind="previous"]');
  await expect(previous).toHaveAttribute("aria-disabled", "true");
  await expect(previous).toHaveAttribute("data-disabled", "");
  expect(await previous.getAttribute("href")).toBeNull();
  expect(await previous.evaluate((el) => (el as HTMLElement).tabIndex), "an anchor without href is not tabbable").toBe(-1);
  expect(await previous.evaluate((el) => getComputedStyle(el).pointerEvents)).toBe("none");
  expect(Number(await previous.evaluate((el) => getComputedStyle(el).opacity))).toBeLessThan(1);

  const next = last.locator('[data-slot="pagination-link"][data-kind="next"]');
  await expect(next).toHaveAttribute("aria-disabled", "true");
  expect(await next.getAttribute("href")).toBeNull();

  // The enabled ends and the page links are unchanged.
  for (const enabled of [first.locator('[data-kind="next"]'), last.locator('[data-kind="previous"]'), first.locator('a[aria-current="page"]')]) {
    await expect(enabled).toHaveAttribute("href", "#");
    await expect(enabled).not.toHaveAttribute("aria-disabled", /.*/);
    await expect(enabled).not.toHaveAttribute("data-disabled", /.*/);
    expect(await enabled.evaluate((el) => (el as HTMLElement).tabIndex)).toBe(0);
  }

  const url = page.url();
  await previous.click({ force: true });
  expect(page.url(), "a disabled link must not navigate").toBe(url);
});
