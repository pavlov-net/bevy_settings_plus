//! Spawns a tagged camera after a state transition instead of at `Startup`.
//! The camera still gets the current render settings, because they are applied
//! when `SettingsCamera` is added.
//!
//! ```bash
//! cargo run --example deferred_camera
//! ```

use bevy::prelude::*;
use bevy_anti_alias::taa::TemporalAntiAliasing;
use bevy_settings_plus::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SettingsPlusPlugins::new(
            "net.pavlov.bevy_settings_plus_deferred_example",
        ))
        .init_state::<AppState>()
        .add_systems(Startup, kick_off_loading)
        .add_systems(OnEnter(AppState::InGame), spawn_camera)
        .add_observer(verify_taa_attached)
        .run();
}

/// Stands in for loading finishing.
fn kick_off_loading(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::InGame);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        SettingsCamera,
    ));
}

/// Logs when `TemporalAntiAliasing` (the desktop default) is attached to the
/// tagged camera.
fn verify_taa_attached(
    add: On<Add<TemporalAntiAliasing>>,
    cameras: Query<(), With<SettingsCamera>>,
) {
    if cameras.contains(add.entity) {
        info!("TemporalAntiAliasing attached to the deferred camera");
    }
}
