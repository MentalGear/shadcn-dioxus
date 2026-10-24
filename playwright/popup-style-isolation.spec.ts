/**
 * ORACLE: a popup rendered INSIDE another component must look exactly like the same popup rendered alone.
 *
 * The class this closes. Our popovers and dialogs are top-layer elements (`popover` attribute / native
 * `<dialog>`), but top-layer promotion changes PAINTING, not the DOM: the panel stays a descendant of its
 * trigger's wrapper. Two things reach it from the host:
 *   1. a DESCENDANT SELECTOR (`.dx-input-group-addon button`, `.dx-alert-description a`, ...): the Date
 * Picker's "natural language" demo hangs its calendar inside `.dx-input-group-addon`, whose
 *      `button { height; padding }` and `svg { width/height: 1rem }` would shrink every calendar day cell to 28x24
 *      and the chevrons to 16px, and nothing else flags it because the popover looks right on its own page;
 * 2. plain INHERITANCE (`font-size`, `color`, `text-align`, `line-height`, `white-space`, `cursor`, ...): a
 *      Select inside a 14px card footer would draw its scroll buttons at 14px (alone: 16px), and the toast region
 *      inside a demo card would take the card's 14px.
 *
 * What the construction guarantees, and what this spec proves of it:
 *   - Inheritance is cut ONCE, at the popup root: `preview/assets/dx-components-theme.css` "Popup root" is
 *     one zero-specificity rule over `[popover]`/`dialog` with a `dx-` class that gives the root the theme's
 *     root value of EVERY inherited property (`scripts/check-popup-root-reset.sh` keeps that list complete
 *     and free of `direction`, which RTL needs, and of custom properties, which keep composing). So the host
 *     cannot matter for inheritance, and a representative popup per component in one hostile host proves
 *     the cut, instead of every popup on every page.
 *   - Descendant selectors are closed statically by `scripts/check-popup-boundary.sh` (a generic-subject
 *     descendant rule must end with `:not(:where([popover], dialog) *)`); the browser half here is the
 *     natural nesting that class shows up in (a popup hosted in an input group addon, a card, a toolbar...),
 *     compared against the same popup alone.
 *
 * The test, by construction rather than by listing the components that happened to be hit. For each popup
 * component (and the site pages that compose several), the first trigger that opens each popup KIND (the
 * root's `dx-` classes) is opened twice:
 *   - ALONE-vs-NESTED: as the page renders it, then
 *   - HOSTILE: after the popup's own parent is given a hostile value for every inherited property that can show
 *     (23px heavy italic monospace uppercase red text, 3x line height, letter and word spacing, right aligned,
 *     nowrap, indented, shadowed, crosshair, no list marker, unselectable, SVG paint...).
 * and the computed style of the panel and of every element in it is snapshotted, the panel is cloned to
 * `<body>` (the same markup with no ancestor component), shown, snapshotted again, and the two must agree on
 * the properties a stray ancestor rule or inherited value moves: font, colour, padding, margin, border widths
 * and radii, gap, display, plus the rendered size of every svg, button, input and role=gridcell cell. A
 * difference is an ancestor reaching into the popup. (The panel's own box is deliberately not compared, only
 * what it inherits: the component that owns a popup styles its root through a scope a clone in <body> lacks,
 * and an anchored panel is sized from its trigger. Everything INSIDE the panel is compared in full.) A page
 * lists the kinds it must open; a kind it never opens is a failure (a trigger that stopped working would
 * otherwise pass for "nothing to compare").
 *
 * It also asserts the premise the construction relies on: every popup that opens is a top-layer element, i.e. matches
 * `[popover], dialog` (`POPUP_BOUNDARY` in `scripts/check-popup-boundary.sh`). A role=menu/listbox/tooltip/
 * dialog panel that opens WITHOUT one has no boundary a stylesheet can stop at.
 *
 * The cure for a failure is never here: a leaked inherited property means the popup-root rule lost a property
 * (the gate says which); a descendant rule means the ancestor's sheet needs the boundary, `.dx-x button:not(
 * :where([popover], dialog) *)`, or should select its own part by class.
 *
 * Run against a debug or release SSG build alike: nothing here is a timing assertion.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { readCatalog } from "./catalog";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const COMPONENT_NAMES = new Set(readCatalog().map((r) => r.name));

/**
 * The popups, as `{ page, kinds }`: `kinds` are the popup roots (a `dx-` class of the `[popover]`/`<dialog>`)
 * that page must open at least once. A component missing from the catalog fails the "listed" test below, so a
 * page that changes name cannot silently drop out.
 */
