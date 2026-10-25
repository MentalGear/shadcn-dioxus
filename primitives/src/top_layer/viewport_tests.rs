use super::VIEWPORT_METRICS_JS;

/// The clamp/flip scripts must measure the viewport ONLY through
/// [`VIEWPORT_METRICS_JS`]: a direct `window.innerWidth` /
/// `visualViewport.width` read overshoots the content area by the scrollbar
/// gutter (an overlay lands ~11px past it). Source-level guard so a
/// new collision script cannot quietly bypass the shared helper.
#[test]
fn viewport_metrics_js_is_the_only_viewport_read() {
    // The module's non-test sources, one `(file, text)` per part.
    let parts: [(&str, &str); 6] = [
        ("mod.rs", include_str!("mod.rs")),
        ("anchor_fallback.rs", include_str!("anchor_fallback.rs")),
        ("attributes.rs", include_str!("attributes.rs")),
        ("popover_sync.rs", include_str!("popover_sync.rs")),
        ("scroll_clamp.rs", include_str!("scroll_clamp.rs")),
        ("styles.rs", include_str!("styles.rs")),
    ];
    let mut verbatim = false;
    for (file, source) in parts {
        let rest = source.replacen(VIEWPORT_METRICS_JS, "", 1);
        verbatim |= rest.len() < source.len();
        scan(file, &rest);
    }
    assert!(
        verbatim,
        "VIEWPORT_METRICS_JS text must appear verbatim in one of the top_layer parts"
    );
}

fn scan(file: &str, code: &str) {
    for (n, line) in code.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        for needle in [
            "window.innerWidth",
            "window.innerHeight",
            "vv.width",
            "vv.height",
            "visualViewport.width",
            "visualViewport.height",
        ] {
            assert!(
                !line.contains(needle),
                "{file}:{} reads `{needle}` outside VIEWPORT_METRICS_JS; use viewportMetrics()",
                n + 1
            );
        }
    }
}

#[test]
fn viewport_metrics_js_takes_the_smaller_of_visual_and_client() {
    let js = VIEWPORT_METRICS_JS;
    assert!(js.contains("Math.min(vv.width, clientW)"));
    assert!(js.contains("Math.min(vv.height, clientH)"));
    assert!(js.contains("documentElement"));
    assert!(js.contains("clientWidth"));
    assert!(js.contains("clientHeight"));
}
