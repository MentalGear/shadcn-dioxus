import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// Class guard: a listener that reaches a Button (or any element built through `merge_attributes`)
// from TWO sources -- the component's own prop AND a caller's `attributes` spread -- must never
// be dropped. Row 155 group 6 (`r#as` on Button) once merged Button's own always-present
// onclick/onmousedown/onmouseup/onkeydown LAST, so a `DialogClose`/`SheetClose`/`DrawerClose`
// rendered `as` a Button, or a menu trigger rendered `as` a Button, lost its listener: dialogs
// would not close and menus would not open (34 Playwright failures on the merged tree, each
// group green alone). `merge_attributes` now chains same-named listeners
// (primitives/src/lib.rs unit tests); these specs assert the user-visible outcomes.

const GOTO = { timeout: 20 * 60 * 1000 };
const page_of = (name: string) => `${BASE_URL}/component/${name}/`;

async function openByKeyboard(page: Page, url: string, label: string) {
  await gotoHydrated(page, url, GOTO);
  await page.getByRole("button", { name: label, exact: true }).focus();
  await page.keyboard.press("Enter");
}

test("Button onclick prop fires (a Button opens a Sheet)", async ({ page }) => {
  await gotoHydrated(page, page_of("sheet"), GOTO);
  await page.getByRole("button", { name: "Right", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
});

test("Button `as` link fires the onclick prop on the rendered anchor", async ({ page }) => {
  await gotoHydrated(page, page_of("button"), GOTO);
  const demo = page.locator("#component-preview-frame-as_link").first();
  await expect(demo.getByText("Clicked 0 time(s)")).toBeVisible();
  await demo.getByRole("link", { name: "Login" }).click();
  await expect(demo.getByText("Clicked 1 time(s)")).toBeVisible();
});

test("DialogClose rendered as a Button (attributes) closes: the X", async ({ page }) => {
  await openByKeyboard(page, page_of("dialog"), "Show Dialog");
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await dialog.locator('[data-slot="dialog-close"]').click();
  await expect(dialog).toHaveCount(0);
});

test("DialogClose rendered as a Button (attributes) closes: the footer Close", async ({ page }) => {
  await openByKeyboard(page, page_of("dialog"), "Share");
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Close", exact: true }).last().click();
  await expect(dialog).toHaveCount(0);
});

test("SheetClose rendered as a Button closes the sheet", async ({ page }) => {
  await gotoHydrated(page, page_of("sheet"), GOTO);
  await page.getByRole("button", { name: "Right", exact: true }).click();
  const sheet = page.getByRole("dialog");
  await expect(sheet).toBeVisible();
  await sheet.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(sheet).toHaveCount(0);
});

test("DrawerClose rendered as a Button closes the drawer", async ({ page }) => {
  await gotoHydrated(page, page_of("drawer"), GOTO);
  await page.getByRole("button", { name: "Move Goal", exact: true }).click();
  const drawer = page.getByRole("dialog");
  await expect(drawer).toBeVisible();
  await drawer.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(drawer).toHaveCount(0);
});

test("a DropdownMenu trigger rendered as a Button opens the menu", async ({ page }) => {
  await gotoHydrated(page, page_of("dropdown_menu"), GOTO);
  const trigger = page.getByRole("button", { name: "Account switcher" });
  await trigger.click();
  await expect(page.getByRole("menu")).toBeVisible();
  await expect(page.getByRole("menuitem").first()).toBeVisible();
});
