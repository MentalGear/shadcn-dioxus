/**
 * The optional `markdown` component (backlog row 213; owner decisions 2026-10-15). What the specs pin:
 *
 * - structure: the main demo renders headings, lists, task checkboxes, a table and a code block;
 * - the link policy: web links open in a new tab with `rel="noopener noreferrer"` and a real popup has no
 *   opener; relative links do not; a `javascript:` link keeps its label and has no `href` anywhere;
 * - images are off by default (no `<img>`, no request) and appear only after the opt-in, never for a script URL;
 * - an XSS attempt renders inert: no script, image, iframe or handler reaches the DOM and nothing runs;
 * - the code-block slot receives the language and the code;
 * - the streaming heal pass: while a reply streams, no frame shows raw `**`, a backtick or `](`; with the
 *   heal switched off the markers do show (so the first claim is not vacuous); the scroller follows the end;
 * - axe on every demo, light and dark.
 *
 * Pure-function coverage (link allowlist, heal on every prefix, autolinks, options) is the component's
 * `cargo test` module; these specs are the browser half.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO = { timeout: 20 * 60 * 1000 };
const URL = `${BASE_URL}/component/markdown/`;

const main = (page: Page) => page.locator("#component-preview-frame").first();
const variant = (page: Page, name: string) => page.locator(`#component-preview-frame-${name}`).first();
const prose = (frame: Locator) => frame.locator('[data-slot="markdown"]').first();

async function open(page: Page, frame: (page: Page) => Locator = main) {
  await gotoHydrated(page, URL, GOTO);
  const root = frame(page);
  await root.scrollIntoViewIfNeeded();
  await expect(prose(root)).toBeVisible({ timeout: 60_000 });
  await stylesApplied(root);
  return root;
}

/** The component's stylesheet is linked after mount: wait until it has applied. */
async function stylesApplied(root: Locator) {
  await expect
    .poll(() => prose(root).evaluate((el) => getComputedStyle(el).overflowWrap), { timeout: 60_000 })
    .toBe("anywhere");
}

/** The streaming demo has no text until a reply is streamed: its "stream-reply" button starts one. */
async function openStreaming(page: Page) {
  await gotoHydrated(page, URL, GOTO);
  const root = variant(page, "streaming");
  await root.scrollIntoViewIfNeeded();
  await expect(root.getByTestId("stream-reply")).toBeEnabled({ timeout: 60_000 });
  return root;
}

test("the main demo renders headings, lists, task checkboxes, a table and a code block", async ({ page }) => {
  const root = await open(page);
  await expect(root.locator("h2")).toHaveText("Release notes");
  await expect(root.locator("ol > li")).toHaveCount(2);
  await expect(root.locator("ol ul > li")).toHaveCount(2);
  const tasks = root.locator('input[type="checkbox"]');
  await expect(tasks).toHaveCount(2);
  await expect(tasks.first()).toBeChecked();
  await expect(tasks.first()).toBeDisabled();
  await expect(tasks.first()).toHaveAccessibleName("Completed");
  await expect(root.locator("table th[scope=col]")).toHaveCount(2);
  await expect(root.locator("td").last()).toHaveText("frame");
  await expect(root.locator("pre code.language-rust")).toContainText("println!");
  await expect(root.locator("blockquote")).toBeVisible();
  await expect(root.locator("del")).toHaveText("strikethrough");
  // The bare URL became a link.
  await expect(root.locator('a[href="https://example.com/docs"]')).toHaveCount(1);
});

