// The scenario binding: one JSON file per `site!` (`bindings/<site>.json`), authored with the witness by the
// non-owning author. A vector is `decide`'s input, not a page, so the binding supplies what a vector cannot: the
// route, the host locator, the setup steps and the vector's event replayed as trusted input on the host. Command-kind
// sites also get the library's two scenarios (`mount`, `toggle`), so that including or omitting a mount vector cannot
// choose the result. JSON, so that a gate and a Rust test read the same file.

import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import type { WitnessEffect, WitnessSentinel } from "../../primitives/ts-test/witness";
import type { DefaultActionEvent } from "./probes/default-action";

export type Step =
  | { op: "click" | "hover" | "focus"; selector: string }
  | { op: "fill"; selector: string; text: string }
  | { op: "press"; selector: string; key: string };

/** An event replayed as trusted input (CDP), never a synthetic dispatch. */
export type Replay =
  | { kind: "key"; key: string }
  | { kind: "text"; text: string }
  | { kind: "click"; selector: string }
  | { kind: "wheel"; selector: string; dx: number; dy: number };

export interface Binding {
  /** The `site!` id; the file is named for it. */
  site: string;
  /** The behaviour's own files (repository paths): a change to one selects this site's witnesses (`select.mjs`). */
  files: string[];
  effects: WitnessEffect[];
  /** A prerendered route, or a self-test fixture page (`self/fixtures.ts`). */
  page: { route: string } | { fixture: string };
  /** CSS locator of the host element the component renders. */
  host: string;
  /** The host component's `style.css`: its declared properties join the projection. */
  sheet?: string;
  setup: Step[];
  event: Replay;
  eventType: DefaultActionEvent;
  /** Completed paints recorded after the event. */
  frames: number;
  /** When the counterfactual is armed: before the page's scripts run (a mount) or just before the event. */
  armAt?: "load" | "trigger";
  /** The next key is dispatched this many completed paints after the event (the focus probe). */
  keyAfterPaints?: number;
  sentinels?: WitnessSentinel[];
  /** A command kind's library scenarios: a client mount after hydration, and a runtime toggle from the other state. */
  scenarios?: ("mount" | "toggle")[];
  toggle?: Replay;
}

const EFFECTS = ["prevent", "stop", "dataTransfer", "attrs", "setVar", "scrollBy", "scrollTo", "scrollIntoView", "setValue", "focus", "capture", "show", "hide", "setProp"];
const EVENTS = ["keydown", "wheel", "touchmove", "beforeinput", "mousedown", "drop", "dragover", "click"];

function fail(file: string, why: string): never {
  throw new Error(`witness binding ${file}: ${why}`);
}

const isObject = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);
const strings = (v: unknown): v is string[] => Array.isArray(v) && v.every((x) => typeof x === "string");

function checkReplay(file: string, key: string, v: unknown): void {
  if (!isObject(v)) fail(file, `${key} must be an object`);
  const need: Record<string, string[]> = { key: ["key"], text: ["text"], click: ["selector"], wheel: ["selector", "dx", "dy"] };
  const fields = typeof v.kind === "string" ? need[v.kind] : undefined;
  if (!fields) fail(file, `${key}.kind must be one of ${Object.keys(need).join(", ")}`);
  for (const f of fields) if (v[f] === undefined) fail(file, `${key}.${f} is required for kind "${v.kind}"`);
}

/** The binding a file holds, or an error naming the file and the first thing wrong with it. */
export function validateBinding(file: string, value: unknown): Binding {
  if (!isObject(value)) fail(file, "must be an object");
  const name = file.replace(/^.*\//, "").replace(/\.json$/, "");
  if (value.site !== name) fail(file, `"site" must be "${name}" (the file is named for its site)`);
  if (!strings(value.files) || !value.files.length) fail(file, '"files" must list the behaviour\'s own files');
  if (!strings(value.effects) || !value.effects.length || !value.effects.every((e) => EFFECTS.includes(e))) fail(file, `"effects" must be a non-empty subset of ${EFFECTS.join(", ")}`);
  const page = value.page;
  if (!isObject(page) || (typeof page.route !== "string" && typeof page.fixture !== "string")) fail(file, '"page" must be {route} or {fixture}');
  if (typeof value.host !== "string" || !value.host) fail(file, '"host" must be a CSS locator');
  if (!Array.isArray(value.setup)) fail(file, '"setup" must be an array of steps');
  for (const [i, step] of value.setup.entries()) {
    if (!isObject(step) || !["click", "hover", "focus", "fill", "press"].includes(step.op as string) || typeof step.selector !== "string") fail(file, `setup[${i}] needs an op and a selector`);
    if (step.op === "fill" && typeof step.text !== "string") fail(file, `setup[${i}] fill needs text`);
    if (step.op === "press" && typeof step.key !== "string") fail(file, `setup[${i}] press needs key`);
  }
  checkReplay(file, "event", value.event);
  if (!EVENTS.includes(value.eventType as string)) fail(file, `"eventType" must be one of ${EVENTS.join(", ")}`);
  if (!Number.isInteger(value.frames) || (value.frames as number) < 1) fail(file, '"frames" must be a positive integer');
  if (value.armAt !== undefined && value.armAt !== "load" && value.armAt !== "trigger") fail(file, '"armAt" must be "load" or "trigger"');
  if (value.scenarios !== undefined && !(strings(value.scenarios) && value.scenarios.every((s) => s === "mount" || s === "toggle"))) fail(file, '"scenarios" must be a subset of mount, toggle');
  if (value.scenarios?.includes("toggle" as never)) checkReplay(file, "toggle", value.toggle);
  return value as unknown as Binding;
}

/** The binding of `site`: `bindings/<site>.json`, or `bindings/_self/<site>.json` for the harness's own fixtures. */
export function binding(site: string): Binding {
  for (const dir of ["bindings", "bindings/_self"]) {
    const file = join(__dirname, dir, `${site}.json`);
    if (existsSync(file)) return validateBinding(file, JSON.parse(readFileSync(file, "utf8")));
  }
  throw new Error(`witness binding: no bindings/${site}.json`);
}
