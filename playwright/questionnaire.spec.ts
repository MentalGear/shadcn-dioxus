/**
 * Questionnaire (backlog row 136): the browser half of the port of shadcn's questionnaire suites
 * (`packages/react/src/questionnaire/questionnaire.browser.test.tsx` and the DOM-dependent cases of
 * `questionnaire.test.tsx`). The decisions themselves (validity, navigation, key classification, shortcuts,
 * defaults, reset, conditional items) are unit tested in `primitives/src/questionnaire/model_tests.rs`; these
 * specs pin what only a browser shows:
 *
 * - the server-rendered page is the complete initial state (JS off): active item, hidden + inert inactive
 *   items, progress, which buttons apply;
 * - navigation and validation move focus: a failed Next focuses the first answer, a successful one the new
 *   `fieldset`; errors are `role=alert`, wired into `aria-describedby`, and `aria-invalid`;
 * - keyboard: letter and number shortcuts, Enter on the selected answer, Command/Control+Enter, horizontal
 *   arrows between items (never an implicit skip), vertical arrows across answers and the native radio group;
 * - the freeform answer replaces a radio and is a draft (no name, `form=""`) once a radio is chosen again;
 * - native form serialization: FormData at submit, a skipped item submits nothing, a conditional item that
 *   does not apply submits nothing, the submit never navigates;
 * - resume and reset, external (schema) validation, controlled navigation, opt-in disabled navigation;
 * - axe on every demo, light and dark, including the error state.
 *
 * Controls are located by role, slot and position inside each demo frame, never by demo copy (row 220).
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { expectNoAxeViolations } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };
const URL = `${BASE_URL}/component/questionnaire/`;
const DEMOS = [
  "main",
  "multiple",
  "freeform",
  "skip",
  "shortcuts",
  "validation",
  "controlled",
  "resume",
  "conditional",
  "navigation_state",
  "progress",
] as const;
type Demo = (typeof DEMOS)[number];

const frameSelector = (name: Demo) => (name === "main" ? "#component-preview-frame" : `#component-preview-frame-${name}`);
const frame = (page: Page, name: Demo) => page.locator(frameSelector(name)).first();
const form = (root: Locator) => root.locator('form[data-slot="questionnaire"]');
const items = (root: Locator) => root.locator('fieldset[data-slot="questionnaire-item"]');
const active = (root: Locator) => root.locator('fieldset[data-slot="questionnaire-item"][data-active]');
const radios = (scope: Locator) => scope.locator('input[data-slot="questionnaire-choice-input"]');
const rows = (scope: Locator) => scope.locator('label[data-slot="questionnaire-choice"]');
const freeform = (scope: Locator) => scope.locator('input[data-slot="questionnaire-input"]');
const error = (scope: Locator) => scope.locator('[data-slot="questionnaire-error"]');
const button = (root: Locator, slot: "previous" | "skip" | "next" | "submit") =>
  root.locator(`button[data-slot="questionnaire-${slot}"]`);
const progress = (root: Locator) => root.getByRole("progressbar");

async function open(page: Page, name: Demo = "main") {
  await gotoHydrated(page, URL, GOTO);
  const root = frame(page, name);
  await root.scrollIntoViewIfNeeded();
  await expect(active(root)).toBeVisible({ timeout: 60_000 });
  // The stylesheet is linked after mount: the active item is a flex column once it applies.
  await expect.poll(() => active(root).evaluate((el) => getComputedStyle(el).display), { timeout: 60_000 }).toBe("flex");
  return root;
}

/** What the native form would submit now. */
const formData = (root: Locator) =>
  form(root).evaluate((f: HTMLFormElement) => [...new FormData(f)].map(([k, v]) => [k, String(v)]));

