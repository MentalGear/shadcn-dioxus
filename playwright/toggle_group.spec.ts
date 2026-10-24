import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// The main demo is `#component-preview-frame`; variants carry `-<name>` suffixes.
const mainDemo = (page: Page) => page.locator("#component-preview-frame");

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
  const b_button = mainDemo(page).getByRole("button", { name: "Toggle bold", exact: true });
  const i_button = mainDemo(page).getByRole("button", { name: "Toggle italic", exact: true });
  const u_button = mainDemo(page).getByRole("button", { name: "Toggle strikethrough", exact: true });

  // The buttons should not be selected initially
  await expect(b_button).toHaveAttribute("data-state", "off");
  await expect(i_button).toHaveAttribute("data-state", "off");
  await expect(u_button).toHaveAttribute("data-state", "off");

  // Click the "Toggle bold" button and check its state
  await b_button.click();
  await expect(b_button).toHaveAttribute("data-state", "on");

  // Pressing right arrow should focus the "Toggle italic" button
  await page.keyboard.press("ArrowRight");
  await expect(i_button).toBeFocused();

  // Pressing enter should press the "Toggle italic" button
  await page.keyboard.press("Enter");
  await expect(i_button).toHaveAttribute("data-state", "on");

  // Pressing right two more times should bring us back to "Toggle bold"
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  await expect(b_button).toBeFocused();
});

