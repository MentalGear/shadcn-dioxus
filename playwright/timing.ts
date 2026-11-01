/**
 * The one marker of a timing case.
 *
 * A timing case asserts something about how long the main thread took: a long task, an interaction, a per-frame
 * cost (`main-thread.ts`, `frame-cost.ts`), and so only means anything on a release build of a machine that is doing
 * nothing else. Every such case is declared with `{ tag: TIMING_TAG }`:
 *
 *     test("title", { tag: TIMING_TAG }, async ({ page }) => { ... });
 *
 * The suite runner (`tools/run-suite.sh`) and `test:changed` (`tools/test-changed.sh`) run everything else first, with
 * their workers, and then every `@timing` case alone in `tools/timing-pass.sh`: one worker, started only once
 * `tools/wait-quiet.sh` sees the machine idle. `scripts/check-timing-tags.sh` fails a case that measures time without
 * the tag. dev-docs/dev-loop.md ("Running tests") is the rule.
 */
export const TIMING_TAG = "@timing";
