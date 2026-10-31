import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const URL = `${BASE_URL}/component/input_otp/`;

test("alphanumeric: letters and digits fill the slots; attributes derive from the pattern", async ({ page }) => {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const input = page.locator("#otp-alnum");
  await expect(input).toHaveAttribute("inputmode", "text");
  await expect(input).toHaveAttribute("pattern", "[A-Za-z0-9]*");
  await input.click();
  await page.keyboard.type("aB3xY9");
  await expect(input).toHaveValue("aB3xY9");
  await expect(page.locator('#otp-alnum + div [data-slot-index="0"]')).toHaveText("a");
  await expect(page.locator('#otp-alnum + div [data-slot-index="1"]')).toHaveText("B");
  await expect(page.locator("#input-otp-alnum-value")).toContainText("Value: aB3xY9");
});

test("alphanumeric: rejects punctuation as typed, keeps the caret after accepted characters", async ({ page }) => {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const input = page.locator("#otp-alnum");
  await input.click();
  await page.keyboard.type("a-b_c!");
  await expect(input).toHaveValue("abc");
  await page.keyboard.type("d");
  await expect(input).toHaveValue("abcd");
  await expect(page.locator("#input-otp-alnum-value")).toContainText("Value: abcd");
});

test("digits-only (default) rejects letters and keeps numeric inputmode", async ({ page }) => {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const input = page.locator("#otp-main");
  await expect(input).toHaveAttribute("inputmode", "numeric");
  await expect(input).toHaveAttribute("pattern", "[0-9]*");
  await input.click();
  await page.keyboard.type("1a2b3c");
  await expect(input).toHaveValue("123");
  await expect(page.locator("#input-otp-value")).toContainText("Value: 123");
});

test("paste is filtered", async ({ page }) => {
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const input = page.locator("#otp-alnum");
  await input.click();
  await page.evaluate((t) => navigator.clipboard.writeText(t), "Ab 1!");
  await input.press("Control+V");
  await expect(input).toHaveValue("Ab1");
  const digits = page.locator("#otp-main");
  await digits.click();
  await page.evaluate((t) => navigator.clipboard.writeText(t), "12ab34");
  await digits.press("Control+V");
  await expect(digits).toHaveValue("1234");
});

// Row 474: the browser trims a paste to `maxlength` before the filter runs, so a code pasted with
// separators lost its tail digits. The control reads the whole paste, filters it, then applies the length.
for (const [pasted, expected] of [
  ["123-456", "123456"],
  ["123 456", "123456"],
  ["12-34-56", "123456"],
  ["1234567890", "123456"],
] as const) {
  test(`paste "${pasted}" fills six digit slots as "${expected}"`, async ({ page }) => {
    await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const input = page.locator("#otp-main");
    await input.click();
    await page.evaluate((t) => navigator.clipboard.writeText(t), pasted);
    await input.press("Control+V");
    await expect(input).toHaveValue(expected);
    await expect(page.locator("#input-otp-value")).toContainText(`Value: ${expected}`);
    for (let i = 0; i < expected.length; i++) {
      await expect(page.locator(`#otp-main + div [data-slot-index="${i}"]`)).toHaveText(expected[i]);
    }
  });
}

test("paste with separators into the alphanumeric control keeps every accepted character", async ({ page }) => {
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const input = page.locator("#otp-alnum");
  await input.click();
  await page.evaluate((t) => navigator.clipboard.writeText(t), "aB3-xY9");
  await input.press("Control+V");
  await expect(input).toHaveValue("aB3xY9");
});

test("typing into a full control is still blocked, and a paste into a partly filled one is cut to the free slots", async ({
  page,
}) => {
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const input = page.locator("#otp-main");
  await input.click();
  await page.keyboard.type("123456");
  await page.keyboard.press("Home");
  await page.keyboard.type("9");
  await expect(input).toHaveValue("123456");
  await input.fill("");
  await page.keyboard.type("12");
  await page.evaluate((t) => navigator.clipboard.writeText(t), "3-4-5-6-7-8");
  await input.press("Control+V");
  await expect(input).toHaveValue("123456");
});

test(":invalid follows the pattern", async ({ page }) => {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const mismatch = (sel: string, v: string) =>
    page.evaluate(([s, val]) => {
      const el = document.querySelector(s) as HTMLInputElement;
      const prev = el.value;
      el.value = val;
      const r = el.validity.patternMismatch;
      el.value = prev;
      return r;
    }, [sel, v]);
  expect(await mismatch("#otp-alnum", "ab12")).toBe(false);
  expect(await mismatch("#otp-alnum", "ab-2")).toBe(true);
  expect(await mismatch("#otp-main", "12ab")).toBe(true);
  expect(await mismatch("#otp-main", "1234")).toBe(false);
});

// The decorative slots are pointer-events:none; the real input is the hit target, so click the slot's centre by coordinates.
async function clickSlot(page: import("@playwright/test").Page, i: number) {
  const box = await page.locator(`#otp-alnum + div [data-slot-index="${i}"]`).boundingBox();
  if (!box) throw new Error("slot has no box");
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
}

test("click-to-slot correction works with letters", async ({ page }) => {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
  const input = page.locator("#otp-alnum");
  await input.click();
  await page.keyboard.type("abcdef");
  await clickSlot(page, 2);
  await page.keyboard.type("Z");
  await expect(input).toHaveValue("abZdef");
  await clickSlot(page, 4);
  await page.keyboard.type("!");
  await expect(input).toHaveValue("abZdef");
});
