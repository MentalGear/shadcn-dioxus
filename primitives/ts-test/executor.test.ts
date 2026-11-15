// The page executor (`src/ts/lib/executor.ts`, row 449) run in the SHIPPED bundle against the fake browser: the one
// writer of a registered element's open level, the nonce rule, the page-caused report and the discovery.

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
  const level = (target: string, level: boolean, nonce: number, mode = "immediate") =>
    s.run({ cmd: "executor", target, kind: "popover", mode, level, nonce, site: "executor.popover" });
  return { win, doc, popover, ch, s, level };
};
const calls = (el: any) => el.calls.map((c: any) => c[0]);

describe("the page executor", () => {
  test("applies the level and never reports the transition its own write causes", () => {
    const { popover, ch, level } = setup();
    const el = popover("panel");
    level("panel", true, 1);
    expect(el.popoverOpen).toBe(true);
    level("panel", false, 2);
    expect(calls(el)).toEqual(["showPopover", "hidePopover"]);
    expect(ch.sent).toEqual([]);
  });

  test("reports a page-caused transition stamped with the nonce it had applied", () => {
    const { doc, popover, ch, level } = setup();
    const el = popover("panel");
    level("panel", true, 4);
    el.popoverOpen = false;
    doc.dispatch("beforetoggle", { target: el, newState: "closed" });
    expect(ch.sent).toEqual([{ n: 4, open: false }]);
  });

  test("drops a command older than the newest it applied", () => {
    const { popover, level } = setup();
    const el = popover("panel");
    level("panel", true, 3);
    level("panel", false, 2);
    expect(el.popoverOpen).toBe(true);
    expect(calls(el)).toEqual(["showPopover"]);
  });

  test("a mounted element is shown on every open level and never hidden by a command", () => {
    const { popover, level } = setup();
    const el = popover("list");
    level("list", true, 1, "mounted");
    level("list", false, 2, "mounted");
    expect(el.popoverOpen).toBe(true);
    el.popoverOpen = false;
    level("list", true, 3, "mounted");
    expect(calls(el)).toEqual(["showPopover", "showPopover"]);
  });

  test("names the site and the effect while it writes", () => {
    const { win, popover, level } = setup();
    const el = popover("panel");
    const seen: unknown[] = [];
    el.showPopover = () => void seen.push(win.__dxApplying);
    level("panel", true, 1);
    expect(seen).toEqual([{ site: "executor.popover", effect: "show" }]);
  });

  test("a command for an element not rendered yet runs when a childList batch brings it", () => {
    const { win, popover, level } = setup();
    level("late", true, 1);
    const el = popover("late");
    const observer = win.MutationObserver.all.at(-1);
    observer.callback([]);
    expect(el.popoverOpen).toBe(true);
    expect(observer.disconnected).toBe(true);
  });

  test("an ended session reports nothing", () => {
    const { doc, popover, ch, s, level } = setup();
    const el = popover("panel");
    level("panel", true, 1);
    s.dispose();
    doc.dispatch("beforetoggle", { target: el, newState: "closed" });
    expect(ch.sent).toEqual([]);
  });
});
