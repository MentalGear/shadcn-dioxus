# The desktop latency session (owner script)

One session on one of your own devices produces the one real desktop latency distribution that sets the next-frame
budget, the desktop-cadence delay and the pass baseline ([row 450](./backlog/450-phase-0-5-dxbridgestats-and-the-latency-session.md),
[row 442](./backlog/442-desktop-cadence-harness-and-desktop-pass.md), [row 443](./backlog/443-owner-measurement-kit-perf-recorder-and-impl-switch.md)).
Nothing here has been built or run: the desktop binary has never been built ([row 23](./backlog/023-preview-s-desktop-feature-lacks-dioxus.md)).
The first run is also the check of this page.

Time: about 5 minutes of use, after a release build of an estimated 15 to 40 minutes (inferred; fat-LTO crates).
Use a quiet machine: no other builds, power plugged in, display at its normal refresh rate.

## What to build

On the device (Linux needs `webkit2gtk` and gtk development packages; macOS and Windows need nothing extra):

```
dx build --release --platform desktop --package preview --features desktop
```

`dx` prints the path of the binary it wrote. If a flag is refused, `dx build --help` names the desktop spelling;
tell the lane which one worked.

## What to run

```
set -a; . preview/desktop-latency.env; set +a
<the binary dx printed>
```

1. In the window open the Dialog demo and press its trigger once. The stats start with the first message on the
   bridge, so a page that never talks to it writes no file.
2. Stay on the Demos page and use it for about 3 minutes: scroll, hover rows, open and close a popover or a menu,
   type in an input. The environment's ping adds one trivial message per frame (about 60 a second) under whatever
   the page renders, so the distribution fills even while you are idle.
3. Close the window.

## What to send back

`~/dx-bridge-stats.json` (the path in `preview/desktop-latency.env`), plus the device (model, OS, webview engine and
version, display refresh rate). The file is written through a temporary file and a rename about once a second, so
closing the window never leaves half a file.

## What the file holds

- `rust.round_trip_us`: microseconds from sending an asking message to its answer, with `p50`, `p95`, `p99`,
  `max` and the last 4,096 samples. `rust.queue_us` is the wait in the outbox before the send.
- `rust.fired`, `asked`, `ops`, `bytes_out`, `bytes_in`: message counts and bytes each way.
- `page.channels[]`: per channel, messages `sent` and `recv` and `turn`, the page's share of a round trip in
  milliseconds. A round trip minus its turn is the transport's share.
- `arm`: `webview` (this session) or `native` (no JavaScript, `page` is `null`).

## On the web, without a desktop build

Open any page in Chrome or Safari: `window.__dxBridgeStats` is the same object as `page` above.
`?dx-bridge-spin=3` burns 3 ms on every bridge message each way, an emulation of the desktop's per-message cost for
a feel test on a trackpad. It is not a desktop model. Rust's own timing (`rust.*`) is read on native targets only.

## What the first run must confirm

- The binary builds with the command above and a window opens.
- The file appears within a second or two of step 1 and `rust.round_trip_us.count` grows while you use the page.
- `page` is an object, not `null` (the page answered the stats request).
- The ping workload does not itself make the window janky: if it does, say so and rerun without `DX_BRIDGE_PING`.
