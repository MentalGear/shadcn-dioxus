/**
 * ORACLE: the shape of every component page's prose (backlog rows 232, 233, 236, 422).
 *
 * The class this closes: a component page that explains installation up to five times (header commands, a
 * one-sentence section, "Manual installation files", the Code tab, docs.md) and puts a wall of docs.md prose
 * under every header. The docs are two groups, each an `h2` and one of the library's own `Accordion`s
 * (`.dx-docs-accordion`), then "Examples" (row 422): Basic (Usage notes, Installation, Component structure,
 * Props reference, the same order on every page) and Component-Specific (the docs.md `##` sections in
 * authored order, then Keyboard, Accessibility, Motion, Direction / RTL, Styling). Every item is closed on
 * load except Usage notes. Installation is written in `ComponentHighlight`, the rest is `build.rs` data
 * (`render_docs_sections`, ordered by `preview/docs_groups.rs`), so a new section is collapsed by
 * construction, and a demo's own notes are a `Collapsible` under the demo (`variants/<variant>/notes.md`).
 * This spec reads the REAL pages for every CATALOG row, so a component added tomorrow is covered the day it
 * is added:
 *
 *  1. exactly ONE Installation section per page, holding exactly one install command (`shadcn-dioxus add
 *     <name>`), and no `dx components add` outside the Code tab (the dx alternative lives once, on /docs);
 *  2. every item is closed on load except Usage notes, "Manual installation files" included (it stays
 *     closed even after its parent Installation is opened, row 236), and every section's `h3` wraps its
 *     trigger so the heading outline and axe `heading-order` keep working;
 *  3. the Basic group renders its four sections in the one order, the Component-Specific group keeps the
 *     recurring topics last in their fixed order, and both group headings are `h2`s that precede their items;
 *  4. no text-only "Examples" section survives inside the docs; a demo's prose is a closed "About this
 *     example" Collapsible under that demo;
 *  5. a URL fragment that points into a closed item opens it (on load and on `hashchange`).
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { readCatalog } from "./catalog";
import { gotoHydrated } from "./hydration";
import { gotoPrerendered } from "./page-kinds";

const NAMES = readCatalog().map((row) => row.name);
const page_url = (name: string) => `${BASE_URL}/component/${name}/`;
/** The accordion item that holds the element with this id. */
/** The recurring topics that close the Component-Specific group, in page order (`preview/docs_groups.rs` `TAIL`). */
const TAIL = ["keyboard", "accessibility", "motion", "direction / rtl", "styling"];
/** The Basic group, top to bottom, as item heading ids (`docs_groups::Basic::id`) and titles. */
const BASIC_IDS = ["docs-usage-notes", "installation", "docs-component-structure", "docs-props-reference"];
const BASIC_TITLES = ["Usage notes", "Installation", "Component structure", "Props reference"];
/** The position of the TAIL topic a heading starts with (whole word, any case), or -1 for a page's own section. */
const tailRank = (title: string) =>
  TAIL.findIndex((topic) => {
    const t = title.trim().toLowerCase();
    return t.startsWith(topic) && !/^[a-z0-9]/.test(t.slice(topic.length));
  });
const itemOf = (page: import("@playwright/test").Page, id: string) =>
  page.locator(".dx-docs-accordion > .dx-accordion-item", { has: page.locator(`[id="${id}"]`) });

