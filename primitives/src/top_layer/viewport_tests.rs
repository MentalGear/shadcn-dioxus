use super::PLACEMENT_HELPERS_JS;

/// The placement scripts must measure the viewport ONLY through
/// [`PLACEMENT_HELPERS_JS`]: a direct `window.innerWidth` /
/// `visualViewport.width` read overshoots the content area by the scrollbar
/// gutter (an overlay lands ~11px past it). Source-level guard so a
/// new placement script cannot quietly bypass the shared helper.
#[test]
fn viewport_metrics_js_is_the_only_viewport_read() {
    // The module's non-test sources, one `(file, text)` per part.
    let parts: [(&str, &str); 7] = [
        ("mod.rs", include_str!("mod.rs")),
        ("anchor_fallback.rs", include_str!("anchor_fallback.rs")),
        ("attributes.rs", include_str!("attributes.rs")),
        ("popover_sync.rs", include_str!("popover_sync.rs")),
        ("scroll_clamp.rs", include_str!("scroll_clamp.rs")),
        ("styles.rs", include_str!("styles.rs")),
        (
            "select/components/list.rs",
            include_str!("../select/components/list.rs"),
        ),
    ];
    let mut verbatim = false;
    for (file, source) in parts {
        let rest = source.replacen(PLACEMENT_HELPERS_JS, "", 1);
        verbatim |= rest.len() < source.len();
        scan(file, &rest);
    }
    assert!(
        verbatim,
        "PLACEMENT_HELPERS_JS text must appear verbatim in one of the top_layer parts"
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
                "{file}:{} reads `{needle}` outside PLACEMENT_HELPERS_JS; use visible()",
                n + 1
            );
        }
    }
}

#[test]
fn placement_helpers_take_the_smaller_of_visual_and_fixed_containing_block() {
    let js = PLACEMENT_HELPERS_JS;
    assert!(js.contains("Math.min(vv ? vv.width : Infinity, p.width)"));
    assert!(js.contains("Math.min(vv ? vv.height : Infinity, p.height)"));
    assert!(js.contains("position:fixed;inset:0"));
}

/// The visible region and every rect are document coordinates: a rect minus the origin of the
/// root element's own rect, never plus a scroll offset (that is right in one engine's rect frame
/// and wrong in the other's).
#[test]
fn placement_helpers_work_in_document_coordinates() {
    let js = PLACEMENT_HELPERS_JS;
    assert!(js.contains("root.getBoundingClientRect()"));
    assert!(js.contains("r.left - v.ox"));
    assert!(js.contains("vv.offsetTop"));
    for needle in ["scrollX", "scrollY", "pageXOffset", "pageYOffset"] {
        assert!(!js.contains(needle), "{needle} mixes coordinate frames");
    }
}
