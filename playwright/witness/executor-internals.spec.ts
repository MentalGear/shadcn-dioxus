// The executor's internals (the exit wait and the watchdog), admitted as a whole by the reds they already have:
// pre-authored as `test.fixme` keyed to the generated site list, live when the executor builds its sites (row 449).
// The exit wait: Rust waits for a `finished` fact, red because an invisible modal holds the page inert. The watchdog:
// dropped, so the CSS fail-open alone must keep a pending state from staying open.

import { test } from "../fixtures";
import { witness } from "./witness";

test.describe("the executor's internals", () => {
  for (const t of [...witness("wrongPaint", "executor.exit_wait", "hide"), ...witness("wrongPaint", "executor.watchdog", "attrs", { mode: "drop" })]) test(t.title, t.run);
});