test("every component page has one closed Installation section with one CLI command", async ({ page }) => {
  test.setTimeout(10 * 60 * 1000);
  const problems: string[] = [];
  for (const name of NAMES) {
    await gotoPrerendered(page, page_url(name), { waitUntil: "domcontentloaded" });
    const facts = await page.evaluate(() => {
      const article = document.querySelector("article.dx-component-page")!;
      const installation = [...article.querySelectorAll("h3")].filter((h) => h.textContent?.trim() === "Installation");
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

test("every docs section is an accordion item, closed except Usage notes, with its h3 wrapping the trigger", async ({ page }) => {
  test.setTimeout(10 * 60 * 1000);
  const problems: string[] = [];
  let sections = 0;
  for (const name of NAMES) {
    await gotoPrerendered(page, page_url(name), { waitUntil: "domcontentloaded" });
    const facts = await page.evaluate(() => {
      const accordions = [...document.querySelectorAll("article.dx-component-page .dx-docs-accordion")];
      const items = accordions.flatMap((a) => [...a.querySelectorAll<HTMLElement>(":scope > .dx-accordion-item")]);
      const docsItems = items.filter((i) => i.querySelector(":scope > h3")?.id !== "installation");
      const openIds = items
        .filter((i) => i.dataset.open !== "false")
        .map((i) => i.querySelector(":scope > h3")?.id ?? "?");
      return {
        sections: docsItems.length,
        openIds,
        // Closed content is in the markup (`hidden="until-found"`), never unmounted or shown.
        notUntilFound: items.filter(
          (i) =>
            i.dataset.open === "false" &&
            i.querySelector(":scope > .dx-accordion-content")?.getAttribute("hidden") !== "until-found",
        ).length,
        // A docs heading that is not an item heading would be an uncollapsed section.
        looseHeading:
          accordions.flatMap((a) => [...a.querySelectorAll("h2")]).length +
          accordions.flatMap((a) => [...a.querySelectorAll("h3")]).filter((h) => !h.matches(".dx-accordion-item > h3")).length,
        // An item heading is one h3 with a `docs-` id that wraps the item's trigger.
        badHeadings: docsItems.filter((i) => {
          const h = i.querySelectorAll(":scope > h3");
          return (
            h.length !== 1 ||
            !h[0].id.startsWith("docs-") ||
            h[0].querySelectorAll(":scope > button.dx-accordion-trigger[aria-expanded]").length !== 1
          );
        }).length,
        // Anything in an accordion that is not an item (the intro is wrapped as "Usage notes").
        strays: accordions.flatMap((a) => [...a.children]).filter((c) => !c.classList.contains("dx-accordion-item")).length,
        hasExamplesH2: accordions.some((a) => [...a.querySelectorAll("h2, h3")].some((h) => h.textContent?.trim() === "Examples")),
        nativeDisclosures: document.querySelectorAll("article.dx-component-page details, article.dx-component-page summary").length,
      };
    });
    sections += facts.sections;
    if (facts.openIds.join() !== "docs-usage-notes") problems.push(`${name}: open on load: [${facts.openIds}], wanted only docs-usage-notes`);
    if (facts.notUntilFound) problems.push(`${name}: ${facts.notUntilFound} closed item(s) not hidden="until-found"`);
    if (facts.looseHeading) problems.push(`${name}: ${facts.looseHeading} docs heading(s) outside an item heading`);
    if (facts.badHeadings) problems.push(`${name}: ${facts.badHeadings} item(s) without one docs- h3 around the trigger`);
    if (facts.strays) problems.push(`${name}: ${facts.strays} docs element(s) outside an item`);
    if (facts.hasExamplesH2) problems.push(`${name}: a text-only "Examples" section survives in the docs`);
    if (facts.nativeDisclosures) problems.push(`${name}: ${facts.nativeDisclosures} native details/summary element(s)`);
  }
  expect(sections, "the pass wrapped nothing").toBeGreaterThan(NAMES.length);
  expect(problems, problems.join("\n")).toEqual([]);
});

test("every component page renders the Basic group in one order and the recurring topics last", async ({ page }) => {
  test.setTimeout(10 * 60 * 1000);
  const problems: string[] = [];
  let specificPages = 0;
  for (const name of NAMES) {
    await gotoPrerendered(page, page_url(name), { waitUntil: "domcontentloaded" });
    const facts = await page.evaluate(() => {
      const article = document.querySelector("article.dx-component-page")!;
      const groups = [...article.querySelectorAll<HTMLElement>("section[data-docs-group]")];
      const describe = (group: HTMLElement | undefined) =>
        group
          ? {
              heading: group.querySelector(":scope > .dx-component-section-heading > h2")?.textContent?.trim() ?? null,
              items: [...group.querySelectorAll<HTMLElement>(":scope > .dx-docs-accordion > .dx-accordion-item > h3")].map(
                (h) => ({ id: h.id, title: h.textContent?.trim() ?? "" }),
              ),
            }
          : null;
      // The headings the docs add to the outline, in document order: group h2s, item h3s, the Examples h2.
      const outline = [...article.querySelectorAll("section.dx-component-section h2, .dx-accordion-item > h3")].map((h) => h.tagName);
      const examples = [...article.querySelectorAll("h2")].find((h) => h.textContent?.trim() === "Examples");
      const lastGroup = groups[groups.length - 1];
      return {
        order: groups.map((g) => g.dataset.docsGroup),
        basic: describe(groups.find((g) => g.dataset.docsGroup === "basic")),
        specific: describe(groups.find((g) => g.dataset.docsGroup === "component-specific")),
        outline,
        examplesAfterGroups:
          !examples || !lastGroup || !!(lastGroup.compareDocumentPosition(examples) & Node.DOCUMENT_POSITION_FOLLOWING),
      };
    });
    const order = facts.order.join();
    if (order !== "basic" && order !== "basic,component-specific") problems.push(`${name}: groups are [${order}]`);
    if (facts.basic?.heading !== "Basic") problems.push(`${name}: Basic group heading is ${facts.basic?.heading}`);
    if (facts.basic?.items.map((i) => i.id).join() !== BASIC_IDS.join())
      problems.push(`${name}: Basic items are [${facts.basic?.items.map((i) => i.id)}], wanted [${BASIC_IDS}]`);
    if (facts.basic?.items.map((i) => i.title).join() !== BASIC_TITLES.join())
      problems.push(`${name}: Basic titles are [${facts.basic?.items.map((i) => i.title)}], wanted [${BASIC_TITLES}]`);
    if (facts.specific) {
      specificPages++;
      if (facts.specific.heading !== "Component-Specific") problems.push(`${name}: specific heading is ${facts.specific.heading}`);
      const ranks = facts.specific.items.map((i) => tailRank(i.title));
      const firstTail = ranks.findIndex((r) => r >= 0);
      const tail = firstTail < 0 ? [] : ranks.slice(firstTail);
      if (tail.some((r) => r < 0)) problems.push(`${name}: a page-specific section follows a recurring topic`);
      if (tail.some((r, i) => i > 0 && r < tail[i - 1])) problems.push(`${name}: recurring topics out of order: [${tail}]`);
      if (facts.specific.items.some((i) => BASIC_IDS.includes(i.id))) problems.push(`${name}: a Basic section sits in Component-Specific`);
    }
    // Outline: a group `h2` comes before the item `h3`s it holds, and Examples closes the docs.
    if (facts.outline[0] !== "H2") problems.push(`${name}: an item heading comes before the first group heading`);
    if (!facts.examplesAfterGroups) problems.push(`${name}: Examples comes before the docs groups`);
  }
  expect(specificPages, "the Component-Specific group never rendered").toBeGreaterThan(NAMES.length / 2);
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
  // Usage notes is the one item open without a fragment.
  await expect(itemOf(page, "docs-usage-notes")).toHaveAttribute("data-open", "true");
  const installation = itemOf(page, "installation");
  await expect(installation).toHaveAttribute("data-open", "false");

  // The header link opens Installation and scrolls to it.
  await page.locator("header a.dx-component-install-link").click();
  await expect(installation).toHaveAttribute("data-open", "true");
  await expect(page.locator("#installation")).toBeInViewport();
  // Opening Installation does not open "Manual installation files" (row 236).
  await expect(installation.locator(".dx-component-manual-install")).toHaveAttribute("data-open", "false");
});
