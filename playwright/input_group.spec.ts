import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/input_group/`, { timeout: 20 * 60 * 1000 });
  const search = page.getByPlaceholder("Search...");
  await expect(search).toBeVisible();
  await search.fill("hello");
  await expect(search).toHaveValue("hello");

  await expect(page.getByText("$", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Convert" })).toBeVisible();
});

test.describe("Axe automated scan", () => {
  // Input Group has no overlay/expand/select interaction -- one state to scan.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/input_group/`, { timeout: 20 * 60 * 1000 });
    await expectNoAxeViolations(page, "input_group: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

/*
 * The wrapped input carries no border/radius of its own -- regression
 * guard for a real bug found and fixed 2026-09-14. `Input`'s component
 * used to render `class: "dx-input"` as a literal RSX attribute and then
 * spread `..attributes` after it; Dioxus's plain "later duplicate
 * attribute wins" rule meant `InputGroupInput`'s own `class:
 * "dx-input-group-control"` fully *replaced* `"dx-input"` rather than
 * being combined with it, so the rendered `<input>` carried only
 * `"dx-input-group-control"` and never got `"dx-input"`'s own UA-style
 * reset. `.dx-input-group .dx-input-group-control`'s
 * `border-radius: 0; box-shadow: none;` rule (this file's own style.css)
 * never had anything to override at the browser's *native* default input
 * chrome, which showed through inside `.dx-input-group`'s own CSS-drawn
 * border -- a visible double-border/"pill within a pill" seam. Fixed by
 * having `Input` compose its base class via `merge_attributes` (which
 * concatenates `class` values space-joined) instead of a plain spread.
 * Asserts both halves directly: the class union survived the merge, and
 * the *effect* that class union is supposed to produce (no border/radius
 * of the input's own) actually holds.
 */
test("the input inside the group keeps its base class and shows no border of its own", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/input_group/`, { timeout: 20 * 60 * 1000 });
  const search = page.getByPlaceholder("Search...");
  await expect(search).toBeVisible();

  const className = await search.evaluate((el) => el.className);
  expect(className.split(/\s+/)).toEqual(expect.arrayContaining(["dx-input", "dx-input-group-control"]));

  const style = await search.evaluate((el) => {
    const cs = getComputedStyle(el);
    return { boxShadow: cs.boxShadow, borderRadius: cs.borderRadius };
  });
  expect(style.boxShadow, JSON.stringify(style)).toBe("none");
  expect(parseFloat(style.borderRadius), JSON.stringify(style)).toBe(0);
});

/*
 * Invalid and disabled (backlog row 133). The wrapper draws the field chrome and its control is
 * stripped bare, so the WRAPPER must carry both states: the shared invalid recipe
 * (`--dx-ring-invalid-field`, the same token `Input` draws on itself) and Nova's
 * `has-disabled:opacity-50` (`--dx-opacity-disabled`) so the addons dim with the control. The
 * control itself must draw neither a second time. The demo has no invalid or disabled case, so
 * the state is applied to the existing "Search..." group the way `Field` / a form would
 * (`aria-invalid` / `disabled` on the control); the rules are `:has()` selectors on the
 * attributes, so setting them directly is the same input to the stylesheet.
 */
test.describe("Invalid and disabled states (row 133)", () => {
  const open = async (page: import("@playwright/test").Page) => {
    await gotoHydrated(page, `${BASE_URL}/component/input_group/`, { timeout: 20 * 60 * 1000 });
    const search = page.getByPlaceholder("Search...");
    await expect(search).toBeVisible();
    const wrapper = page.locator(".dx-input-group", { has: search }).first();
    await expect(wrapper).toBeVisible();
    return { search, wrapper };
  };

  /** What a custom-property expression resolves to as a computed property, via a probe element. */
  const resolved = (page: import("@playwright/test").Page, property: "boxShadow" | "opacity", expr: string) =>
    page.evaluate(
      ([prop, value]) => {
        const probe = document.createElement("i");
        (probe.style as unknown as Record<string, string>)[prop] = value;
        document.body.append(probe);
        const out = (getComputedStyle(probe) as unknown as Record<string, string>)[prop];
        probe.remove();
        return out;
      },
      [property, expr] as const,
    );

  test("an invalid control puts the invalid recipe on the wrapper, not on the bare control", async ({ page }) => {
    const { search, wrapper } = await open(page);
    const resting = await wrapper.evaluate((el) => getComputedStyle(el).boxShadow);
    await search.evaluate((el) => el.setAttribute("aria-invalid", "true"));

    const expected = await resolved(page, "boxShadow", "var(--dx-ring-invalid-field)");
    await expect.poll(() => wrapper.evaluate((el) => getComputedStyle(el).boxShadow)).toBe(expected);
    expect(expected, "the invalid recipe is not the resting edge").not.toBe(resting);
    expect(await search.evaluate((el) => getComputedStyle(el).boxShadow), "the bare control draws no ring of its own").toBe(
      "none",
    );
  });

  test("a disabled control dims the whole wrapper once, and the control does not dim again", async ({ page }) => {
    const { search, wrapper } = await open(page);
    expect(await wrapper.evaluate((el) => getComputedStyle(el).opacity)).toBe("1");
    await search.evaluate((el) => ((el as HTMLInputElement).disabled = true));

    const expected = await resolved(page, "opacity", "var(--dx-opacity-disabled)");
    await expect.poll(() => wrapper.evaluate((el) => getComputedStyle(el).opacity)).toBe(expected);
    expect(expected).not.toBe("1");
    expect(await search.evaluate((el) => getComputedStyle(el).opacity), "the control stays at full opacity inside the dimmed wrapper").toBe(
      "1",
    );
  });
});
