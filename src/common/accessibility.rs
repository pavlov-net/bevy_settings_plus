//! `AccessibilitySettings`: subtitles, motion and colour-blind mode.
//!
//! Nothing is applied automatically; read the resource in your own UI and
//! shaders.

use bevy_app::{App, Plugin};
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use bevy_reflect::prelude::ReflectDefault;
use bevy_settings::{ReflectSettingsGroup, SettingsGroup};

use crate::caps::{AdapterCaps, PlatformTarget};
use crate::caps_aware::{CapsAware, ReflectCapsAware};

/// Colour-blind compensation mode.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum ColorBlindMode {
    #[default]
    None,
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

/// Accessibility preferences.
#[derive(Resource, SettingsGroup, Reflect, Debug, Clone, PartialEq)]
#[reflect(Resource, SettingsGroup, CapsAware, Default)]
#[settings_group(group = "accessibility")]
pub struct AccessibilitySettings {
    pub subtitles_on: bool,
    pub subtitle_scale: f32,
    pub reduce_motion: bool,
    pub color_blind_mode: ColorBlindMode,
}

impl AccessibilitySettings {
    /// The default values for `target`.
    pub fn platform_default(_target: PlatformTarget) -> Self {
        Self {
            subtitles_on: false,
            subtitle_scale: 1.0,
            reduce_motion: false,
            color_blind_mode: ColorBlindMode::None,
        }
    }
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self::platform_default(PlatformTarget::detect())
    }
}

impl CapsAware for AccessibilitySettings {
    fn clamp_to(&mut self, _caps: AdapterCaps) {
        self.subtitle_scale = self.subtitle_scale.clamp(0.5, 3.0);
    }
}

/// Registers [`AccessibilitySettings`].
pub struct AccessibilitySettingsPlugin;

impl Plugin for AccessibilitySettingsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AccessibilitySettings>();
    }
}
