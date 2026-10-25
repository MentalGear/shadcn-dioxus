import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
  // Scope to the demo frame: the component page renders its own source in a
  // code viewer and carries sitewide chrome, either of which can contribute
  // another role="group" and make an unscoped locator ambiguous.
  const groups = page.locator("#component-preview-frame").first().getByRole("group");
  await expect(groups).toHaveCount(2);
  await expect(groups.first().getByRole("button", { name: "Today" })).toBeVisible();
  await expect(groups.last().getByRole("button", { name: "More" })).toBeVisible();
});

test.describe("Axe automated scan", () => {
  // Button Group has no overlay/expand/select interaction -- one state to scan.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
    await expectNoAxeViolations(page, "button_group: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// Backlog row 188 (g): the popover demo's chevron trigger wears the Button recipe (shadcn's
// `<Button variant="outline" size="icon">`), so it is the icon Button's 32px square with a 16px icon,
// as tall as the "Assistant" button it is joined to.
test("popover trigger is an outline icon Button beside its group sibling", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
  const frame = page.locator("#component-preview-frame-popover").first();
  const trigger = frame.getByRole("button", { name: "Assistant options" });
  await expect(trigger).toBeVisible({ timeout: 60_000 });
  const box = await trigger.evaluate((e) => {
    const r = e.getBoundingClientRect();
    const icon = e.querySelector("svg")!.getBoundingClientRect();
    return { w: r.width, h: r.height, iconW: icon.width, iconH: icon.height };
  });
  const siblingH = await frame
    .getByRole("button", { name: "Assistant", exact: true })
    .evaluate((e) => e.getBoundingClientRect().height);
  expect(box.w).toBeCloseTo(32, 0);
  expect(box.h).toBeCloseTo(32, 0);
  expect(box.iconW).toBeCloseTo(16, 0);
  expect(box.iconH).toBeCloseTo(16, 0);
  expect(box.h).toBeCloseTo(siblingH, 0);
});

// Joined children sit edge to edge and a follower leaves out its start edge, so each seam is the previous child's
// end edge painted once and, on the RENDERED page, the same colour as the group's outer edge. Dark edges and fills
// are translucent, so a second layer on the seam would draw it heavier; the check reads screenshot pixels because
// only the composite shows it.
test.describe("Seams match the outer edge", () => {
  for (const mode of ["light", "dark"] as const) {
    test(`${mode}: every seam pixel equals the outer edge pixel`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/button_group/`, { timeout: 20 * 60 * 1000 });
      await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
      await page.mouse.move(0, 0);
      const groups = page.locator(
        "#component-preview-frame .dx-button-group:not([data-orientation='vertical']):not(:has(.dx-button-group, .dx-button-group-separator))",
      );
      expect(await groups.count()).toBeGreaterThan(0);
      for (const group of await groups.all()) {
        await group.scrollIntoViewIfNeeded();
        const geo = await group.evaluate((g) => {
          const r = g.getBoundingClientRect();
          return {
            h: Math.round(r.height),
            // The seam is the previous child's end edge: the column just before a follower (LTR) or just after it (RTL).
            seams: [...g.children].slice(1).map((c) => {
              const cr = c.getBoundingClientRect();
              return getComputedStyle(g).direction === "rtl" ? Math.round(cr.right - r.left) : Math.round(cr.left - r.left) - 1;
            }),
          };
        });
        const png = (await group.screenshot({ animations: "disabled" })).toString("base64");
        const cols = await page.evaluate(
          async ({ png, geo }) => {
            const bmp = await createImageBitmap(await (await fetch(`data:image/png;base64,${png}`)).blob());
            const ctx = new OffscreenCanvas(bmp.width, bmp.height).getContext("2d")!;
            ctx.drawImage(bmp, 0, 0);
            const at = (x: number) => Array.from(ctx.getImageData(x, Math.floor(geo.h / 2), 1, 1).data.slice(0, 3));
            return { outer: at(0), seams: geo.seams.map(at) };
          },
          { png, geo },
        );
        for (const seam of cols.seams) {
          seam.forEach((v, i) => expect(Math.abs(v - cols.outer[i]), `seam ${seam} vs outer ${cols.outer}`).toBeLessThanOrEqual(2));
        }
      }
    });
  }
});
