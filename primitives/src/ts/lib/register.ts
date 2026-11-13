// What every behaviour module does last: hand its adapter to the page's bridge. Typed against the
// GENERATED behaviour union, so a module for a kind Rust does not declare, or with the wrong parameters,
// is a compile error. Bundled into each behaviour entry (this is `ts/lib/`, not an entry).

import { KINDS, type Behaviour, type Kind } from "../behaviours/behaviours.gen";

/** The generated event-field and attribute-read lists of each kernel kind. */
export { ADAPTER } from "../behaviours/behaviours.gen";

export function register<K extends Kind>(kind: K, impl: DxBehaviourImpl<Extract<Behaviour, { kind: K }>>): void {
  if (!(KINDS as readonly string[]).includes(kind)) throw new Error("dx behaviour " + kind + " is not a generated kind");
  const bridge = window.__dxBridge;
  if (!bridge) throw new Error("dx behaviour " + kind + " loaded before the bridge");
  bridge.register(kind, impl);
}

/** The adapter of a behaviour whose decision is Rust: the generated lists, and `decide` through the kernel. */
export function kernelBehaviour<K extends Kind>(
  kind: K,
  lists: { readonly fields: readonly string[]; readonly reads: readonly string[] },
): DxBehaviourImpl<Extract<Behaviour, { kind: K }>> {
  return {
    fields: lists.fields,
    reads: lists.reads,
    decide: (behaviour, event, snapshot, memory) => window.__dxBridge!.kernel.decide(kind, behaviour, event, snapshot, memory),
  };
}
