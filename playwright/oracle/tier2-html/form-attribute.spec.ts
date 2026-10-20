/**
 * ORACLE: tier 2 (HTML) -- rule 8, the `form` attribute.
 *
 * Source: WHATWG HTML, "Association of controls and forms"
 *   https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#association-of-controls-and-forms
 * A submittable element whose `form` attribute names a `<form>` by id is that form's control even when it is
 * not a DOM descendant of it, so `new FormData(form)` includes it and `form.reset()` restores it.
 *
 * Fixture: `FormFixture`'s "Form attribute" section (/component/?name=form). The `Checkbox`, `Switch` and
 * `RadioGroup` there sit OUTSIDE `#outside-form` and pass `form: "outside-form"`, which the primitives forward
 * to their hidden mirror input. A native `<input form=...>` is the CALIBRATION control (a failure there means
 * the test is wrong, not the component).
 */
import { test, expect } from "../../fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "../../base-url";
import { gotoHydrated } from "../../hydration";

const NAV_TIMEOUT = 20 * 60 * 1000;
const gotoForm = (page: Page) =>
  gotoHydrated(page, `${BASE_URL}/component/form/`, { timeout: NAV_TIMEOUT });

async function submitAndRead(page: Page): Promise<string[]> {
  const result = page.locator("#outside-result");
  const before = parseInt((await result.getAttribute("data-submit-count")) ?? "0", 10);
  await page.locator("#outside-submit").click();
  await expect(result).toHaveAttribute("data-submit-count", String(before + 1));
  return ((await result.textContent()) ?? "").split("\n").map((l) => l.trim()).filter(Boolean);
}

test.describe("Rule 8 -- the `form` attribute associates a control outside its <form>", () => {
  test("the controls are not DOM descendants of the form", async ({ page }) => {
    await gotoForm(page);
    for (const id of ["chk-outside", "switch-outside", "plan-outside-pro"]) {
      expect(await page.locator(`#${id}`).evaluate((el) => !!el.closest("#outside-form")), id).toBe(false);
    }
  });

  test("CALIBRATION: native checkbox with form= contributes to the entry list", async ({ page }) => {
    await gotoForm(page);
    expect(await submitAndRead(page)).toContainEqual("terms-outside-native=accepted");
  });

  test("Checkbox, Switch and RadioGroup contribute through form=", async ({ page }) => {
    await gotoForm(page);
    await page.locator("#plan-outside-pro").click();
    await expect(page.locator("#plan-outside-pro")).toHaveAttribute("data-state", "checked");
    const entries = await submitAndRead(page);
    expect(entries).toContainEqual("terms-outside-lib=accepted");
    expect(entries).toContainEqual("notify-outside-lib=subscribed");
    expect(entries).toContainEqual("plan-outside-lib=pro");
  });

  test("an unchecked outside control contributes nothing", async ({ page }) => {
    await gotoForm(page);
    await page.locator("#chk-outside").click();
    await page.locator("#switch-outside").click();
    const entries = await submitAndRead(page);
    expect(entries.some((l) => l.startsWith("terms-outside-lib="))).toBe(false);
    expect(entries.some((l) => l.startsWith("notify-outside-lib="))).toBe(false);
  });

  test("the form's reset restores the outside controls", async ({ page }) => {
    await gotoForm(page);
    await page.locator("#chk-outside").click();
    await page.locator("#switch-outside").click();
    await page.locator("#plan-outside-pro").click();
    await page.locator("#outside-reset").click();
    await expect(page.locator("#chk-outside")).toHaveAttribute("data-state", "checked");
    await expect(page.locator("#switch-outside")).toHaveAttribute("data-state", "checked");
    await expect(page.locator("#plan-outside-starter")).toHaveAttribute("data-state", "checked");
    const entries = await submitAndRead(page);
    expect(entries).toContainEqual("terms-outside-lib=accepted");
    expect(entries).toContainEqual("notify-outside-lib=subscribed");
    expect(entries).toContainEqual("plan-outside-lib=starter");
  });
});
