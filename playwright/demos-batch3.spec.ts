/**
 * Row 188 / 187, the component "parts", each a typed prop with a demo:
 * AlertDialog `size` / `AlertDialogMedia` / `AlertDialogActionVariant`, Card `size`, Spinner `size`,
 * Button `radius`. Also pins the demos for parts that already existed (InputGroup block addons, Button
 * `as`, Dialog built-in close, Sheet `side`). Controls are located by role inside the demo frame;
 * axe runs in light and dark.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const frame = (page: Page, variant: string) => page.locator(variant === "main" ? "#component-preview-frame" : `#component-preview-frame-${variant}`).first();

async function open(page: Page, name: string, variant: string, mode: "light" | "dark") {
  await gotoHydrated(page, `${BASE_URL}/component/${name}/`, GOTO_OPTS);
  await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
  const root = frame(page, variant);
  await root.scrollIntoViewIfNeeded();
  return root;
}

const px = (el: import("@playwright/test").Locator, prop: "width" | "height") =>
  el.evaluate((e, p) => parseFloat(getComputedStyle(e)[p]), prop);
const bg = (el: import("@playwright/test").Locator) => el.evaluate((e) => getComputedStyle(e).backgroundColor);

for (const mode of ["light", "dark"] as const) {
  test.describe(`demos batch 3 (parts), ${mode}`, () => {
    test.use({ colorScheme: mode });

    test(`alert_dialog/small: data-size sm, two actions on one row`, async ({ page }) => {
      const root = await open(page, "alert_dialog", "small", mode);
      await root.getByRole("button", { name: "Show Small Dialog" }).click();
      const dialog = page.getByRole("alertdialog");
      await expect(dialog).toBeVisible();
      await expect(dialog).toHaveAttribute("data-size", "sm");
      const w = (await dialog.boundingBox())!.width;
      expect(w).toBeLessThanOrEqual(320.5);
      const a = (await dialog.getByRole("button", { name: "Don't allow" }).boundingBox())!;
      const b = (await dialog.getByRole("button", { name: "Allow", exact: true }).boundingBox())!;
      expect(Math.abs(a.y - b.y)).toBeLessThan(2);
      await expectNoAxeViolations(page, `alert_dialog/small ${mode}`);
      await dialog.getByRole("button", { name: "Allow", exact: true }).click();
      await expect(dialog).toHaveCount(0);
    });

    test(`alert_dialog/media: the media tile sits above the title, sized by size`, async ({ page }) => {
      const root = await open(page, "alert_dialog", "media", mode);
      await root.getByRole("button", { name: "Share Project" }).click();
      let dialog = page.getByRole("alertdialog");
      await expect(dialog).toBeVisible();
      const media = dialog.locator("[data-slot=alert-dialog-media]");
      const m = (await media.boundingBox())!;
      const t = (await dialog.getByRole("heading").boundingBox())!;
      expect(await px(media, "width")).toBeCloseTo(64, 0);
      expect(m.y + m.height).toBeLessThanOrEqual(t.y + 1);
      expect(await px(media.locator("svg"), "width")).toBeCloseTo(32, 0);
      await expectNoAxeViolations(page, `alert_dialog/media default ${mode}`);
      await dialog.getByRole("button", { name: "Cancel" }).click();
      await expect(dialog).toHaveCount(0);

      await root.getByRole("button", { name: "Pair Device" }).click();
      dialog = page.getByRole("alertdialog");
      await expect(dialog).toBeVisible();
      const sm = dialog.locator("[data-slot=alert-dialog-media]");
      expect(await px(sm, "width")).toBeCloseTo(40, 0);
      expect(await px(sm.locator("svg"), "width")).toBeCloseTo(24, 0);
      await expectNoAxeViolations(page, `alert_dialog/media small ${mode}`);
    });

    test(`alert_dialog/non_destructive: the primary action is not the destructive fill`, async ({ page }) => {
      const root = await open(page, "alert_dialog", "non_destructive", mode);
      await root.getByRole("button", { name: "Publish changes" }).click();
      const dialog = page.getByRole("alertdialog");
      await expect(dialog).toBeVisible();
      const action = dialog.getByRole("button", { name: "Publish", exact: true });
      await expect(action).toHaveAttribute("data-style", "primary");
      // Same fill as a primary Button; the main demo's destructive action differs.
      const primary = await bg(action);
      await dialog.getByRole("button", { name: "Not yet" }).click();
      await expect(dialog).toHaveCount(0);
      const main = frame(page, "main");
      await main.scrollIntoViewIfNeeded();
      await main.getByRole("button", { name: "Show Alert Dialog" }).click();
      const destructive = await bg(page.getByRole("alertdialog").getByRole("button", { name: "Delete" }));
      expect(destructive).not.toBe(primary);
      await page.keyboard.press("Escape");
      await expect(page.getByRole("alertdialog")).toHaveCount(0);
      await root.scrollIntoViewIfNeeded();
      await root.getByRole("button", { name: "Publish changes" }).click();
      await expectNoAxeViolations(page, `alert_dialog/non_destructive ${mode}`);
      await page.getByRole("alertdialog").getByRole("button", { name: "Publish", exact: true }).click();
      await expect(root.getByText("Page published.")).toBeVisible();
    });

    test(`card/small: typed size renders data-size and tighter spacing`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/card/`, GOTO_OPTS);
      const small = frame(page, "small").locator(".dx-card").first();
      await expect(small).toHaveAttribute("data-size", "sm");
      expect(await small.evaluate((e) => getComputedStyle(e).rowGap)).toBe("12px");
      const dflt = frame(page, "main").locator(".dx-card").first();
      await expect(dflt).toHaveAttribute("data-size", "default");
      expect(await dflt.evaluate((e) => getComputedStyle(e).rowGap)).toBe("16px");
    });

    test(`spinner/size: 12, 16, 24 and 32 px`, async ({ page }) => {
      const root = await open(page, "spinner", "size", mode);
      const edge = (name: string) =>
        px(root.getByRole("status", { name }).locator("svg"), "width");
      await expect(root.getByRole("status", { name: "Loading small" })).toBeVisible({ timeout: 60_000 });
      expect(await edge("Loading small")).toBeCloseTo(12, 0);
      expect(await edge("Loading default")).toBeCloseTo(16, 0);
      expect(await edge("Loading large")).toBeCloseTo(24, 0);
      expect(await edge("Loading extra large")).toBeCloseTo(32, 0);
      await expectNoAxeViolations(page, `spinner/size ${mode}`);
    });

    test(`button/radius: Full is a pill, an icon Full is a circle, default keeps the size radius`, async ({ page }) => {
      const root = await open(page, "button", "radius", mode);
      const radius = (name: string) =>
        root.getByRole("button", { name }).evaluate((e) => parseFloat(getComputedStyle(e).borderTopLeftRadius));
      await expect(root.getByRole("button", { name: "Rounded" })).toBeVisible({ timeout: 60_000 });
      expect(await radius("Default radius")).toBeLessThan(12);
      expect(await radius("Rounded")).toBeGreaterThan(100);
      expect(await radius("Large pill")).toBeGreaterThan(100);
      expect(await radius("Open")).toBeGreaterThan(100);
      await expectNoAxeViolations(page, `button/radius ${mode}`);
    });

    test(`button/as_link: renders an anchor that still takes the click handler`, async ({ page }) => {
      const root = await open(page, "button", "as_link", mode);
      const link = root.getByRole("link", { name: "Login" });
      await expect(link).toBeVisible({ timeout: 60_000 });
      await link.click();
      await expect(root.getByText("Clicked 1 time(s)")).toBeVisible();
      await expectNoAxeViolations(page, `button/as_link ${mode}`);
    });

    test(`input_group/block: block addons stack above and below the control`, async ({ page }) => {
      const root = await open(page, "input_group", "block", mode);
      const end = root.getByRole("textbox", { name: "Message" });
      await expect(end).toBeVisible({ timeout: 60_000 });
      const send = (await root.getByRole("button", { name: "Send message" }).boundingBox())!;
      expect(send.y).toBeGreaterThan((await end.boundingBox())!.y);
      const reply = root.getByRole("textbox", { name: "Reply", exact: true });
      const cancel = (await root.getByRole("button", { name: "Cancel" }).boundingBox())!;
      expect(cancel.y).toBeLessThan((await reply.boundingBox())!.y);
      await expectNoAxeViolations(page, `input_group/block ${mode}`);
    });

    test(`dialog: the built-in close button closes the dialog`, async ({ page }) => {
      const root = await open(page, "dialog", "share", mode);
      await root.getByRole("button", { name: "Share" }).click();
      const dialog = page.getByRole("dialog");
      await expect(dialog).toBeVisible();
      await expectNoAxeViolations(page, `dialog/share ${mode}`);
      await dialog.locator("[data-slot=dialog-close-slot]").getByRole("button").click();
      await expect(dialog).toHaveCount(0);
    });

    test(`sheet: the typed side puts the panel on each edge`, async ({ page }) => {
      const root = await open(page, "sheet", "main", mode);
      for (const side of ["Top", "Right", "Bottom", "Left"]) {
        await root.getByRole("button", { name: side, exact: true }).click();
        const panel = page.getByRole("dialog");
        await expect(panel).toBeVisible();
        await expect(panel).toHaveAttribute("data-side", side.toLowerCase());
        await page.keyboard.press("Escape");
        await expect(panel).toHaveCount(0);
      }
    });
  });
}