test("link policy: new tab for web links with noopener, none for relative, no target for script links", async ({
  page,
}) => {
  const root = await open(page, (p) => variant(p, "links"));
  const web = root.locator('a[href="https://example.com/page"]');
  await expect(web).toHaveAttribute("target", "_blank");
  await expect(web).toHaveAttribute("rel", "noopener noreferrer");
  const relative = root.locator('a[href="/component/button/"]');
  await expect(relative).not.toHaveAttribute("target", /.*/);
  await expect(root.locator('a[href^="mailto:"]')).not.toHaveAttribute("target", /.*/);
  await expect(root.locator('a[href^="tel:"]')).toHaveCount(1);

  // The refused links keep their label and have no target at all.
  const blocked = root.locator('[data-blocked="link"]');
  await expect(blocked).toHaveCount(2);
  await expect(blocked.first()).toHaveText("A script link");
  await expect(page.locator('a[href^="javascript" i], a[href^="data:" i]')).toHaveCount(0);
  await expect(root.locator("a:not([href])")).toHaveCount(0);

  // A real click: a new tab opens and it has no `window.opener`.
  await page.context().route("https://example.com/**", (route) => route.fulfill({ status: 200, body: "<title>ok</title>" }));
  const [popup] = await Promise.all([page.waitForEvent("popup"), web.click()]);
  expect(await popup.evaluate(() => window.opener)).toBeNull();
  await popup.close();
  await expect(page).toHaveURL(/\/component\/markdown/);
});

test("images are off by default and load only after the opt-in; a script source never loads", async ({ page }) => {
  const requested: string[] = [];
  page.on("request", (request) => {
    if (request.resourceType() === "image") requested.push(request.url());
  });
  const root = await open(page, (p) => variant(p, "images"));
  const box = root.locator('input[type="checkbox"]').first();
  await expect(box).not.toBeChecked();
  await expect(prose(root).locator("img")).toHaveCount(0);
  const alts = prose(root).locator('[data-blocked="image"]');
  await expect(alts).toHaveCount(2);
  await expect(alts.first()).toHaveText("the shadcn-dioxus logo");
  expect(requested.filter((u) => u.includes("favicon"))).toEqual([]);

  await box.check();
  const img = prose(root).locator("img");
  await expect(img).toHaveCount(1);
  await expect(img).toHaveAttribute("referrerpolicy", "no-referrer");
  await expect(img).toHaveAttribute("loading", "lazy");
  await expect(img).toHaveAttribute("alt", "the shadcn-dioxus logo");
  // The script-source image stays text even with images on.
  await expect(prose(root).locator('[data-blocked="image"]')).toHaveText("a script image");

  await box.uncheck();
  await expect(prose(root).locator("img")).toHaveCount(0);
});

test("an XSS attempt renders inert: the markup is text and nothing runs", async ({ page }) => {
  const dialogs: string[] = [];
  page.on("dialog", (d) => {
    dialogs.push(d.message());
    void d.dismiss();
  });
  const root = await open(page, (p) => variant(p, "xss"));
  const text = prose(root);
  for (const tag of ["script", "img", "iframe", "svg", "b", "a", "style"]) {
    await expect(text.locator(tag), `<${tag}> must not be an element`).toHaveCount(0);
  }
  // The attempts are visible as literal text.
  await expect(text).toContainText("<script>window.__pwned = true</script>");
  await expect(text).toContainText('<img src=x onerror="window.__pwned = true">');
  await expect(text).toContainText("raw anchor");
  // No element carries an event handler or a script URL.
  expect(
    await text.evaluate(
      (el) =>
        [el, ...el.querySelectorAll("*")].filter(
          (n) =>
            [...n.attributes].some((a) => /^on/i.test(a.name)) ||
            [...n.attributes].some((a) => /^\s*(javascript|vbscript|data):/i.test(a.value)),
        ).length,
    ),
  ).toBe(0);
  // The two well-formed script links are inert labels. A destination with a tab or space is not a link to
  // the parser at all, so that one stays literal text (`safe_href` also refuses it, unit-tested).
  await expect(text.locator('[data-blocked="link"]')).toHaveCount(2);
  await expect(text).toContainText("[tab](java");
  // Give any injected handler time to fire, then prove nothing ran.
  await page.waitForTimeout(500);
  expect(await page.evaluate(() => (window as unknown as { __pwned?: boolean }).__pwned)).toBeUndefined();
  expect(dialogs).toEqual([]);
});