/** Records the FormData of the next submit event, read when the browser fires it (before any handler). */
async function captureSubmit(root: Locator) {
  await form(root).evaluate((f: HTMLFormElement) => {
    const w = window as unknown as { __qSubmits: string[][][] };
    w.__qSubmits = [];
    // listener-ok: test instrumentation, removed with the page
    f.addEventListener("submit", () => w.__qSubmits.push([...new FormData(f)].map(([k, v]) => [k, String(v)])), {
      capture: true,
    });
  });
}
const submits = (page: Page) => page.evaluate(() => (window as unknown as { __qSubmits: string[][][] }).__qSubmits);

test("the server-rendered page is the complete initial state (JavaScript off)", async ({ browser }) => {
  const ctx = await browser.newContext({ javaScriptEnabled: false });
  const page = await ctx.newPage();
  await gotoHydrated(page, `${BASE_URL}/component/questionnaire/`, { waitUntil: "load", ...GOTO });
  const root = frame(page, "main");
  await expect(items(root)).toHaveCount(3);
  await expect(items(root).nth(0)).toBeVisible();
  for (const i of [1, 2]) {
    await expect(items(root).nth(i)).toBeHidden();
    await expect(items(root).nth(i)).toHaveAttribute("inert", /.*/);
  }
  await expect(progress(root)).toHaveAttribute("aria-valuetext", "Question 1 of 3");
  await expect(progress(root)).toHaveText("Question 1 of 3");
  await expect(button(root, "next")).toBeVisible();
  for (const slot of ["previous", "skip", "submit"] as const) await expect(button(root, slot)).toBeHidden();
  // The letter shortcuts are server-rendered too.
  await expect(radios(items(root).nth(0)).nth(0)).toHaveAttribute("aria-keyshortcuts", "A");
  // The resume demo opens on its saved item with its saved answers.
  const resume = frame(page, "resume");
  await expect(items(resume).nth(1)).toBeVisible();
  await expect(radios(items(resume).nth(1)).nth(0)).toBeChecked();
  await expect(radios(items(resume).nth(1)).nth(1)).toBeChecked();
  await ctx.close();
});

test("a failed Next focuses the first answer and shows the error; a successful one focuses the new item", async ({ page }) => {
  const root = await open(page);
  const first = items(root).nth(0);
  await button(root, "next").click();
  await expect(error(first)).toBeVisible();
  await expect(error(first)).toHaveAttribute("role", "alert");
  await expect(first).toHaveAttribute("aria-invalid", "true");
  const errorId = await error(first).getAttribute("id");
  const descriptionId = await first.locator('[data-slot="questionnaire-description"]').getAttribute("id");
  await expect(first).toHaveAttribute("aria-describedby", `${descriptionId} ${errorId}`);
  await expect(radios(first).nth(0)).toHaveAttribute("aria-invalid", "true");
  await expect(radios(first).nth(0)).toBeFocused();
  await expect(progress(root)).toHaveAttribute("aria-valuenow", "1");

  // Answering clears the error while it stays attempted.
  await rows(first).nth(1).click();
  await expect(radios(first).nth(1)).toBeChecked();
  await expect(error(first)).toBeHidden();
  await expect(first).not.toHaveAttribute("aria-invalid", /.*/);
  await expect(first).toHaveAttribute("aria-describedby", `${descriptionId}`);

  await button(root, "next").click();
  const second = items(root).nth(1);
  await expect(second).toBeFocused();
  await expect(first).toBeHidden();
  await expect(first).toHaveAttribute("inert", /.*/);
  await expect(progress(root)).toHaveAttribute("aria-valuetext", "Question 2 of 3");
  // An optional item offers Skip; Previous applies after the first item.
  await expect(button(root, "skip")).toBeVisible();
  await expect(button(root, "previous")).toBeVisible();

  await button(root, "previous").click();
  await expect(first).toBeFocused();
  await expect(radios(first).nth(1)).toBeChecked();
});

