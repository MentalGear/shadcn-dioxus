// The typed violation a witness passes on. It is constructed only in this directory (a `tsc`-API ban in
// `scripts/check-bridge-behaviours.sh`), so a thrown timeout, a locator error or a spec-authored assertion can never
// stand in for a probe's verdict.

export type ProbeName = "wrongPaint" | "defaultAction" | "outcomeOrder" | "focusAtNextInput";

export class WitnessViolation extends Error {
  readonly probe: ProbeName;
  readonly detail: string;
  constructor(probe: ProbeName, detail: string) {
    super(`${probe}: ${detail}`);
    this.name = "WitnessViolation";
    this.probe = probe;
    this.detail = detail;
  }
}

/** The one way a probe reports a red. */
export function violation(probe: ProbeName, detail: string): WitnessViolation {
  return new WitnessViolation(probe, detail);
}
