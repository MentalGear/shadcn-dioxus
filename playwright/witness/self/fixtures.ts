// The harness's own pages: the shipped bridge bundle and the fixture behaviour on a hand-written page, so the
// harness is proved without a site build. Each page is one small mechanism whose counterfactual is known red (or known
// green), which is what proves a probe can go red at all. A real binding never names a fixture page.

import { readFileSync } from "node:fs";
import { join } from "node:path";

const JS = join(__dirname, "..", "..", "..", "primitives", "src", "js");
const BRIDGE = readFileSync(join(JS, "bridge.js"), "utf8");
const FIXTURE = readFileSync(join(JS, "behaviours", "fixture.js"), "utf8");

/** Installs the fixture behaviour on `#<target>` the way Rust does, with a channel stub that notes every fact. */
const install = (site: string, target: string, event: string, cancel: boolean, capture = cancel): string => `
const stub = { send() { window.__dxWitness.note("fact"); }, recv: () => new Promise(() => {}) };
window.__dxBridge.scope(stub).run({
  cmd: "behaviour", target: ${JSON.stringify(target)},
  behaviour: { kind: "fixture", site: ${JSON.stringify(site)}, event: ${JSON.stringify(event)}, attr: "data-dx-fixture", value: "on", capture: ${capture}, cancel: ${cancel}, leaseMs: null },
  declared: { target: "element", event: ${JSON.stringify(event)}, capture: ${capture}, passive: ${!cancel}, leaseMs: null },
  epoch: { key: "behaviour:fixture", n: 1 },
});`;

interface Page {
  markup: string;
  script: string;
}

export const PAGES: Record<string, Page> = {
  // A write the page needs before the paint after its trigger: the panel opens by a page script and the fixture's
  // attribute places it. Delay the attribute and the panel paints open and unplaced.
  paint: {
    markup: `<style>
#host{position:relative;width:200px;height:120px} #panel{display:none;position:absolute;left:0;top:50px;width:30px;height:30px;background:#09f}
#panel.open{display:block} #host[data-dx-fixture] #panel{left:90px}</style>
<div id="host"><button id="go">go</button><div id="panel"></div></div>`,
    script: `document.getElementById("go").addEventListener("click", () => document.getElementById("panel").classList.add("open"));${install("fixture.paint", "host", "click", false)}`,
  },
  // A write that is the whole state change: the panel is hidden until the attribute places it. Delayed, the panel
  // paints the state it painted before the click, then the real one: a time-shifted copy of the real trajectory.
  lag: {
    markup: `<style>#host{width:200px;height:120px} #panel{display:none;width:30px;height:30px;background:#09f} #host[data-dx-fixture] #panel{display:block}</style>
<div id="host"><button id="go">go</button><div id="panel"></div></div>`,
    script: install("fixture.lag", "host", "click", false),
  },
  // A default action only the cancel stops: the checkbox does not toggle while the fixture cancels the click.
  cancel: { markup: `<div id="host"><input id="cb" type="checkbox"><button id="other">other</button></div>`, script: install("fixture.cancel", "cb", "click", true) },
  // A fact that must reach Rust before a later listener on the path runs.
  order: { markup: `<div id="host"><button id="go">go</button></div>`, script: install("fixture.order", "host", "click", false) },
  // A focus write the next keystroke needs: the click handler moves focus to the second field.
  focus: {
    markup: `<div id="host"><button id="go">go</button><input id="a"><input id="b"></div>`,
    script: `document.getElementById("go").addEventListener("click", () => window.__dxWitness.mark("fixture.focus", "focus", () => document.getElementById("b").focus()));`,
  },
};

export function fixtureHtml(name: string): string {
  const page = PAGES[name];
  if (page === undefined) throw new Error(`witness fixture: no page "${name}" (${Object.keys(PAGES).join(", ")})`);
  const scripts = [BRIDGE, FIXTURE, page.script].map((js) => `<script>${js}</script>`).join("");
  return `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width"><body style="margin:0">${page.markup}${scripts}`;
}
