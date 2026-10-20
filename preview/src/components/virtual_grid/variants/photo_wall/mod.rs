//! A photo wall of 90,000 generated photos of mixed aspect ratios, in two layouts behind one
//! toggle:
//!
//! - **Grid**: the two-axis `VirtualGrid`, 300 x 300 structurally identical square cells. The
//!   grid's rows and columns never depend on the photos, so both axes window independently and the
//!   grid's keyboard model is untouched; how a photo fits its cell (`contain` on a mat, or
//!   `cover`) is presentation.
//! - **Justified**: `VirtualJustifiedGrid` over the same photos and aspects, on a canvas wide
//!   enough for a roughly square wall that scrolls both ways: every photo uncropped at its own
//!   aspect, rows filling the canvas edge to edge, rows and the tiles of each row windowed.
//!
//! Each photo is one `<img src="data:image/svg+xml,...">` (one DOM node per tile, not a dozen SVG
//! elements). The counter, drag-to-pan, the minimap, the random jump and the preview work in both.
use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogHeader, DialogTitle};
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use crate::components::switch::Switch;
use crate::components::toggle::ToggleVariant;
use crate::components::toggle_group::{ToggleGroup, ToggleItem};
use dioxus::prelude::*;
use std::collections::HashSet;
use dioxus_primitives::js_listener::{use_js_listeners, JsListeners};

/// Tiles per side: the wall is `SIDE x SIDE` = 90,000 photos.
const SIDE: usize = 300;
/// A cell's width and height in px (the grid's row height and column width).
const TILE: u32 = 128;
/// The justified wall's canvas width in px: wide enough that 90,000 photos at a `TILE` row
/// height make a roughly square wall (about 300 photos per row), like the grid's.
const JUSTIFIED_CANVAS: f64 = 46_000.0;
/// The element id the bridge script finds the grid and the minimap under.
const ROOT_ID: &str = "photo-wall-root";

/// Deterministic aspect ratio (width, height) of photo `index`: portrait 2:3 and 3:4, square,
/// landscape 4:3 and 3:2 and 16:9, and the occasional 3:1 panorama.
fn aspect(index: usize) -> (u32, u32) {
    const SET: [(u32, u32); 10] =
        [(2, 3), (3, 2), (1, 1), (4, 3), (3, 4), (3, 2), (16, 9), (4, 3), (2, 3), (3, 1)];
    let mixed = index.wrapping_mul(2_654_435_761) >> 7;
    SET[(mixed ^ (index / 7)) % SET.len()]
}

/// The `data:` URI of generated photo `index`: a sky gradient, a sun and two hills whose colours
/// and positions derive from the index, drawn at the photo's own aspect ratio (so the image has
/// that intrinsic ratio: `width`/`height` are twice the viewBox's, in the same ratio; a 2:1 box
/// around every drawing once made `contain` letterbox each photo a second time inside its own
/// image, so a photo filled about half its cell). Nothing is fetched.
fn photo_src(index: usize) -> String {
    let (aw, ah) = aspect(index);
    let w = 100.0 * aw as f32 / ah as f32;
    let hue = (index * 47 + index / SIDE * 13) % 360;
    let peak = 48 + (index * 29) % 22;
    let sun_x = w * (0.2 + ((index * 37) % 60) as f32 / 100.0);
    let sun_y = 16 + (index * 13) % 22;
    let (a, b, c) = (w * 0.3, w * 0.6, w * 0.55);
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {w:.1} 100' width='{wp:.0}' height='200'>\
         <defs><linearGradient id='g' x1='0' y1='0' x2='0' y2='1'>\
         <stop offset='0' stop-color='hsl({hue} 70% 72%)'/><stop offset='1' stop-color='hsl({h2} 75% 88%)'/>\
         </linearGradient></defs><rect width='{w:.1}' height='100' fill='url(#g)'/>\
         <circle cx='{sun_x:.1}' cy='{sun_y}' r='8' fill='hsl(48 95% 62%)'/>\
         <path d='M0 {p1} Q {a:.1} {p2} {b:.1} {p3} T {w:.1} {peak} V 100 H 0 Z' fill='hsl({h3} 35% 55%)'/>\
         <path d='M0 82 Q {a:.1} {p4} {c:.1} 80 T {w:.1} 78 V 100 H 0 Z' fill='hsl({h4} 45% 38%)'/></svg>",
        wp = w * 2.0,
        h2 = (hue + 40) % 360,
        h3 = (hue + 130) % 360,
        h4 = (hue + 150) % 360,
        p1 = peak + 12,
        p2 = peak - 8,
        p3 = peak + 6,
        p4 = peak + 4,
    );
    // Percent-encode what a data URI cannot carry raw (`%` first, so the escapes below survive).
    let encoded = svg
        .replace('%', "%25")
        .replace('#', "%23")
        .replace('<', "%3C")
        .replace('>', "%3E");
    format!("data:image/svg+xml,{encoded}")
}

