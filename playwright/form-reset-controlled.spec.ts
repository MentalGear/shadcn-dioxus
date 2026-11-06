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
