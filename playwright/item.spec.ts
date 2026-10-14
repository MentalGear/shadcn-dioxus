import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";

// Backlog row 155 group 7: `size="xs"` on Item (shadcn's `cn-item-size-xs`: `gap-2 px-2.5 py-2`, a `text-xs`
// description, and `gap-2` between items of a group); `default` and `sm` share `gap-2.5 px-3 py-2.5`.
test.describe("Item sizes", () => {
  test("xs is tighter than sm and default, with a text-xs description", async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=item&`, { timeout: 20 * 60 * 1000, waitUntil: "networkidle" });
    const frame = page.locator("#component-preview-frame-size");
    const read = (size: string) =>
      frame.locator(`.dx-item[data-size="${size}"]`).evaluate((el) => {
        const s = getComputedStyle(el);
        const d = getComputedStyle(el.querySelector(".dx-item-description")!);
        return {
          paddingBlock: parseFloat(s.paddingTop),
          paddingInline: parseFloat(s.paddingLeft),
          gap: parseFloat(s.columnGap),
          descriptionSize: parseFloat(d.fontSize),
        };
      });
    const xs = await read("xs");
    expect(xs).toEqual({ paddingBlock: 8, paddingInline: 10, gap: 8, descriptionSize: 12 });
    const sm = await read("sm");
    const def = await read("default");
    expect(sm).toEqual({ paddingBlock: 10, paddingInline: 12, gap: 10, descriptionSize: 14 });
    expect(def).toEqual(sm);
  });

  test("xs items are shorter than default ones", async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=item&`, { timeout: 20 * 60 * 1000, waitUntil: "networkidle" });
    const frame = page.locator("#component-preview-frame-size");
    const h = async (size: string) => (await frame.locator(`.dx-item[data-size="${size}"]`).boundingBox())!.height;
    expect(await h("xs")).toBeLessThan(await h("default"));
  });
});