/// What the bridge script reports: the mounted tile count and the scroll geometry.
#[derive(Clone, PartialEq)]
struct WallState {
    /// How many `[data-grid-cell]` tiles are in the DOM right now.
    count: usize,
    /// Distance scrolled from the inline start, px.
    left: f64,
    /// Distance scrolled from the top, px.
    top: f64,
    /// Scroll width of the whole wall, px.
    sw: f64,
    /// Scroll height of the whole wall, px.
    sh: f64,
    /// Visible width, px.
    cw: f64,
    /// Visible height, px.
    ch: f64,
}

impl WallState {
    /// Reads the bridge's `[count, left, top, sw, sh, cw, ch]` message.
    fn from_message(m: &[f64]) -> Option<Self> {
        let [count, left, top, sw, sh, cw, ch] = *m else { return None };
        Some(Self { count: count as usize, left, top, sw, sh, cw, ch })
    }
}

impl Default for WallState {
    fn default() -> Self {
        let side = (SIDE as u32 * TILE) as f64;
        Self { count: 0, left: 0.0, top: 0.0, sw: side, sh: side, cw: 640.0, ch: 448.0 }
    }
}

/// Runs in the page for the life of the demo; every listener goes through `listen`, so the scope
/// removes them. It reports scroll geometry and the mounted tile count (one message per frame),
/// pans the wall by dragging with a mouse or pen (touch keeps its native scroll; a press that
/// never travels 4px stays a click) and moves the viewport when the minimap is pressed.
const BRIDGE_JS: &str = r#"
    const root = document.getElementById(await dioxus.recv());
    if (!root) return;
    const grid = root.querySelector('[data-virtual-grid]');
    const mini = root.querySelector('[data-photo-wall-minimap]');
    if (!grid) return;
    const isRtl = () => getComputedStyle(grid).direction === 'rtl';

    let raf = 0;
    let last = '';
    const report = () => {
        raf = 0;
        const state = [
            grid.querySelectorAll('[data-grid-cell]').length,
            Math.abs(grid.scrollLeft), grid.scrollTop,
            grid.scrollWidth, grid.scrollHeight,
            grid.clientWidth, grid.clientHeight,
        ];
        const key = state.join(' ');
        if (key === last) return;
        last = key;
        dioxus.send(state);
    };
    const schedule = () => { if (!raf) raf = requestAnimationFrame(report); };
    listen(grid, 'scroll', schedule, { passive: true });
    const mutations = new MutationObserver(schedule);
    mutations.observe(grid, { childList: true, subtree: true });
    const resizes = new ResizeObserver(schedule);
    resizes.observe(grid);
    cleanup(() => { mutations.disconnect(); resizes.disconnect(); if (raf) cancelAnimationFrame(raf); });

    let press = null;
    let panned = false;
    const inScrollport = (e) => {
        const box = grid.getBoundingClientRect();
        const x = e.clientX - box.left - grid.clientLeft;
        const y = e.clientY - box.top - grid.clientTop;
        return x >= 0 && y >= 0 && x < grid.clientWidth && y < grid.clientHeight;
    };
    listen(grid, 'pointerdown', (e) => {
        if (e.pointerType === 'touch' || e.button !== 0 || !inScrollport(e)) return;
        press = { x: e.clientX, y: e.clientY, left: grid.scrollLeft, top: grid.scrollTop, id: e.pointerId, moving: false };
    });
    listen(grid, 'pointermove', (e) => {
        if (!press || e.pointerId !== press.id) return;
        const dx = e.clientX - press.x;
        const dy = e.clientY - press.y;
        if (!press.moving) {
            if (Math.hypot(dx, dy) < 4) return;
            press.moving = true;
            grid.setPointerCapture(e.pointerId);
            grid.style.cursor = 'grabbing';
        }
        grid.scrollLeft = press.left - dx;
        grid.scrollTop = press.top - dy;
    });
    const release = (e) => {
        if (!press || e.pointerId !== press.id) return;
        panned = press.moving;
        press = null;
        grid.style.cursor = '';
        // The click that follows this pointerup ends a pan; it is not a tile choice.
        if (panned) setTimeout(() => { panned = false; }, 0);
    };
    listen(grid, 'pointerup', release);
    listen(grid, 'pointercancel', release);
    listen(grid, 'click', (e) => {
        if (!panned) return;
        e.stopPropagation();
        e.preventDefault();
        panned = false;
    }, true);

    if (mini) {
        const jump = (e) => {
            const box = mini.getBoundingClientRect();
            const rtl = isRtl();
            const fx = Math.min(1, Math.max(0, (rtl ? box.right - e.clientX : e.clientX - box.left) / box.width));
            const fy = Math.min(1, Math.max(0, (e.clientY - box.top) / box.height));
            const left = Math.max(0, fx * grid.scrollWidth - grid.clientWidth / 2);
            const top = Math.max(0, fy * grid.scrollHeight - grid.clientHeight / 2);
            grid.scrollTo({ left: rtl ? -left : left, top, behavior: 'instant' });
        };
        let dragging = false;
        listen(mini, 'pointerdown', (e) => {
            if (e.button !== 0) return;
            dragging = true;
            mini.setPointerCapture(e.pointerId);
            jump(e);
        });
        listen(mini, 'pointermove', (e) => { if (dragging) jump(e); });
        const stop = () => { dragging = false; };
        listen(mini, 'pointerup', stop);
        listen(mini, 'pointercancel', stop);
    }
    report();
