// The eval channel Dioxus hands every `document::eval` script, typed once so the TypeScript that
// talks to Rust is checked (row 434 step 3). A script body is run as `new Function("dioxus", body)`
// inside an `async function`: `dioxus.send(value)` reaches `Eval::recv` on the Rust side and
// `await dioxus.recv()` reads what Rust `Eval::send` sent. Both ends are JSON, so only
// JSON-serialisable values cross.

/** The channel of one `document::eval`. */
interface DioxusChannel {
  /** JS to Rust. Asynchronous: the message reaches Rust after the current turn. */
  send(message: unknown): void;
  /** Rust to JS. Resolves with the next message Rust sent. */
  recv(): Promise<unknown>;
}

/** The channel as a script body sees it (a parameter of the wrapping function). */
declare const dioxus: DioxusChannel;
