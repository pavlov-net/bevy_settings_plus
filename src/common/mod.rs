//! The ready-made settings. Each is a settings resource with a plugin that
//! registers it:
//!
//! - [`DisplaySettings`]: window mode, monitor, resolution, vsync.
//! - [`RenderSettings`]: anti-aliasing, MSAA, upscaler, render scale.
//! - [`AccessibilitySettings`]: subtitles, motion, colour-blind mode.

mod accessibility;
mod display;
mod render;

pub use accessibility::*;
pub use display::*;
pub use render::*;
