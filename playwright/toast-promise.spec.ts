import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";

/** Each toast demo owns one provider whose region carries a distinct name (axe landmark-unique). */
const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => page.goto(`${BASE_URL}/component/?name=toast&`);

const R = "Promise toast notifications";

/** Counts the live regions (role status/alert) inserted into the toast region over the test. */
async function watchLiveRegions(page: Page, name: string) {
  await region(page, name).evaluate((root) => {
    const w = window as unknown as { __live: { added: string[]; toasts: number } };
    w.__live = { added: [], toasts: 0 };
    new MutationObserver((records) => {
      for (const r of records)
        for (const n of Array.from(r.addedNodes)) {
          if (!(n instanceof HTMLElement)) continue;
          // A toast arrives inside its <li>, so look at the added subtree, not just its root.
          for (const el of [n, ...Array.from(n.querySelectorAll("[role]"))]) {
            const role = el.getAttribute("role") ?? "";
            if (role === "alertdialog") w.__live.toasts++;
            if (role === "status" || role === "alert") w.__live.added.push(role);
          }
        }
    }).observe(root, { childList: true, subtree: true });
  });
}
const live = (page: Page) =>
  page.evaluate(() => (window as unknown as { __live: { added: string[]; toasts: number } }).__live);

test.describe("toast: promise (row 138 step 3)", () => {
  test("success: one toast goes loading -> success in place, announced politely", async ({ page }) => {
    await open(page);
    await watchLiveRegions(page, R);
    await page.getByRole("button", { name: "Promise (success)" }).click();
    const t = toasts(page, R);
    await expect(t).toHaveCount(1);
    await expect(t).toHaveAttribute("data-type", "loading");
    await expect(t.getByRole("status")).toHaveText("Saving event...");
    await t.evaluate((el) => ((el as HTMLElement).dataset.probe = "same-node"));

    await expect(t).toHaveAttribute("data-type", "success");
    await expect(t.getByRole("status")).toHaveText("Event has been created");
    await expect(t).toHaveCount(1);
    await expect(t).toHaveAttribute("data-probe", "same-node");
    const l = await live(page);
    expect(l.toasts, "exactly one toast element was ever inserted").toBe(1);
    expect(l.added, "never an assertive region for a success").not.toContain("alert");
  });

  test("error: the same toast becomes a role=alert region, announced once", async ({ page }) => {
    await open(page);
    await watchLiveRegions(page, R);
    await page.getByRole("button", { name: "Promise (error)" }).click();
    const t = toasts(page, R);
    await expect(t.getByRole("status")).toHaveText("Saving event...");
    await t.evaluate((el) => ((el as HTMLElement).dataset.probe = "same-node"));

    await expect(t.getByRole("alert")).toHaveText("Could not save: Server unavailable");
    await expect(t).toHaveAttribute("data-type", "error");
    await expect(t.getByRole("status")).toHaveCount(0);
    await expect(t).toHaveCount(1);
    await expect(t).toHaveAttribute("data-probe", "same-node");
    const l = await live(page);
    expect(l.toasts).toBe(1);
    expect(l.added.filter((r) => r === "alert")).toHaveLength(1);
  });

  test("once settled, the toast starts its normal countdown (loading no longer pins it)", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Promise (success)" }).click();
    const t = toasts(page, R);
    await expect(t).toHaveAttribute("data-type", "success");
    // Default duration 5 s from settling.
    await expect(t).toHaveCount(0, { timeout: 9000 });
  });
});
