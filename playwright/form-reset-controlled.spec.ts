/**
 * `<form>` reset on a text control whose value the owner's signal holds (row 468). The browser restores
 * the DOM value and tells Rust nothing, so the signal kept the old text and the owner's next render wrote
 * it back. The wrappers report the reset as an edit to the default (`primitives/src/form_reset.rs`).
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

test("input/reset: a form reset empties a controlled Input, a Textarea and returns an uncontrolled one to its default", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/input/`, GOTO_OPTS);
  const root = page.locator("#component-preview-frame-reset").first();
  await root.scrollIntoViewIfNeeded();
  const controlled = root.locator("#input-reset-controlled");
  const uncontrolled = root.locator("#input-reset-uncontrolled");
  const textarea = root.locator("#input-reset-textarea");
  const state = root.locator("#input-reset-state");

  await controlled.fill("abc");
  await textarea.fill("some bio");
  await uncontrolled.fill("typed");
  await expect(state).toContainText("name:abc bio:some bio");

  await root.locator("#input-reset-button").click();

  // The DOM went back to the defaults, and the owner's signals followed.
  await expect(controlled).toHaveValue("");
  await expect(textarea).toHaveValue("");
  await expect(uncontrolled).toHaveValue("preset");
  await expect(state).toContainText("name: bio:");

  // A later render of the owner must not write the old text back.
  await root.locator("#input-reset-rerender").click();
  await expect(state).toContainText("renders:1");
  await expect(controlled).toHaveValue("");
  await expect(textarea).toHaveValue("");
  await expect(uncontrolled).toHaveValue("preset");
});

test("input/reset: a form reset returns a controlled NativeSelect and Checkbox to their defaults and tells the owner", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/input/`, GOTO_OPTS);
  const root = page.locator("#component-preview-frame-reset").first();
  await root.scrollIntoViewIfNeeded();
  const select = root.locator("#input-reset-select");
  const checkbox = root.locator("#input-reset-checkbox");
  const state = root.locator("#input-reset-state");

  await select.selectOption("banana");
  await checkbox.click();
  await expect(state).toContainText("fruit:banana agree:Checked");

  await root.locator("#input-reset-button").click();

  await expect(select).toHaveValue("apple");
  await expect(checkbox).toHaveAttribute("aria-checked", "false");
  await expect(state).toContainText("fruit:apple agree:Unchecked");

  // A later render of the owner must not write the old state back.
  await root.locator("#input-reset-rerender").click();
  await expect(state).toContainText("renders:1");
  await expect(select).toHaveValue("apple");
  await expect(checkbox).toHaveAttribute("aria-checked", "false");
});

test("input/reset: form state the browser restores on history navigation reaches the owner, and no mirror input holds a value Rust does not", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/input/`, GOTO_OPTS);
  const root = page.locator("#component-preview-frame-reset").first();
  await root.scrollIntoViewIfNeeded();
  await root.locator("#input-reset-controlled").fill("abc");
  await root.locator("#input-reset-textarea").fill("some bio");
  await root.locator("#input-reset-select").selectOption("banana");
  await root.locator("#input-reset-checkbox").click();
  await expect(root.locator("#input-reset-state")).toContainText("name:abc bio:some bio fruit:banana agree:Checked");

  // Leave and come back: the browser rebuilds the page from the server HTML and restores the typed
  // values into the controls before the wasm hydrates (Playwright's Chromium has the back/forward
  // cache off, so this is the form-state restore path, not a frozen page).
  await gotoHydrated(page, `${BASE_URL}/component/button/`, GOTO_OPTS);
  await page.goBack({ waitUntil: "networkidle", timeout: GOTO_OPTS.timeout });
  await page.waitForSelector('html[data-hydrated="true"]', { state: "attached" });
  const back = page.locator("#component-preview-frame-reset").first();
  await back.scrollIntoViewIfNeeded();

  // The text controls keep what the browser restored and the owner's signals agree with it.
  const state = back.locator("#input-reset-state");
  const dom = await back.locator("#input-reset-controlled").inputValue();
  const bio = await back.locator("#input-reset-textarea").inputValue();
  const fruit = await back.locator("#input-reset-select").inputValue();
  expect(dom, "the browser restored the typed text, so this test exercises the restore").toBe("abc");
  await expect(state).toContainText(`name:${dom} bio:${bio} fruit:${fruit}`);

  // The checkbox's hidden mirror is Rust's output: it never holds a state the visible checkbox lacks.
  const visible = (await back.locator("#input-reset-checkbox").getAttribute("aria-checked")) === "true";
  const mirror = await back.locator('input[type="checkbox"][name="agree"]').isChecked();
  expect(mirror).toBe(visible);
  await expect(state).toContainText(visible ? "agree:Checked" : "agree:Unchecked");

  // And a later render of the owner changes none of it.
  await back.locator("#input-reset-rerender").click();
  await expect(state).toContainText("renders:1");
  expect(await back.locator("#input-reset-controlled").inputValue()).toBe(dom);
  expect(await back.locator("#input-reset-textarea").inputValue()).toBe(bio);
  expect(await back.locator("#input-reset-select").inputValue()).toBe(fruit);
});
