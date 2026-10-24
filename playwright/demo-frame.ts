import type { Locator, Page } from "@playwright/test";

/**
 * Locating a demo's controls from a SHARED spec (backlog row 220).
 *
 * A shared oracle spec (keyboard matrix, main-thread, top-layer ink, rtl, ...) drives the demo pages of many
 * components. Locating a control there by the demo's visible COPY (`getByRole("button", { name: "Open Menu" })`,
 * `getByText("Selected: Duplicate")`) ties every one of those specs to wording nobody meant as a contract: a demo that mirrors shadcn's example more
 * closely (row 212's labelled demos, row 155 group 3's toggles) renames a button and breaks `keyboard-matrix`,
 * `main-thread`, `rtl` and `theme-preset-contrast` at once.
 *
 * The construction: the spec names a control by its ARIA ROLE and its POSITION inside the demo frame --
 * `demoRole(page, "button")` is the first button of the page's main demo, `demoRole(page, "tab", 2)` its third
 * tab -- and an item of an OPEN popup by its position in that popup (`popupItem(page, "menu", "menuitem", 2)`).
 * The main demo's frame has the one stable id the docs page template gives it (`#component-preview-frame`;
 * every other variant's is `component-preview-frame-<name>`, preview/src/main.rs). Names that belong to the
 * COMPONENT rather than to a demo (a Close button's `aria-label`, a carousel's "Previous slide") are not demo
 * copy and may still be used; `scripts/check-demo-text-locators.sh` fails any other text locator in a shared
 * spec, with a reasoned allowlist.
 *
 * Positions are the demo's CURRENT order, written next to the helper call in a comment where it is not obvious
 * (`// dropdown_menu demo: Edit, Undo (disabled), Duplicate, More tools, Delete`). A demo that adds a control
 * BEFORE the one a spec drives moves the position; that is a visible, local edit, where a copy change breaks
 * the spec silently from the other side.
 */
export const DEMO_FRAME = "#component-preview-frame";

type Role = Parameters<Page["getByRole"]>[0];

/**
 * A demo's frame: the page's main demo (`variant` omitted or `"main"`, id `component-preview-frame`) or another
 * variant by its code name (`"checkboxes"` -> `#component-preview-frame-checkboxes`). The variant names are the
 * directory names under `preview/src/components/<c>/variants/`, i.e. identifiers, not demo copy.
 */
export function demoFrame(page: Page, variant = "main"): Locator {
  return page.locator(variant === "main" ? DEMO_FRAME : `${DEMO_FRAME}-${variant}`).first();
}

/** The `n`th (document order, 0-based) element of ARIA role `role` inside a demo frame (the main one by default). */
export function demoRole(page: Page, role: Role, n = 0, variant = "main"): Locator {
  return demoFrame(page, variant).getByRole(role).nth(n);
}

/** Every element of ARIA role `role` inside a demo frame (the main one by default). */
export function demoRoles(page: Page, role: Role, variant = "main"): Locator {
  return demoFrame(page, variant).getByRole(role);
}

/**
 * The `n`th (0-based) `item` (`menuitem`, `option`, ...) of the LAST `container` popup (`menu`, `listbox`,
 * ...) in the document: popups portal to the end of the body, so the last one is the most recently opened (and
 * a submenu comes after its parent). Scoped to the container role, not to the frame, because a popup is not a
 * DOM descendant of the demo.
 */
export function popupItem(page: Page, container: Role, item: Role, n = 0): Locator {
  return page.getByRole(container).last().getByRole(item).nth(n);
}

/** All `item`s of the LAST `container` popup, for `.first()` / `.last()` / `.count()`. */
export function popupItems(page: Page, container: Role, item: Role): Locator {
  return page.getByRole(container).last().getByRole(item);
}
