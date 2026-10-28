import { test, expect } from "./fixtures";
import { type Locator, type Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const frame = (page: Page, variant: string) =>
    page.locator(`#component-preview-frame-${variant}`).first();

async function open(page: Page, variant: string, mode: "light" | "dark" = "light") {
    await gotoHydrated(page, `${BASE_URL}/component/combobox/`, GOTO_OPTS);
    await page.evaluate(
        (m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m),
        mode,
    );
    const root = frame(page, variant);
    await root.scrollIntoViewIfNeeded();
    return root;
}

const field = (root: Locator) => root.locator("[data-dx-combobox-chips]");
const chips = (root: Locator) => root.locator("[data-dx-combobox-chip]");
const chipLabels = (root: Locator) => chips(root).locator("> span");
const listbox = (page: Page) => page.locator("[role='listbox'][data-state='open']");

test("chips render the selection; options toggle and the list stays open", async ({ page }) => {
    const root = await open(page, "multiple");
    const input = root.getByRole("combobox", { name: "Choose frameworks" });
    await expect(chipLabels(root)).toHaveText(["Next.js", "Astro"]);
    await expect(input).toHaveValue("");

    await input.click();
    const list = listbox(page);
    await expect(list).toBeVisible();
    await expect(list).toHaveAttribute("aria-multiselectable", "true");
    await expect(list.getByRole("option", { name: "Next.js" })).toHaveAttribute("aria-selected", "true");
    await expect(list.getByRole("option", { name: "Remix" })).toHaveAttribute("aria-selected", "false");

    await list.getByRole("option", { name: "Remix" }).click();
    await expect(list).toBeVisible();
    await expect(chipLabels(root)).toHaveText(["Next.js", "Astro", "Remix"]);
    await expect(list.getByRole("option", { name: "Remix" })).toHaveAttribute("aria-selected", "true");
    await expect(root.getByTestId("combobox-multiple-value")).toHaveText("next,astro,remix");

    // Picking a selected option again removes it.
    await list.getByRole("option", { name: "Next.js" }).click();
    await expect(chipLabels(root)).toHaveText(["Astro", "Remix"]);
    await expect(list).toBeVisible();
});

test("keyboard: type, Enter toggles and keeps the popup open, text is cleared", async ({ page }) => {
    const root = await open(page, "multiple");
    const input = root.getByRole("combobox", { name: "Choose frameworks" });
    await input.click();
    await page.keyboard.type("sve");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await expect(chipLabels(root)).toHaveText(["Next.js", "Astro", "SvelteKit"]);
    await expect(listbox(page)).toBeVisible();
    await expect(input).toHaveValue("");
    await expect(listbox(page).getByRole("option")).toHaveCount(7);
});

test("Home and End move the caret; Escape on a closed list clears every chip", async ({ page }) => {
    const root = await open(page, "multiple");
    const input = root.getByRole("combobox", { name: "Choose frameworks" });
    await input.click();
    await page.keyboard.type("abc");
    await page.keyboard.press("Home");
    expect(await input.evaluate((el: HTMLInputElement) => el.selectionStart), "Home moves the caret to the start").toBe(0);
    await page.keyboard.press("End");
    expect(await input.evaluate((el: HTMLInputElement) => el.selectionStart), "End moves the caret to the end").toBe(3);
    await expect(chipLabels(root), "Home and End leave the selection alone").toHaveText(["Next.js", "Astro"]);

    // Escape on the open list closes it and keeps the chips; the next Escape, on the closed list, clears them.
    await page.keyboard.press("Escape");
    await expect(listbox(page)).toHaveCount(0);
    await expect(chipLabels(root)).toHaveText(["Next.js", "Astro"]);
    await page.keyboard.press("Escape");
    await expect(chips(root)).toHaveCount(0);
    await expect(root.getByTestId("combobox-multiple-value")).toHaveText("");
});

test("only the multi-select list is aria-multiselectable", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/combobox/`, GOTO_OPTS);
    const single = page.getByRole("combobox", { name: "Select framework", exact: true });
    await single.click();
    await expect(listbox(page)).toBeVisible();
    await expect(listbox(page)).not.toHaveAttribute("aria-multiselectable", /.*/);
});

test("Backspace in the empty input removes the last chip; with text it edits", async ({ page }) => {
    const root = await open(page, "multiple");
    const input = root.getByRole("combobox", { name: "Choose frameworks" });
    await input.click();
    await page.keyboard.type("x");
    await page.keyboard.press("Backspace");
    await expect(chipLabels(root)).toHaveText(["Next.js", "Astro"]);
    await page.keyboard.press("Backspace");
    await expect(chipLabels(root)).toHaveText(["Next.js"]);
    await page.keyboard.press("Backspace");
    await expect(chips(root)).toHaveCount(0);
    await page.keyboard.press("Backspace");
    await expect(chips(root)).toHaveCount(0);
});

test("chips are removable by mouse and by keyboard", async ({ page }) => {
    const root = await open(page, "multiple");
    const input = root.getByRole("combobox", { name: "Choose frameworks" });
    await root.getByRole("button", { name: "Remove Next.js" }).click();
    await expect(chipLabels(root)).toHaveText(["Astro"]);
    await expect(input).toBeFocused();

    // Keyboard: arrow back from the empty input onto the chip, Enter removes it.
    await page.keyboard.type("");
    await page.keyboard.press("ArrowLeft");
    const remove = root.getByRole("button", { name: "Remove Astro" });
    await expect(remove).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(chips(root)).toHaveCount(0);
    await expect(input).toBeFocused();
});

test("chip keyboard: Delete removes, arrows move between chips", async ({ page }) => {
    const root = await open(page, "multiple_clear");
    const input = root.getByRole("combobox", { name: "Frameworks with clear" });
    await expect(chipLabels(root)).toHaveText(["SvelteKit", "Remix", "Dioxus"]);
    await input.focus();
    await page.keyboard.press("ArrowLeft");
    await expect(root.getByRole("button", { name: "Remove Dioxus" })).toBeFocused();
    await page.keyboard.press("ArrowLeft");
    await expect(root.getByRole("button", { name: "Remove Remix" })).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect(root.getByRole("button", { name: "Remove Dioxus" })).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect(input).toBeFocused();

    await page.keyboard.press("ArrowLeft");
    await page.keyboard.press("ArrowLeft");
    await page.keyboard.press("Delete");
    await expect(chipLabels(root)).toHaveText(["SvelteKit", "Dioxus"]);
    await expect(input).toBeFocused();
});

test("RTL: the physical keys are mirrored", async ({ page }) => {
    const root = await open(page, "rtl");
    const input = root.getByRole("combobox", { name: "RTL frameworks" });
    await input.focus();
    await page.keyboard.press("ArrowLeft"); // next, not previous, in RTL: stays put
    await expect(input).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect(root.getByRole("button", { name: "Remove Nuxt.js" })).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect(root.getByRole("button", { name: "Remove Next.js" })).toBeFocused();
    await page.keyboard.press("ArrowLeft");
    await expect(root.getByRole("button", { name: "Remove Nuxt.js" })).toBeFocused();
});

test("clear button removes every chip and disappears", async ({ page }) => {
    const root = await open(page, "multiple_clear");
    const clear = root.getByRole("button", { name: "Clear" });
    await expect(clear).toBeVisible();
    await clear.click();
    await expect(chips(root)).toHaveCount(0);
    await expect(clear).toHaveCount(0);
    await expect(root.getByRole("combobox")).toBeFocused();
});

test("the field anchors the popup and sets its width", async ({ page }) => {
    const root = await open(page, "multiple");
    await root.getByRole("combobox", { name: "Choose frameworks" }).click();
    const list = listbox(page);
    await expect(list).toBeVisible();
    const f = await measureAtRest(field(root));
    const l = await measureAtRest(list);
    expect(Math.abs(l.x - f.x)).toBeLessThan(2);
    expect(Math.abs(l.width - f.width)).toBeLessThan(2);
    expect(l.y).toBeGreaterThanOrEqual(f.y + f.height - 1);
});

test("form: every selected value is submitted; reset restores the default", async ({ page }) => {
    const root = await open(page, "multiple_form");
    const input = root.getByRole("combobox", { name: "Form frameworks" });
    const entries = () =>
        page.evaluate(() => {
            const form = document.querySelector("#combobox-multiple-form") as HTMLFormElement;
            return new FormData(form).getAll("frameworks");
        });
    expect(await entries()).toEqual(["next"]);

    await input.click();
    await listbox(page).getByRole("option", { name: "Nuxt.js" }).click();
    await listbox(page).getByRole("option", { name: "Dioxus" }).click();
    expect(await entries()).toEqual(["next", "nuxt", "dioxus"]);

    await root.getByRole("button", { name: "Remove Next.js" }).click();
    expect(await entries()).toEqual(["nuxt", "dioxus"]);

    await root.getByRole("button", { name: "Reset" }).click();
    await expect(chipLabels(root)).toHaveText(["Next.js"]);
    expect(await entries()).toEqual(["next"]);
});

for (const mode of ["light", "dark"] as const) {
    test(`axe (${mode}): chips closed and list open`, async ({ page }) => {
        const root = await open(page, "multiple_clear", mode);
        const opts = { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] };
        await expectNoAxeViolations(page, `combobox multiple (${mode}): loaded`, opts);
        await root.getByRole("combobox", { name: "Frameworks with clear" }).click();
        await expect(listbox(page)).toBeVisible();
        await expectNoAxeViolations(page, `combobox multiple (${mode}): open`, opts);
    });
}
