import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const SIDEBAR_RENDER_TIMEOUT = 30 * 1000;

async function gotoSidebarBlock(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/component/block/sidebar/main/`, {
    timeout: 20 * 60 * 1000,
    waitUntil: 'load'
  });

  await expect(page.locator('[data-slot="sidebar-wrapper"]')).toBeVisible({
    timeout: SIDEBAR_RENDER_TIMEOUT,
  });
}

test("sidebar: preview page renders block", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/sidebar/`, {
    timeout: 20 * 60 * 1000,
    waitUntil: 'load'
  });
  const iframe = page.locator("iframe").first();
  await expect(iframe).toBeVisible({ timeout: SIDEBAR_RENDER_TIMEOUT });
  // Path-segment form (dev-docs/backlog.md row 46): `preview/src/main.rs`'s
  // `BlockComponentVariantHighlight` builds this iframe's `src` from the
  // canonical, SSG-enumerable `Route::ComponentBlockDemoPath`
  // (`/component/block/<name>/<variant>/`). The query form
  // (`?name=...&variant=...`) redirects client-side to this same path.
  await expect(iframe).toHaveAttribute(
    "src",
    /component\/block\/sidebar\/main\//,
    { timeout: SIDEBAR_RENDER_TIMEOUT },
  );

  await expect(
    page.frameLocator("iframe").first().locator('[data-slot="sidebar-wrapper"]'),
  ).toBeVisible({ timeout: SIDEBAR_RENDER_TIMEOUT });
});

