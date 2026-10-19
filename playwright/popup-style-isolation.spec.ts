/**
 * ORACLE: a popup rendered INSIDE another component must look exactly like the same popup rendered alone.
 *
 * The class this closes. Our popovers and dialogs are top-layer elements (`popover` attribute / native
 * `<dialog>`), but top-layer promotion changes PAINTING, not the DOM: the panel stays a descendant of its
 * trigger's wrapper, so an ancestor component's DESCENDANT selector (`.dx-input-group-addon button`,
 * `.dx-input-group-addon svg`, `.dx-alert-description a`, ...) matches elements inside it, and so does plain
 * inheritance (`color`, `font-size`, `line-height`, `text-align`, a custom property). The Date Picker's
 * "natural language" demo hangs its calendar popover inside `.dx-input-group-addon`; that sheet's
 * `button { height: ...-xs; padding: 0 ... }` and `svg { width/height: 1rem }` shrank every calendar day
 * cell to 28x24 and the month-nav chevrons to 16px, and nothing flagged it because the popover looks right
 * on its own page.
 *
 * The test, by construction rather than by listing the components that happened to be hit:
 *   for EVERY trigger on every component page (click / hover / focus / right-click, whichever opens it),
 *   take the popup that opens, snapshot the computed style of the panel and of every element in it, then
 *   clone the panel to `<body>` (the same markup with no ancestor component), show it, snapshot again, and
 *   require the two to agree on the properties a stray ancestor rule or inherited value moves: font, colour,
 *   padding, margin, border widths and radii, gap, display, plus the rendered size of every svg, button,
 *   input and role=gridcell/option/menuitem... cell. A difference is an ancestor reaching into the popup.
 *   (The panel's own box is deliberately not compared, only what it inherits: the component that owns a popup
 *   styles its root through a scope a clone in <body> lacks, and an anchored panel is sized from its trigger.
 *   Everything INSIDE the panel is compared in full.)
 *
 * It also asserts the premise the fix relies on: every popup that opens is a top-layer element, i.e. matches
 * `[popover], dialog` (`POPUP_BOUNDARY` in `scripts/check-popup-boundary.sh`). A role=menu/listbox/tooltip/
 * dialog panel that opens WITHOUT one has no boundary a stylesheet can stop at.
 *
 * The cure for a failure is in the ancestor's stylesheet, never here: end the descendant selector at the popup
 * boundary, `.dx-x button:not(:where([popover], dialog) *)`, or select the component's own part by class or
 * child combinator. `ALLOWED` holds only the latent root font-size inheritance described there.
 *
 * Run against a debug or release SSG build alike: nothing here is a timing assertion.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { readBlockDemos, readCatalog } from "./catalog";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const COMPONENTS = readCatalog().map((r) => r.name);
/** Block demos (`variants/demo.css` components) are pages of their own, `/component/block/<name>/<variant>/`. */
const BLOCKS = readBlockDemos().map((b) => ({ id: `block-${b.name}-${b.variant}`, path: b.path }));
/** Every page a popup can be nested on: each component page, the block demos, and the site pages that compose several components. */
const ROUTES = [
  ...COMPONENTS.map((name) => ({ id: name, path: `/component/${name}/` })),
  ...BLOCKS,
  { id: "site-home", path: "/" },
  { id: "site-demos", path: "/demos" },
  { id: "site-charts", path: "/charts/" },
];

/** `<popup> | <element> | <property>` (on any page) -> reason. An entry needs a reason a reviewer can check; fix the sheet instead. */
const ROOT_FONT_SIZE =
  "latent: the panel ROOT inherits its host's font-size (14px in a breadcrumb/card header/item, 16px alone) because no sheet " +
  "gives the popup root its own; every element inside declares its own size, so nothing renders differently. Cure is one " +
  "theme rule resetting inherited typography at the popup boundary (preview/assets/dx-components-theme.css, owned by the theme lane); " +
  "delete these entries with it.";
const ALLOWED: Record<string, string> = {
  "div.dx-dropdown-menu-content.dx-anchor-dropdown-menu[role=menu] | (panel) | font-size": ROOT_FONT_SIZE,
  "div.dx-select-list.dx-anchor-select[role=listbox] | (panel) | font-size": ROOT_FONT_SIZE,
  // The same inheritance one level down: SelectGroup's wrapper declares no size either, and its label and items do.
  "div.dx-select-list.dx-anchor-select[role=listbox] | div[role=group] | font-size": ROOT_FONT_SIZE,
};

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
  diffs: Diff[];
  /** nested = the popup's parent chain contains a `dx-` element */
  hosts: string[];
}