const PAGES: Array<{ id: string; path: string; kinds: string[] }> = [
  { id: "tooltip", path: "/component/tooltip/", kinds: ["dx-tooltip-content"] },
  { id: "hover_card", path: "/component/hover_card/", kinds: ["dx-hover-card-content"] },
  { id: "popover", path: "/component/popover/", kinds: ["dx-popover-content"] },
  { id: "dropdown_menu", path: "/component/dropdown_menu/", kinds: ["dx-dropdown-menu-content"] },
  { id: "context_menu", path: "/component/context_menu/", kinds: ["dx-context-menu-content"] },
  { id: "menubar", path: "/component/menubar/", kinds: ["dx-menubar-content"] },
  { id: "navigation_menu", path: "/component/navigation_menu/", kinds: ["dx-navigation-menu-content"] },
  { id: "select", path: "/component/select/", kinds: ["dx-select-list"] },
  { id: "combobox", path: "/component/combobox/", kinds: ["dx-combobox-list"] },
  { id: "dialog", path: "/component/dialog/", kinds: ["dx-dialog"] },
  { id: "alert_dialog", path: "/component/alert_dialog/", kinds: ["dx-alert-dialog"] },
  { id: "sheet", path: "/component/sheet/", kinds: ["dx-sheet"] },
  { id: "drawer", path: "/component/drawer/", kinds: ["dx-drawer"] },
  { id: "command", path: "/component/command/", kinds: ["dx-command-dialog"] },
  { id: "date_picker", path: "/component/date_picker/", kinds: ["dx-date-picker-popover-content"] },
  { id: "color_picker", path: "/component/color_picker/", kinds: ["dx-color-picker-popover"] },
  { id: "toast", path: "/component/toast/", kinds: ["dx-toast-container"] },
  // The site pages compose several components inside 14px cards (the host that makes a Select's scroll buttons
  // and the toast region 14px without the root reset): the kinds their cards hold.
  {
    id: "site-home",
    path: "/",
    kinds: [
      "dx-toast-container", "dx-color-picker-popover", "dx-combobox-list", "dx-alert-dialog", "dx-select-list", "dx-drawer",
      "dx-dropdown-menu-content", "dx-menubar-content", "dx-navbar-content", "dx-navigation-menu-content", "dx-popover-content",
    ],
  },
  { id: "site-charts", path: "/charts/", kinds: ["dx-tooltip-content", "dx-sheet", "dx-select-list"] },
];

/**
 * A hostile host: a deliberately absurd value for every inherited property that can show in a popup, set on the
 * parent of the popup that just opened. Components declare their own typography where it matters, so
 * any popup that still shows a trace of these inherited it. (`direction`, `visibility` and `pointer-events` are
 * left out on purpose: RTL is meant to reach a popup, and the other two would hide or disable the trigger.)
 */
const HOSTILE =
  "font-family: monospace; font-size: 23px; font-weight: 800; font-style: italic; line-height: 3; letter-spacing: 3px; " +
  "word-spacing: 9px; text-transform: uppercase; text-align: right; text-indent: 17px; white-space: nowrap; " +
  "color: rgb(255, 0, 0); text-shadow: 2px 2px 0 rgb(0, 0, 255); cursor: crosshair; list-style-type: square; " +
  "user-select: none; font-variant-caps: small-caps; tab-size: 3; overflow-wrap: anywhere; hyphens: auto; word-break: break-all; " +
  "caret-color: rgb(255, 0, 255); fill: rgb(255, 0, 0); stroke: rgb(0, 0, 255); stroke-width: 5px;";

