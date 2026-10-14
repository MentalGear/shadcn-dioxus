import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL = `${BASE_URL}/component/?name=progress&`;
const LOAD_TIMEOUT = 20 * 60 * 1000;

// The `label` variant's bar (the `controlled` variant also has an `aria-label="Upload progress"` bar, without parts).
const labelled = (page: Page) => page.locator('[role="progressbar"]:has(.dx-progress-label)').filter({ hasText: "Upload progress" });

// `ProgressLabel` + `ProgressValue` (shadcn's `progress-label`, backlog row 155 group 9).
test.describe("label and value", () => {
  test("the bar is named by its label and the value reads the percent", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: LOAD_TIMEOUT });
    const bar = labelled(page);
    await expect(bar).toBeVisible();
    await expect(bar).toHaveAccessibleName("Upload progress");
    await expect(bar).toHaveAttribute("aria-valuenow", "56");
    const label = bar.locator(".dx-progress-label");
    const value = bar.locator(".dx-progress-value");
    await expect(label).toHaveText("Upload progress");
    await expect(value).toHaveText("56%");
    await expect(value).toHaveAttribute("aria-hidden", "true");
    // The root points at the label's id.
    const id = await label.getAttribute("id");
    expect(id).toBeTruthy();
    await expect(bar).toHaveAttribute("aria-labelledby", id!);
  });

  test("the label starts the first line, the value ends it, the track is full width below", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: LOAD_TIMEOUT });
    const bar = labelled(page);
    const root = (await bar.boundingBox())!;
    const label = (await bar.locator(".dx-progress-label").boundingBox())!;
    const value = (await bar.locator(".dx-progress-value").boundingBox())!;
    const track = (await bar.locator(".dx-progress-track").boundingBox())!;
    expect(Math.abs(label.x - root.x)).toBeLessThan(1);
    expect(Math.abs(root.x + root.width - (value.x + value.width))).toBeLessThan(1);
    expect(Math.abs(label.y - value.y)).toBeLessThan(8);
    expect(track.y).toBeGreaterThanOrEqual(label.y + label.height);
    expect(Math.abs(track.width - root.width)).toBeLessThan(1);
    expect(Math.round(track.height)).toBe(4);
    // 56% of the track is filled.
    const fill = (await bar.locator(".dx-progress-indicator").boundingBox())!;
    expect(Math.abs(fill.width / track.width - 0.56)).toBeLessThan(0.01);
  });

  test("a bare Progress is still the 4px track and keeps its name from aria-label", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: LOAD_TIMEOUT });
    const bare = page.getByRole("progressbar", { name: "Progressbar Demo" });
    await expect(bare).toBeVisible();
    await expect(bare.locator(".dx-progress-label")).toHaveCount(0);
    expect(Math.round((await bare.locator(".dx-progress-track").boundingBox())!.height)).toBe(4);
  });

  test("RTL mirrors the row: the label at the right, the value at the left", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: LOAD_TIMEOUT });
    const bar = page.getByRole("progressbar", { name: "جارٍ الرفع..." });
    await expect(bar).toBeVisible();
    const root = (await bar.boundingBox())!;
    const label = (await bar.locator(".dx-progress-label").boundingBox())!;
    const value = (await bar.locator(".dx-progress-value").boundingBox())!;
    expect(Math.abs(root.x + root.width - (label.x + label.width))).toBeLessThan(1);
    expect(Math.abs(value.x - root.x)).toBeLessThan(1);
  });
});

test.describe("Axe automated scan", () => {
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: LOAD_TIMEOUT });
    await expectNoAxeViolations(page, "progress: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
