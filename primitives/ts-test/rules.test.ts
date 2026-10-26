// Every rule kind has a case here (`scripts/check-bridge-rules.sh` greps this file for each kind): the
// kinds of the interpreter's switch and of the Rust `Rule` enum are `fixture` today; a later lane adds its
// own `describe` for each family it fills in.

import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { loadBridge, makeChannel } from "./fake-dom";

const vectors = JSON.parse(readFileSync(join(import.meta.dir, "../conformance/rules.json"), "utf8")).vectors as {
  name: string;
  rule: { kind: string };
  steps: { dispatch: string; event?: Record<string, unknown> }[];
  expect: { attrs: Record<string, string>; messages: unknown[]; prevented?: boolean[] };
}[];

describe("conformance vectors (primitives/conformance/rules.json, shared with cargo test)", () => {
  for (const v of vectors) {
    test(v.name, () => {
      const { bridge, win } = loadBridge();
      const el = win.document.add("host");
      const ch = makeChannel();
      const s = bridge.scope(ch);
      bridge.rules.install("host", v.rule, s);
      const prevented: boolean[] = [];
      for (const step of v.steps) {
        const event = { defaultPrevented: false, preventDefault() { event.defaultPrevented = true; }, ...step.event };
        el.dispatch(step.dispatch, event);
        prevented.push(event.defaultPrevented);
      }
      for (const [name, value] of Object.entries(v.expect.attrs)) expect(el.getAttribute(name)).toBe(value);
      expect(el.attrs.size).toBe(Object.keys(v.expect.attrs).length);
      expect(ch.sent).toEqual(v.expect.messages);
      if (v.expect.prevented) expect(prevented).toEqual(v.expect.prevented);
    });
  }
});

describe('rule kind "fixture"', () => {
  test("on the event it sets the attribute and tells Rust how often it fired", () => {
    const { bridge, win } = loadBridge();
    const el = win.document.add("f");
    const ch = makeChannel();
    const s = bridge.scope(ch);
    bridge.rules.install("f", { kind: "fixture", event: "pointerdown", attr: "data-x", value: "1" }, s);
    el.dispatch("pointerdown");
    expect(el.getAttribute("data-x")).toBe("1");
    expect(ch.sent).toEqual([{ rule: "fixture", fired: 1 }]);
  });
});