interface Diff {
  popup: string;
  element: string;
  prop: string;
  nested: string;
  alone: string;
}
interface Probe {
  /** popups that opened since the last probe (each processed once) */
  opened: number;
  /** the root kind (its `dx-` classes) of each */
  kinds: string[];
  diffs: Diff[];
  /** nested = the popup's parent chain contains a `dx-` element */
  hosts: string[];
}

/** Everything the probe does runs in the page: one evaluate, so a snapshot is atomic with the clone. */
async function probe(page: Page, skipKinds: string[], hostile: boolean): Promise<Probe> {
  return page.evaluate(({ skipKinds, hostileCss }) => {
    const PROPS = [
      "display", "font-size", "font-weight", "font-family", "line-height", "letter-spacing", "text-align", "text-transform",
      "text-decoration-line", "color", "background-color", "opacity", "cursor", "gap", "flex-direction", "align-items",
      "justify-content", "font-style", "word-spacing", "text-indent", "text-shadow", "white-space", "list-style-type", "user-select",
      "font-variant-caps", "tab-size", "overflow-wrap", "hyphens", "word-break", "caret-color", "fill", "stroke", "stroke-width", "pointer-events",
      "padding-top", "padding-right", "padding-bottom", "padding-left", "margin-top", "margin-right",
      "margin-bottom", "margin-left", "border-top-width", "border-right-width", "border-bottom-width", "border-left-width",
      "border-top-left-radius", "border-top-right-radius", "border-bottom-left-radius", "border-bottom-right-radius",
    ];
    // The panel's OWN box (padding, border, radius, background, size) is legitimately styled by the component
    // that owns the popup through a scope it brings (`.dx-date-picker-group .dx-date-picker-popover-content`),
    // which a clone in <body> does not have; only what the panel INHERITS is compared at the root.
    const INHERITED = new Set([
      "font-size", "font-weight", "font-family", "line-height", "letter-spacing", "text-align", "text-transform", "color", "cursor", "font-style",
      "word-spacing", "text-indent", "text-shadow", "white-space", "list-style-type", "user-select", "font-variant-caps", "tab-size", "overflow-wrap",
      "hyphens", "word-break", "caret-color", "fill", "stroke", "stroke-width", "pointer-events",
    ]);
    // Rendered size is compared where it is fixed by the component, not by the panel's width: an icon's
    // width and height, a control's or calendar cell's height. (An option or a menu item flows from the
    // panel, which an anchored popup sizes from its trigger, so its box is not a style difference.)
    const BOTH_AXES = new Set(["svg", "img"]);
    const HEIGHT_ONLY = new Set(["button", "input", "select", "textarea"]);
    const w = window as unknown as { __pwSeen?: WeakSet<Element> };
    const seen = (w.__pwSeen ??= new WeakSet());

    const isOpenTopLayer = (e: Element) =>
      e.matches(":popover-open") || (e.tagName === "DIALOG" && (e as HTMLDialogElement).open);
    const label = (e: Element) => {
      const cls = (e.getAttribute("class") || "").split(/\s+/).filter(Boolean).slice(0, 2).join(".");
      const role = e.getAttribute("role");
      return `${e.tagName.toLowerCase()}${cls ? "." + cls : ""}${role ? `[role=${role}]` : ""}`;
    };
    const snap = (root: Element) =>
      [root, ...root.querySelectorAll("*")].map((e) => {
        const cs = getComputedStyle(e);
        const o: Record<string, string> = {};
        for (const p of PROPS) o[p] = cs.getPropertyValue(p);
        const tag = e.tagName.toLowerCase();
        const both = BOTH_AXES.has(tag);
        if (both || HEIGHT_ONLY.has(tag) || e.getAttribute("role") === "gridcell") {
          const r = e.getBoundingClientRect(); // rest-ok: motion is frozen page-wide by the addStyleTag in this spec's setup
          if (both) o["rendered-width"] = `${Math.round(r.width * 10) / 10}px`;
          o["rendered-height"] = `${Math.round(r.height * 10) / 10}px`;
        }
        return o;
      });

    const fresh = [...document.querySelectorAll("[popover], dialog")].filter((e) => isOpenTopLayer(e) && !seen.has(e));
    const out: Probe = { opened: fresh.length, kinds: [], diffs: [], hosts: [] };
    // The kind of popup: its `dx-` classes except the shared anchor marker (`dx-popover-content dx-date-picker-popover-content`).
    const kindOf = (e: Element) =>
      (e.getAttribute("class") || "").split(/\s+/).filter((t) => t.startsWith("dx-") && !t.startsWith("dx-anchor-")).join(" ") || "(no dx- class)";

    for (const p of fresh) {
      seen.add(p);
      out.kinds.push(kindOf(p));
      // Already compared (this kind, this pass): opening it again would only repeat the snapshot.
      if (skipKinds.includes(kindOf(p))) continue;
      // The hostile host: the popup's own parent takes the hostile inherited values, so everything the root
      // fails to cut reaches it directly. (Set after the popup opened, so no click lands on a moved control.)
      if (hostileCss && p.parentElement) p.parentElement.style.cssText += hostileCss;
      // A focused control is a state, not an ancestry difference (AlertDialog focuses Cancel on open).
      const focused = document.activeElement as HTMLElement | null;
      if (focused && p.contains(focused)) focused.blur();
      const nodes = [p, ...p.querySelectorAll("*")];
      const nested = snap(p);
      out.hosts.push(
        (() => {
          const chain: string[] = [];
          for (let a = p.parentElement; a && a !== document.body; a = a.parentElement) {
            if ((a.getAttribute("class") || "").includes("dx-")) chain.push(label(a));
          }
          return chain.slice(0, 3).join(" < ") || "(none)";
        })(),
      );
      const clone = p.cloneNode(true) as HTMLElement;
      clone.removeAttribute("id");
      clone.setAttribute("data-pw-clone", "");
      // Direction is inherited ON PURPOSE (an RTL page's popups are RTL), so the reference keeps it.
      clone.setAttribute("dir", getComputedStyle(p).direction);
      document.body.appendChild(clone);
      try {
        if (clone.hasAttribute("popover")) (clone as HTMLElement & { showPopover(): void }).showPopover();
        else if (clone.tagName === "DIALOG") (clone as HTMLDialogElement).show();
      } catch {
        /* a clone that cannot be shown is compared as it is (display:none props still compare) */
      }
      // showPopover()/show() move focus into the clone; the original was blurred above, so blur the clone too.
      (document.activeElement as HTMLElement | null)?.blur?.();
      const alone = snap(clone);
      for (let i = 0; i < nested.length; i++) {
        for (const prop of Object.keys(nested[i])) {
          if (nested[i][prop] === alone[i][prop]) continue;
          if (i === 0 && !INHERITED.has(prop)) continue;
          const a = parseFloat(nested[i][prop]);
          const b = parseFloat(alone[i][prop]);
          if (prop.startsWith("rendered-") && Math.abs(a - b) < 0.6) continue;
          out.diffs.push({
            popup: label(p),
            element: i === 0 ? "(panel)" : label(nodes[i]),
            prop,
            nested: nested[i][prop],
            alone: alone[i][prop],
          });
        }
      }
      clone.remove();
    }

    return out;
  }, { skipKinds, hostileCss: hostile ? HOSTILE : "" });
}

