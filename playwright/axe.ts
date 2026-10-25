/**
 * ORACLE: static accessibility rules (axe-core), shared across specs.
 *
 * Source: `axe-core`'s own rule set, tagged to the WCAG success criteria it
 * implements (`wcag2a`/`wcag2aa`/`wcag21a`/`wcag21aa`) plus its curated
 * `best-practice` rules (things every mature a11y linter flags -- e.g.
 * duplicate landmarks -- that are not themselves a numbered WCAG SC).
 *
 * This is a *different* class of check than every other oracle in this
 * harness. Tiers 1-3 (`dev-docs/conformance-harness.md`) test *behaviour*:
 * does Escape move focus where APG says, does a required control block
 * form submission, does an anchored overlay track its trigger. axe tests
 * *static* accessibility rules on whatever DOM exists at the moment it
 * runs: valid ARIA role/attribute combinations, accessible names, unique
 * landmarks, label association, contrast, list/table structure. A
 * component can pass every keyboard/behaviour oracle in this repo and
 * still fail axe (a duplicate landmark, an invalid `aria-required` on a
 * `button`, a `role="menu"` popup with no accessible name) -- exactly the
 * defect class this file exists to catch (backlog row 25: ContextMenu/Menubar menu popups
 * with no accessible name; the toast `aria-required` cases).
 *
 * ## Where each state is scanned
 *
 * At rest: `all-routes.spec.ts` scans every page of the preview (every
 * CATALOG component page, the site pages, every block demo) as it loads,
 * in light and in dark, with this helper and no rule disabled. A component
 * spec therefore does not scan its own page at load: the sweep covers the same page, the
 * same tag set and both schemes (backlog row 152).
 * `tools/select.mjs` adds the changed component's `all-routes` test to a
 * `test:changed` run, so a lane still gets the at-rest scan of what it
 * touched.
 *
 * Reached by interaction: every component spec that reaches an
 * open/expanded/selected/invalid state scans it after its existing "open
 * the overlay" step -- reusing whatever locator/interaction that spec
 * already has, never a new fixture invented for this file. A spec also
 * keeps an at-load scan the sweep does not reproduce: another viewport
 * (`drawer.spec.ts` runs at phone width), a page outside the sweep
 * (`/charts/<tab>/`, the dashboard), or markup that depends on a request
 * the sweep aborts (`avatar.spec.ts`'s pending image).
 *
 * ## Exclusions are audited, never a way to force green
 *
 * `disableRules` takes a `reason` per rule id and refuses (throws) an
 * empty one -- see `expectNoAxeViolations` below. This is deliberate:
 * excluding a rule silently is how a real, unfixed defect goes invisible
 * to every future run. A rule may be excluded ONLY for a documented false
 * positive at that specific scan site (axe misjudging something that is
 * not actually a defect there); a real, larger issue that is not fixed
 * (a structural markup change, a vendored third-party asset) must stay
 * enabled and RED, recorded in `dev-docs/backlog.md` instead.
 *
 * ## `color-contrast`: fixed by construction, not excluded
 *
 * The scans run with no `color-contrast` exclusion. Text colour is a theme token (`--dx-muted-foreground` and
 * the other roles in `preview/assets/dx-components-theme.css`) that clears the 4.5:1 WCAG asks of normal text on
 * the surfaces it is paired with (shadcn's own values by default; the calibrated `data-dx-contrast="wcag"` set is
 * pinned by `contrast-mode.spec.ts`), so a component never carries a literal colour of its own to fall short.
 * Residue beyond that is listed in `dev-docs/backlog.md` row 39.
 *
 * `EXCLUDE_VENDORED_CODE_HIGHLIGHT` scopes out nothing: the syntax-highlighted code spans (the "Manual
 * installation"/component-source viewer, `preview/src/site/code.rs`'s `CodeBlock` wrapping a `PreviewCode`, and the
 * same highlighter's output in a component's "Usage notes" prose, all under `.dx-preview-code-theme`) clear
 * contrast by construction. The `github-light` theme's CSS is not source this repo owns: it is generated from
 * the `dioxus-code`/`arborium-theme` crates into `docs/assets/github-light*.css` on every
 * `scripts/deploy-preview.sh` run, so the contrast fix lives in `preview/assets/main.css`: an override rule
 * `.dx-preview-code-theme <theme class>` (two classes, beating the theme's own single-class rule whatever
 * the stylesheet order) for the theme's three comment-family custom properties
 * (`--dxc-*-a-c-color`/`--dxc-*-a-cd-color`/`--dxc-*-var-muted`) across every class variant the crate
 * generates for `github-light` (plain, `-system-light-`, `-system-dark-`). The upstream comment token
 * `#6e7781` is below 4.5:1 on the code block's `--dx-card` background, so the override sets it to `#59636e`
 * (GitHub's own light-theme comment color). The paired `github-dark` theme (`data-theme=dark`) needs no
 * override. `preview/assets/main.css` carries the same detail beside the rule.
 *
 * The constant stays exported as an `AxeRegionExclusion` with a selector that excludes nothing (see
 * `expectNoAxeViolations`'s handling of an empty selector, below), purely for call-site compatibility: the spec
 * files that `import` it pass it to `expectNoAxeViolations({ excludeRegions: [...] })`. It goes, with every call
 * site, the next time those spec files are touched.
 *
 * ## Readiness: every scan waits for the app's first paint
 *
 * `expectNoAxeViolations` waits (`waitForAppReady`, below) before it scans.
 * `page.goto()` resolving only means the CSR shell loaded, not that
 * Dioxus's wasm client has painted anything into it yet; scanning in that
 * gap yields two false violations
 * (`landmark-one-main`/`page-has-heading-one`, both against the bare
 * `<html>` element) on whichever demo page happens to be fastest. See
 * `waitForAppReady`'s own doc comment for the full mechanism and why two
 * waits, not one. This lives in the shared helper, not each call site, per
 * this repo's "same problem more than once -> fix the class" rule.
 */

import { expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

/** The WCAG/best-practice tag set every scan in this repo runs against. */
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"];

/** One excluded rule id, with the mandatory reason it is safe to exclude here. */
export interface AxeExclusion {
  /** The axe rule id (or ids) to disable, e.g. "color-contrast". */
  ids: string | string[];
  /**
   * Why this specific scan site is a false positive for this rule. Must be non-empty -- see
   * `expectNoAxeViolations`.
   */
  reason: string;
}

/**
 * One excluded *region*: a CSS selector axe should skip scanning entirely
 * (axe's `.exclude()`, a scan-context change -- every rule is skipped for
 * that subtree, not just one), with the mandatory reason it is safe to
 * exclude here. Scoped, unlike `AxeExclusion`/`disableRules`, which turns
 * a rule off for the *whole* page: prefer a region exclusion whenever the
 * false positive is isolated to a specific, identifiable subtree, so the
 * rest of the page (a component's own markup included) stays checked by
 * every rule.
 */
export interface AxeRegionExclusion {
  /** CSS selector (or a frame-traversal chain) to exclude from the scan. */
  selector: string | string[];
  /** Why this specific region is a false positive. Must be non-empty. */
  reason: string;
}

/**
 * Kept only for call-site compatibility; slated for removal.
 *
 * The defect it would exclude (the vendored `github-light` syntax-highlighting theme's comment token, `#6e7781`,
 * below 4.5:1 against this app's code-block background) is fixed in source -- NOT in the
 * crate-generated `docs/assets/github-light*.css` (regenerated by every `scripts/deploy-preview.sh` run)
 * but as a higher-specificity override in `preview/assets/main.css`, which sets the theme's comment-token
 * custom properties to `#59636e` on `.dx-preview-code-theme <theme class>`.
 * `.dx-preview-code-theme` has no `color-contrast` violation to exclude. See this file's header doc
 * ("`EXCLUDE_VENDORED_CODE_HIGHLIGHT` scopes out nothing") and `preview/assets/main.css`'s own comment on
 * that override rule.
 *
 * The constant stays exported, with a selector that excludes nothing (see
 * `expectNoAxeViolations`'s empty-selector handling below), purely so the
 * many spec files that still `import` and pass it to
 * `expectNoAxeViolations({ excludeRegions: [...] })` keep compiling and
 * running unchanged. Once every call site is updated to drop this import,
 * delete it along with this comment.
 */
export const EXCLUDE_VENDORED_CODE_HIGHLIGHT: AxeRegionExclusion = {
  selector: "",
  reason:
    "no-op, kept only for call-site compatibility -- nothing needs excluding: " +
    "preview/assets/main.css overrides the github-light theme's comment " +
    "tokens (#6e7781 on #fbfbfb is 4.39:1) with #59636e (5.91:1), because " +
    "the theme's own generated CSS is reverted by every deploy; see " +
    "dev-docs/backlog.md row 39 and this file's header doc",
};

export interface AxeScanOptions {
  /** Scope the scan to one or more CSS selectors (axe's `.include()`). */
  include?: string | string[];
  /** Rule exclusions, each requiring a written `reason` (see `AxeExclusion`). */
  exclude?: AxeExclusion[];
  /** Region exclusions, each requiring a written `reason` (see `AxeRegionExclusion`). */
  excludeRegions?: AxeRegionExclusion[];
}

function formatViolations(
  violations: Awaited<ReturnType<AxeBuilder["analyze"]>>["violations"],
  label: string,
): string {
  const lines = [`axe: ${violations.length} violation(s) at "${label}"`];
  for (const v of violations) {
    lines.push(`  [${v.id}] impact=${v.impact ?? "unknown"} — ${v.help}`);
    lines.push(`    ${v.helpUrl}`);
    for (const node of v.nodes) {
      const html = node.html.length > 300 ? `${node.html.slice(0, 300)}…` : node.html;
      lines.push(`    - target: ${node.target.join(" ")}`);
      lines.push(`      html:   ${html}`);
    }
  }
  return lines.join("\n");
}

/**
 * How long `waitForAppReady` will wait for either readiness signal before
 * giving up. Bounded well above every *passing* scan's observed duration
 * in this suite (a few seconds) but far below Playwright's own per-test ceiling (5min in this repo's
 * config), so a page that genuinely never renders (a real defect, not a
 * race) still fails fast with a clear "waiting for locator(...) to be
 * attached" timeout instead of silently eating the whole test budget.
 */
const APP_READY_TIMEOUT_MS = 15_000;

/**
 * Wait for Dioxus's first client-side paint before axe scans `page`.
 *
 * `page.goto()` resolving (the `load` event) only proves the CSR shell
 * (`preview/index.html`, an empty `<div id="main">` -- note: a `div`
 * *carrying* the id `main`, not a `<main>` element, so it can never
 * satisfy either wait below on its own) was fetched. It says nothing
 * about whether the wasm client has run its first render pass yet, and
 * for a CSR app the `load` event routinely fires before that paint.
 * Scanning in that gap is exactly what produces the two false violations
 * this wait prevents: `landmark-one-main`/`page-has-heading-one`, both
 * targeting the bare `<html>` element, on whichever demo page's scan
 * happens to run fastest (almost nothing in the DOM yet to scan, so it
 * finishes well under a normal scan's time).
 *
 * Two waits, not one, because the race produces *two* violations and
 * every route this suite scans (`preview/src/site/home.rs`'s `Home`,
 * `preview/src/site/docs_page.rs`'s `Docs`, `preview/src/site/demos.rs`'s `Demos`,
 * `preview/src/site/component_page.rs`'s `ComponentHighlight`,
 * `preview/src/site/demos.rs`'s `ComponentDemo`, the dashboard) renders
 * both together as part of the same first paint -- each either rendering
 * its own top-level `main { .. }` + `h1` (`Home`, `Docs`, `Demos`) or
 * composing `DocsLayout`, whose `SidebarInset` always renders the page's
 * one `<main>` (`preview/src/components/sidebar/component.rs`), around
 * content that always carries an `h1`:
 *   1. the page's one `<main>` landmark, and
 *   2. its level-one heading (an `h1`, or an ARIA `role="heading"` +
 *      `aria-level="1"` equivalent -- `getByRole` matches either).
 * Both checks use `state: "attached"` only (no visibility/animation
 * wait, no `networkidle`), so this stays cheap and matches the exact two
 * rules the race violates -- a direct fix for the failure mode, not a general "wait a bit longer."
 */
async function waitForAppReady(page: Page): Promise<void> {
  await page
    .locator("main")
    .first()
    .waitFor({ state: "attached", timeout: APP_READY_TIMEOUT_MS });
  await page
    .getByRole("heading", { level: 1 })
    .first()
    .waitFor({ state: "attached", timeout: APP_READY_TIMEOUT_MS });
}

/**
 * Run axe-core's full WCAG 2.0/2.1 A+AA + best-practice rule set against
 * `page` (or a subset of it, via `opts.include`) and fail with a readable
 * table (rule id, impact, help URL, every offending node's target + html)
 * if anything is found. `label` identifies the scan site in that message
 * (spec name + state, e.g. "context-menu: submenu open").
 *
 * Waits for the app's first paint first (`waitForAppReady`, above) -- see
 * that function's doc comment and this file's header doc ("Readiness")
 * for why.
 */
export async function expectNoAxeViolations(
  page: Page,
  label: string,
  opts: AxeScanOptions = {},
): Promise<void> {
  await waitForAppReady(page);

  // `resultTypes: ["violations"]`: every rule still runs on every node and `violations` is complete, but axe stops building
  // the selector, source and check data of the passing and incomplete nodes this function never reads (14,700 nodes on the
  // home page). On a debug build (8 routes) that is -20 % scan time with identical violations on the real pages and on a page
  // with five injected defects.
  let builder = new AxeBuilder({ page }).withTags(TAGS).options({ resultTypes: ["violations"] });

  if (opts.include) {
    builder = builder.include(opts.include);
  }

  const disabledIds: string[] = [];
  for (const exclusion of opts.exclude ?? []) {
    if (!exclusion.reason || exclusion.reason.trim().length === 0) {
      throw new Error(
        `expectNoAxeViolations("${label}"): exclusion of rule(s) ` +
          `${JSON.stringify(exclusion.ids)} requires a non-empty "reason" ` +
          `(axe.ts's exclusion-with-reason rule — see this file's header doc)`,
      );
    }
    disabledIds.push(...(Array.isArray(exclusion.ids) ? exclusion.ids : [exclusion.ids]));
  }
  if (disabledIds.length > 0) {
    builder = builder.disableRules(disabledIds);
  }

  for (const region of opts.excludeRegions ?? []) {
    if (!region.reason || region.reason.trim().length === 0) {
      throw new Error(
        `expectNoAxeViolations("${label}"): region exclusion of ` +
          `${JSON.stringify(region.selector)} requires a non-empty "reason" ` +
          `(axe.ts's exclusion-with-reason rule — see this file's header doc)`,
      );
    }
    // A no-op region exclusion (empty selector, or an array of only empty/
    // blank selectors) is ignored rather than passed to axe's `.exclude()`.
    // This is what lets `EXCLUDE_VENDORED_CODE_HIGHLIGHT` stay a real,
    // importable `AxeRegionExclusion` although its underlying defect is fixed
    // by construction (see that constant's doc comment): its selector is
    // `""`, and every existing call site keeps compiling and scanning the
    // page normally instead of passing an empty string straight to axe
    // (which is not guaranteed to safely mean "exclude nothing").
    const selectors = Array.isArray(region.selector) ? region.selector : [region.selector];
    const isNoop = selectors.every((s) => s.trim().length === 0);
    if (isNoop) {
      continue;
    }
    builder = builder.exclude(region.selector);
  }

  const results = await builder.analyze();
  expect(results.violations, formatViolations(results.violations, label)).toEqual([]);
}
