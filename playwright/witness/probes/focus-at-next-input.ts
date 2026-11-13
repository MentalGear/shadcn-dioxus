import { violation } from "./violation";

/** Red iff the next input, driven at the platform's key-repeat rate and dispatched at a frame count, lands on a
 * different element in the counterfactual arm: the focus write arrived too late for the next keystroke. */
export function focusAtNextInput(real: string[], counterfactual: string[]): void {
  if (JSON.stringify(real) !== JSON.stringify(counterfactual)) {
    throw violation("focusAtNextInput", `the next input reached ${JSON.stringify(counterfactual)} instead of ${JSON.stringify(real)}`);
  }
}