"#;


/// Scrolls tile `row:column` to the middle of the wall and focuses it once the grid has mounted it.
const JUMP_JS: &str = r#"
    const [rootId, row, column, tile] = await dioxus.recv();
    const root = document.getElementById(rootId);
    const grid = root && root.querySelector('[data-virtual-grid]');
    if (!grid) return;
    const rtl = getComputedStyle(grid).direction === 'rtl';
    const left = Math.max(0, column * tile - (grid.clientWidth - tile) / 2);
    const top = Math.max(0, row * tile - (grid.clientHeight - tile) / 2);
    grid.scrollTo({ left: rtl ? -left : left, top, behavior: 'instant' });
    let tries = 0;
    const go = () => {
        const cell = grid.querySelector('[data-grid-cell="' + row + ':' + column + '"]');
        if (!cell) {
            if (tries++ < 120) requestAnimationFrame(go);
            return;
        }
        // focus-ok: the button press is a user action that asks for this tile; the grid mounts it on scroll
        cell.focus({ preventScroll: true });
    };
    go();
"#;

/// The two layouts of the wall.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// Uniform cells on the two-axis `VirtualGrid`.
    Grid,
    /// Justified rows on `VirtualJustifiedGrid`.
    Justified,
}

#[component]
pub fn Demo() -> Element {
    let mut wall = use_signal(WallState::default);
    let selected = use_signal(|| 0usize);
    let mut preview_open = use_signal(|| false);
    // The active photo by index, shared by both layouts (the justified grid owns it two-way).
    let mut active_index = use_signal(|| 0usize);
    let mut cover = use_signal(|| false);
    let mut layout = use_signal(|| Layout::Grid);
    let aspects = use_signal(|| {
        (0..SIDE * SIDE)
            .map(|index| {
                let (w, h) = aspect(index);
                f64::from(w) / f64::from(h)
            })
            .collect::<Vec<f64>>()
    });

    // Reinstalled when the layout changes: the script binds to the grid element, which is new.
    use_js_listeners(move || {
        let _ = layout();
        Some(
            JsListeners::install(BRIDGE_JS)
                .arg(ROOT_ID)
                .on_message(move |message: Vec<f64>| {
                    if let Some(state) = WallState::from_message(&message) {
                        if *wall.peek() != state {
                            wall.set(state);
                        }
                    }
                }),
        )
    });

    let state = wall();
    let pct = |part: f64, whole: f64| (part / whole.max(1.0) * 100.0).clamp(0.0, 100.0);
    let view_left = pct(state.left, state.sw);
    let view_top = pct(state.top, state.sh);
    let view_width = pct(state.cw, state.sw);
    let view_height = pct(state.ch, state.sh);
    let current = active_index();
    let chosen = selected();
    let (aw, ah) = aspect(chosen);
    let justified = layout() == Layout::Justified;
    let open_photo = move |index: usize| {
        let (mut selected, mut preview_open) = (selected, preview_open);
        selected.set(index);
        preview_open.set(true);
    };

    rsx! {
        div {
            id: ROOT_ID,
            "data-fit": if cover() { "cover" } else { "contain" },
            "data-photo-wall-layout": if justified { "justified" } else { "grid" },
            style: "display: flex; flex-direction: column; gap: var(--dx-space-2); width: 100%; min-width: 0;",
            // How a photo fits its cell: `contain` (the whole photo on a mat) or `cover` (cropped to fill).
            // The `var()` shorthands of the minimap box and the preview live here, not in their inline
            // `style`: those strings change with the scroll position and the chosen photo, and Dioxus's
            // web renderer drops a `var()` shorthand from an inline `style` the first time it is re-set
            // (the box lost its border and fill once the wall mounted its first tiles).
            style {
                "[data-photo-img] {{ display: block; width: 100%; height: 100%; object-fit: contain; }} [data-fit=\"cover\"] [data-photo-img] {{ object-fit: cover; }} [data-photo-wall-viewport] {{ border: 2px solid var(--dx-primary); background: var(--dx-accent); }} [data-photo-preview] {{ border-radius: var(--dx-radius-md); }}"
            }
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-2) var(--dx-space-4);",
                ToggleGroup {
                    horizontal: true,
                    role: "group",
                    variant: ToggleVariant::Outline,
                    pressed: Some(HashSet::from([usize::from(justified)])),
                    on_pressed_change: move |pressed: HashSet<usize>| {
                        // Radio-style: pressing the pressed item again keeps it.
                        if pressed.contains(&1) {
                            layout.set(Layout::Justified);
                        } else if pressed.contains(&0) {
                            layout.set(Layout::Grid);
                        }
                    },
                    aria_label: "Layout",
                    ToggleItem { index: 0usize, "Grid" }
                    ToggleItem { index: 1usize, "Justified" }
                }
                if !justified {
                    Field { orientation: FieldOrientation::Horizontal,
                        Switch {
                            id: "photo-wall-cover",
                            checked: cover(),
                            on_checked_change: move |value| cover.set(value),
                        }
                        FieldLabel { html_for: "photo-wall-cover", "Crop photos to fill their cells" }
                    }
                }
            }
            div {
                // Enter or Space on the focused tile opens it, as a click does.
                onkeydown: move |event: KeyboardEvent| {
                    if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
                        event.prevent_default();
                        open_photo(*active_index.peek());
                    }
                },
                if justified {
                    VirtualJustifiedGrid {
                        aspects,
                        target_row_height: f64::from(TILE),
                        canvas_width: Some(JUSTIFIED_CANVAS),
                        active: active_index,
                        render_tile: move |(index, _rect): (usize, TileRect)| rsx! {
                            img {
                                "data-photo-img": "{aspect(index).0}:{aspect(index).1}",
                                src: photo_src(index),
                                alt: "Generated landscape {index + 1}",
                                draggable: "false",
                                style: "cursor: pointer; border-radius: inherit;",
                                onclick: move |_| open_photo(index),
                            }
                        },
                        "aria-label": "Justified photo wall, 90,000 photos",
                        style: "height: 28rem; cursor: grab; touch-action: pan-x pan-y;",
                    }
                } else {
                    VirtualGrid {
                        row_count: SIDE,
                        column_count: SIDE,
                        row_height: |_| TILE,
                        column_width: |_| TILE,
                        render_cell: move |(row, column): (usize, usize)| {
                            let index = row * SIDE + column;
                            rsx! {
                                div {
                                    // Fills the cell past the grid sheet's inline padding: a thin mat the photo sits on
                                    // (6px of 128: the long side of every photo fills about 90% of the cell).
                                    style: "box-sizing: border-box; width: calc(100% + 2 * var(--dx-space-2)); height: 100%; margin-inline: calc(-1 * var(--dx-space-2)); padding: 6px; background: var(--dx-muted); cursor: pointer; user-select: none; -webkit-user-select: none;",
                                    onclick: move |_| open_photo(index),
                                    img {
                                        "data-photo-img": "{aspect(index).0}:{aspect(index).1}",
                                        src: photo_src(index),
                                        alt: "Generated landscape {index + 1}",
                                        draggable: "false",
                                    }
                                }
                            }
                        },
                        row_buffer: 3usize,
                        column_buffer: 3usize,
                        on_active_change: move |(row, column): (usize, usize)| active_index.set(row * SIDE + column),
                        "aria-label": "Photo wall, 300 by 300 photos",
                        style: "height: 28rem; cursor: grab; touch-action: pan-x pan-y;",
                    }
                }
            }
            div { style: "display: flex; flex-wrap: wrap; gap: var(--dx-space-3); align-items: flex-end;",
                div { style: "display: flex; flex-direction: column; gap: var(--dx-space-2); align-items: flex-start; min-width: 0;",
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            spawn(async move {
                                let mut random = document::eval("dioxus.send([Math.random(), Math.random()])");
                                if let Ok([r, c]) = random.recv::<[f64; 2]>().await {
                                    let row = ((r * SIDE as f64) as usize).min(SIDE - 1);
                                    let column = ((c * SIDE as f64) as usize).min(SIDE - 1);
                                    if *layout.peek() == Layout::Justified {
                                        // The justified grid reveals and focuses a tile written into its active signal.
                                        active_index.set(row * SIDE + column);
                                    } else {
                                        let _ = document::eval(JUMP_JS).send((ROOT_ID, row, column, TILE));
                                    }
                                }
                            });
                        },
                        "Jump to random photo"
                    }
                    p {
                        "data-photo-wall-count": "true",
                        // Wide enough for the longest count, so the number arriving after mount moves nothing.
                        style: "margin: 0; min-width: 30ch; font-size: var(--dx-text-sm); font-variant-numeric: tabular-nums;",
                        "{state.count} of 90,000 photos rendered"
                    }
                    p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                        if justified {
                            "Photo {current + 1}. Scroll, drag the wall, or use the arrow keys (up and down pick the nearest photo); Enter opens a photo."
                        } else {
                            "Photo {current + 1} (row {current / SIDE + 1}, column {current % SIDE + 1}). Scroll, drag the wall, or use the arrow keys; Enter opens a photo."
                        }
                    }
                }
                // The whole wall, scaled: the outlined box is what the grid shows. Press or drag it to move there.
                div {
                    "data-photo-wall-minimap": "true",
                    "aria-hidden": "true",
                    style: "position: relative; flex: none; width: 7.5rem; height: 7.5rem; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-md); background: var(--dx-muted); cursor: pointer; touch-action: none; overflow: hidden;",
                    div {
                        "data-photo-wall-viewport": "true",
                        style: "position: absolute; box-sizing: border-box; inset-inline-start: {view_left}%; top: {view_top}%; width: {view_width}%; height: {view_height}%; min-width: 4px; min-height: 4px; pointer-events: none;",
                    }
                }
            }
            Dialog { open: preview_open(), on_open_change: move |open| preview_open.set(open),
                DialogHeader {
                    DialogTitle { "Photo {chosen + 1}" }
                    DialogDescription {
                        "Row {chosen / SIDE + 1}, column {chosen % SIDE + 1} of the grid; aspect ratio {aw}:{ah}."
                    }
                }
                // The photo at its real aspect ratio, whole.
                img {
                    "data-photo-preview": "{aw}:{ah}",
                    src: photo_src(chosen),
                    alt: "Generated landscape {chosen + 1}",
                    style: "display: block; width: 100%; max-width: 28rem; max-height: 60vh; aspect-ratio: {aw} / {ah}; object-fit: contain; margin-inline: auto;",
                }
            }
        }
    }
}
