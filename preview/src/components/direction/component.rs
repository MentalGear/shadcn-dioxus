//! Text direction: the provider and hook every direction-aware component reads.
//!
//! A re-export of `dioxus_primitives::direction`, so an app has one import path for it next to
//! its other components. There is nothing to style (the provider renders no element); the
//! stylesheet exists so the component installs like every other.
#[allow(unused_imports, reason = "re-export for apps; the preview itself only uses the demo")]
pub use dioxus_primitives::direction::{
    use_direction, Direction, DirectionProvider, HorizontalNav,
};
