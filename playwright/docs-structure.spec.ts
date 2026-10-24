/**
 * ORACLE: the shape of every component page's prose (backlog rows 232, 233, 236).
 *
 * The class this closes: a component page that explains installation up to five times (header commands, a
 * one-sentence section, "Manual installation files", the Code tab, docs.md) and puts a wall of docs.md prose
 * under every header. The page is ONE of the library's own `Accordion`s (`.dx-docs-accordion`), all items
 * closed: "Installation" (written in `ComponentHighlight`), then every docs.md `##` section (data from
 * `build.rs`, `render_docs_sections`, so a new section is collapsed by construction), and a demo's own notes
 * are a `Collapsible` under the demo (`variants/<variant>/notes.md`). This spec reads the REAL pages for every
 * CATALOG row, so a component added tomorrow is covered the day it is added:
 *
 *  1. exactly ONE Installation section per page, holding exactly one install command (`shadcn-dioxus add
 *     <name>`), and no `dx components add` outside the Code tab (the dx alternative lives once, on /docs);
 *  2. every item is closed on load, "Manual installation files" included (it stays closed even after its
 *     parent Installation is opened, row 236), and every section's `h2` wraps its trigger so the
 *     heading outline and axe `heading-order` keep working;
 *  3. no text-only "Examples" section survives inside the docs; a demo's prose is a closed "About this
 *     example" Collapsible under that demo;
 *  4. a URL fragment that points into a closed item opens it (on load and on `hashchange`).
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { readCatalog } from "./catalog";
import { gotoHydrated } from "./hydration";

const NAMES = readCatalog().map((row) => row.name);
const page_url = (name: string) => `${BASE_URL}/component/${name}/`;
/** The accordion item that holds the element with this id. */
const itemOf = (page: import("@playwright/test").Page, id: string) =>
  page.locator(".dx-docs-accordion > .dx-accordion-item", { has: page.locator(`[id="${id}"]`) });

test("every component page has one closed Installation section with one CLI command", async ({ page }) => {
  test.setTimeout(10 * 60 * 1000);
  const problems: string[] = [];
  for (const name of NAMES) {
    // goto-ok: reads prerendered markup only (no interaction), faster than waiting for hydration on every component
    await page.goto(page_url(name), { waitUntil: "domcontentloaded" });
    const facts = await page.evaluate(() => {
      const article = document.querySelector("article.dx-component-page")!;
      const installation = [...article.querySelectorAll("h2")].filter((h) => h.textContent?.trim() === "Installation");
      const item = installation[0]?.closest(".dx-accordion-item") as HTMLElement | null;
      // Visible prose only: the Code tab's source is in the DOM but is not "explaining" anything.
      const visible = (article as HTMLElement).innerText;
      const manual = [...article.querySelectorAll<HTMLElement>(".dx-component-manual-install")].filter((d) =>
        d.querySelector(":scope > .dx-collapsible-trigger")?.textContent?.includes("Manual installation files"),
      );
      return {
        h2Installation: installation.length,
        installationOpen: item ? item.dataset.open : null,
        commands: item?.querySelectorAll(".dx-component-install-commands code").length ?? 0,
        commandText: item?.querySelector(".dx-component-install-commands code")?.textContent ?? "",
        manualCount: manual.length,
        manualInsideInstallation: manual.every((m) => item?.contains(m)),
        manualOpen: manual.some((m) => m.dataset.open === "true"),
        headerHasCommand: !!article.querySelector("header .dx-component-inline-command"),
        anyCommandOutsideInstallation: [...article.querySelectorAll(".dx-component-inline-command")].some(
          (c) => !item?.contains(c),
        ),
        installLinkHref: article.querySelector("header a.dx-component-install-link")?.getAttribute("href"),
        dxInProse: /dx components add/.test(visible),
      };
    });
    const expected = `shadcn-dioxus add ${name}`;
    if (facts.h2Installation !== 1) problems.push(`${name}: ${facts.h2Installation} Installation headings`);
    if (facts.installationOpen !== "false") problems.push(`${name}: Installation is not closed (${facts.installationOpen})`);
    if (facts.commands !== 1) problems.push(`${name}: ${facts.commands} install commands in Installation`);
    if (facts.commandText !== expected) problems.push(`${name}: command is "${facts.commandText}", wanted "${expected}"`);
    if (facts.anyCommandOutsideInstallation || facts.headerHasCommand)
      problems.push(`${name}: an install command outside the Installation section`);
    if (facts.manualCount !== 1) problems.push(`${name}: ${facts.manualCount} "Manual installation files" collapsibles`);
    if (!facts.manualInsideInstallation) problems.push(`${name}: "Manual installation files" is outside Installation`);
    if (facts.manualOpen) problems.push(`${name}: "Manual installation files" is open on load`);
    if (facts.installLinkHref !== "#installation") problems.push(`${name}: header link is ${facts.installLinkHref}`);
    if (facts.dxInProse) problems.push(`${name}: visible text still mentions \`dx components add\``);
  }
  expect(problems, problems.join("\n")).toEqual([]);
});

