import { test, expect, type Browser, type Page } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { gotoPrerendered } from "./page-kinds";
import { measureAtRest, staticBox } from "./animations";
import { noWebFonts } from "./web-fonts";

const GROUPS = [
    { label: "Fruits", first: "Apple" },
    { label: "Vegetables", first: "Broccoli" },
    { label: "Grains", first: "Barley" },
];

const url = (component: string) => `${BASE_URL}/component/${component}/`;
const frame = (page: Page) => page.locator("#component-preview-frame-groups");

/** Every group heading of the Groups demo, whether or not it is rendered. */
const headings = (page: Page) => frame(page).locator(".dx-combobox-group-label");

/** The closed Groups demo's root takes exactly its input row: a heading in the flow adds height. */
async function expectClosedDemoIsOneRow(page: Page) {
    const root = frame(page).locator(".dx-combobox");
    const row = frame(page).locator(".dx-combobox-input-wrapper");
    await expect(root).toBeVisible();
    const [rootBox, rowBox] = await Promise.all([staticBox(root), staticBox(row)]);
    expect(rootBox.height).toBeLessThanOrEqual(rowBox.height + 1);
}

async function expectNoVisibleHeading(page: Page) {
    await expect(headings(page).filter({ visible: true })).toHaveCount(0);
    for (const { label } of GROUPS) {
        await expect(frame(page).getByText(label, { exact: true })).toBeHidden();
    }
}

async function withoutScript<T>(browser: Browser, run: (page: Page) => Promise<T>) {
    const context = await browser.newContext({ javaScriptEnabled: false });
    await noWebFonts(context);
    try {
        return await run(await context.newPage());
    } finally {
        await context.close();
    }
}

test("closed Groups demo shows no group heading in the server markup", async ({ browser }) => {
    await withoutScript(browser, async (page) => {
        await gotoPrerendered(page, url("combobox"));
        await expectNoVisibleHeading(page);
        await expectClosedDemoIsOneRow(page);
    });
});

test("closed Groups demo shows no group heading after hydration", async ({ page }) => {
    await gotoHydrated(page, url("combobox"));
    await expectNoVisibleHeading(page);
    await expectClosedDemoIsOneRow(page);
});

test("open Groups demo shows each heading inside the popup above its group's options", async ({ page }) => {
    await gotoHydrated(page, url("combobox"));
    await frame(page).getByRole("combobox", { name: "Grouped food" }).click();
    const popup = page.locator("[role='listbox'][data-open]");
    await expect(popup).toBeVisible();
    for (const { label, first } of GROUPS) {
        const heading = popup.getByText(label, { exact: true });
        await expect(heading).toBeVisible();
        const option = popup.getByRole("option", { name: first });
        const [h, o] = await Promise.all([measureAtRest(heading), measureAtRest(option)]);
        expect(h.y + h.height).toBeLessThanOrEqual(o.y + 1);
        // The group element names itself by this heading and holds both.
        const group = popup.getByRole("group", { name: label });
        await expect(group).toBeVisible();
        await expect(group.getByText(label, { exact: true })).toBeVisible();
        await expect(group.getByRole("option", { name: first })).toBeVisible();
    }
});

// Components with popup content beside their items: none of it is visible while the popup is closed.
const POPUP_PARTS = [
    "[role=listbox]",
    "[role=option]",
    "[role=separator]",
    "[role=menuitem]",
    "[class*='group-label']",
    "[class*='menu-label']",
].join(",");

for (const component of ["combobox", "select", "dropdown_menu", "context_menu", "navigation_menu"]) {
    test(`${component}: no popup part is visible while closed, before and after hydration`, async ({ browser, page }) => {
        await withoutScript(browser, async (bare) => {
            await gotoPrerendered(bare, url(component));
            await expect(bare.locator(".dx-component-preview-frame").locator(POPUP_PARTS).filter({ visible: true })).toHaveCount(0);
        });
        await gotoHydrated(page, url(component));
        await expect(page.locator(".dx-component-preview-frame").locator(POPUP_PARTS).filter({ visible: true })).toHaveCount(0);
    });
}
