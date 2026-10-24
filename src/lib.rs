//! Graphics-aware settings for Bevy, built on [`bevy_settings`].
//!
//! - [`AdapterCaps`](caps::AdapterCaps) describes the GPU (backend, vendor,
//!   ray-query and DLSS support). It is detected once at startup and inserted
//!   as a resource.
//! - [`CapsAware`](caps_aware::CapsAware) lets a settings resource clamp itself
//!   to what the GPU supports. [`CapsAwarePlugin`] clamps every such resource
//!   after settings are loaded.
//! - [`DisplaySettings`](common::DisplaySettings),
//!   [`RenderSettings`](common::RenderSettings) and
//!   [`AccessibilitySettings`](common::AccessibilitySettings) are ready-made
//!   settings with platform defaults. Display and render settings are applied
//!   to the primary window and to cameras tagged
//!   [`SettingsCamera`](common::SettingsCamera).
//!
//! ## Quickstart
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings_plus::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(SettingsPlusPlugins::new("net.pavlov.my_game"))
//!     .add_systems(Startup, |mut commands: Commands| {
//!         commands.spawn((Camera3d::default(), SettingsCamera));
//!     })
//!     .run();
//! ```
//!
//! To save changes, for example from an "Apply" button:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings_plus::prelude::*;
//!
//! fn save_button(mut commands: Commands) {
//!     commands.queue(SaveSettingsDeferred::default());
//! }
//! ```
//!
//! ## Cargo features
//!
//! - `dlss`: [DLSS](https://developer.nvidia.com/rtx/dlss) upscaling through
//!   `bevy_anti_alias`. Needs the NVIDIA DLSS SDK at build time.

pub mod caps;
pub mod caps_aware;
pub mod common;

use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_ecs::schedule::SystemSet;
use bevy_settings::SettingsPlugin;

use crate::caps_aware::CapsAwarePlugin;
use crate::common::{AccessibilitySettingsPlugin, DisplaySettingsPlugin, RenderSettingsPlugin};

/// System set in `PostUpdate` that applies settings to the window and cameras.
///
/// Order your own systems against it with `.before(ApplyBindings)` or
/// `.after(ApplyBindings)`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ApplyBindings;

/// Adds everything in the right order:
///
/// 1. [`DisplaySettingsPlugin`], [`RenderSettingsPlugin`] and
///    [`AccessibilitySettingsPlugin`] register the settings types.
/// 2. [`bevy_settings::SettingsPlugin`] loads every registered
///    [`SettingsGroup`](bevy_settings::SettingsGroup) from disk.
/// 3. [`CapsAwarePlugin`] detects [`AdapterCaps`](caps::AdapterCaps) and clamps
///    every [`CapsAware`](caps_aware::CapsAware) resource to it.
///
/// Add it after `DefaultPlugins`, so the render app exists when caps are
/// detected.
///
/// Remove a part with `.disable::<T>()`:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_settings_plus::prelude::*;
/// # use bevy_app::PluginGroup;
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(
///         SettingsPlusPlugins::new("net.pavlov.my_game")
///             .build()
///             .disable::<AccessibilitySettingsPlugin>(),
///     );
/// ```
pub struct SettingsPlusPlugins {
    app_name: String,
}

impl SettingsPlusPlugins {
    /// `app_name` is a reverse-domain application id such as
    /// `"net.pavlov.my_game"`. It names the settings directory; see
    /// [`SettingsPlugin::new`].
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }
}

impl PluginGroup for SettingsPlusPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // Types must be registered before `SettingsPlugin` scans the registry.
            .add(DisplaySettingsPlugin)
            .add(RenderSettingsPlugin)
            .add(AccessibilitySettingsPlugin)
            .add(SettingsPlugin::new(&self.app_name))
            .add(CapsAwarePlugin)
    }
}

pub mod prelude {
    //! The types most users need, including the `bevy_settings` save commands.

    pub use crate::caps::{AdapterCaps, Backend, GpuVendor, PlatformTarget};
    pub use crate::caps_aware::{CapsAware, CapsAwarePlugin, ReflectCapsAware};
    pub use crate::common::{
        AccessibilitySettings, AccessibilitySettingsPlugin, AntiAlias, ColorBlindMode,
        DisplaySettings, DisplaySettingsPlugin, FpsCap, HdrPreference, MonitorSelection, MsaaLevel,
        RenderSettings, RenderSettingsPlugin, Resolution, SettingsCamera, Upscaler, UpscalerPreset,
        VsyncMode, WindowMode,
    };
    pub use crate::{ApplyBindings, SettingsPlusPlugins};
    pub use bevy_settings::{
        SaveSettings, SaveSettingsDeferred, SaveSettingsSync, SettingsGroup, SettingsPlugin,
    };
}
