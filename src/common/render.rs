//! `RenderSettings`: anti-aliasing, MSAA, upscaler, render scale, sharpness and
//! ray tracing.
//!
//! [`RenderSettingsPlugin`] applies anti-aliasing, MSAA and the upscaler to
//! cameras tagged with [`SettingsCamera`].

use bevy_anti_alias::fxaa::Fxaa;
use bevy_anti_alias::smaa::Smaa;
use bevy_anti_alias::taa::TemporalAntiAliasing;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::prelude::*;
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_log::warn;
use bevy_reflect::Reflect;
use bevy_reflect::prelude::ReflectDefault;
use bevy_render::view::Msaa;
use bevy_settings::{ReflectSettingsGroup, SettingsGroup};

use crate::ApplyBindings;
use crate::caps::{AdapterCaps, PlatformTarget};
use crate::caps_aware::{CapsAware, ReflectCapsAware};

/// Anti-aliasing technique. Only one is active on a camera at a time.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum AntiAlias {
    #[default]
    None,
    Fxaa,
    Smaa,
    Taa,
}

/// MSAA sample count.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum MsaaLevel {
    #[default]
    Off,
    Sample2,
    Sample4,
    Sample8,
}

/// Upscaler. Only `Native` and `Dlss` are implemented; the others log a
/// warning and behave like `Native`.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum Upscaler {
    #[default]
    Native,
    Dlss(UpscalerPreset),
    Fsr(UpscalerPreset),
    Xess(UpscalerPreset),
    Tsr(UpscalerPreset),
}

/// Quality preset for upscalers.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum UpscalerPreset {
    /// DLSS only: pick the tier from the display resolution.
    Auto,
    /// DLSS only: run at native resolution as anti-aliasing.
    Dlaa,
    #[default]
    Quality,
    Balanced,
    Performance,
    UltraPerformance,
}

#[cfg(feature = "dlss")]
impl From<UpscalerPreset> for bevy_anti_alias::dlss::DlssPerfQualityMode {
    fn from(preset: UpscalerPreset) -> Self {
        match preset {
            UpscalerPreset::Auto => Self::Auto,
            UpscalerPreset::Dlaa => Self::Dlaa,
            UpscalerPreset::Quality => Self::Quality,
            UpscalerPreset::Balanced => Self::Balanced,
            UpscalerPreset::Performance => Self::Performance,
            UpscalerPreset::UltraPerformance => Self::UltraPerformance,
        }
    }
}

/// Marks a camera whose anti-aliasing, MSAA and upscaler follow
/// [`RenderSettings`].
///
/// ```ignore
/// commands.spawn((Camera3d::default(), SettingsCamera));
/// ```
///
/// Settings are applied when the marker is added, so cameras can be spawned at
/// any time.
///
/// **2D cameras:** the desktop default `AntiAlias::Taa` needs 3D prepasses and
/// misrenders on a `Camera2d`. Set `anti_alias` to `Fxaa` or `Smaa` first.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SettingsCamera;

/// Render preferences.
#[derive(Resource, SettingsGroup, Reflect, Debug, Clone, PartialEq)]
#[reflect(Resource, SettingsGroup, CapsAware, Default)]
#[settings_group(group = "render")]
pub struct RenderSettings {
    pub anti_alias: AntiAlias,
    pub msaa: MsaaLevel,
    pub upscaler: Upscaler,
    /// Fraction of native resolution to render at, in `[0.25, 1.0]`. Not
    /// applied automatically.
    pub render_scale: f32,
    /// Sharpening strength, in `[0, 1]`. Not applied automatically.
    pub sharpness: f32,
    /// Ray-tracing switch, forced off without ray-query support. Not applied
    /// automatically; gate your ray-tracing plugins on it.
    pub ray_tracing: bool,
}

impl RenderSettings {
    /// The default values for `target`. [`Default`] uses the current platform.
    pub fn platform_default(target: PlatformTarget) -> Self {
        Self {
            anti_alias: if matches!(target, PlatformTarget::Wasm) {
                AntiAlias::None
            } else {
                AntiAlias::Taa
            },
            msaa: MsaaLevel::Off,
            upscaler: Upscaler::Native,
            render_scale: 1.0,
            sharpness: 0.0,
            ray_tracing: false,
        }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self::platform_default(PlatformTarget::detect())
    }
}

impl CapsAware for RenderSettings {
    fn clamp_to(&mut self, caps: AdapterCaps) {
        if self.ray_tracing && !caps.ray_query {
            self.ray_tracing = false;
        }
        // The `dlss` cargo feature is checked when the upscaler is applied.
        if matches!(self.upscaler, Upscaler::Dlss(_)) && !caps.supports_dlss {
            self.upscaler = Upscaler::Native;
        }
        self.render_scale = self.render_scale.clamp(0.25, 1.0);
        self.sharpness = self.sharpness.clamp(0.0, 1.0);
    }
}

/// Registers [`RenderSettings`] and applies it to tagged cameras, both when it
/// changes and when a camera is tagged.
pub struct RenderSettingsPlugin;

impl Plugin for RenderSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RenderSettings>();
        app.add_systems(
            PostUpdate,
            apply_render
                .in_set(ApplyBindings)
                .run_if(resource_changed::<RenderSettings>),
        );
        app.add_observer(apply_render_on_camera_add);
    }
}

