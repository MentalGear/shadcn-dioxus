// Ambient types of the bridge (row 434 step 3): what `ts/bridge.ts` implements and what a behaviour module
// (`ts/behaviours/<kind>.ts`) registers into. Global on purpose: every entry is its own bundle, so the
// shared shapes cannot be imported at run time, only declared.

type DxTarget =
  | { kind: "window" }
  | { kind: "document" }
  | { kind: "id"; id: string }
  | { kind: "selector"; selector: string }
  | { kind: "active" };
/** A target as the verbs take it: the spec Rust sends, an element id, or a live object (bespoke scripts). */
type DxTargetish = DxTarget | EventTarget | string | null | undefined;

/** An epoch: a write or install carrying an older `n` than the target already saw for `key` is dropped. */
interface DxEpoch {
  key: string;
  n: number;
}

type DxFields = Record<string, unknown>;

/** What a behaviour decided; the adapter in `bridge.ts` applies it (the only code that cancels an event). */
interface DxDecision {
  /** Cancel the default action. Honoured only for a behaviour Rust declared non-passive. */
  prevent?: boolean;
  /** Stop propagation. Same rule. */
  stop?: boolean;
  /** Attributes to set on the host element. */
  attrs?: Record<string, string>;
  /** A message for Rust, sent after the event. */
  send?: unknown;
  /** Per-install state: merged into the `memory` the next `decide` of this install receives. */
  remember?: Record<string, unknown>;
}

/**
 * A named behaviour: a PURE `decide(behaviour, eventFields, state)` and the closed names of what the
 * adapter must read for it. The adapter owns the listener, its phase and passivity (declared by the Rust
 * `Behaviour`), the lease and the epoch; the module owns only the decision.
 */
interface DxBehaviourImpl<B> {
  /** Event fields (closed `EventField` wire names) read at event time into `event`. */
  readonly fields: readonly string[];
  /** Attributes of the host element read at event time into `state`. */
  readonly reads?: readonly string[];
  decide(behaviour: B, event: DxFields, state: Record<string, string | null>, memory: Record<string, unknown>): DxDecision;
}

/** A command a Rust session sends to `serve`. */
type DxCommand =
  | { cmd: "listen"; target: DxTargetish; event: string; options?: { capture?: boolean; once?: boolean }; fields?: string[]; initial?: boolean }
  | { cmd: "observe"; target: DxTargetish; kind: "resize" | "intersection" | "mutation"; options?: DxFields }
  | { cmd: "media"; query: string }
  | { cmd: "sample"; target: DxTargetish; fields: DxSampleField[]; until: DxSampleUntil; onlyChanged?: boolean }
  | {
      cmd: "behaviour";
      target: DxTargetish;
      behaviour: { kind: string; [param: string]: unknown };
      declared: DxDeclared;
      epoch?: DxEpoch | null;
    };

/** What Rust declares about a behaviour's listener; the page never chooses any of it. */
interface DxDeclared {
  target: "window" | "document" | "element";
  event: string;
  capture: boolean;
  /** `false` is the only value under which `prevent` / `stop` of a decision are honoured. */
  passive: boolean;
  /** Fail-open lease: the listener removes itself this long after the last install or renewal. */
  leaseMs?: number | null;
}

type DxSampleField =
  | string
  | { name: string; mode?: "coalesced" }
  | { name: string; mode: "batched"; event: string; pick?: string[]; capture?: boolean; flush?: "frame" | "task" };
type DxSampleUntil =
  | { kind: "forever" }
  | { kind: "frames"; n: number }
  | { kind: "ms"; ms: number }
  | { kind: "attr"; name: string; value?: string };

/** An op of a transaction (or of the pump). `epoch` makes a call or set droppable when stale. */
type DxOp =
  | { op: "call"; target: DxTargetish; method: string; args?: unknown[]; epoch?: DxEpoch | null }
  | { op: "get"; target: DxTargetish; props: string[]; epoch?: DxEpoch | null }
  | { op: "set"; target: DxTargetish; name: string; value: string | null; epoch?: DxEpoch | null };
/** `dropped`: an op of a newer epoch had already run on the target, so this one did nothing. A reply echoes the epoch it answers. */
type DxOpResult = { ok: unknown; dropped?: true; epoch?: DxEpoch } | { err: string };

/** A message of the ordered channel: `id` set asks for `{ id, results }` back, null is fire-and-forget. */
type DxPumpMessage = { id: number | null; ops: DxOp[] };

interface DxScope {
  /** The channel the scope's messages go through (the bridge-latency mode applies to it). */
  readonly channel: DioxusChannel;
  /** A listener whose removal this scope owns; the scroll-blocking events default to passive. */
  on(
    target: DxTargetish,
    type: string,
    handler: (event: any) => void,
    options?: boolean | { capture?: boolean; passive?: boolean; once?: boolean },
  ): () => void;
  cleanup(fn: () => void): void;
  send(message: unknown): void;
  /** Ends the scope: every registration it made is undone, in reverse order. Idempotent. */
  dispose(): void;
  run(command: DxCommand): void;
}

interface DxBridge {
  version: number;
  channel(raw: DioxusChannel): DioxusChannel;
  scope(raw?: DioxusChannel): DxScope;
  serve(raw: DioxusChannel): Promise<void>;
  pump(raw: DioxusChannel): Promise<void>;
  call(target: DxTargetish, method: string, args?: unknown[]): unknown;
  get(target: DxTargetish, props: string[]): Record<string, unknown>;
  set(target: DxTargetish, name: string, value: string | null): void;
  transaction(ops: DxOp[]): DxOpResult[];
  /** The named behaviours loaded in this page, by kind. */
  behaviours: Record<string, DxBehaviourImpl<any>>;
  /** Called by a behaviour module as it is evaluated. */
  register(kind: string, impl: DxBehaviourImpl<any>): void;
}

interface Window {
  __dxBridge?: DxBridge;
  /**
   * Test only (the bridge-latency harness): defer every message through a bridge channel by this many
   * animation frames, in both directions; `"never"` drops every message to Rust.
   */
  __dxBridgeDelayFrames?: number | "never";
  /** `window.__dxJsListeners.live` maps a registration id to its event type while it is live. */
  __dxJsListeners?: { n: number; live: Map<number, string> };
}