test.describe("sidebar: block route", () => {
  test("desktop: toggles via button and Ctrl+B", async ({ page }) => {
    await gotoSidebarBlock(page);

    const sidebar = page.locator('[data-slot="sidebar"]:not([data-mobile="true"])');
    await expect(sidebar).toHaveAttribute("data-state", "expanded");
    const trigger = page.locator('[data-slot="sidebar-trigger"]');
    await expect(trigger).toHaveAccessibleName("Toggle Sidebar");

    // Toggle via button.
    await trigger.click();
    await expect(sidebar).toHaveAttribute("data-state", "collapsed");
    await trigger.click();
    await expect(sidebar).toHaveAttribute("data-state", "expanded");

    // Toggle via keyboard shortcut (⌘/Ctrl+B).
    await page.keyboard.press("Control+b");
    await expect(sidebar).toHaveAttribute("data-state", "collapsed");
    await page.keyboard.press("Control+b");
    await expect(sidebar).toHaveAttribute("data-state", "expanded");
  });

  test("desktop: Ctrl+B is left to a text field, a contenteditable and a handler that prevented it (row 397)", async ({ page }) => {
    await gotoSidebarBlock(page);
    const sidebar = page.locator('[data-slot="sidebar"]:not([data-mobile="true"])');
    await expect(sidebar).toHaveAttribute("data-state", "expanded");
    await page.evaluate(() => {
      const input = Object.assign(document.createElement("input"), { id: "probe-input", ariaLabel: "probe" });
      const editor = Object.assign(document.createElement("div"), { id: "probe-editor", contentEditable: "true" });
      const bold = Object.assign(document.createElement("button"), { id: "probe-bold", textContent: "probe" });
      // An editor's own Ctrl+B: handled nearer the target, so the page-level shortcut stays out.
      bold.addEventListener("keydown", (event) => event.preventDefault());
      document.body.append(input, editor, bold);
    });

    for (const id of ["probe-input", "probe-editor", "probe-bold"]) {
      await page.locator(`#${id}`).focus();
      const prevented = await page.evaluate(() => {
        const event = new KeyboardEvent("keydown", { key: "b", ctrlKey: true, bubbles: true, cancelable: true });
        // Dispatched on the focused element, the way a real key press starts.
        document.activeElement!.dispatchEvent(event);
        return event.defaultPrevented;
      });
      if (id !== "probe-bold") expect(prevented, `${id}: the key is not swallowed`).toBe(false);
      await page.keyboard.press("Control+b");
      await expect(sidebar, `${id}: Ctrl+B does not toggle the sidebar`).toHaveAttribute("data-state", "expanded");
    }

    // Outside an editable target the shortcut still toggles.
    await page.evaluate(() => (document.activeElement as HTMLElement).blur());
    await page.keyboard.press("Control+b");
    await expect(sidebar).toHaveAttribute("data-state", "collapsed");
  });

  test("desktop: side switch updates data-side", async ({ page }) => {
    await gotoSidebarBlock(page);

    const sidebar = page.locator('[data-slot="sidebar"]:not([data-mobile="true"])');
    await expect(sidebar).toHaveAttribute("data-side", "left");

    await page.getByRole("button", { name: "Right" }).click();
    await expect(sidebar).toHaveAttribute("data-side", "right");
    await page.getByRole("button", { name: "Left" }).click();
    await expect(sidebar).toHaveAttribute("data-side", "left");
  });

  // `collapsible: SidebarCollapsible::None` renders a parallel `.dx-sidebar-static`
  // element, which must set `data-side` too, or the side-mirroring CSS rule
  // (`.dx-sidebar-wrapper:has([data-slot="sidebar"]
  // [data-side="right"]) { flex-direction: row-reverse; }`) never matches --
  // the "Right" control would have no visible effect while "None" is selected, the
  // sidebar always rendering flush against the left edge. Geometry-based per
  // this repo's convention (see select.spec.ts's "Trigger minimum width").
  test("desktop: side switch mirrors sidebar with collapsible=none", async ({ page }) => {
    await gotoSidebarBlock(page);

    const wrapper = page.locator('[data-slot="sidebar-wrapper"]');
    const sidebar = page.locator('[data-slot="sidebar"]:not([data-mobile="true"])');
    const inset = page.locator('[data-slot="sidebar-inset"]');

    await page.getByRole("button", { name: "None" }).click();
    await expect(sidebar).toHaveAttribute("data-side", "left");

    const leftBox = await sidebar.boundingBox();
    expect(leftBox).not.toBeNull();
    expect(leftBox!.x).toBeCloseTo(0, 0);

    await page.getByRole("button", { name: "Right" }).click();
    await expect(sidebar).toHaveAttribute("data-side", "right");

    const rightBox = await sidebar.boundingBox();
    const insetBox = await inset.boundingBox();
    expect(rightBox).not.toBeNull();
    expect(insetBox).not.toBeNull();
    // Sidebar must have moved to the right edge of ITS OWN wrapper (not
    // page.viewportSize() / documentElement.clientWidth -- this route's
    // <body> renders a few px narrower than the outer viewport for
    // unrelated reasons, so the wrapper is the only trustworthy width
    // reference), and the main content (inset) must now occupy the left
    // side, starting at 0 -- i.e. the two must have swapped, not stayed put.
    const wrapperBox = await wrapper.boundingBox();
    expect(wrapperBox).not.toBeNull();
    expect(rightBox!.x + rightBox!.width).toBeCloseTo(wrapperBox!.x + wrapperBox!.width, 0);
    expect(insetBox!.x).toBeCloseTo(wrapperBox!.x, 0);
    expect(insetBox!.x + insetBox!.width).toBeCloseTo(rightBox!.x, 0);

    await page.getByRole("button", { name: "Left" }).click();
    await expect(sidebar).toHaveAttribute("data-side", "left");
  });

  test("desktop: icon collapse shows tooltip on focus and preserves accessible names", async ({
    page,
  }) => {
    await gotoSidebarBlock(page);

    const sidebar = page.locator('[data-slot="sidebar"]:not([data-mobile="true"])');
    const trigger = page.locator('[data-slot="sidebar-trigger"]');

    await page.getByRole("button", { name: "Icon" }).click();
    await trigger.click();

    await expect(sidebar).toHaveAttribute("data-state", "collapsed");
    await expect(sidebar).toHaveAttribute("data-collapsible", "icon");

    // In icon-collapsed mode, tooltips should appear on keyboard focus.
    const playground = page
      .locator('[data-sidebar="menu-button"]')
      .filter({ hasText: "Playground" })
      .first();

    await playground.focus();

    const tooltip = page.getByRole("tooltip");
    await expect(tooltip).toBeVisible();
    await expect(tooltip).toContainText("Playground");

    // Even when labels are visually hidden in icon mode, the control should still have an accessible name.
    await expect(playground).toHaveAccessibleName("Playground");
  });

  test("mobile: opens as a sheet and closes with Escape (focus restored)", async ({
    page,
  }) => {
    await gotoSidebarBlock(page);

    const trigger = page.locator('[data-slot="sidebar-trigger"]');
    await trigger.tap();

    const sheet = page.locator('[data-slot="sheet-root"]');
    await expect(sheet).toHaveAttribute("data-state", "open");
    await page.keyboard.press("Escape");
    await expect(sheet).toHaveCount(0);
    await expect(trigger).toBeFocused();
  });
});

// dev-docs/backlog.md row 38: the `/component/block/
// ?name=sidebar&variant=main&` route (this file's `gotoSidebarBlock`) is
// normally embedded in an `<iframe>` from the main preview page (see the
// "preview page renders block" test above), where a missing page-level
// `<h1>`/`<main>` is a non-issue (the iframe's own document is a separate
// accessibility-tree root). Visiting it directly, as both tests below do,
// would surface `page-has-heading-one`/`region` without them, so
// `ComponentBlockDemo` (`preview/src/main.rs`) renders a
// visually-hidden `<h1>` naming the block, in its own `<header>` landmark;
// and `Sidebar` (`preview/src/components/sidebar/component.rs`) gives
// its own content a `complementary` landmark too, so nothing on the page
// sits outside a landmark. The collapsed scan below and the `all-routes`
// sweep's at-rest scan of the same route (`block-sidebar-main`) pass clean.
test.describe("Axe automated scan", () => {
  test("collapsed has no automatically detectable a11y issues", async ({ page }) => {
    await gotoSidebarBlock(page);
    const sidebar = page.locator('[data-slot="sidebar"]:not([data-mobile="true"])');
    const trigger = page.locator('[data-slot="sidebar-trigger"]');
    await trigger.click();
    await expect(sidebar).toHaveAttribute("data-state", "collapsed");
    await expectNoAxeViolations(page, "sidebar: collapsed", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