test("the highlight slot receives the language and the code", async ({ page }) => {
  const root = await open(page, (p) => variant(p, "code_slot"));
  const blocks = root.locator('[data-highlighter="demo"]');
  await expect(blocks).toHaveCount(2);
  await expect(blocks.first()).toHaveClass(/language-rust/);
  await expect(blocks.first().locator("span[style*='display: block']")).toHaveCount(3);
  await expect(blocks.first()).toContainText("a + b");
  await expect(blocks.first().locator("strong").first()).toHaveText("fn ");
  // No language: the slot still runs, with `text`.
  await expect(blocks.nth(1)).toHaveClass(/language-text/);
  // The default block (inline code in the prose) is untouched.
  await expect(root.locator("code.dx-markdown-code").first()).toBeVisible();
});

/** Streams the demo's reply and returns every distinct text the markdown showed, from first to last mutation. */
async function streamAndSample(page: Page, root: Locator): Promise<string[]> {
  await root.evaluate((el) => {
    const w = window as unknown as { __samples: string[] };
    w.__samples = [];
    const read = () => el.querySelector('[data-slot="markdown"]')?.textContent ?? "";
    new MutationObserver(() => {
      const t = read();
      if (t && w.__samples[w.__samples.length - 1] !== t) w.__samples.push(t);
    }).observe(el, { subtree: true, childList: true, characterData: true });
  });
  const run = root.getByTestId("stream-reply");
  await run.click();
  await expect(run).toBeDisabled();
  await expect(run).toBeEnabled({ timeout: 60_000 });
  return page.evaluate(() => (window as unknown as { __samples: string[] }).__samples);
}

test("streaming: the heal pass keeps raw markers out of every frame, and the scroller follows the end", async ({
  page,
}) => {
  const root = await openStreaming(page);
  const samples = await streamAndSample(page, root);
  expect(samples.length, "the reply arrived in many frames").toBeGreaterThan(20);
  const raw = samples.filter((t) => t.includes("**") || t.includes("`") || t.includes("]("));
  expect(raw, "no frame shows a raw `**`, backtick or `](`").toEqual([]);
  // The finished reply is real markup.
  const md = prose(root);
  await expect(md.locator("strong").first()).toHaveText("quick plan");
  await expect(md.locator("pre code.language-rust")).toContainText("ship it");
  await expect(md.locator("table")).toBeVisible();
  await expect(md.locator('a[href="https://example.com/faq"]')).toHaveCount(1);
  // At the live edge the scroller stayed with the newest text.
  const gap = await root
    .locator("[data-message-scroller-viewport]")
    .evaluate((el) => el.scrollHeight - el.clientHeight - el.scrollTop);
  expect(gap).toBeLessThanOrEqual(16);
});

test("streaming: with the heal switched off the markers do show (the heal claim is not vacuous)", async ({
  page,
}) => {
  const root = await openStreaming(page);
  await root.getByRole("switch").click();
  await expect(root.getByRole("switch")).not.toBeChecked();
  const samples = await streamAndSample(page, root);
  expect(samples.some((t) => t.includes("**") || t.includes("`") || t.includes("](")), "raw markers appear mid-stream").toBe(true);
  // Once the reply is complete the final render is the same real markup either way.
  await expect(prose(root).locator("strong").first()).toHaveText("quick plan");
});

for (const scheme of ["light", "dark"] as const) {
  test(`axe: every markdown demo (${scheme})`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: scheme });
    await open(page);
    for (const name of ["main", "streaming", "links", "images", "code_slot", "xss"]) {
      const frame = name === "main" ? main(page) : variant(page, name);
      await frame.scrollIntoViewIfNeeded();
      if (name === "streaming") {
        const run = frame.getByTestId("stream-reply");
        await run.click();
        await expect(run).toBeEnabled({ timeout: 60_000 });
      }
      await expect(prose(frame)).toBeVisible();
      await stylesApplied(frame);
      await expectNoAxeViolations(page, `markdown: ${name} demo, ${scheme}`, {
        include: name === "main" ? "#component-preview-frame" : `#component-preview-frame-${name}`,
        excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT],
      });
    }
  });
}