/**
 * Tag the controls that might open a popup with `data-pw-cand="<n>"` and return the indices in the order to try them.
 * Anything a user can press inside <main> qualifies (a Dialog's trigger is a plain Button with no marker at
 * all), de-duplicated by a signature of the control AND the first three `dx-` components around it, so a
 * calendar's 35 identical day cells are one candidate while the same Button inside an Input Group addon and
 * inside a Card are two. Links are left out (they navigate), as are tabs and the sidebar toggle (they only
 * rearrange the page the other candidates live on).
 */
async function tagCandidates(page: Page): Promise<number[]> {
  return page.evaluate(() => {
    const SEL =
      'button, [role="button"], [role="combobox"], [aria-haspopup]:not([aria-haspopup="false"]), [aria-expanded], [class*="-trigger"]';
    const root = document.querySelector("main") ?? document.body;
    const seen = new Set<string>();
    let n = 0;
    // Controls that announce a popup (aria-haspopup, aria-expanded, combobox, `-trigger`) are tried before plain buttons.
    const announced: number[] = [];
    const plain: number[] = [];
    for (const e of root.querySelectorAll(SEL)) {
      if (e.closest("svg") || e.tagName === "A" || e.getAttribute("role") === "tab" || e.classList.contains("dx-sidebar-trigger")) continue;
      if (e.closest("[popover], dialog, pre, .dx-code-block, [data-pw-clone]")) continue;
      if ((e as HTMLButtonElement).disabled || e.getAttribute("aria-disabled") === "true") continue;
      const r = e.getBoundingClientRect(); // rest-ok: motion is frozen page-wide by the addStyleTag in this spec's setup
      if (r.width === 0 || r.height === 0) continue;
      const chain: string[] = [];
      for (let a = e.parentElement; a && a !== root && chain.length < 3; a = a.parentElement) {
        const c = (a.getAttribute("class") || "").split(/\s+/).find((t) => t.startsWith("dx-"));
        if (c) chain.push(c);
      }
      const sig = [
        e.tagName,
        e.getAttribute("class"),
        e.getAttribute("role"),
        e.getAttribute("aria-haspopup"),
        e.getAttribute("data-size"),
        e.getAttribute("data-style"),
        chain.join("<"),
      ].join("|");
      if (seen.has(sig)) continue;
      seen.add(sig);
      (e.matches('[role="combobox"], [aria-haspopup]:not([aria-haspopup="false"]), [aria-expanded], [class*="-trigger"]') ? announced : plain).push(n);
      e.setAttribute("data-pw-cand", String(n++));
    }
    return [...announced, ...plain];
  });
}