test("letter shortcuts select, Enter confirms, horizontal arrows move between items", async ({ page }) => {
  const root = await open(page);
  const [first, second] = [items(root).nth(0), items(root).nth(1)];
  await first.focus();
  await page.keyboard.press("b");
  await expect(radios(first).nth(1)).toBeChecked();
  await expect(radios(first).nth(1)).toBeFocused();
  await expect(radios(first).nth(1)).toHaveAttribute("aria-keyshortcuts", "B Enter");
  await expect(first).toHaveAttribute("aria-keyshortcuts", "Meta+Enter Control+Enter ArrowUp ArrowDown ArrowRight");

  await page.keyboard.press("Enter");
  await expect(second).toBeFocused();
  await page.keyboard.press("ArrowLeft");
  await expect(first).toBeFocused();
  await page.keyboard.press("ArrowRight");
  await expect(second).toBeFocused();

  // ArrowRight on an unanswered optional item is not an implicit skip.
  await page.keyboard.press("ArrowRight");
  await expect(second).toBeFocused();
  await expect(second).toHaveAttribute("data-status", "unanswered");

  // A shortcut toggles a checkbox.
  await page.keyboard.press("a");
  await expect(radios(second).nth(0)).toBeChecked();
  await page.keyboard.press("a");
  await expect(radios(second).nth(0)).not.toBeChecked();
});

test("under RTL the horizontal arrows swap, in aria-keyshortcuts too (page toggle, row 137)", async ({ page }) => {
  const root = await open(page);
  const toggle = page.locator("#dx-direction-toggle");
  await toggle.click();
  await expect(root).toHaveAttribute("dir", "rtl");
  const [first, second] = [items(root).nth(0), items(root).nth(1)];
  await first.focus();
  await page.keyboard.press("b");
  await expect(first).toHaveAttribute("aria-keyshortcuts", "Meta+Enter Control+Enter ArrowUp ArrowDown ArrowLeft");
  await first.focus();
  await page.keyboard.press("ArrowLeft");
  await expect(second).toBeFocused();
  await expect(second).toHaveAttribute("aria-keyshortcuts", /ArrowRight/);
  await page.keyboard.press("ArrowRight");
  await expect(first).toBeFocused();
});

test("Command/Control+Enter validates from anywhere and Skip records a skip", async ({ page }) => {
  const root = await open(page);
  const [first, second, third] = [items(root).nth(0), items(root).nth(1), items(root).nth(2)];
  await rows(first).nth(0).click();
  await page.keyboard.press("ControlOrMeta+Enter");
  await expect(second).toBeFocused();
  // An unanswered optional item needs an answer or an explicit skip.
  await page.keyboard.press("ControlOrMeta+Enter");
  await expect(error(second)).toBeVisible();
  await expect(radios(second).nth(0)).toBeFocused();
  await button(root, "skip").click();
  await expect(third).toBeFocused();
  await expect(second).toHaveAttribute("data-status", "skipped");
  await expect(error(second)).toBeHidden();
});

test("vertical arrows enter, cross and leave the answers; radios keep the browser's own group", async ({ page }) => {
  const root = await open(page);
  const first = items(root).nth(0);
  await first.focus();
  await page.keyboard.press("ArrowDown");
  await expect(radios(first).nth(0)).toBeFocused();
  await expect(radios(first).nth(0)).toBeChecked();
  // Radio to radio is the native radio group.
  await page.keyboard.press("ArrowDown");
  await expect(radios(first).nth(1)).toBeFocused();
  await expect(radios(first).nth(1)).toBeChecked();
  await page.keyboard.press("ArrowDown");
  await expect(radios(first).nth(2)).toBeChecked();
  // The last radio moves into the freeform input without selecting it.
  await page.keyboard.press("ArrowDown");
  await expect(freeform(first)).toBeFocused();
  await expect(radios(first).nth(2)).toBeChecked();
  // An empty text input lets ArrowUp leave it.
  await page.keyboard.press("ArrowUp");
  await expect(radios(first).nth(2)).toBeFocused();

  // Checkboxes: the focus moves, nothing toggles.
  await page.keyboard.press("ControlOrMeta+Enter");
  const second = items(root).nth(1);
  await expect(second).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await expect(radios(second).nth(0)).toBeFocused();
  await page.keyboard.press("ArrowUp");
  await expect(radios(second).nth(3)).toBeFocused();
  for (let i = 0; i < 4; i++) await expect(radios(second).nth(i)).not.toBeChecked();
});