// HARDENING (dev-docs/backlog.md row 89) -- see
// `toggle.spec.ts`'s identical test for the full writeup (same
// `:where(:hover)` construction: correct via specificity, not source order).
for (const dark of [false, true]) {
  test(`a pressed item keeps its "on" background while hovered (${dark ? "dark" : "light"} mode)`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/toggle_group/${dark ? "?dark_mode=true" : ""}`);
    const bButton = mainDemo(page).getByRole("button", { name: "Toggle bold", exact: true });

    await bButton.click();
    await expect(bButton).toHaveAttribute("data-state", "on");
    await page.locator("body").hover({ position: { x: 0, y: 0 } });
    // Wait past the CSS transition (--dx-motion-duration-slow, 200ms)
    // before reading computed style -- a mid-transition read returns an
    // interpolated, neither-old-nor-new color.
    await page.waitForTimeout(400);
    const onAtRest = await bButton.evaluate((el) => getComputedStyle(el).backgroundColor);

    await bButton.hover();
    await page.waitForTimeout(400);
    expect(await bButton.evaluate((el) => el.matches(":hover"))).toBe(true);
    const onAndHovered = await bButton.evaluate((el) => getComputedStyle(el).backgroundColor);

    // The invariant the `:where()` construction makes durable (pressed keeps its own
    // background under the pointer), not a particular colour -- the pressed
    // colour is derived from the preset-aware role tokens.
    expect(onAndHovered, `onAtRest=${onAtRest} onAndHovered=${onAndHovered}`).toBe(onAtRest);
  });
}

// The pressed background is derived from the preset-aware role tokens (not the ramp step
// `--primary-color-7`, which is grey under every base), so under a TINTED base it must (a) carry the tint and
// (b) still be three visibly different backgrounds: resting, hovered, pressed. The exhaustive base x
// accent x mode proof is `theme-preset-contrast.spec.ts`; this keeps the same assertion next to the
// component's own specs.
const GAP_PRESSED = 16; // per channel, 0-255
const GAP_HOVER = 6;

async function paintedRgb(page: Page, css: string): Promise<[number, number, number]> {
  return page.evaluate((c) => {
    const cx = document.createElement("canvas").getContext("2d", { willReadFrequently: true })!;
    cx.canvas.width = cx.canvas.height = 1;
    cx.fillStyle = c;
    cx.fillRect(0, 0, 1, 1);
    const d = cx.getImageData(0, 0, 1, 1).data;
    return [d[0], d[1], d[2]] as [number, number, number];
  }, css);
}

const gap = (a: number[], b: number[]) => Math.max(...a.map((v, i) => Math.abs(v - b[i])));

for (const dark of [false, true]) {
  for (const base of ["slate", "gray"]) {
    test(`resting, hovered and pressed items are three different backgrounds under a ${base} base (${dark ? "dark" : "light"} mode)`, async ({ page }) => {
      // Settled first: a late (re)render of the page would drop the attribute set below.
      await gotoHydrated(page, `${BASE_URL}/component/toggle_group/${dark ? "?dark_mode=true" : ""}`);
      await page.evaluate((b) => {
        document.documentElement.setAttribute("data-theme-base", b);
        // Settled values, not a mid-transition read.
        document.head.insertAdjacentHTML("beforeend", "<style>*{transition:none!important}</style>");
      }, base);
      const item = (name: string) => mainDemo(page).getByRole("button", { name, exact: true });
      const paint = async (name: string) =>
        paintedRgb(page, await item(name).evaluate((el) => getComputedStyle(el).backgroundColor));

      // Bold pressed, italic hovered, strikethrough resting: all three states at once.
      await item("Toggle bold").click();
      await expect(item("Toggle bold")).toHaveAttribute("data-state", "on");
      await item("Toggle italic").hover();
      expect(await item("Toggle italic").evaluate((el) => el.matches(":hover"))).toBe(true);
      const pressed = await paint("Toggle bold");
      const hovered = await paint("Toggle italic");
      const rest = await paint("Toggle strikethrough");

      const msg = `rest=${rest} hovered=${hovered} pressed=${pressed}`;
      expect(gap(pressed, hovered), msg).toBeGreaterThanOrEqual(GAP_PRESSED);
      expect(gap(pressed, rest), msg).toBeGreaterThanOrEqual(GAP_PRESSED);
      expect(gap(hovered, rest), msg).toBeGreaterThanOrEqual(GAP_HOVER);
      // The tint reaches the pressed state: the ramp step is pure grey (r == g == b) under every base.
      expect(new Set(pressed).size, `pressed ${pressed} is neutral grey, not ${base}-tinted`).toBeGreaterThan(1);
    });
  }
}

// Row 155 group 3: `variant`, `size` and `spacing` on ToggleGroup (shadcn: default spacing 2, `0` joins).
const frameOf = (page: Page, name: string) => page.locator(`#component-preview-frame-${name}`);
const group = (page: Page, name: string, label: string) => frameOf(page, name).getByRole("group", { name: label });
const num = (el: import("@playwright/test").Locator, prop: string) =>
  el.evaluate((e, p) => parseFloat(getComputedStyle(e).getPropertyValue(p)), prop);
const box = async (el: import("@playwright/test").Locator) => (await el.boundingBox())!;

test("the main demo is shadcn's: an outline, multiple-select group of icon items, spaced 8px apart", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
  const root = mainDemo(page).locator(".dx-toggle-group");
  await expect(root).toHaveAttribute("data-variant", "outline");
  await expect(root).toHaveAttribute("data-spacing", "2");
  const [b, i] = [
    await box(mainDemo(page).getByRole("button", { name: "Toggle bold", exact: true })),
    await box(mainDemo(page).getByRole("button", { name: "Toggle italic", exact: true })),
  ];
  expect(i.x - (b.x + b.width), "default spacing 2 = 8px between separate toggles").toBeCloseTo(8, 0);
  expect(await num(mainDemo(page).getByRole("button", { name: "Toggle bold", exact: true }), "border-top-width")).toBe(1);
});

test("spacing 2 keeps separate rounded toggles; spacing 0 joins them into one bar", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
  const spaced = group(page, "spacing", "Spaced placement").getByRole("button");
  const joined = group(page, "spacing", "Joined placement").getByRole("button");
  const [s0, s1] = [await box(spaced.nth(0)), await box(spaced.nth(1))];
  expect(s1.x - (s0.x + s0.width)).toBeCloseTo(8, 0);
  const r = (el: import("@playwright/test").Locator, p: string) => num(el, p);
  // Spaced: every item is a full toggle with the large radius on all four corners.
  const full = await r(spaced.nth(1), "border-top-left-radius");
  expect(full).toBeGreaterThan(0);
  expect(await r(spaced.nth(1), "border-top-right-radius")).toBe(full);

  const [j0, j1, j2] = [await box(joined.nth(0)), await box(joined.nth(1)), await box(joined.nth(2))];
  expect(j1.x - (j0.x + j0.width), "joined items touch").toBeCloseTo(0, 0);
  expect(j2.x - (j1.x + j1.width), "joined items touch").toBeCloseTo(0, 0);
  // Square inner corners, radius only on the two ends of the bar.
  expect(await r(joined.nth(1), "border-top-left-radius")).toBe(0);
  expect(await r(joined.nth(1), "border-top-right-radius")).toBe(0);
  expect(await r(joined.nth(0), "border-top-left-radius")).toBeGreaterThan(0);
  expect(await r(joined.nth(0), "border-top-right-radius")).toBe(0);
  expect(await r(joined.nth(3), "border-top-right-radius")).toBeGreaterThan(0);
  expect(await r(joined.nth(3), "border-top-left-radius")).toBe(0);
  // One shared edge: all but the first drop the leading border.
  expect(await r(joined.nth(0), "border-left-width")).toBe(1);
  expect(await r(joined.nth(1), "border-left-width")).toBe(0);
  expect(await r(joined.nth(1), "border-right-width")).toBe(1);
  // Joined items use px-2 (8px) where separate ones use px-2.5 (10px).
  expect(await r(joined.nth(1), "padding-left")).toBe(8);
  expect(await r(spaced.nth(1), "padding-left")).toBe(10);
});