async function countVisibleRolePanels(page: Page): Promise<number> {
  return page.evaluate(
    () =>
      [...document.querySelectorAll('[role="menu"], [role="listbox"], [role="tooltip"], [role="dialog"], [role="alertdialog"]')].filter(
        (e) => {
          if (e.closest("[popover], dialog")) return false;
          const r = e.getBoundingClientRect(); // rest-ok: motion is frozen page-wide by the addStyleTag in this spec's setup
          return r.width > 0 && r.height > 0 && getComputedStyle(e).visibility !== "hidden";
        },
      ).length,
  );
}

async function reset(page: Page, url: string) {
  await page.keyboard.press("Escape").catch(() => undefined);
  await page.keyboard.press("Escape").catch(() => undefined);
  await page.waitForTimeout(80);
  // The toast region is a manual popover that stays open for as long as a toast is up: Escape does not close it and
  // it is not a stuck popup (reloading for it cost a hydration per trigger on the home page).
  const stuck = await page.evaluate(() =>
    [...document.querySelectorAll("[popover], dialog")].some(
      (e) => !e.classList.contains("dx-toast-base-region") && (e.matches(":popover-open") || (e.tagName === "DIALOG" && (e as HTMLDialogElement).open)),
    ),
  );
  if (stuck || page.url() !== url) await prepare(page, url);
}

/** Load the page hydrated and freeze motion (a half-finished transition would read as a style difference). */
async function prepare(page: Page, url: string): Promise<number[]> {
  await gotoHydrated(page, url, { timeout: 5 * 60 * 1000 });
  await page.addStyleTag({ content: "*, *::before, *::after { transition: none !important; animation: none !important; }" });
  return tagCandidates(page);
}

/** How a trigger opens its popup: hover/focus for tooltips and hover cards, click and right-click for menus, click otherwise. */
function waysToOpen(cls: string, haspopup: string): Array<"click" | "hover" | "focus" | "contextmenu"> {
  return /tooltip-trigger|hover-card-trigger/.test(cls) ? ["hover", "focus"] : haspopup === "menu" ? ["click", "contextmenu"] : ["click"];
}

/**
 * Open `trig` and return the probe of the popups it opens, or null when nothing opened. `wanted` (popup kinds) keeps
 * polling until all of them have opened: a persistent popup such as the toast region is already open and would
 * otherwise end the wait before the popup this trigger opens does.
 */
