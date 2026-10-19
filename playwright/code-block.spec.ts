import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL_ = `${BASE_URL}/component/?name=code_block&`;
const GOTO = { timeout: 20 * 60 * 1000 };
const frame = (page: Page, name?: string) =>
  page.locator(name ? `#component-preview-frame-${name}` : "#component-preview-frame").first();
const styled = (page: Page) =>
  expect
    .poll(() => frame(page).locator('[data-slot="code-block"]').first().evaluate((el) => getComputedStyle(el).display), {
      timeout: 60_000,
    })
    .toBe("flex");

test("copy button copies the verbatim code and its name flips to Copied", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]).catch(() => {});
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page);
  const button = demo.getByRole("button", { name: "Copy" });
  await expect(button).toBeVisible();
  await button.click();
  await expect(demo.getByRole("button", { name: "Copied" })).toBeVisible();
  await expect(demo.locator('[aria-live="polite"]')).toHaveText("Copied");
  const text = await page.evaluate(() => navigator.clipboard.readText()).catch(() => null);
  if (text !== null) expect(text).toContain("streamText({");
  // Reverts after about two seconds.
  await expect(demo.getByRole("button", { name: "Copy" })).toBeVisible({ timeout: 5000 });
});

test("the code region is focusable, ltr, and focus rings the whole block", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const pre = frame(page).locator('[data-slot="code-block-content"]');
  await expect(pre).toHaveAttribute("tabindex", "0");
  await expect(pre).toHaveAttribute("dir", "ltr");
  await pre.focus();
  const shadow = await frame(page)
    .locator('[data-slot="code-block"]')
    .evaluate((el) => getComputedStyle(el).boxShadow);
  expect(shadow).not.toBe("none");
});

test("line numbers are CSS counters (not text), start_line offsets them", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page, "line_numbers");
  const blocks = demo.locator('[data-slot="code-block-content"]');
  await expect(blocks).toHaveCount(2);
  await expect(blocks.first()).toHaveAttribute("data-numbered", "true");
  const first = blocks.first().locator('[data-slot="code-block-line"]').first();
  expect(await first.evaluate((el) => getComputedStyle(el, "::before").content)).toContain("counter(line)");
  expect(await first.innerText()).not.toMatch(/^1\s/);
  expect(await blocks.first().evaluate((el) => el.style.counterReset)).toBe("line 0");
  expect(await blocks.nth(1).evaluate((el) => el.style.counterReset)).toBe("line 40");
});

test("diff lines carry a gutter marker and variant", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page, "diff");
  const added = demo.locator('[data-slot="code-block-line"][data-variant="added"]');
  const removed = demo.locator('[data-slot="code-block-line"][data-variant="removed"]');
  await expect(added).toHaveCount(1);
  await expect(removed).toHaveCount(1);
  expect(await added.evaluate((el) => getComputedStyle(el, "::after").content)).toBe('"+"');
  expect(await removed.evaluate((el) => getComputedStyle(el, "::after").content)).toContain("−");
});

test("wrap wraps and the default scrolls", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const blocks = frame(page, "wrap_and_footer").locator('[data-slot="code-block-content"]');
  const [scroll, wrap] = await blocks.evaluateAll((els) =>
    els.map((el) => ({ over: el.scrollWidth > el.clientWidth + 1, ws: getComputedStyle(el).whiteSpace })),
  );
  expect(scroll.over).toBe(true);
  expect(wrap.over).toBe(false);
  expect(wrap.ws).toBe("pre-wrap");
});

test("floating actions are hidden until hover or focus", async ({ page }) => {
  await gotoHydrated(page, URL_, GOTO);
  await styled(page);
  const demo = frame(page, "floating_actions");
  const actions = demo.locator('[data-slot="code-block-actions"]');
  const opacity = () => actions.evaluate((el) => getComputedStyle(el).opacity);
  expect(await opacity()).toBe("0");
  await demo.getByRole("button", { name: "Copy" }).focus();
  await expect.poll(opacity).toBe("1");
});

for (const mode of ["light", "dark"] as const) {
  test(`code block demos have no axe violations (${mode})`, async ({ page }) => {
    await gotoHydrated(page, URL_, GOTO);
    await styled(page);
    await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
    for (const name of ["main", "diff", "destructive", "line_numbers", "floating_actions"]) {
      await expectNoAxeViolations(page, `code_block ${name} ${mode}`, {
        include: name === "main" ? "#component-preview-frame" : `#component-preview-frame-${name}`,
      });
    }
  });
}