test("the freeform answer replaces a radio and becomes a draft when a radio is chosen again", async ({ page }) => {
  const root = await open(page);
  const first = items(root).nth(0);
  const input = freeform(first);
  await expect(input).toHaveAttribute("form", "");
  await expect(input).not.toHaveAttribute("name", /.*/);
  await rows(first).nth(0).click();
  await input.click();
  await expect(radios(first).nth(0)).toBeChecked(); // focus alone selects nothing
  await input.fill("Something else");
  await expect(radios(first).nth(0)).not.toBeChecked();
  await expect(input).toHaveAttribute("name", "direction");
  await expect(input).not.toHaveAttribute("form", /.*/);
  await expect(input).toHaveAttribute("data-filled", /.*/);
  await expect(input).toHaveAttribute("aria-keyshortcuts", "Enter");
  expect(await formData(root)).toEqual([["direction", "Something else"]]);
  // Arrows keep the caret inside a filled input.
  await page.keyboard.press("ArrowUp");
  await expect(input).toBeFocused();
  // Enter on the filled answer confirms (no implicit form submission).
  const before = page.url();
  await page.keyboard.press("Enter");
  await expect(items(root).nth(1)).toBeFocused();
  expect(page.url()).toBe(before);

  await button(root, "previous").click();
  await rows(first).nth(2).click();
  await expect(input).toHaveValue("Something else");
  await expect(input).not.toHaveAttribute("name", /.*/);
  await expect(input).toHaveAttribute("form", "");
  expect(await formData(root)).toEqual([["direction", "handoffs"]]);
});

test("submit validates every item, returns to the first invalid one and serializes the native form", async ({ page }) => {
  const root = await open(page);
  await captureSubmit(root);
  const [first, second, third] = [items(root).nth(0), items(root).nth(1), items(root).nth(2)];
  await rows(first).nth(1).click();
  await button(root, "next").click();
  await rows(second).nth(0).click();
  await rows(second).nth(2).click();
  await button(root, "next").click();
  await expect(third).toBeFocused();
  // The last item unanswered: submit is blocked and focuses its first answer.
  await button(root, "submit").click();
  await expect(error(third)).toBeVisible();
  await expect(radios(third).nth(0)).toBeFocused();
  await rows(third).nth(0).click();
  const before = page.url();
  await button(root, "submit").click();
  await expect.poll(() => submits(page).then((s) => s.length)).toBeGreaterThan(1);
  const last = (await submits(page)).at(-1);
  expect(last).toEqual([
    ["direction", "approvals"],
    ["signals", "progress"],
    ["signals", "risks"],
    ["timing", "now"],
  ]);
  await expect(root.getByRole("status")).toContainText("timing: now");
  expect(page.url()).toBe(before);

  // A skipped item submits nothing.
  await button(root, "previous").click();
  await button(root, "skip").click();
  await button(root, "submit").click();
  await expect.poll(async () => (await submits(page)).at(-1)).toEqual([
    ["direction", "approvals"],
    ["timing", "now"],
  ]);
  await expect(root.getByRole("status")).toContainText("signals: None");
});

