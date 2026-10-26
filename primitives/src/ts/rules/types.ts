// The closed set of rules Rust can hand the interpreter (`primitives/src/bridge/rules.rs`'s `Rule`
// enum, serde `tag = "kind"`, kebab-case). One member per Rust variant, inside the same marked
// region; `scripts/check-bridge-rules.sh` compares the Rust enum with the interpreter's switch in
// `rules/index.ts` one to one.

/** What an installed rule may use of the scope that owns it. */
export interface RuleScope {
  /** Registers a listener whose removal this scope owns (the passive default of `wheel`/`touch*` applies). */
  on(
    target: EventTarget | null,
    type: string,
    handler: (event: any) => void,
    options?: boolean | { capture?: boolean; passive?: boolean; once?: boolean },
  ): () => void;
  /** Registers other teardown, run when the scope ends. */
  cleanup(fn: () => void): void;
  /** Sends a message to Rust, if the scope has a channel; otherwise nothing. */
  send(message: unknown): void;
}

// region:B1 begin (event gates: a decision inside the listener call)
// region:B1 end

// region:B2 begin (drag and swipe recogniser)
// region:B2 end

// region:B3 begin (measurement publisher)
// region:B3 end

// region:B5 begin (reveal and settle)
// region:B5 end

// region:FIXTURE begin (test fixtures only, no component uses these)
/** On `event` at the host element, sets `attr` to `value` and tells Rust how often it fired. */
export interface FixtureRule {
  kind: "fixture";
  event: string;
  attr: string;
  value: string;
}
// region:FIXTURE end

/** Every rule, discriminated by `kind`. */
export type Rule = FixtureRule;
