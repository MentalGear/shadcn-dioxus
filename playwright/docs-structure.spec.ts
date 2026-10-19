/**
 * ORACLE: the shape of every component page's prose (backlog rows 232, 233, 236).
 *
 * The class this closes: a component page used to explain installation up to five times (header commands, a
 * one-sentence section, "Manual installation files", the Code tab, docs.md) and put a wall of docs.md prose
 * under every header. The page is now ONE accordion of closed native `<details class="dx-docs-disclosure">`:
 * "Installation" (written in `ComponentHighlight`), then every docs.md `##` section (written by `build.rs`,
 * `wrap_sections_in_disclosures`, so a new section is collapsed by construction), and a demo's own notes
 * under the demo (`variants/<variant>/notes.md`). This spec reads the REAL pages for every CATALOG row, so a
 * component added tomorrow is covered the day it is added:
 *
 *  1. exactly ONE Installation section per page, holding exactly one install command (`shadcn-dioxus add
 *     <name>`), and no `dx components add` outside the Code tab (the dx alternative lives once, on /docs);
 *  2. every disclosure is closed on load, "Manual installation files" included (it stays closed even after
 *     its parent Installation is opened: owner, row 236), and every docs.md `h2` sits inside a `<summary>`
 *     so the heading outline and axe `heading-order` keep working;
 *  3. no text-only "Examples" section survives; a demo's prose is a closed "About this example" disclosure
 *     under that demo;
 *  4. a URL fragment that points into a closed disclosure opens it (on load and on `hashchange`).
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { readCatalog } from "./catalog";
import { gotoHydrated } from "./hydration";

const NAMES = readCatalog().map((row) => row.name);
const page_url = (name: string) => `${BASE_URL}/component/${name}/`;

test("every component page has one closed Installation section with one CLI command", async ({ page }) => {
  test.setTimeout(10 * 60 * 1000);
  const problems: string[] = [];
  for (const name of NAMES) {
    // goto-ok: reads prerendered markup only (no interaction), faster than waiting for hydration on every component
    await page.goto(page_url(name), { waitUntil: "domcontentloaded" });
    const facts = await page.evaluate(() => {
      const article = document.querySelector("article.dx-component-page")!;
      const installation = [...article.querySelectorAll("h2")].filter((h) => h.textContent?.trim() === "Installation");
      const details = installation[0]?.closest("details");
      // Visible prose only: the Code tab's source is in the DOM but is not "explaining" anything.
      const visible = (article as HTMLElement).innerText;
      const manual = [...article.querySelectorAll("details")].filter((d) =>
        d.querySelector(":scope > summary")?.textContent?.includes("Manual installation files"),
      );
      return {
        h2Installation: installation.length,
        installationOpen: details?.open ?? null,
        commands: details?.querySelectorAll(".dx-component-install-commands code").length ?? 0,
        commandText: details?.querySelector(".dx-component-install-commands code")?.textContent ?? "",
        manualCount: manual.length,
        manualInsideInstallation: manual.every((m) => details?.contains(m)),
        manualOpen: manual.some((m) => m.open),
        headerHasCommand: !!article.querySelector("header .dx-component-inline-command"),
        anyCommandOutsideInstallation: [...article.querySelectorAll(".dx-component-inline-command")].some(
          (c) => !details?.contains(c),
        ),
        installLinkHref: article.querySelector("header a.dx-component-install-link")?.getAttribute("href"),
        dxInProse: /dx components add/.test(visible),
      };
    });
    const expected = `shadcn-dioxus add ${name}`;
    if (facts.h2Installation !== 1) problems.push(`${name}: ${facts.h2Installation} Installation headings`);
    if (facts.installationOpen !== false) problems.push(`${name}: Installation is not closed (${facts.installationOpen})`);
    if (facts.commands !== 1) problems.push(`${name}: ${facts.commands} install commands in Installation`);
    if (facts.commandText !== expected) problems.push(`${name}: command is "${facts.commandText}", wanted "${expected}"`);
    if (facts.anyCommandOutsideInstallation || facts.headerHasCommand)
      problems.push(`${name}: an install command outside the Installation section`);
    if (facts.manualCount !== 1) problems.push(`${name}: ${facts.manualCount} "Manual installation files" disclosures`);
    if (!facts.manualInsideInstallation) problems.push(`${name}: "Manual installation files" is outside Installation`);
    if (facts.manualOpen) problems.push(`${name}: "Manual installation files" is open on load`);
    if (facts.installLinkHref !== "#installation") problems.push(`${name}: header link is ${facts.installLinkHref}`);
    if (facts.dxInProse) problems.push(`${name}: visible text still mentions \`dx components add\``);
  }
  expect(problems, problems.join("\n")).toEqual([]);
});

test("every docs.md section is a closed disclosure with its h2 inside the summary", async ({ page }) => {
  test.setTimeout(10 * 60 * 1000);
  const problems: string[] = [];
  let sections = 0;
  for (const name of NAMES) {
    // goto-ok: reads prerendered markup only (no interaction), faster than waiting for hydration on every component
    await page.goto(page_url(name), { waitUntil: "domcontentloaded" });
    const facts = await page.evaluate(() => {
      // The page's own docs.md prose, not a demo's "About this example" disclosure body: that one wears
      // `.dx-component-description` on purpose (the prose typography in main.css) and precedes the page
      // description in the DOM on pages whose first demo has notes (attachment, avatar).
      const prose = document.querySelector("article.dx-component-page .dx-component-description:not(.dx-variant-notes *)")!;
      const disclosures = [...prose.querySelectorAll(":scope > div > details.dx-docs-disclosure")];
      return {
        disclosures: disclosures.length,
        open: disclosures.filter((d) => (d as HTMLDetailsElement).open).length,
        // A docs h2 outside a summary would be an uncollapsed section.
        looseH2: [...prose.querySelectorAll("h2")].filter((h) => !h.closest("summary")).length,
        // A summary holds exactly one h2 with an id (the anchor target).
        badSummaries: disclosures.filter((d) => {
          const h = d.querySelectorAll(":scope > summary > h2");
          return h.length !== 1 || !h[0].id.startsWith("docs-");
        }).length,
        // Anything in the prose that is not a disclosure (the intro is wrapped as "Usage notes").
        strays: [...(prose.firstElementChild?.children ?? [])].filter((c) => c.tagName !== "DETAILS").length,
        hasExamplesH2: [...prose.querySelectorAll("h2")].some((h) => h.textContent?.trim() === "Examples"),
      };
    });
    sections += facts.disclosures;
    if (facts.open) problems.push(`${name}: ${facts.open} docs disclosure(s) open on load`);
    if (facts.looseH2) problems.push(`${name}: ${facts.looseH2} docs h2 outside a summary`);
    if (facts.badSummaries) problems.push(`${name}: ${facts.badSummaries} summary without exactly one docs- h2`);
    if (facts.strays) problems.push(`${name}: ${facts.strays} docs element(s) outside a disclosure`);
    if (facts.hasExamplesH2) problems.push(`${name}: a text-only "Examples" section survives`);
  }
  expect(sections, "the pass wrapped nothing").toBeGreaterThan(NAMES.length);
  expect(problems, problems.join("\n")).toEqual([]);
});

test("a demo's prose is a closed 'About this example' disclosure under the demo", async ({ page }) => {
  // goto-ok: reads prerendered markup only (no interaction), faster than waiting for hydration on every component
  await page.goto(page_url("attachment"), { waitUntil: "domcontentloaded" });
  const notes = page.locator("details.dx-variant-notes");
  // One per attachment variant that has a notes.md (nine), each closed, each right after its own demo.
  await expect(notes).toHaveCount(9);
  for (const note of await notes.all()) {
    await expect(note).not.toHaveJSProperty("open", true);
    await expect(note.locator("> summary")).toHaveText("About this example");
  }
  const dialog = page.locator("h3", { hasText: /^dialog$/i });
  const dialogNote = dialog.locator("xpath=following-sibling::details[contains(@class,'dx-variant-notes')][1]");
  await expect(dialogNote).toContainText("AttachmentTrigger");
  await dialogNote.locator("> summary").click();
  await expect(dialogNote).toHaveJSProperty("open", true);
  await expect(dialogNote.locator("code").first()).toBeVisible();
});

test("a fragment into a closed disclosure opens it, on load and on hashchange", async ({ page }) => {
  await gotoHydrated(page, `${page_url("button")}#docs-component-structure`);
  const structure = page.locator("details.dx-docs-disclosure", { has: page.locator("#docs-component-structure") });
  await expect(structure).toHaveJSProperty("open", true);
  const installation = page.locator("details.dx-docs-disclosure", { has: page.locator("#installation") });
  await expect(installation).toHaveJSProperty("open", false);

  // The header link opens Installation and scrolls to it.
  await page.locator("header a.dx-component-install-link").click();
  await expect(installation).toHaveJSProperty("open", true);
  await expect(page.locator("#installation")).toBeInViewport();
  // Opening Installation does not open "Manual installation files" (row 236).
  await expect(installation.locator("details.dx-component-manual-install")).toHaveJSProperty("open", false);
});