test("a conditional item joins and leaves the flow and submits nothing when it does not apply", async ({ page }) => {
  const root = await open(page, "conditional");
  await captureSubmit(root);
  const [first, environment, approval] = [items(root).nth(0), items(root).nth(1), items(root).nth(2)];
  await expect(progress(root)).toHaveAttribute("aria-valuemax", "2");
  // Playwright counts only form controls as natively disabled, not a fieldset.
  await expect(environment).toHaveJSProperty("disabled", true);
  await rows(first).nth(1).click();
  await expect(progress(root)).toHaveAttribute("aria-valuemax", "3");
  await expect(environment).toHaveJSProperty("disabled", false);
  await button(root, "next").click();
  await expect(environment).toBeFocused();
  await button(root, "previous").click();
  await rows(first).nth(0).click();
  await expect(progress(root)).toHaveAttribute("aria-valuemax", "2");
  await button(root, "next").click();
  await expect(approval).toBeFocused();
  await rows(approval).nth(0).click();
  await button(root, "submit").click();
  await expect.poll(async () => (await submits(page)).at(-1)).toEqual([
    ["runtime", "local"],
    ["approval", "writes"],
  ]);
});

test("resume opens on the saved item with saved answers, and reset restores them", async ({ page }) => {
  const root = await open(page, "resume");
  const [change, verification, notes] = [items(root).nth(0), items(root).nth(1), items(root).nth(2)];
  await expect(verification).toBeVisible();
  await expect(progress(root)).toHaveAttribute("aria-valuetext", "Question 2 of 3");
  await expect(freeform(notes)).toHaveAttribute("name", "notes");
  expect(await formData(root)).toEqual([
    ["change", "incremental"],
    ["verification", "tests"],
    ["verification", "typecheck"],
    ["notes", "Keep the existing public API stable."],
  ]);
  await rows(verification).nth(0).click();
  await rows(verification).nth(2).click();
  await button(root, "previous").click();
  await rows(change).nth(1).click();
  expect(await formData(root)).toEqual([
    ["change", "cutover"],
    ["verification", "typecheck"],
    ["verification", "manual"],
    ["notes", "Keep the existing public API stable."],
  ]);
  await root.locator('button[type="reset"]').click();
  await expect(verification).toBeFocused();
  await expect(radios(verification).nth(0)).toBeChecked();
  await expect(radios(verification).nth(2)).not.toBeChecked();
  expect(await formData(root)).toEqual([
    ["change", "incremental"],
    ["verification", "tests"],
    ["verification", "typecheck"],
    ["notes", "Keep the existing public API stable."],
  ]);
});

test("external validation returns to and blocks the invalid item with its own message", async ({ page }) => {
  const root = await open(page, "validation");
  const [detail, audience] = [items(root).nth(0), items(root).nth(1)];
  await rows(detail).nth(0).click();
  await button(root, "next").click();
  await expect(audience).toBeFocused();
  await rows(audience).nth(1).click();
  await button(root, "submit").click();
  await expect(detail).toBeFocused();
  await expect(detail).toHaveAttribute("aria-invalid", "true");
  await expect(error(detail)).toBeVisible();
  const message = (await error(detail).textContent()) ?? "";
  expect(message).not.toBe("Choose an answer to continue.");
  expect(message.length).toBeGreaterThan(0);
  // Blocked until the error is cleared by a new answer.
  await button(root, "next").click();
  await expect(detail).toHaveAttribute("data-active", /.*/);
  await rows(detail).nth(1).click();
  await expect(error(detail)).toBeHidden();
  await button(root, "next").click();
  await button(root, "submit").click();
  await expect(root.getByRole("status")).toContainText("detail: complete");
});

test("controlled navigation follows the host's item and focuses it", async ({ page }) => {
  const root = await open(page, "controlled");
  const jump = root.getByRole("group").first().getByRole("button");
  await jump.nth(2).click();
  await expect(items(root).nth(2)).toHaveAttribute("data-active", /.*/);
  await expect(items(root).nth(2)).toBeFocused();
  await jump.nth(0).click();
  await expect(items(root).nth(0)).toBeFocused();
  await rows(items(root).nth(0)).nth(0).click();
  await button(root, "next").click();
  await expect(items(root).nth(1)).toBeFocused();
});