/** Everything the probe does runs in the page: one evaluate, so a snapshot is atomic with the clone. */
async function probe(page: Page): Promise<Probe> {
  return page.evaluate(() => {
    const PROPS = [
      "display", "font-size", "font-weight", "font-family", "line-height", "letter-spacing", "text-align", "text-transform",
      "text-decoration-line", "color", "background-color", "opacity", "cursor", "gap", "flex-direction", "align-items",
      "justify-content", "padding-top", "padding-right", "padding-bottom", "padding-left", "margin-top", "margin-right",
      "margin-bottom", "margin-left", "border-top-width", "border-right-width", "border-bottom-width", "border-left-width",
      "border-top-left-radius", "border-top-right-radius", "border-bottom-left-radius", "border-bottom-right-radius",
    ];
    // The panel's OWN box (padding, border, radius, background, size) is legitimately styled by the component
    // that owns the popup through a scope it brings (`.dx-date-picker-group .dx-date-picker-popover-content`),
    // which a clone in <body> does not have; only what the panel INHERITS is compared at the root.
    const INHERITED = new Set(["font-size", "font-weight", "font-family", "line-height", "letter-spacing", "text-align", "text-transform", "color", "cursor"]);
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
    const out: Probe = { opened: fresh.length, diffs: [], hosts: [] };

    for (const p of fresh) {
      seen.add(p);
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
  });
}

/**
 * Tag the controls that might open a popup with `data-pw-cand="<n>"` and return how many there are.
 * Anything a user can press inside <main> qualifies (a Dialog's trigger is a plain Button with no marker at
 * all), de-duplicated by a signature of the control AND the first three `dx-` components around it, so a
 * calendar's 35 identical day cells are one candidate while the same Button inside an Input Group addon and
 * inside a Card are two. Links are left out (they navigate), as are tabs and the sidebar toggle (they only
 * rearrange the page the other candidates live on).
 */
async function tagCandidates(page: Page): Promise<number> {
  return page.evaluate(() => {
    const SEL =
      'button, [role="button"], [role="combobox"], [aria-haspopup]:not([aria-haspopup="false"]), [aria-expanded], [class*="-trigger"]';
    const root = document.querySelector("main") ?? document.body;
    const seen = new Set<string>();
    let n = 0;
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
      e.setAttribute("data-pw-cand", String(n++));
    }
    return n;
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
  const stuck = await page.evaluate(
    () => [...document.querySelectorAll("[popover], dialog")].some((e) => e.matches(":popover-open") || (e.tagName === "DIALOG" && (e as HTMLDialogElement).open)),
  );
  if (stuck || page.url() !== url) await prepare(page, url);
}

/** Load the page hydrated and freeze motion (a half-finished transition would read as a style difference). */
async function prepare(page: Page, url: string): Promise<number> {
  await gotoHydrated(page, url, { timeout: 5 * 60 * 1000 });
  await page.addStyleTag({ content: "*, *::before, *::after { transition: none !important; animation: none !important; }" });
  return tagCandidates(page);
}

test.describe("popup style isolation: a popup nested in a component looks like the same popup alone", () => {
  test.describe.configure({ timeout: 240_000 });
  for (const { id: name, path } of ROUTES) {
    test(`${name}`, async ({ page }) => {
      const url = `${BASE_URL}${path}`;
      // Hermetic, as all-routes.spec.ts: nothing leaves the static server.
      await page.route((u) => !u.href.startsWith(BASE_URL), (r) => r.abort());
      const total = await prepare(page, url);
      const failures = new Map<string, number>(); // one line per distinct difference, with how many elements share it
      let opened = 0;
      for (let i = 0; i < total; i++) {
        const trig = page.locator(`[data-pw-cand="${i}"]`);
        const cls = (await trig.getAttribute("class", { timeout: 1000 }).catch(() => null)) ?? "";
        const haspopup = (await trig.getAttribute("aria-haspopup", { timeout: 1000 }).catch(() => null)) ?? "";
        if (!(await trig.isVisible().catch(() => false)) || (await trig.isDisabled().catch(() => false))) continue;
        const panelsBefore = await countVisibleRolePanels(page);
        const how: Array<"click" | "hover" | "focus" | "contextmenu"> = /tooltip-trigger|hover-card-trigger/.test(cls)
          ? ["hover", "focus"]
          : haspopup === "menu"
            ? ["click", "contextmenu"]
            : ["click"];
        let result: Probe | null = null;
        for (const h of how) {
          try {
            if (h === "click") await trig.click({ timeout: 2500 });
            else if (h === "hover") await trig.hover({ timeout: 2500 });
            else if (h === "focus") await trig.focus({ timeout: 2500 });
            else await trig.click({ button: "right", timeout: 2500 });
          } catch {
            continue;
          }
          // The pointer must not rest on the panel's controls (a `:hover` fill is a state, not an ancestry difference);
          // hover/focus-opened popups (tooltip, hover card) need the pointer where it is.
          if (h === "click" || h === "contextmenu") await page.mouse.move(1, 1);
          for (let t = 0; t < (h === "hover" || h === "focus" ? 10 : 5) && !result; t++) {
            await page.waitForTimeout(h === "hover" || h === "focus" ? 150 : 100);
            const p = await probe(page);
            if (p.opened > 0) result = p;
          }
          if (result) break;
        }
        if (result) {
          opened += result.opened;
          if (process.env.POPUP_ISOLATION_LOG) console.log(`  ${name} #${i} ${cls.split(/\s+/)[0]} -> ${result.opened} popup(s), host ${result.hosts.join(" ; ")}, ${result.diffs.length} diffs`);
          const where = `${name} | trigger ${cls.split(/\s+/)[0] || "(no class)"} | host ${result.hosts.join(" ; ")}`;
          for (const d of result.diffs) {
            const key = `${d.popup} | ${d.element} | ${d.prop}`;
            if (key in ALLOWED) continue;
            const line = `${where}\n    ${d.popup} > ${d.element} { ${d.prop}: ${d.nested} } but alone ${d.alone}`;
            failures.set(line, (failures.get(line) ?? 0) + 1);
          }
        } else if ((await countVisibleRolePanels(page)) > panelsBefore) {
          failures.set(`${name} | trigger ${cls.split(/\s+/)[0]} opened a role=menu/listbox/tooltip/dialog panel that is not a [popover]/<dialog> top-layer element`, 1);
        }
        await reset(page, url);
      }
      if (process.env.POPUP_ISOLATION_LOG) console.log(`${name}: ${opened} popups from ${total} triggers`);
      const report = [...failures].map(([line, n]) => (n > 1 ? `${line}  (x${n})` : line));
      expect(
        report.length,
        `popups nested in a component whose computed style differs from the same popup alone (${opened} popups opened from ${total} triggers):\n${report.slice(0, 40).join("\n")}`,
      ).toBe(0);
    });
  }
});