fn apply_render_to_entity(
    commands: &mut Commands,
    entity: Entity,
    render: &RenderSettings,
    caps: Option<&AdapterCaps>,
) {
    apply_anti_alias(commands, entity, render.anti_alias);
    apply_msaa(commands, entity, render.msaa);
    apply_upscaler(commands, entity, render.upscaler, caps);
}

// Only re-applies the settings that changed: re-inserting
// `TemporalAntiAliasing` resets its history.
fn apply_render(
    render: Res<RenderSettings>,
    caps: Option<Res<AdapterCaps>>,
    cameras: Query<Entity, With<SettingsCamera>>,
    mut commands: Commands,
    mut prev: Local<Option<RenderSettings>>,
) {
    let aa_changed = prev
        .as_ref()
        .is_none_or(|p| p.anti_alias != render.anti_alias);
    let msaa_changed = prev.as_ref().is_none_or(|p| p.msaa != render.msaa);
    let upscaler_changed = prev.as_ref().is_none_or(|p| p.upscaler != render.upscaler);

    if aa_changed || msaa_changed || upscaler_changed {
        for entity in &cameras {
            if aa_changed {
                apply_anti_alias(&mut commands, entity, render.anti_alias);
            }
            if msaa_changed {
                apply_msaa(&mut commands, entity, render.msaa);
            }
            if upscaler_changed {
                apply_upscaler(&mut commands, entity, render.upscaler, caps.as_deref());
            }
        }
    }

    *prev = Some(render.clone());
}

// Configures a camera as soon as it is tagged. `RenderSettings` is optional
// because a camera can be spawned before the resource is inserted.
fn apply_render_on_camera_add(
    add: On<Add<SettingsCamera>>,
    render: Option<Res<RenderSettings>>,
    caps: Option<Res<AdapterCaps>>,
    mut commands: Commands,
) {
    let Some(render) = render else { return };
    apply_render_to_entity(&mut commands, add.entity, &render, caps.as_deref());
}

fn apply_anti_alias(commands: &mut Commands, entity: Entity, mode: AntiAlias) {
    let mut e = commands.entity(entity);
    match mode {
        AntiAlias::None => {
            e.remove::<(TemporalAntiAliasing, Fxaa, Smaa)>();
        }
        AntiAlias::Fxaa => {
            e.remove::<(TemporalAntiAliasing, Smaa)>()
                .insert(Fxaa::default());
        }
        AntiAlias::Smaa => {
            e.remove::<(TemporalAntiAliasing, Fxaa)>()
                .insert(Smaa::default());
        }
        AntiAlias::Taa => {
            e.remove::<(Fxaa, Smaa)>()
                .insert(TemporalAntiAliasing::default());
        }
    }
}

fn apply_msaa(commands: &mut Commands, entity: Entity, level: MsaaLevel) {
    let msaa = match level {
        MsaaLevel::Off => Msaa::Off,
        MsaaLevel::Sample2 => Msaa::Sample2,
        MsaaLevel::Sample4 => Msaa::Sample4,
        MsaaLevel::Sample8 => Msaa::Sample8,
    };
    commands.entity(entity).insert(msaa);
}

fn apply_upscaler(
    commands: &mut Commands,
    entity: Entity,
    upscaler: Upscaler,
    caps: Option<&AdapterCaps>,
) {
    clear_dlss_if_present(commands, entity);
    match upscaler {
        Upscaler::Native => {}
        Upscaler::Dlss(preset) => apply_dlss(commands, entity, preset, caps),
        Upscaler::Fsr(_) | Upscaler::Xess(_) | Upscaler::Tsr(_) => {
            warn!("Upscaler {upscaler:?} is not implemented; using Native");
        }
    }
}

#[cfg(feature = "dlss")]
fn clear_dlss_if_present(commands: &mut Commands, entity: Entity) {
    use bevy_anti_alias::dlss::{Dlss, DlssSuperResolutionFeature};
    commands
        .entity(entity)
        .remove::<Dlss<DlssSuperResolutionFeature>>();
}

#[cfg(not(feature = "dlss"))]
fn clear_dlss_if_present(_commands: &mut Commands, _entity: Entity) {}

#[cfg(feature = "dlss")]
fn apply_dlss(
    commands: &mut Commands,
    entity: Entity,
    preset: UpscalerPreset,
    caps: Option<&AdapterCaps>,
) {
    if !caps.is_some_and(|c| c.supports_dlss) {
        warn!(
            "Upscaler::Dlss selected but the adapter does not support DLSS \
             (needs Vulkan and an NVIDIA GPU); using Native"
        );
        return;
    }
    use bevy_anti_alias::dlss::{Dlss, DlssSuperResolutionFeature};
    use core::marker::PhantomData;
    commands
        .entity(entity)
        .insert(Dlss::<DlssSuperResolutionFeature> {
            perf_quality_mode: preset.into(),
            reset: false,
            _phantom_data: PhantomData,
        });
}

#[cfg(not(feature = "dlss"))]
fn apply_dlss(
    _commands: &mut Commands,
    _entity: Entity,
    _preset: UpscalerPreset,
    _caps: Option<&AdapterCaps>,
) {
    warn!("Upscaler::Dlss selected but the `dlss` cargo feature is disabled; using Native");
}