test("opt-in disabled navigation waits for an answer", async ({ page }) => {
  const root = await open(page, "navigation_state");
  await expect(button(root, "next")).toBeDisabled();
  await rows(items(root).nth(0)).nth(0).click();
  await expect(button(root, "next")).toBeEnabled();
  await expect(button(root, "next")).toHaveAttribute("aria-keyshortcuts", "Enter");
  await button(root, "next").click();
  await expect(button(root, "submit")).toBeDisabled();
  await expect(button(root, "submit")).not.toHaveAttribute("aria-keyshortcuts", /.*/);
});

test("shortcut modes: none, letters, numbers", async ({ page }) => {
  const root = await open(page, "shortcuts");
  const modes = root.getByRole("group").first().getByRole("button");
  const item = items(root).nth(0);
  const shortcut = item.locator('[data-slot="questionnaire-choice-shortcut"]');
  await expect(form(root)).toHaveAttribute("data-shortcuts", "letters");
  await expect(shortcut.nth(2)).toHaveText("C");
  await modes.nth(0).click();
  await expect(form(root)).not.toHaveAttribute("data-shortcuts", /.*/);
  await expect(shortcut.nth(0)).toBeHidden();
  await expect(radios(item).nth(0)).not.toHaveAttribute("aria-keyshortcuts", /.*/);
  await modes.nth(2).click();
  await expect(shortcut.nth(1)).toHaveText("2");
  await item.focus();
  await page.keyboard.press("3");
  await expect(radios(item).nth(2)).toBeChecked();
  // A digit typed in letter mode is not a shortcut.
  await modes.nth(1).click();
  await item.focus();
  await page.keyboard.press("1");
  await expect(radios(item).nth(2)).toBeChecked();
  await page.keyboard.press("a");
  await expect(radios(item).nth(0)).toBeChecked();
});

test("a11y attributes: named progressbar, legend-named fieldsets, hidden and inert inactive parts", async ({ page }) => {
  const root = await open(page);
  await expect(progress(root)).toHaveAccessibleName("Questionnaire progress");
  await expect(progress(root)).toHaveAttribute("aria-valuemin", "1");
  await expect(progress(root)).toHaveAttribute("aria-live", "polite");
  const first = items(root).nth(0);
  const legend = (await first.locator("legend").textContent()) ?? "";
  await expect(first).toHaveAccessibleName(legend.trim());
  await expect(first).toHaveAttribute("tabindex", "-1");
  for (const slot of ["previous", "skip", "submit"] as const) {
    await expect(button(root, slot)).toHaveAttribute("inert", /.*/);
    await expect(button(root, slot)).toHaveAttribute("aria-hidden", "true");
    await expect(button(root, slot)).toHaveAttribute("tabindex", "-1");
  }
  await expect(button(root, "next")).toHaveAttribute("aria-keyshortcuts", "Enter");
  // A choice row's accessible name is its label, without the aria-hidden shortcut key.
  const name = await radios(first).nth(1).evaluate((el) => (el as HTMLInputElement).labels?.[0]?.innerText ?? "");
  expect(name.length).toBeGreaterThan(0);
  await expect(freeform(first)).toHaveAccessibleName(/.+/);
});

for (const scheme of ["light", "dark"] as const) {
  test(`axe: every questionnaire demo, and the error state (${scheme})`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: scheme });
    const root = await open(page);
    for (const name of DEMOS) {
      const f = frame(page, name);
      await f.scrollIntoViewIfNeeded();
      await expect(active(f)).toBeVisible();
      await expectNoAxeViolations(page, `questionnaire: ${name} demo, ${scheme}`, { include: frameSelector(name) });
    }
    await root.scrollIntoViewIfNeeded();
    await button(root, "next").click();
    await expect(error(items(root).nth(0))).toBeVisible();
    await expectNoAxeViolations(page, `questionnaire: main demo error state, ${scheme}`, { include: frameSelector("main") });
  });
}
