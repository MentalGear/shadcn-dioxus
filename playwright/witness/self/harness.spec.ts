// The harness's own self-tests (row 446): the probes judge what they claim to judge, the control arm is real against
// real, the marker decides what a counterfactual may touch, and the pages here have known-red and known-green
// counterfactuals. No site is loaded: every page is a fixture on the shipped bridge bundle.

import { expect, test } from "../../fixtures";
import { gotoReference } from "../../page-kinds";
import { witnessPage } from "../../../primitives/ts-test/witness";
import { binding, validateBinding } from "../binding";
import { counterfactual, observe } from "../runner";
import { siteList } from "../sites";
import { propsOfSheet, sheetProps } from "../projection";
import { admits, parseVerdict, readVerdict } from "../verdict";
import { PROBE_FOR, judge, witness } from "../witness";
import { OBSERVABLES, defaultAction, leftTrajectory } from "../probes/default-action";
import { focusAtNextInput } from "../probes/focus-at-next-input";
import type { Snapshot, Trajectory } from "../probes/observations";
import { firstDifference, outcomeOrder } from "../probes/outcome-order";
import { WitnessViolation } from "../probes/violation";
import { paintDisagreement, stray, wrongPaint } from "../probes/wrong-paint";

const trajectory = (elements: Record<string, string[]>): Trajectory => ({ frames: 3, elements });
const snapshot = (over: Partial<Snapshot> = {}): Snapshot => ({
  activeElement: "body", scrollChain: [[0, 0]], scrollHost: [[0, 0]], fieldValue: null, fieldSelection: null, selection: "", open: [],
  activation: [], location: "http://x/", insertion: "", outcome: [], typedInto: [], ...over,
});
const violates = (fn: () => void): WitnessViolation => {
  try {
    fn();
  } catch (error) {
    if (error instanceof WitnessViolation) return error;
    throw error;
  }
  throw new Error("expected a WitnessViolation");
};

test.describe("the probes judge by their one criterion", () => {
  test("wrongPaint: a time-shifted copy of the real trajectory is green, an unplaced state is red", () => {
    const real = trajectory({ "": ["hidden", "placed"] });
    expect(() => wrongPaint(real, trajectory({ "": ["hidden", "hidden", "hidden", "placed"] }))).not.toThrow();
    expect(violates(() => wrongPaint(real, trajectory({ "": ["hidden", "unplaced", "placed"] }))).probe).toBe("wrongPaint");
    // an element the real arm never has is a state it never painted
    expect(stray(real, trajectory({ "": ["hidden"], "0": ["new"] }))).toEqual(["0: new"]);
  });

  test("wrongPaint: two real runs agree when they paint the same states, whatever the frame counts", () => {
    expect(paintDisagreement(trajectory({ "": ["a", "a", "b"] }), trajectory({ "": ["a", "b", "b", "b"] }))).toBeNull();
    expect(paintDisagreement(trajectory({ "": ["a", "b"] }), trajectory({ "": ["a", "c"] }))).toContain("only the second run painted");
  });

  test("defaultAction: only the observables of the event's own row are compared", () => {
    const real = snapshot();
    expect(() => defaultAction("click", real, snapshot({ scrollChain: [[0, 90]] }))).not.toThrow();
    expect(violates(() => defaultAction("click", real, snapshot({ activation: [["0", true]] }))).probe).toBe("defaultAction");
    expect(leftTrajectory("wheel", real, snapshot({ scrollChain: [[0, 90]] }))).toEqual(["scrollChain"]);
    expect(Object.keys(OBSERVABLES).sort()).toEqual(["beforeinput", "click", "dragover", "drop", "keydown", "mousedown", "touchmove", "wheel"]);
  });

  test("outcomeOrder and focusAtNextInput: a different sequence is red, the same one is green", () => {
    expect(() => outcomeOrder(["fact@0", "later@1"], ["fact@0", "later@1"])).not.toThrow();
    expect(violates(() => outcomeOrder(["fact@0", "later@1"], ["later@0", "fact@1"])).probe).toBe("outcomeOrder");
    expect(firstDifference(["a"], ["a", "b"])).toBe("length 1 against 2");
    expect(() => focusAtNextInput(["2"], ["2"])).not.toThrow();
    expect(violates(() => focusAtNextInput(["2"], [])).probe).toBe("focusAtNextInput");
  });

  test("the library chooses the probe: every effect has one, and a delayed write is judged by wrongPaint", () => {
    expect(PROBE_FOR.attrs).toEqual(["wrongPaint"]);
    expect(PROBE_FOR.focus).toEqual(["focusAtNextInput"]);
    expect(() => witness("wrongPaint", "fixture.cancel", "prevent")).toThrow("the probe for \"prevent\"");
    expect(() => witness("wrongPaint", "fixture.paint", "focus")).toThrow("does not list the effect");
  });
});

