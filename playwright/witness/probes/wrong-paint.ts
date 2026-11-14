import { violation } from "./violation";
import type { Trajectory } from "./observations";

/** The states `other` painted for an element that `real` never painted for it. A lagging effect paints a time-shifted
 * copy of the real trajectory (every state is one the real arm also painted) and is green here: that effect is next
 * frame by derivation, and only a `lag` or `order` verdict admits it. */
export function stray(real: Trajectory, other: Trajectory): string[] {
  const out: string[] = [];
  for (const [path, states] of Object.entries(other.elements)) {
    const seen = new Set(real.elements[path] ?? []);
    for (const state of new Set(states)) if (!seen.has(state)) out.push(`${path || "host"}: ${state}`);
  }
  return out;
}

/** Whether two runs of the same arm paint the same states for every element (the control arm's determinism check). */
export function paintDisagreement(a: Trajectory, b: Trajectory): string | null {
  const ab = stray(a, b);
  const ba = stray(b, a);
  return ab.length || ba.length ? `only the second run painted ${ab.slice(0, 2).join(" | ") || "nothing"}; only the first painted ${ba.slice(0, 2).join(" | ") || "nothing"}` : null;
}

/** Red iff the counterfactual arm paints, for some element, a projected state the real arm never painted for it. */
export function wrongPaint(real: Trajectory, counterfactual: Trajectory): void {
  const strays = stray(real, counterfactual);
  if (strays.length) throw violation("wrongPaint", `the counterfactual painted ${strays.length} state(s) the real arm never painted: ${strays[0]}`);
}