async function openAndProbe(
  page: Page,
  trig: ReturnType<Page["locator"]>,
  skipKinds: string[],
  hostile: boolean,
  wanted: string[] = [],
): Promise<Probe | null> {
  const cls = (await trig.getAttribute("class", { timeout: 1000 }).catch(() => null)) ?? "";
  const haspopup = (await trig.getAttribute("aria-haspopup", { timeout: 1000 }).catch(() => null)) ?? "";
  if (!(await trig.isVisible().catch(() => false)) || (await trig.isDisabled().catch(() => false))) return null;
  let acc: Probe | null = null;
  for (const h of waysToOpen(cls, haspopup)) {
    try {
      if (h === "click") await trig.click({ timeout: 2500 });
      else if (h === "hover") await trig.hover({ timeout: 2500 });
      else if (h === "focus") await trig.focus({ timeout: 2500 });
      else await trig.click({ button: "right", timeout: 2500 });
    } catch (e) {
      if (process.env.POPUP_ISOLATION_LOG) console.log(`  ${h} failed: ${String(e).split("\n").slice(0, 3).join(" | ")}`);
      continue;
    }
    // The pointer must not rest on the panel's controls (a `:hover` fill is a state, not an ancestry difference);
    // hover/focus-opened popups (tooltip, hover card) need the pointer where it is.
    if (h === "click" || h === "contextmenu") await page.mouse.move(1, 1);
    for (let t = 0; t < (h === "hover" || h === "focus" ? 10 : 5); t++) {
      await page.waitForTimeout(h === "hover" || h === "focus" ? 150 : 100);
      const p = await probe(page, skipKinds, hostile);
      if (p.opened > 0) {
        acc = acc
          ? { opened: acc.opened + p.opened, kinds: [...acc.kinds, ...p.kinds], diffs: [...acc.diffs, ...p.diffs], hosts: [...acc.hosts, ...p.hosts] }
          : p;
      }
      if (acc && wanted.every((k) => acc!.kinds.includes(k))) return acc;
    }
    if (acc) return acc;
  }
  return acc;
}

/** Let the next probe look at popups of these kinds again (a popup that stays mounted between opens is the same element). */
async function forget(page: Page, kinds: string[]) {
  await page.evaluate((kinds) => {
    const w = window as unknown as { __pwSeen?: WeakSet<Element> };
    for (const e of document.querySelectorAll("[popover], dialog")) {
      const kind = (e.getAttribute("class") || "").split(/\s+/).filter((t) => t.startsWith("dx-") && !t.startsWith("dx-anchor-")).join(" ");
      if (kinds.includes(kind)) w.__pwSeen?.delete(e);
    }
  }, kinds);
}

