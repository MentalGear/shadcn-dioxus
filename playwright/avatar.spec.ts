import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/avatar/`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes
  // The demo avatars are bundled geometric SVGs (preview/assets/avatars), not
  // hot-linked photos: the src is the hashed local asset and it must actually load.
  const image = page.locator('img[alt="Avery Lin"]').first();
  await expect(image).toHaveAttribute("src", /\/avery-lin[^/]*\.svg$/);
  await expect(page.getByLabel("Basic avatar")).toHaveAttribute("data-state", "loaded");

  // The main demo is shadcn's avatar-demo: a grayscale avatar, one with a green badge, a grayscale group ending in "+3".
  const demo = page.locator("#component-preview-frame").first();
  await expect(demo.locator(".dx-avatar-badge")).toHaveCount(1);
  expect(await demo.locator(".dx-avatar-image").first().evaluate((el) => getComputedStyle(el).filter)).toBe("grayscale(1)");
  await expect(demo.getByLabel("Team").locator(".dx-avatar-group-count")).toHaveText("+3");

  // A broken image falls back to the initials.
  await expect(page.getByLabel("Error avatar").getByText("JR")).toBeVisible();
});

test.describe("Axe automated scan", () => {
  // Avatar has no overlay/expand/select interaction -- one state to scan.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/avatar/`, { timeout: 20 * 60 * 1000 });
    await expectNoAxeViolations(page, "avatar: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// `AvatarBadge`, `AvatarGroup`, `AvatarGroupCount` (shadcn's small avatar parts, backlog row 155 group 9).
test.describe("badge and group", () => {
  test("the badge sits on the end-bottom corner, outside the circle, and scales with the avatar", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/avatar/`, { timeout: 20 * 60 * 1000 });
    const dot = page.getByLabel("Avery Lin, online");
    await expect(dot).toBeVisible();
    // The root does not clip: a clip would cut the badge, which sits on the corner of the square.
    expect(await dot.evaluate((el) => getComputedStyle(el).overflow)).toBe("visible");
    const badge = dot.locator(".dx-avatar-badge");
    const a = (await dot.boundingBox())!;
    const b = (await badge.boundingBox())!;
    expect(Math.abs(a.x + a.width - (b.x + b.width))).toBeLessThan(1);
    expect(Math.abs(a.y + a.height - (b.y + b.height))).toBeLessThan(1);
    expect(Math.round(b.width)).toBe(10);
    // Painted above the picture (hit-test the badge's centre, in the viewport).
    await badge.scrollIntoViewIfNeeded();
    expect(
      await badge.evaluate((el) => {
        const r = el.getBoundingClientRect();
        return document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2) === el;
      }),
    ).toBe(true);

    const small = page.getByLabel("Small avatar with badge").locator(".dx-avatar-badge");
    const large = page.getByLabel("Large avatar with badge").locator(".dx-avatar-badge");
    expect(Math.round((await small.boundingBox())!.width)).toBe(8);
    expect(Math.round((await large.boundingBox())!.width)).toBe(12);
    // An icon is hidden in the small avatar and 8px in the others.
    await expect(small.locator("svg")).toBeHidden();
    expect(Math.round((await large.locator("svg").boundingBox())!.width)).toBe(8);
  });

  test("the group overlaps its avatars by 8px, rings them, and the count tile matches their size", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/avatar/`, { timeout: 20 * 60 * 1000 });
    const group = page.getByLabel("Team with a count");
    await expect(group).toBeVisible();
    const avatars = group.locator(".dx-avatar");
    await expect(avatars).toHaveCount(3);
    const first = (await avatars.nth(0).boundingBox())!;
    const second = (await avatars.nth(1).boundingBox())!;
    expect(Math.round(first.x + first.width - second.x)).toBe(8);
    expect(await avatars.nth(1).evaluate((el) => getComputedStyle(el).boxShadow)).toMatch(/0px 0px 0px 2px/);

    const count = group.locator(".dx-avatar-group-count");
    await expect(count).toHaveText("+3");
    const c = (await count.boundingBox())!;
    expect(Math.round(c.width)).toBe(32);
    expect(Math.round(c.height)).toBe(32);
    const third = (await avatars.nth(2).boundingBox())!;
    expect(Math.round(third.x + third.width - c.x)).toBe(8);
    // The icon variant holds an svg.
    await expect(page.getByLabel("Team with an icon tile").locator(".dx-avatar-group-count svg")).toBeVisible();
  });

  test("the picture and the fallback keep the circle (children round themselves, the root does not clip)", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/avatar/`, { timeout: 20 * 60 * 1000 });
    const image = page.getByLabel("Avery Lin, online").locator(".dx-avatar-image");
    const radius = await image.evaluate((el) => getComputedStyle(el).borderTopLeftRadius);
    expect(radius).toBe("50%");
  });
});
