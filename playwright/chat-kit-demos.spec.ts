import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };
const variant = (page: Page, name: string) => page.locator(`#component-preview-frame-${name}`).first();

test("message code_block_chat: framed block with filename and copy button inside a reply", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/message/`, GOTO);
  const demo = variant(page, "code_block_chat");
  await expect(demo.getByText("app/api/chat/route.ts")).toBeVisible({ timeout: 60_000 });
  await expect(demo.getByRole("button", { name: "Copy" })).toBeVisible();
  await expect(demo.locator('[data-slot="code-block"]')).toHaveCount(1);
});

test("message message_pattern: pressed-state feedback toggles and copy says Copied", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/message/`, GOTO);
  const demo = variant(page, "message_pattern");
  const good = demo.getByRole("button", { name: "Good response" });
  await expect(good).toHaveAttribute("aria-pressed", "false", { timeout: 60_000 });
  await good.click();
  await expect(good).toHaveAttribute("aria-pressed", "true");
  await demo.getByRole("button", { name: "Bad response" }).click();
  await expect(good).toHaveAttribute("aria-pressed", "false");
  await demo.getByRole("button", { name: "Copy response" }).click();
  await expect(demo.getByRole("button", { name: "Copied" })).toBeVisible();
  await expect(demo.getByRole("button", { name: "Copy response" })).toBeVisible({ timeout: 5000 });
});

test("scroller agent_chat: replay streams trace, answer; restore drops the answer", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/message_scroller/`, GOTO);
  const demo = variant(page, "agent_chat");
  await expect(demo.locator('[data-slot="code-block"]')).toHaveCount(1, { timeout: 60_000 });
  const restore = demo.getByRole("button", { name: /Restore checkpoint: Before the answer/ });
  await restore.click();
  await expect(demo.locator('[data-slot="code-block"]')).toHaveCount(0);
  await expect(restore).toBeDisabled();
  await demo.getByRole("button", { name: "Run again" }).click();
  await expect(demo.locator('[data-slot="trace-line"][data-status="running"]').first()).toBeVisible();
  await expect(demo.locator('[data-slot="code-block"]')).toHaveCount(1, { timeout: 15_000 });
  await expect(demo.getByRole("button", { name: /Thought/ })).toHaveAttribute("aria-expanded", "false");
});

for (const mode of ["light", "dark"] as const) {
  test(`chat demos have no axe violations (${mode})`, async ({ page }) => {
    for (const [name, v] of [
      ["message", "code_block_chat"],
      ["message", "message_pattern"],
      ["message_scroller", "agent_chat"],
    ]) {
      await gotoHydrated(page, `${BASE_URL}/component/${name}/`, GOTO);
      await expect(variant(page, v)).toBeVisible({ timeout: 60_000 });
      await page.evaluate((m) => (window as unknown as { __dxApplyMode: (m: string) => void }).__dxApplyMode(m), mode);
      await expectNoAxeViolations(page, `${name} ${v} ${mode}`, { include: `#component-preview-frame-${v}` });
    }
  });
}