test.describe("the bindings, the site list, the sheet and the verdict", () => {
  test("a binding is validated and named for its site", () => {
    expect(binding("fixture.paint").page).toEqual({ fixture: "paint" });
    const ok = JSON.parse(JSON.stringify(binding("fixture.paint")));
    expect(() => validateBinding("x/other.json", ok)).toThrow('"site" must be "other"');
    expect(() => validateBinding("x/fixture.paint.json", { ...ok, effects: ["nonsense"] })).toThrow('"effects" must be');
    expect(() => validateBinding("x/fixture.paint.json", { ...ok, event: { kind: "click" } })).toThrow("event.selector is required");
    expect(() => validateBinding("x/fixture.paint.json", { ...ok, frames: 0 })).toThrow('"frames" must be a positive integer');
    expect(() => validateBinding("x/fixture.paint.json", { ...ok, scenarios: ["toggle"] })).toThrow("toggle must be an object");
  });

  test("the pre-authored bindings are valid", () => {
    for (const site of ["input_otp.filter", "executor.exit_wait", "executor.watchdog"]) expect(binding(site).site).toBe(site);
  });

  test("the site list is empty until a generator writes it", () => {
    expect(siteList("/nonexistent/sites.gen.json")).toEqual([]);
  });

  test("a sheet's declared properties join the projection", () => {
    expect(sheetProps("/* x: y */ .a:hover { color: red; --dx-gap: 2px; } @media (a) { .b { opacity: 0; transform: none } }")).toEqual(["color", "--dx-gap", "opacity", "transform"]);
    expect(propsOfSheet("preview/src/components/input_otp/style.css").length).toBeGreaterThan(5);
    expect(propsOfSheet(undefined)).toEqual([]);
  });

  test("verdict.json: only a measured fail for lag or order admits an effect", () => {
    const text = (verdict: string, reason: string) => JSON.stringify({ verdict, reason, devices: ["pixel"], traces: ["a.json.gz"] });
    expect(admits(parseVerdict("v", text("fail", "lag")))).toBe(true);
    expect(admits(parseVerdict("v", text("fail", "order")))).toBe(true);
    expect(admits(parseVerdict("v", text("fail", "parity")))).toBe(false);
    expect(admits(parseVerdict("v", text("pass", "lag")))).toBe(false);
    expect(admits(readVerdict("/nonexistent/verdict.json"))).toBe(false);
    expect(() => parseVerdict("v", text("maybe", "lag"))).toThrow('"verdict" must be');
    expect(() => parseVerdict("v", JSON.stringify({ verdict: "fail", reason: "lag", devices: [], traces: [] }))).toThrow('"devices"');
  });
});

