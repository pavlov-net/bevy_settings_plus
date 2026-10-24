//! Logs the detected GPU caps and the resolved settings at startup, and spawns
//! a tagged camera.
//!
//! ```bash
//! cargo run --example basic
//! ```

use bevy::prelude::*;
use bevy_settings_plus::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SettingsPlusPlugins::new(
            "net.pavlov.bevy_settings_plus_basic_example",
        ))
        .add_systems(Startup, (spawn_camera, log_resolved_settings))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        SettingsCamera,
    ));
}

fn log_resolved_settings(
    caps: Option<Res<AdapterCaps>>,
    display_cfg: Res<DisplaySettings>,
    render_cfg: Res<RenderSettings>,
) {
    info!("=== bevy_settings_plus: Startup ===");
    if let Some(caps) = caps {
        info!(
            "AdapterCaps: backend={:?} vendor={:?} ray_query={} dlss_supported={}",
            caps.backend, caps.vendor, caps.ray_query, caps.supports_dlss
        );
    } else {
        info!("AdapterCaps not available (headless?).");
    }
    info!(
        "DisplaySettings: window_mode={:?} vsync={:?}",
        display_cfg.window_mode, display_cfg.vsync
    );
    info!(
        "RenderSettings: anti_alias={:?} msaa={:?} upscaler={:?} ray_tracing={}",
        render_cfg.anti_alias, render_cfg.msaa, render_cfg.upscaler, render_cfg.ray_tracing
    );
}