test.describe("popup style isolation: a popup nested in a component looks like the same popup alone", () => {
  test.describe.configure({ timeout: 240_000 });

  test("every listed page is a catalog component (or a site page)", () => {
    const missing = PAGES.filter((p) => p.path.startsWith("/component/") && !COMPONENT_NAMES.has(p.id)).map((p) => p.id);
    expect(missing, "PAGES names components the catalog does not have").toEqual([]);
  });

  // The nesting this class shows up in: a popup whose trigger sits in an Input Group addon, where
  // `.dx-input-group-addon button` / `svg` would shrink the Date Picker's calendar. Every popup the addons hold.
  for (const { id, path, min } of [
    { id: "date_picker", path: "/component/date_picker/", min: 1 },
    { id: "input_group", path: "/component/input_group/", min: 0 },
  ]) {
    test(`${id}: popups hosted in an input group addon`, async ({ page }) => {
      const url = `${BASE_URL}${path}`;
      await page.route((u) => !u.href.startsWith(BASE_URL), (r) => r.abort());
      await prepare(page, url);
      const triggers = page.locator(
        '.dx-input-group-addon :is(button, [aria-haspopup]:not([aria-haspopup="false"]), [role="combobox"]):not(dialog *, [popover] *)',
      );
      const n = await triggers.count();
      const lines: string[] = [];
      let opened = 0;
      for (let i = 0; i < n; i++) {
        const result = await openAndProbe(page, triggers.nth(i), [], false);
        if (result) {
          opened += result.opened;
          for (const d of result.diffs) lines.push(`${d.popup} > ${d.element} { ${d.prop}: ${d.nested} } but alone ${d.alone}`);
        }
        await reset(page, url);
      }
      expect(opened, `${id}: expected at least ${min} popup(s) opened from the ${n} addon controls`).toBeGreaterThanOrEqual(min);
      expect(lines, `${id}: popups inside an input group addon differ from the same popup alone`).toEqual([]);
    });
  }

  for (const { id: name, path, kinds: expected } of PAGES) {
    test(`${name}`, async ({ page }) => {
      const url = `${BASE_URL}${path}`;
      // Hermetic, as all-routes.spec.ts: nothing leaves the static server.
      await page.route((u) => !u.href.startsWith(BASE_URL), (r) => r.abort());
      let order = await prepare(page, url);
      const failures = new Map<string, number>(); // one line per distinct difference, with how many elements share it
      const record = (where: string, r: Probe) => {
        for (const d of r.diffs) {
          const line = `${where}\n    ${d.popup} > ${d.element} { ${d.prop}: ${d.nested} } but alone ${d.alone}`;
          failures.set(line, (failures.get(line) ?? 0) + 1);
        }
      };
      const trigClass = async (trig: ReturnType<Page["locator"]>) =>
        ((await trig.getAttribute("class", { timeout: 1000 }).catch(() => null)) ?? "").split(/\s+/)[0] || "(no class)";
      const covered = new Set<string>(); // popup kinds compared as the page renders them
      const hostileDone = new Set<string>(); // popup kinds compared in the hostile host
      const isCovered = (token: string) => [...hostileDone].some((k) => k.split(" ").includes(token));

      // Each trigger, in order, opens its popup as the page renders it; a kind seen for the first time is then
      // opened once more in a hostile host. Stops once every listed kind has been compared both ways (site pages
      // list the kinds their cards hold, and stop the same way).
      for (let n = 0; n < order.length && !(expected.length > 0 && expected.every(isCovered)); n++) {
        const i = order[n];
        const trig = page.locator(`[data-pw-cand="${i}"]`);
        const panelsBefore = await countVisibleRolePanels(page);
        const label = await trigClass(trig);
        const result = await openAndProbe(page, trig, [...covered], false);
        if (result) {
          if (process.env.POPUP_ISOLATION_LOG) console.log(`  ${name} #${i} ${label} -> ${result.kinds.join(", ")}, host ${result.hosts.join(" ; ")}, ${result.diffs.length} diffs`);
          record(`${name} | trigger ${label} | host ${result.hosts.join(" ; ")}`, result);
          const fresh = result.kinds.filter((k) => !covered.has(k));
          fresh.forEach((k) => covered.add(k));
          if (fresh.length > 0) {
            await reset(page, url);
            await forget(page, fresh);
            const hostile = await openAndProbe(page, page.locator(`[data-pw-cand="${i}"]`), [...hostileDone], true, fresh);
            if (hostile) {
              hostile.kinds.forEach((k) => hostileDone.add(k));
              record(`${name} | HOSTILE host, trigger ${label}`, hostile);
            } else {
              failures.set(`${name} | trigger ${label} opened ${fresh.join(", ")} as rendered but not a second time in the hostile host`, 1);
            }
          }
        } else if ((await countVisibleRolePanels(page)) > panelsBefore) {
          failures.set(`${name} | trigger ${label} opened a role=menu/listbox/tooltip/dialog panel that is not a [popover]/<dialog> top-layer element`, 1);
        }
        await reset(page, url);
      }

      for (const k of expected) {
        if (!isCovered(k)) failures.set(`${name} | no trigger on the page opened a ${k} popup (opened: ${[...hostileDone].join(", ") || "none"})`, 1);
      }
      if (process.env.POPUP_ISOLATION_LOG) console.log(`${name}: kinds ${[...hostileDone].join(", ")} from ${order.length} triggers`);
      const report = [...failures].map(([line, n]) => (n > 1 ? `${line}  (x${n})` : line));
      expect(
        report.length,
        `popups nested in a component whose computed style differs from the same popup alone (${hostileDone.size} kinds compared, ${order.length} triggers):\n${report.slice(0, 40).join("\n")}`,
      ).toBe(0);
    });
  }
});
