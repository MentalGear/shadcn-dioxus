//! The ordered channel's counters and their read-out.
//!
//! Rust owns the aggregation: this webview's [`Stats`] count messages, ops and bytes each way and keep the
//! last [`RING`] samples of two durations of an asking message, the wait in the outbox (`queue_us`) and the
//! round trip from the send to the answer (`round_trip_us`). The page counts at the bridge
//! (`window.__dxBridgeStats`, written by `channel()` in `ts/bridge.ts`) and Rust reads that object over the
//! channel it already has when it writes the read-out.
//!
//! The read-out is the file named by the environment variable [`READOUT_VAR`]: a release webview has its
//! inspector off, so the numbers leave through the file system. [`PING_VAR`] adds one trivial asking
//! message per frame, the workload that fills the round-trip distribution under whatever the page renders.
//! Neither variable is read on `wasm32`; the web arm's spec reads `window.__dxBridgeStats` itself.
//! Timing uses `Instant`, which `wasm32-unknown-unknown` cannot read: [`now`] is `None` there.

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};

/// The environment variable that names the file the stats are written to (about once a second).
pub(in crate::bridge) const READOUT_VAR: &str = "DX_BRIDGE_STATS";
/// The environment variable that adds a ping per frame (any non-empty value).
pub(in crate::bridge) const PING_VAR: &str = "DX_BRIDGE_PING";
/// Samples kept per duration; older ones are overwritten.
const RING: usize = 4096;

/// A point in time, `None` where the platform has no clock.
pub(in crate::bridge) type Stamp = Option<Instant>;

pub(in crate::bridge) fn now() -> Stamp {
    #[cfg(not(target_family = "wasm"))]
    return Some(Instant::now());
    #[cfg(target_family = "wasm")]
    None
}

/// Microseconds from `from` to `to`, when both are known.
pub(in crate::bridge) fn micros(from: Stamp, to: Stamp) -> Option<u64> {
    Some(to?.saturating_duration_since(from?).as_micros() as u64)
}

pub(in crate::bridge) fn readout_path() -> Option<PathBuf> {
    #[cfg(not(target_family = "wasm"))]
    return std::env::var_os(READOUT_VAR)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);
    #[cfg(target_family = "wasm")]
    None
}

pub(in crate::bridge) fn ping_enabled() -> bool {
    #[cfg(not(target_family = "wasm"))]
    return std::env::var_os(PING_VAR).is_some_and(|value| !value.is_empty());
    #[cfg(target_family = "wasm")]
    false
}

/// The last [`RING`] samples of one duration, in microseconds.
#[derive(Default)]
struct Ring {
    samples: Vec<u64>,
    seen: u64,
}

impl Ring {
    fn push(&mut self, micros: u64) {
        if self.samples.len() < RING {
            self.samples.push(micros);
        } else {
            self.samples[(self.seen % RING as u64) as usize] = micros;
        }
        self.seen += 1;
    }

    /// Nearest-rank percentiles of the kept samples, plus the samples themselves.
    fn summary(&self) -> Value {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let at = |p: usize| match sorted.len() {
            0 => 0,
            n => sorted[(n * p).div_ceil(100).clamp(1, n) - 1],
        };
        json!({
            "count": self.seen,
            "p50": at(50),
            "p95": at(95),
            "p99": at(99),
            "max": sorted.last().copied().unwrap_or(0),
            "samples": self.samples,
        })
    }
}

/// One webview's counters.
pub(in crate::bridge) struct Stats {
    /// The read-out file, read from the environment once; bytes are counted only when it is set.
    pub(in crate::bridge) readout: Option<PathBuf>,
    fired: u64,
    asked: u64,
    ops: u64,
    answers: u64,
    bytes_out: u64,
    bytes_in: u64,
    queue: Ring,
    trip: Ring,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            readout: readout_path(),
            fired: 0,
            asked: 0,
            ops: 0,
            answers: 0,
            bytes_out: 0,
            bytes_in: 0,
            queue: Ring::default(),
            trip: Ring::default(),
        }
    }
}

impl Stats {
    /// One message sent to the page: `ops` ops, `bytes` of JSON (0 when not counted).
    pub(in crate::bridge) fn out(&mut self, asks: bool, ops: usize, bytes: usize) {
        if asks {
            self.asked += 1;
        } else {
            self.fired += 1;
        }
        self.ops += ops as u64;
        self.bytes_out += bytes as u64;
    }

