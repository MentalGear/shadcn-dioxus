/**
 * ORACLE: an instance of a component nested inside ANOTHER VARIANT of the same component must look exactly like
 * the same instance outside it (backlog row 218, item 1).
 *
 * The class this closes. A variant attribute (`data-variant`, `data-size`, `data-orientation`, `data-spacing`, `data-align`)
 * sits on a component root, and a rule written as a descendant selector from it,
 *
 *   .dx-card[data-size="sm"] .dx-card-title { font-size: var(--dx-text-sm); }
 *
 * also matches the title of every default Card nested anywhere inside the small one. The docs page's DEMO/CODE
 * `line` Tabs restyled every Tabs demo inside it that way. The construction (Tabs' since `lane/p5tabs`): the root
 * re-declares a slot variable for EVERY variant and the part reads it, so a nested root re-declares its own value;
 * `scripts/check-variant-scope.sh` keeps the descendant form out of the sheets statically.
 *
 * The test, by construction rather than by listing the rules that happened to leak. On the page of every component
 * whose sheet has a variant-attribute selector, for each of the component's own `dx-` classes that appears with
 * more than one variant signature (the four attributes' values, missing included): take one element per
 * signature as the OUTER instance and, for every OTHER signature, clone an element of it twice -- once into a
 * neutral host appended INSIDE the outer instance, once into the same host placed right AFTER it (same parent, so
 * every ancestor rule above the outer instance is the same). The hosts are `all: initial` (no inherited value from
 * the outer instance; custom properties are not reset by `all`, so a slot variable the outer root sets DOES reach
 * the nested clone unless the clone's root re-declares it -- that is part of what is tested), a fixed 640px wide
 * block, a containing block (`position: relative`) and an inline-size container. Every computed property of every element of the two clones must agree; a
 * difference is a rule of the outer instance reaching into the nested one.
 *
 * CANNOT_NEST lists the (component, class) pairs whose element cannot contain another one of its own kind, with
 * the reason; the same pairs are allowlisted in the gate.
 *
 * Run against a debug or release SSG build alike: nothing here is a timing assertion.
 */
