// `verdict.json`: the owner's measurement of a next-frame row, which is that row's form of the witness (the same
// counterfactual, run on hardware). The kit writes it from the owner's dump and only the owner commits it.
//   { "verdict": "pass" | "fail", "reason": "parity" | "lag" | "order" | "integrity", "devices": [...], "traces": [...] }
// A `fail` with the reason `lag` or `order` admits the effect (it is red where a witness would have been red); a
// `fail` with `parity` or `integrity` means the Rust arm is to be fixed; a `pass` admits nothing.

import { existsSync, readFileSync } from "node:fs";

export type VerdictReason = "parity" | "lag" | "order" | "integrity";
export interface Verdict {
  verdict: "pass" | "fail";
  reason: VerdictReason;
  devices: string[];
  traces: string[];
}

const REASONS = ["parity", "lag", "order", "integrity"];

export function parseVerdict(file: string, text: string): Verdict {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch (error) {
    throw new Error(`${file}: not JSON (${String(error)})`);
  }
  const v = value as Record<string, unknown>;
  const list = (x: unknown): x is string[] => Array.isArray(x) && x.every((s) => typeof s === "string");
  if (typeof v !== "object" || v === null) throw new Error(`${file}: must be an object`);
  if (v.verdict !== "pass" && v.verdict !== "fail") throw new Error(`${file}: "verdict" must be "pass" or "fail"`);
  if (typeof v.reason !== "string" || !REASONS.includes(v.reason)) throw new Error(`${file}: "reason" must be one of ${REASONS.join(", ")}`);
  if (!list(v.devices) || !v.devices.length) throw new Error(`${file}: "devices" must list the devices measured`);
  if (!list(v.traces)) throw new Error(`${file}: "traces" must be a list of trace file names`);
  return v as unknown as Verdict;
}

/** The verdict at `file`, or null when none has been measured yet. */
export function readVerdict(file: string): Verdict | null {
  return existsSync(file) ? parseVerdict(file, readFileSync(file, "utf8")) : null;
}

/** Whether a verdict admits the effect it was measured for: it failed, for a reason only a late or reordered effect has. */
export function admits(verdict: Verdict | null): boolean {
  return verdict !== null && verdict.verdict === "fail" && (verdict.reason === "lag" || verdict.reason === "order");
}
