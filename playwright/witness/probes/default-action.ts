import { violation } from "./violation";
import type { Snapshot } from "./observations";

export type DefaultActionEvent = "keydown" | "wheel" | "touchmove" | "beforeinput" | "mousedown" | "drop" | "dragover" | "click";

/** The closed table of default-action observables per event type (a snapshot key each), defined by the library. */
export const OBSERVABLES: Readonly<Record<DefaultActionEvent, readonly (keyof Snapshot)[]>> = {
  keydown: ["open", "activeElement", "scrollChain", "fieldValue", "fieldSelection", "activation"],
  wheel: ["scrollChain"],
  touchmove: ["scrollChain"],
  beforeinput: ["fieldValue", "fieldSelection"],
  mousedown: ["selection", "activeElement"],
  drop: ["location", "insertion"],
  dragover: ["outcome"],
  click: ["activation", "location", "open"],
};

/** The observables of `event` that differ between two snapshots, by name. */
export function leftTrajectory(event: DefaultActionEvent, real: Snapshot, other: Snapshot): string[] {
  return OBSERVABLES[event].filter((key) => JSON.stringify(real[key]) !== JSON.stringify(other[key]));
}

/** Red iff the counterfactual arm's default-action observables leave the real arm's. */
export function defaultAction(event: DefaultActionEvent, real: Snapshot, counterfactual: Snapshot): void {
  const left = leftTrajectory(event, real, counterfactual);
  if (left.length) throw violation("defaultAction", `after a ${event} the counterfactual's ${left.join(", ")} differ from the real arm's`);
}
