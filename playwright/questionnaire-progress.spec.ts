/**
 * Questionnaire polish: the `progress` demo's checkpoint bar paints done segments in the primary colour and
 * pending segments in a visible muted track (light and dark), and the `animate_height` demo renders with a
 * switch that flips `data-animate-height` on the form live.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };
const URL = `${BASE_URL}/component/questionnaire/`;
const PROGRESS = "#component-preview-frame-progress";
const HEIGHT = "#component-preview-frame-animate_height";

const mode = (page: Page, m: "light" | "dark") =>
  page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), m);

/** Computed background of each checkpoint segment of the progress demo. */
const segments = (page: Page) =>
  page.locator(`${PROGRESS} [role="progressbar"] [data-slot="questionnaire-checkpoint"]`).evaluateAll((els) =>
    els.map((el) => getComputedStyle(el).backgroundColor),
  );

const token = (page: Page, name: string) =>
  page.evaluate((name) => {
    const probe = document.createElement("div");
    probe.style.background = `var(${name})`;
    document.body.appendChild(probe);
    const color = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return color;
  }, name);

const transparent = (c: string) => c === "rgba(0, 0, 0, 0)" || c === "transparent";

for (const m of ["light", "dark"] as const) {
  test(`progress demo: done segments are primary, pending are a visible muted track (${m})`, async ({ page }) => {
    await gotoHydrated(page, URL, GOTO);
    await mode(page, m);
    const root = page.locator(PROGRESS);
    await root.scrollIntoViewIfNeeded();
    const primary = await token(page, "--dx-primary");
    const muted = await token(page, "--dx-muted");
    const background = await token(page, "--dx-background");
    expect(primary).not.toBe(muted);
    expect(muted).not.toBe(background);

    // Walk to the third checkpoint, checking the bar at every step.
    for (let current = 1; current <= 3; current++) {
      const colours = await segments(page);
      expect(colours).toHaveLength(4);
      colours.forEach((c, i) => {
        expect(transparent(c), `segment ${i + 1} of checkpoint ${current} is painted`).toBe(false);
        expect(c, `segment ${i + 1} of checkpoint ${current}`).toBe(i < current ? primary : muted);
        expect(c).not.toBe(background);
      });
      if (current === 3) break;
      await root.locator('input[data-slot="questionnaire-choice-input"]:visible').first().check();
      await root.getByRole("button", { name: "Next" }).click();
      await expect(root.getByRole("progressbar")).toContainText(`Checkpoint ${current + 1} of 4`);
    }
  });
}

test("animate_height demo renders and its switch flips data-animate-height live", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO);
  const root = page.locator(HEIGHT);
  await root.scrollIntoViewIfNeeded();
  const form = root.locator('form[data-slot="questionnaire"]');
  const toggle = root.getByRole("switch", { name: "Animate height" });
  await expect(form).toHaveAttribute("data-animate-height", "true");
  await expect(toggle).toBeChecked();
  await toggle.click();
  await expect(toggle).not.toBeChecked();
  await expect(form).not.toHaveAttribute("data-animate-height", /.*/);
  await toggle.click();
  await expect(form).toHaveAttribute("data-animate-height", "true");

  // The items differ clearly in height, so the glide has something to show.
  const heights: number[] = [];
  for (let i = 0; i < 2; i++) {
    heights.push((await form.boundingBox())!.height);
    await root.locator('input[data-slot="questionnaire-choice-input"]:visible').first().check();
    await root.getByRole("button", { name: "Next" }).click();
    await expect(form.locator('fieldset[data-active]')).toHaveAttribute("data-name", /.*/).catch(() => {});
    await expect.poll(async () => (await form.boundingBox())!.height).not.toBe(heights[i]);
  }
  expect(Math.abs(heights[1] - heights[0])).toBeGreaterThan(60);
});
