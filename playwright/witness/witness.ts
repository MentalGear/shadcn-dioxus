// `witness(kind, site, effect)`: the one way a counterfactual witness is written. It returns a PAIR of tests, which the
// spec registers with its own `test(...)` call (Playwright attributes a test to the file that calls `test`, and the
// durations table, the budget gate and the selector all key on spec files, so a registration inside this file would
// file every witness under `witness/witness.ts`):
//
//   for (const t of witness("defaultAction", "input_otp.filter", "prevent")) test(t.title, t.run);
//
//   control:  two independent REAL runs (witness mode off) must agree under the probe's own criterion. That is also
//             the determinism check: a control that only required the real arm not to throw would be vacuous for a
//             criterion that compares two arms.
//   witness:  wraps `test.fail()`. With the effect moved to its next-best placement the probe must throw its typed
//             `WitnessViolation`; the test passes only then. A timeout, a locator error, a counterfactual that
//             intercepted nothing or a probe that stays green all fail the test, so no trivially red route exists.
//
// The probe is chosen by the library from the effect (`PROBE_FOR`), never by the behaviour's author, and no probe takes
// an author argument. A witness for a site the generated site list does not name is `test.fixme` until the site is built.

import type { Browser, TestInfo } from "@playwright/test";
import { test } from "../fixtures";
import type { WitnessEffect } from "../../primitives/ts-test/witness";
import { binding, type Binding } from "./binding";
import { counterfactual, observe, type Scenario } from "./runner";
import { siteList } from "./sites";
import { defaultAction, leftTrajectory } from "./probes/default-action";
import { focusAtNextInput } from "./probes/focus-at-next-input";
import type { Observation } from "./probes/observations";
import { firstDifference, outcomeOrder } from "./probes/outcome-order";
import { WitnessViolation, type ProbeName } from "./probes/violation";
import { paintDisagreement, wrongPaint } from "./probes/wrong-paint";

/** The probes that can see each effect's commit point. An order-ground decision is `outcomeOrder` for any effect. */
export const PROBE_FOR: Readonly<Record<WitnessEffect, readonly ProbeName[]>> = {
  prevent: ["defaultAction", "outcomeOrder"],
  stop: ["outcomeOrder"],
  dataTransfer: ["outcomeOrder"],
  attrs: ["wrongPaint"],
  setVar: ["wrongPaint"],
  scrollBy: ["wrongPaint"],
  scrollTo: ["wrongPaint"],
  scrollIntoView: ["wrongPaint"],
  setValue: ["wrongPaint"],
  focus: ["focusAtNextInput"],
  capture: ["outcomeOrder"],
  show: ["wrongPaint"],
  hide: ["wrongPaint"],
  setProp: ["wrongPaint"],
};

/** The criterion the control arm applies: null when two runs agree. */
function disagreement(kind: ProbeName, b: Binding, a: Observation, c: Observation): string | null {
  switch (kind) {
    case "wrongPaint":
      return paintDisagreement(a.trajectory, c.trajectory);
    case "defaultAction": {
      const left = leftTrajectory(b.eventType, a.snapshot, c.snapshot);
      return left.length ? left.join(", ") : null;
    }
    case "outcomeOrder":
      return firstDifference(a.outcome, c.outcome);
    case "focusAtNextInput":
      return firstDifference(a.snapshot.typedInto, c.snapshot.typedInto);
  }
}

/** Throws the probe's `WitnessViolation` when the counterfactual observation leaves the real one's. */
export function judge(kind: ProbeName, b: Binding, real: Observation, cf: Observation): void {
  switch (kind) {
    case "wrongPaint":
      return wrongPaint(real.trajectory, cf.trajectory);
    case "defaultAction":
      return defaultAction(b.eventType, real.snapshot, cf.snapshot);
    case "outcomeOrder":
      return outcomeOrder(real.outcome, cf.outcome);
    case "focusAtNextInput":
      return focusAtNextInput(real.snapshot.typedInto, cf.snapshot.typedInto);
  }
}

export interface WitnessOptions {
  /** The counterfactual's placement, when not the effect's next-best one. */
  mode?: "drop" | "delay" | "rehome" | "order";
  frames?: number;
  /** A command kind's library scenario. */
  scenario?: Scenario;
}

/** One test of a pair, in the shape `test(title, run)` takes. */
export interface WitnessTest {
  title: string;
  run: (fixtures: { browser: Browser }, testInfo: TestInfo) => Promise<void>;
}

export function witness(kind: ProbeName, site: string, effect: WitnessEffect, options: WitnessOptions = {}): WitnessTest[] {
  const b = binding(site);
  if (!b.effects.includes(effect)) throw new Error(`witness ${site}: the binding does not list the effect "${effect}"`);
  if (options.mode !== "order" && !PROBE_FOR[effect].includes(kind)) {
    throw new Error(`witness ${site}: the probe for "${effect}" is ${PROBE_FOR[effect].join(" or ")}, not ${kind} (the library chooses the probe)`);
  }
  const title = `${kind} ${site} ${effect}${options.scenario ? ` (${options.scenario})` : ""}`;
  const live = "fixture" in b.page || siteList().includes(site);
  const scenario = options.scenario ?? "event";

  const control: WitnessTest = { title: `control: ${title}`, run: async ({ browser }) => {
    test.fixme(!live, `site ${site} is not in the generated site list`);
    const first = await observe(browser, b, null, scenario);
    const second = await observe(browser, b, null, scenario);
    const differ = disagreement(kind, b, first, second);
    if (differ) throw new Error(`the control arm is not deterministic: two real runs disagree (${differ})`);
    if (first.hits + second.hits) throw new Error("the real arm was intercepted: witness mode is not off");
  } };

  const counter: WitnessTest = { title: `witness: ${title}`, run: async ({ browser }, testInfo) => {
    test.fixme(!live, `site ${site} is not in the generated site list`);
    test.fail();
    try {
      const real = await observe(browser, b, null, scenario);
      const cf = await observe(browser, b, counterfactual(b, effect, options), scenario);
      if (cf.hits === 0) throw new Error("the counterfactual intercepted nothing: the effect never ran under the marker, so the witness proves nothing");
      judge(kind, b, real, cf);
    } catch (error) {
      if (error instanceof WitnessViolation) throw error;
      // not the probe's verdict: the body must not throw, so that `test.fail()` reports the witness red
      testInfo.annotations.push({ type: "witness-error", description: String(error) });
    }
  } };

  return [control, counter];
}