test.describe("known-red and known-green counterfactuals on the fixture pages", () => {
  for (const t of [
    ...witness("wrongPaint", "fixture.paint", "attrs"),
    ...witness("defaultAction", "fixture.cancel", "prevent", { mode: "drop" }),
    ...witness("outcomeOrder", "fixture.order", "attrs", { mode: "order" }),
    ...witness("focusAtNextInput", "fixture.focus", "focus"),
  ]) test(t.title, t.run);

  test("a lagging effect paints a time-shifted copy and is green: the effect is next frame by derivation", async ({ browser }) => {
    const b = binding("fixture.lag");
    const real = await observe(browser, b, null);
    const cf = await observe(browser, b, counterfactual(b, "attrs"));
    expect(cf.hits).toBeGreaterThan(0);
    expect(() => judge("wrongPaint", b, real, cf)).not.toThrow();
  });

  test("the mount scenario arms before the page's scripts and records from the first frame the host exists", async ({ browser }) => {
    const b = binding("fixture.paint");
    const real = await observe(browser, b, null, "mount");
    const armed = await observe(browser, b, counterfactual(b, "attrs"), "mount");
    expect(real.trajectory.frames).toBeGreaterThan(b.frames - 1);
    expect(Object.keys(real.trajectory.elements).sort()).toEqual(["", "0", "1"]);
    expect(armed.hits).toBe(0); // nothing applies an attribute on mount here, so nothing is intercepted
    expect(paintDisagreement(real.trajectory, armed.trajectory)).toBeNull();
  });

  test("a cancel moved to the bubble phase at an ancestor of the target is green by construction; off the path it is red", async ({ browser }) => {
    const b = binding("fixture.cancel");
    const real = await observe(browser, b, null);
    const ancestor = await observe(browser, b, counterfactual(b, "prevent", { mode: "rehome" }));
    expect(ancestor.hits).toBeGreaterThan(0);
    expect(() => judge("defaultAction", b, real, ancestor)).not.toThrow();
    const offPath = await observe(browser, b, { ...counterfactual(b, "prevent", { mode: "rehome" }), host: "#other" });
    expect(violates(() => judge("defaultAction", b, real, offPath)).probe).toBe("defaultAction");
  });

  test("the marker names the site and the effect: another site's counterfactual, and unmarked calls, run untouched", async ({ browser }) => {
    const b = binding("fixture.paint");
    const real = await observe(browser, b, null);
    const elsewhere = await observe(browser, b, { ...counterfactual(b, "attrs"), site: "some.other.site" });
    expect(elsewhere.hits).toBe(0);
    expect(paintDisagreement(real.trajectory, elsewhere.trajectory)).toBeNull();
    const otherEffect = await observe(browser, b, counterfactual(b, "stop"));
    expect(otherEffect.hits).toBe(0);
  });

  test("the real arm sees real decisions: the fixture has not stood down, so a red is not silence", async ({ browser }) => {
    const paint = await observe(browser, binding("fixture.paint"), null);
    expect(paint.trajectory.elements[""].some((state) => state.includes("data-dx-fixture=on"))).toBe(true);
    const cancel = await observe(browser, binding("fixture.cancel"), null);
    expect(cancel.snapshot.activation).toEqual([["0", false]]);
    expect(cancel.trajectory.elements["0"].some((state) => state.includes("data-dx-fixture=on"))).toBe(true);
  });

  test("the control arm is real against real: two independent runs agree and nothing is intercepted", async ({ browser }) => {
    for (const site of ["fixture.paint", "fixture.cancel", "fixture.order", "fixture.focus"]) {
      const b = binding(site);
      const [one, two] = [await observe(browser, b, null), await observe(browser, b, null)];
      expect(paintDisagreement(one.trajectory, two.trajectory), site).toBeNull();
      expect(firstDifference(one.outcome, two.outcome), site).toBeNull();
      expect(one.hits + two.hits, site).toBe(0);
    }
  });

  test("the adapter's marker is read-only and unset between writes", async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();
    await gotoReference(page, "about:blank");
    await page.addScriptTag({ path: "../primitives/src/js/bridge.js" });
    const answer = await page.evaluate(() => {
      const w = window as any;
      const before = w.__dxApplying;
      w.__dxApplying = { site: "x", effect: "prevent" };
      let redefined = "no error";
      try {
        Object.defineProperty(window, "__dxApplying", { value: 1 });
      } catch (error) {
        redefined = String(error).slice(0, 20);
      }
      return { before, after: w.__dxApplying, redefined };
    });
    await context.close();
    expect(answer).toEqual({ before: null, after: null, redefined: "TypeError: Cannot re" });
  });

  test("the debug pause freezes the host subtree's own animations and marks every unrelated sibling", async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();
    await page.addInitScript(witnessPage);
    await page.route("http://witness.fixture/pause.html", (route) =>
      route.fulfill({
        contentType: "text/html",
        body: `<style>i{animation:a 1s infinite;width:1px;display:block}@keyframes a{to{width:9px}}</style>
<div id="host"><i id="inner"></i></div><div id="other"><i id="far"></i></div>`,
      }),
    );
    await gotoReference(page, "http://witness.fixture/pause.html");
    await page.evaluate(() => (window as any).__dxWitness.pause("#host", true));
    const state = await page.evaluate(() => ({
      inner: document.getElementById("inner")!.getAnimations().map((a) => a.playState),
      far: document.getElementById("far")!.getAnimations().map((a) => a.playState),
      marked: document.getElementById("other")!.getAttribute("data-dx-motion"),
      host: document.getElementById("host")!.hasAttribute("data-dx-motion"),
    }));
    await context.close();
    expect(state).toEqual({ inner: ["paused"], far: ["running"], marked: "paused", host: false });
  });
});