test("every docs.md section is a closed accordion item with its h2 wrapping the trigger", async ({ page }) => {
  test.setTimeout(10 * 60 * 1000);
  const problems: string[] = [];
  let sections = 0;
  for (const name of NAMES) {
    // goto-ok: reads prerendered markup only (no interaction), faster than waiting for hydration on every component
    await page.goto(page_url(name), { waitUntil: "domcontentloaded" });
    const facts = await page.evaluate(() => {
      const accordion = document.querySelector("article.dx-component-page .dx-docs-accordion")!;
      const items = [...accordion.querySelectorAll<HTMLElement>(":scope > .dx-accordion-item")];
      const docsItems = items.filter((i) => i.querySelector(":scope > h2")?.id !== "installation");
      return {
        sections: docsItems.length,
        open: items.filter((i) => i.dataset.open !== "false").length,
        // Closed content is in the markup (`hidden="until-found"`), never unmounted or shown.
        notUntilFound: items.filter(
          (i) => i.querySelector(":scope > .dx-accordion-content")?.getAttribute("hidden") !== "until-found",
        ).length,
        // A docs h2 that is not an item heading would be an uncollapsed section.
        looseH2: [...accordion.querySelectorAll("h2")].filter((h) => !h.matches(".dx-accordion-item > h2")).length,
        // An item heading is one h2 with a `docs-` id that wraps the item's trigger.
        badHeadings: docsItems.filter((i) => {
          const h = i.querySelectorAll(":scope > h2");
          return (
            h.length !== 1 ||
            !h[0].id.startsWith("docs-") ||
            h[0].querySelectorAll(":scope > button.dx-accordion-trigger[aria-expanded]").length !== 1
          );
        }).length,
        // Anything in the accordion that is not an item (the intro is wrapped as "Usage notes").
        strays: [...accordion.children].filter((c) => !c.classList.contains("dx-accordion-item")).length,
        hasExamplesH2: [...accordion.querySelectorAll("h2")].some((h) => h.textContent?.trim() === "Examples"),
        nativeDisclosures: document.querySelectorAll("article.dx-component-page details, article.dx-component-page summary").length,
      };
    });
    sections += facts.sections;
    if (facts.open) problems.push(`${name}: ${facts.open} docs item(s) open on load`);
    if (facts.notUntilFound) problems.push(`${name}: ${facts.notUntilFound} closed item(s) not hidden="until-found"`);
    if (facts.looseH2) problems.push(`${name}: ${facts.looseH2} docs h2 outside an item heading`);
    if (facts.badHeadings) problems.push(`${name}: ${facts.badHeadings} item(s) without one docs- h2 around the trigger`);
    if (facts.strays) problems.push(`${name}: ${facts.strays} docs element(s) outside an item`);
    if (facts.hasExamplesH2) problems.push(`${name}: a text-only "Examples" section survives in the docs`);
    if (facts.nativeDisclosures) problems.push(`${name}: ${facts.nativeDisclosures} native details/summary element(s)`);
  }
  expect(sections, "the pass wrapped nothing").toBeGreaterThan(NAMES.length);
  expect(problems, problems.join("\n")).toEqual([]);
});

test("a demo's prose is a closed 'About this example' Collapsible under the demo", async ({ page }) => {
  await gotoHydrated(page, page_url("attachment"));
  const notes = page.locator(".dx-variant-notes");
  // One per attachment variant that has a notes.md (nine), each closed, each right after its own demo.
  await expect(notes).toHaveCount(9);
  for (const note of await notes.all()) {
    await expect(note).toHaveAttribute("data-open", "false");
    await expect(note.locator("> .dx-collapsible-trigger")).toHaveText("About this example");
  }
  const dialog = page.locator("h3", { hasText: /^dialog$/i });
  const dialogNote = dialog.locator("xpath=following-sibling::div[contains(@class,'dx-variant-notes')][1]");
  await expect(dialogNote).toContainText("AttachmentTrigger");
  await dialogNote.locator("> .dx-collapsible-trigger").click();
  await expect(dialogNote).toHaveAttribute("data-open", "true");
  await expect(dialogNote.locator("code").first()).toBeVisible();
});

test("a fragment into a closed item opens it, on load and on hashchange", async ({ page }) => {
  await gotoHydrated(page, `${page_url("button")}#docs-component-structure`);
  const structure = itemOf(page, "docs-component-structure");
  await expect(structure).toHaveAttribute("data-open", "true");
  const installation = itemOf(page, "installation");
  await expect(installation).toHaveAttribute("data-open", "false");

  // The header link opens Installation and scrolls to it.
  await page.locator("header a.dx-component-install-link").click();
  await expect(installation).toHaveAttribute("data-open", "true");
  await expect(page.locator("#installation")).toBeInViewport();
  // Opening Installation does not open "Manual installation files" (row 236).
  await expect(installation.locator(".dx-component-manual-install")).toHaveAttribute("data-open", "false");
});