test("sizes: sm and default groups are 28 and 32px high", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
  const sm = group(page, "sizes", "Small placement").getByRole("button");
  const def = group(page, "sizes", "Default placement").getByRole("button");
  expect(await num(sm.first(), "height")).toBe(28);
  expect(await num(def.first(), "height")).toBe(32);
  await expect(group(page, "sizes", "Small placement")).toHaveAttribute("data-size", "sm");
  // Single-select, "Top" pressed to start with.
  await expect(sm.first()).toHaveAttribute("data-state", "on");
  await sm.nth(1).click();
  await expect(sm.nth(1)).toHaveAttribute("data-state", "on");
  await expect(sm.first()).toHaveAttribute("data-state", "off");
});

test("outline: a single-select group with All pressed", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
  const all = frameOf(page, "outline").getByRole("button", { name: "Toggle all", exact: true });
  const missed = frameOf(page, "outline").getByRole("button", { name: "Toggle missed", exact: true });
  await expect(all).toHaveAttribute("data-state", "on");
  await missed.click();
  await expect(missed).toHaveAttribute("data-state", "on");
  await expect(all).toHaveAttribute("data-state", "off");
});

test("vertical: a column with spacing 1 (4px), navigated with ArrowUp/ArrowDown", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
  const root = group(page, "vertical", "Vertical text formatting");
  await expect(root).toHaveAttribute("data-orientation", "vertical");
  expect(await root.evaluate((e) => getComputedStyle(e).flexDirection)).toBe("column");
  const items = root.getByRole("button");
  const [a, b] = [await box(items.nth(0)), await box(items.nth(1))];
  expect(a.x).toBeCloseTo(b.x, 0);
  expect(b.y - (a.y + a.height)).toBeCloseTo(4, 0);
  // Bold and italic start pressed.
  await expect(items.nth(0)).toHaveAttribute("data-state", "on");
  await expect(items.nth(1)).toHaveAttribute("data-state", "on");
  await expect(items.nth(2)).toHaveAttribute("data-state", "off");
  await items.nth(0).focus();
  await page.keyboard.press("ArrowDown");
  await expect(items.nth(1)).toBeFocused();
  await page.keyboard.press("ArrowRight");
  await expect(items.nth(1), "ArrowRight is not the vertical axis").toBeFocused();
});

