// One arm of a witness: a fresh browser context, the page side injected before any page script, the host subtree
// recorded after every completed paint, the event replayed as trusted input, and the library's records read back.
// A witness is two arms (the real one and the counterfactual); this file runs one and judges nothing.

import type { Browser, Page } from "@playwright/test";
import { witnessPage } from "../../primitives/ts-test/witness";
import type { WitnessArm, WitnessEffect } from "../../primitives/ts-test/witness";
import { BASE_URL } from "../base-url";
import { gotoHydrated } from "../hydration";
import { gotoReference } from "../page-kinds";
import type { Binding, Replay } from "./binding";
import type { Observation, Snapshot, Trajectory } from "./probes/observations";
import { propsOfSheet } from "./projection";
import { fixtureHtml } from "./self/fixtures";

export type Scenario = "event" | "mount" | "toggle";

/** The next-best placement each effect gets (`sync-js-root`, admission): the counterfactual the witness moves it to. */
const DEFAULT_MODE: Record<WitnessEffect, WitnessArm["mode"]> = {
  prevent: "rehome",
  stop: "drop",
  dataTransfer: "drop",
  attrs: "delay",
  setVar: "delay",
  scrollBy: "delay",
  scrollTo: "delay",
  scrollIntoView: "delay",
  setValue: "delay",
  focus: "delay",
  capture: "delay",
  show: "delay",
  hide: "delay",
  setProp: "delay",
};

/** Completed paints a delayed effect waits: provisionally 3, to be set to the p99 of the latency session. */
export const DELAY_FRAMES = 3;

export function counterfactual(b: Binding, effect: WitnessEffect, over: Partial<Pick<WitnessArm, "mode" | "frames">> = {}): WitnessArm {
  return { site: b.site, effect, mode: over.mode ?? DEFAULT_MODE[effect], frames: over.frames ?? DELAY_FRAMES, host: b.host };
}

const FIXTURE_URL = "http://witness.fixture/page.html";

async function open(page: Page, b: Binding, kernel: boolean): Promise<void> {
  if ("fixture" in b.page) {
    const html = fixtureHtml(b.page.fixture, kernel);
    await page.route(FIXTURE_URL, (route) => route.fulfill({ contentType: "text/html", body: html }));
    await gotoReference(page, FIXTURE_URL);
  } else await gotoHydrated(page, BASE_URL + b.page.route);
}

const paints = (page: Page, n: number): Promise<void> => page.evaluate((count) => new Promise<void>((done) => (window as any).__dxWitness.paints(count, done)), n);

async function replay(page: Page, r: Replay): Promise<void> {
  switch (r.kind) {
    case "key":
      return page.keyboard.press(r.key);
    case "text":
      return page.keyboard.insertText(r.text);
    case "click":
      return page.locator(r.selector).click();
    case "wheel": {
      const box = await page.locator(r.selector).boundingBox();
      if (!box) throw new Error(`witness: ${r.selector} has no box to wheel over`);
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      return page.mouse.wheel(r.dx, r.dy);
    }
  }
}

async function setup(page: Page, b: Binding): Promise<void> {
  for (const step of b.setup) {
    const target = page.locator(step.selector);
    if (step.op === "click") await target.click();
    else if (step.op === "hover") await target.hover();
    else if (step.op === "focus") await target.focus();
    else if (step.op === "fill") await target.fill(step.text);
    else await target.press(step.key);
  }
}

/** Runs one arm: `arm` null is the real run (witness mode off, the patched members never intercept). */
export async function observe(browser: Browser, b: Binding, arm: WitnessArm | null, scenario: Scenario = "event", kernel = true): Promise<Observation> {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    const props = propsOfSheet(b.sheet);
    await page.addInitScript(witnessPage);
    const early = arm !== null && (b.armAt === "load" || scenario === "mount");
    if (early) await page.addInitScript((a) => (window as any).__dxWitness.arm(a), arm);
    if (scenario === "mount") await page.addInitScript((a) => (window as any).__dxWitness.recordWhenPresent(a.host, a.props), { host: b.host, props });
    await open(page, b, kernel);
    await page.evaluate(
      ({ host, sentinels }) => {
        const w = (window as any).__dxWitness;
        w.pause(host, true);
        w.watchTyped(host);
        for (const sentinel of sentinels) w.sentinel(sentinel);
      },
      { host: b.host, sentinels: b.sentinels ?? [] },
    );
    await setup(page, b);
    if (arm && !early) await page.evaluate((a) => (window as any).__dxWitness.arm(a), arm);
    if (scenario !== "mount") {
      await page.evaluate(({ host, props }) => (window as any).__dxWitness.record(host, props), { host: b.host, props });
      await paints(page, 2); // frames of the state before the event, in both arms
      await replay(page, scenario === "toggle" && b.toggle ? b.toggle : b.event);
      if (b.keyAfterPaints !== undefined) {
        await paints(page, b.keyAfterPaints);
        await page.keyboard.press("x");
      }
    }
    await paints(page, b.frames);
    const result = await page.evaluate((host) => {
      const w = (window as any).__dxWitness;
      return { trajectory: w.stopRecording(), snapshot: w.snapshot(host), outcome: w.outcome(), hits: w.hits(), applied: w.applied() };
    }, b.host);
    return result as { trajectory: Trajectory; snapshot: Snapshot; outcome: string[]; hits: number; applied: string[] };
  } finally {
    await context.close();
  }
}
