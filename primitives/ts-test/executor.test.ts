// The page executor (`src/ts/lib/executor.ts`, row 449) run in the SHIPPED bundle against the fake browser: the one
// writer of an element's rendered open level, the nonce rule, and the page-caused report.

import { describe, expect, test } from "bun:test";
import { loadBridge, makeChannel } from "./fake-dom";

/** A popover whose show and hide dispatch `beforetoggle` synchronously, as the browser does. */
const setup = () => {
  const { win, bridge } = loadBridge();
  const doc = win.document;
  const popover = (id: string) => {
    const el = doc.add(id);
    const show = el.showPopover.bind(el);
    const hide = el.hidePopover.bind(el);
    el.showPopover = () => (doc.dispatch("beforetoggle", { target: el, newState: "open" }), show());
    el.hidePopover = () => (doc.dispatch("beforetoggle", { target: el, newState: "closed" }), hide());
    return el;
  };
  const ch = makeChannel();
  const s = bridge.scope(ch);
  s.run({ cmd: "executor", target: "panel" });
  /** Renders the level, then runs the eval Rust issues after the render. */
  const level = (el: any, nonce: number, open: boolean, mode = "immediate") => {
    el.setAttribute("data-dx-level", `${nonce} ${open ? "open" : "closed"} ${mode} executor.popover`);
    bridge.execute();
  };
  return { win, doc, bridge, popover, ch, s, level };
};
const calls = (el: any) => el.calls.map((c: any) => c[0]);

describe("the page executor", () => {
  test("applies the level and never reports the transition its own write causes", () => {
    const { doc, popover, ch, level } = setup();
    const el = popover("panel");
    level(el, 1, true);
    expect(el.popoverOpen).toBe(true);
    doc.dispatch("toggle", { target: el, newState: "open" });
    level(el, 2, false);
    doc.dispatch("toggle", { target: el, newState: "closed" });
    expect(calls(el)).toEqual(["showPopover", "hidePopover"]);
    expect(ch.sent).toEqual([]);
  });

  test("reports a page-caused transition stamped with the nonce it had applied", () => {
    const { doc, popover, ch, level } = setup();
    const el = popover("panel");
    level(el, 4, true);
    el.popoverOpen = false;
    doc.dispatch("beforetoggle", { target: el, newState: "closed" });
    expect(ch.sent, "the report waits for the toggle task").toEqual([]);
    doc.dispatch("toggle", { target: el, newState: "closed" });
    expect(ch.sent).toEqual([{ n: 4, open: false }]);
    doc.dispatch("toggle", { target: el, newState: "closed" });
    expect(ch.sent, "one report per page-caused transition").toEqual([{ n: 4, open: false }]);
  });

  test("applies a level once: a page-caused close stands until the owner renders a newer one", () => {
    const { bridge, popover, level } = setup();
    const el = popover("panel");
    level(el, 3, true);
    el.popoverOpen = false;
    bridge.execute();
    expect(el.popoverOpen).toBe(false);
    level(el, 2, true);
    expect(el.popoverOpen).toBe(false);
    level(el, 4, true);
    expect(calls(el)).toEqual(["showPopover", "showPopover"]);
  });

  test("a mounted element is shown on every open level and never hidden by one", () => {
    const { popover, level } = setup();
    const el = popover("list");
    level(el, 1, true, "mounted");
    level(el, 2, false, "mounted");
    expect(el.popoverOpen).toBe(true);
    el.popoverOpen = false;
    level(el, 3, true, "mounted");
    expect(calls(el)).toEqual(["showPopover", "showPopover"]);
  });

  test("names the site and the effect while it writes", () => {
    const { win, popover, level } = setup();
    const el = popover("panel");
    const seen: unknown[] = [];
    el.showPopover = () => void seen.push(win.__dxApplying);
    level(el, 1, true);
    expect(seen).toEqual([{ site: "executor.popover", effect: "show" }]);
  });

  test("an element without a watching session is applied and reports nothing", () => {
    const { doc, popover, level } = setup();
    const el = popover("other");
    const ch = makeChannel();
    level(el, 1, true);
    el.popoverOpen = false;
    doc.dispatch("beforetoggle", { target: el, newState: "closed" });
    doc.dispatch("toggle", { target: el, newState: "closed" });
    expect(el.calls.length).toBe(1);
    expect(ch.sent).toEqual([]);
  });

  test("an ended session reports nothing, and its listener goes with the last session", () => {
    const { doc, popover, ch, s, level } = setup();
    const el = popover("panel");
    level(el, 1, true);
    s.dispose();
    doc.dispatch("beforetoggle", { target: el, newState: "closed" });
    doc.dispatch("toggle", { target: el, newState: "closed" });
    expect(ch.sent).toEqual([]);
    expect(doc.count("beforetoggle") + doc.count("toggle")).toBe(0);
  });
});
