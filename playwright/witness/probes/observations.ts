// What a run of one arm leaves behind: the library's own records, read from the page by `runner.ts` and judged only by
// the probes of this directory. Nothing here is authored per behaviour.

/** The per-element trajectory: the projected state of every element of the host subtree after each completed paint. */
export interface Trajectory {
  frames: number;
  /** Element path below the host (`""` is the host) to the states it painted, in frame order. */
  elements: Record<string, string[]>;
}

/** The default-action observables read after the event settled (`witness.ts`, `snapshot`). */
export interface Snapshot {
  activeElement: string | null;
  scrollChain: number[][];
  scrollHost: number[][];
  fieldValue: string | null;
  fieldSelection: (number | null)[] | null;
  selection: string;
  open: unknown[];
  activation: unknown[];
  location: string;
  insertion: string | null;
  outcome: string[];
  typedInto: string[];
}

export interface Observation {
  trajectory: Trajectory;
  snapshot: Snapshot;
  /** The sentinel log of the run, in the order the sentinels ran. */
  outcome: string[];
  /** How many calls the counterfactual intercepted (0 for a real arm). */
  hits: number;
}