    /// One answer from the page.
    pub(in crate::bridge) fn record_answer(
        &mut self,
        queue_us: Option<u64>,
        trip_us: Option<u64>,
        bytes: usize,
    ) {
        self.answers += 1;
        self.bytes_in += bytes as u64;
        if let Some(us) = queue_us {
            self.queue.push(us);
        }
        if let Some(us) = trip_us {
            self.trip.push(us);
        }
    }

    /// The read-out document: this webview's counters and the page's `window.__dxBridgeStats` (`null` when
    /// the page could not be asked).
    pub(in crate::bridge) fn document(&self, page: Value) -> Value {
        json!({
            "schema": 1,
            "arm": if crate::NATIVE_ARM { "native" } else { "webview" },
            "rust": {
                "fired": self.fired,
                "asked": self.asked,
                "ops": self.ops,
                "answers": self.answers,
                "bytes_out": self.bytes_out,
                "bytes_in": self.bytes_in,
                "queue_us": self.queue.summary(),
                "round_trip_us": self.trip.summary(),
            },
            "page": page,
        })
    }
}

/// Writes `document` to `path` through a sibling file and a rename, so a reader never sees half a file.
pub(in crate::bridge) fn write_readout(path: &Path, document: &Value) -> std::io::Result<()> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    std::fs::write(&temp, serde_json::to_vec(document)?)?;
    std::fs::rename(&temp, path)
}

/// Starts the task behind the two environment variables, when either is set: a ping per frame under
/// [`PING_VAR`], the read-out file under [`READOUT_VAR`]. Called once, when the pump is created.
pub(in crate::bridge) fn start(hub: super::Hub) {
    let path = readout_path();
    let ping = ping_enabled();
    if path.is_none() && !ping {
        return;
    }
    // interval-ok: the read-out task lives as long as the webview's pump and ends when its channel closes
    crate::dioxus_core::spawn_forever(async move {
        let mut written: Stamp = None;
        loop {
            let frame = std::time::Duration::from_millis(if ping { 16 } else { 1000 });
            crate::sleep::sleep(frame).await;
            if ping {
                let _ = crate::bridge::get(
                    crate::bridge::Target::Window,
                    [crate::bridge::Prop::InnerWidth],
                )
                .await;
            }
            if hub.0.borrow().closed.is_some() {
                break;
            }
            let due = written.is_none() || micros(written, now()).is_some_and(|us| us >= 1_000_000);
            if let (true, Some(path)) = (due, &path) {
                let page = super::page_stats().await;
                let document = hub.0.borrow().stats.document(page);
                if let Err(error) = write_readout(path, &document) {
                    tracing::warn!("dx bridge stats: {error}");
                }
                written = now();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_environment_variable_names_the_readout_file() {
        std::env::set_var(READOUT_VAR, "/tmp/dx-readout-probe.json");
        assert_eq!(
            readout_path().as_deref(),
            Some(std::path::Path::new("/tmp/dx-readout-probe.json"))
        );
        std::env::remove_var(READOUT_VAR);
        assert_eq!(readout_path(), None);
    }

    #[test]
    fn a_readout_file_holds_the_counters_and_the_percentiles() {
        let mut stats = Stats::default();
        stats.out(true, 3, 40);
        stats.out(false, 2, 20);
        for micros in 1..=100u64 {
            stats.record_answer(Some(micros), Some(micros * 10), 8);
        }
        let dir = std::env::temp_dir().join(format!("dx-readout-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("stats.json");
        write_readout(&file, &stats.document(serde_json::Value::Null)).unwrap();
        let doc: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        assert_eq!(doc["rust"]["asked"], 1);
        assert_eq!(doc["rust"]["fired"], 1);
        assert_eq!(doc["rust"]["ops"], 5);
        assert_eq!(doc["rust"]["bytes_out"], 60);
        assert_eq!(doc["rust"]["round_trip_us"]["count"], 100);
        assert_eq!(doc["rust"]["round_trip_us"]["p50"], 500);
        assert_eq!(doc["rust"]["round_trip_us"]["p95"], 950);
        assert_eq!(doc["rust"]["round_trip_us"]["max"], 1000);
        assert!(doc["page"].is_null());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