import { test, expect } from "./fixtures";
import * as fs from "fs";
import * as path from "path";
import { readCatalog } from "./catalog";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const COMPONENTS_DIR = path.join(__dirname, "..", "preview", "src", "components");
const VARIANT_SELECTOR = /\[data-(?:variant|size|orientation|spacing|align)\b/;

/** The components whose own sheet styles by a variant attribute: the ones this spec must cover. */
function variantComponents(): string[] {
  return fs
    .readdirSync(COMPONENTS_DIR)
    .filter((c) => {
      const p = path.join(COMPONENTS_DIR, c, "style.css");
      return fs.existsSync(p) && VARIANT_SELECTOR.test(fs.readFileSync(p, "utf8").replace(/\/\*[\s\S]*?\*\//g, ""));
    })
    .sort();
}

/** (component, class) -> why one element of that class cannot hold another. Mirrors the gate's allowlist. */
const CANNOT_NEST: Record<string, Record<string, string>> = {
  item: {
    "dx-item-media":
      "ItemMedia holds the user's leaf icon or image, never another ItemMedia (gate allowlist: a slot default on the svg/img would override its own width/height attributes)",
  },
};

const PAGES = variantComponents();
const CATALOG = new Set(readCatalog().map((r) => r.name));

interface Diff {
  cls: string;
  outer: string;
  inner: string;
  element: string;
  prop: string;
  nested: string;
  alone: string;
}

test.describe("variant nesting: an instance inside another variant of its component looks like it does alone", () => {
  test.describe.configure({ timeout: 120_000 });

  test("every component with a variant-attribute selector has a catalog page", () => {
    expect(PAGES.length).toBeGreaterThan(10);
    expect(PAGES.filter((c) => !CATALOG.has(c))).toEqual([]);
  });

  for (const comp of PAGES) {
    test(`/component/${comp}/`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/${comp}/`);
      await page.addStyleTag({ content: "*, *::before, *::after { transition: none !important; animation: none !important; }" });
      await page.mouse.move(0, 0);
      const { pairs, diffs } = await page.evaluate(
        ({ comp, skip }) => {
          const ATTRS = ["data-variant", "data-size", "data-orientation", "data-spacing", "data-align"];
          const prefix = `dx-${comp.replace(/_/g, "-")}`;
          const HOST = "all: initial; display: block; position: relative; flex: none; align-self: start; justify-self: start; width: 640px; min-width: 640px; max-width: 640px; container-type: inline-size;";
          const sig = (e: Element) => ATTRS.map((a) => `${a}=${e.getAttribute(a) ?? "-"}`).join(" ");
          const label = (e: Element) => {
            const cls = (e.getAttribute("class") || "").split(/\s+/).filter((t) => t.startsWith("dx-")).slice(0, 2).join(".");
            return `${e.tagName.toLowerCase()}${cls ? "." + cls : ""}`;
          };
          const snap = (root: Element) =>
            [root, ...root.querySelectorAll("*")].map((e) => {
              const cs = getComputedStyle(e);
              const o: Record<string, string> = {};
              for (let i = 0; i < cs.length; i++) {
                const p = cs[i];
                // `animation-play-state` is the theme's offscreen-motion pause (`[data-dx-motion="paused"] *`), a runtime state of the outer root
                if (!p.startsWith("--") && p !== "animation-play-state") o[p] = cs.getPropertyValue(p);
              }
              return o;
            });
          const root = document.querySelector("main") ?? document.body;
          const classes = new Set<string>();
          for (const e of root.querySelectorAll(`[class*="${prefix}"]`)) {
            for (const t of (e.getAttribute("class") || "").split(/\s+/)) {
              if (t === prefix || t.startsWith(prefix + "-")) classes.add(t);
            }
          }
          const out = { pairs: [] as string[], diffs: [] as Diff[] };
          for (const cls of [...classes].sort()) {
            if (skip.includes(cls)) continue;
            const bySig = new Map<string, Element>();
            for (const e of root.querySelectorAll(`.${CSS.escape(cls)}`)) {
              if (e.closest("[data-pw-host]")) continue;
              if (!bySig.has(sig(e))) bySig.set(sig(e), e);
            }
            for (const [outerSig, outer] of bySig) {
              if (!ATTRS.some((a) => outer.hasAttribute(a))) continue;
              // A custom property in the instance's own inline style is the caller's documented setting for that subtree
              // (`--dx-carousel-per-view` on CarouselContent or any wrapper); it inherits by design, so that outer is skipped.
              if (/(^|;)\s*--/.test(outer.getAttribute("style") ?? "")) continue;
              // A form control or replaced element renders no element children, so nothing can be nested in it.
              if (outer.matches("select, input, textarea, img, video, iframe, canvas, option, optgroup, svg")) continue;
              for (const [innerSig, inner] of bySig) {
                if (innerSig === outerSig) continue;
                out.pairs.push(`${cls} [${innerSig}] in [${outerSig}]`);
                const mk = () => {
                  const host = document.createElement("div");
                  host.setAttribute("data-pw-host", "");
                  host.style.cssText = HOST;
                  const c = inner.cloneNode(true) as Element;
                  for (const x of [c, ...c.querySelectorAll("[id]")]) x.removeAttribute("id");
                  host.appendChild(c);
                  return host;
                };
                const nestedHost = mk();
                const aloneHost = mk();
                outer.appendChild(nestedHost);
                outer.after(aloneHost);
                const nodes = [nestedHost.firstElementChild!, ...nestedHost.firstElementChild!.querySelectorAll("*")];
                const nested = snap(nestedHost.firstElementChild!);
                const alone = snap(aloneHost.firstElementChild!);
                for (let i = 0; i < nested.length; i++) {
                  for (const prop of Object.keys(nested[i])) {
                    if (nested[i][prop] === alone[i]?.[prop]) continue;
                    out.diffs.push({
                      cls,
                      outer: outerSig,
                      inner: innerSig,
                      element: i === 0 ? "(root)" : label(nodes[i]),
                      prop,
                      nested: nested[i][prop],
                      alone: alone[i]?.[prop] ?? "(missing)",
                    });
                  }
                }
                nestedHost.remove();
                aloneHost.remove();
              }
            }
          }
          return out;
        },
        { comp, skip: Object.keys(CANNOT_NEST[comp] ?? {}) },
      );
      test.info().annotations.push({ type: "pairs", description: `${pairs.length}: ${pairs.slice(0, 20).join("; ")}` });
      const report = diffs
        .slice(0, 40)
        .map((d) => `${d.cls} [${d.inner}] inside [${d.outer}]: ${d.element} ${d.prop}: nested ${d.nested} / alone ${d.alone}`);
      expect(report, `${diffs.length} computed value(s) differ between a nested and a standalone clone`).toEqual([]);
    });
  }
});
