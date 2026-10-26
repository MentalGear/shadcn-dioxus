// What every behaviour module does last: hand its pure `decide` to the page's bridge. Typed against the
// GENERATED behaviour union, so a module for a kind Rust does not declare, or with the wrong parameters,
// is a compile error. Bundled into each behaviour entry (this is `ts/lib/`, not an entry).

import type { Behaviour, Kind } from "../behaviours/behaviours.gen";

export function register<K extends Kind>(kind: K, impl: DxBehaviourImpl<Extract<Behaviour, { kind: K }>>): void {
  const bridge = window.__dxBridge;
  if (!bridge) throw new Error("dx behaviour " + kind + " loaded before the bridge");
  bridge.register(kind, impl);
}