test("font weight: a controlled single-select of 4rem tiles that names the chosen font-* class", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
  const root = group(page, "font_weight", "Font Weight");
  const tile = (n: string) => root.getByRole("button", { name: n, exact: true });
  await expect(tile("Normal")).toHaveAttribute("data-state", "on");
  expect(await num(tile("Normal"), "height")).toBe(64);
  expect(await num(tile("Normal"), "width")).toBe(64);
  await expect(frameOf(page, "font_weight")).toContainText("font-normal");
  await tile("Bold").click();
  await expect(tile("Bold")).toHaveAttribute("data-state", "on");
  await expect(tile("Normal")).toHaveAttribute("data-state", "off");
  await expect(frameOf(page, "font_weight")).toContainText("font-bold");
  // Single-select: pressing the chosen tile again keeps it chosen.
  await tile("Bold").click();
  await expect(tile("Bold")).toHaveAttribute("data-state", "on");
});

test.describe("Axe automated scan", () => {
  test("an item selected has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
    await mainDemo(page).getByRole("button", { name: "Toggle bold", exact: true }).click();
    await expectNoAxeViolations(page, "toggle_group: B selected", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// Row 364: the pressed set is keyed by item `value`, so reordering the items or inserting one before them never
// moves the pressed state to another button.
test.describe("pressed state follows the item value (row 364)", () => {
  const frame = (page: Page) => frameOf(page, "reorder");
  const multi = (page: Page, v: string) => frame(page).getByRole("group", { name: "Reorderable multiple" }).getByRole("button", { name: `multi ${v}`, exact: true });
  const single = (page: Page, v: string) => frame(page).getByRole("group", { name: "Reorderable single" }).getByRole("button", { name: `single ${v}`, exact: true });
  const order = async (page: Page) =>
    frame(page).getByRole("group", { name: "Reorderable multiple" }).getByRole("button").evaluateAll((els) => els.map((e) => e.getAttribute("aria-label")));

  test("uncontrolled multiple: default and clicked items stay pressed across reverse and insert", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
    await expect(multi(page, "italic")).toHaveAttribute("data-state", "on"); // default_value
    await multi(page, "bold").click();
    await expect(multi(page, "bold")).toHaveAttribute("data-state", "on");
    await expect(multi(page, "underline")).toHaveAttribute("data-state", "off");

    await frame(page).getByRole("button", { name: "Reverse order", exact: true }).click();
    await expect.poll(() => order(page)).toEqual(["multi underline", "multi italic", "multi bold"]);
    await expect(multi(page, "bold")).toHaveAttribute("data-state", "on");
    await expect(multi(page, "italic")).toHaveAttribute("data-state", "on");
    await expect(multi(page, "underline")).toHaveAttribute("data-state", "off");

    await frame(page).getByRole("button", { name: "Insert first", exact: true }).click();
    await expect.poll(() => order(page)).toEqual(["multi new1", "multi underline", "multi italic", "multi bold"]);
    await expect(multi(page, "new1")).toHaveAttribute("data-state", "off");
    await expect(multi(page, "bold")).toHaveAttribute("data-state", "on");
    await expect(multi(page, "italic")).toHaveAttribute("data-state", "on");
    await expect(multi(page, "underline")).toHaveAttribute("data-state", "off");

    // Still toggles by value afterwards (multiple: several may be pressed).
    await multi(page, "new1").click();
    await expect(multi(page, "new1")).toHaveAttribute("data-state", "on");
    await expect(multi(page, "bold")).toHaveAttribute("data-state", "on");
  });

  test("controlled single: the chosen value follows the item, replaces rather than adds", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/toggle_group/`);
    await expect(single(page, "italic")).toHaveAttribute("data-state", "on");
    await frame(page).getByRole("button", { name: "Insert first", exact: true }).click();
    await frame(page).getByRole("button", { name: "Reverse order", exact: true }).click();
    await expect(single(page, "italic")).toHaveAttribute("data-state", "on");
    await expect(single(page, "bold")).toHaveAttribute("data-state", "off");
    await single(page, "bold").click();
    await expect(single(page, "bold")).toHaveAttribute("data-state", "on");
    await expect(single(page, "italic")).toHaveAttribute("data-state", "off");
    await expect(frame(page).getByTestId("toggle-group-chosen")).toHaveText("Chosen: bold");
    await frame(page).getByRole("button", { name: "Insert first", exact: true }).click();
    await expect(single(page, "bold")).toHaveAttribute("data-state", "on");
    await expect(single(page, "new2")).toHaveAttribute("data-state", "off");
  });
});
