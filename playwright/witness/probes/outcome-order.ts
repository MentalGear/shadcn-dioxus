import { violation } from "./violation";

/** The first position at which two sentinel logs differ, or null when they are the same sequence. */
export function firstDifference(a: string[], b: string[]): string | null {
  const at = a.findIndex((entry, i) => entry !== b[i]);
  if (at >= 0) return `at ${at}: ${a[at]} against ${b[at] ?? "nothing"}`;
  return a.length === b.length ? null : `length ${a.length} against ${b.length}`;
}

/** Red iff the counterfactual's outcome differs from the real arm's recorded outcome. There is never an expected
 * order: the sentinels are the harness's own (a later listener on the path, a drop target, a pointer recorder). */
export function outcomeOrder(real: string[], counterfactual: string[]): void {
  const differs = firstDifference(real, counterfactual);
  if (differs) throw violation("outcomeOrder", `the counterfactual's outcome differs from the real arm's ${differs}`);
}
