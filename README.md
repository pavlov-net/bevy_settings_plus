# bevy_settings_plus

Graphics-aware settings for [Bevy](https://bevy.org), built on
[`bevy_settings`](https://docs.rs/bevy-settings). Settings are saved to disk and
adjusted to what the player's GPU supports.

- `DisplaySettings`, `RenderSettings` and `AccessibilitySettings`: ready-made
  settings with platform defaults. Display and render settings are applied to
  the primary window and to cameras tagged with `SettingsCamera`.
- `AdapterCaps`: a resource describing the GPU (backend, vendor, ray-query and
  DLSS support).
- `CapsAware`: implement it on your own settings to clamp them to what the GPU
  supports.

## Usage

```rust
use bevy::prelude::*;
use bevy_settings_plus::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SettingsPlusPlugins::new("net.pavlov.my_game"))
        .add_systems(Startup, spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), SettingsCamera));
}
```

The string passed to `SettingsPlusPlugins::new` names your game's settings
directory. Cameras tagged with `SettingsCamera` follow `RenderSettings`; they
can be spawned at any time.

Change a setting by editing its resource, then save it with the
`bevy_settings` commands:

```rust
fn apply_button(mut commands: Commands) {
    // Waits briefly so a burst of changes becomes one write.
    commands.queue(SaveSettingsDeferred::default());
}
```

To also save on exit, queue `SaveSettingsSync::IfChanged` just before exiting.

## Your own settings

Implement `CapsAware` on a settings resource to clamp it to the GPU:

```rust
use bevy::prelude::*;
use bevy_settings_plus::prelude::*;

#[derive(Resource, SettingsGroup, Reflect, Default, Clone, PartialEq)]
#[reflect(Resource, SettingsGroup, CapsAware, Default)]
#[settings_group(group = "my_game")]
struct MyGameQuality {
    ray_tracing: bool,
    shadow_distance: f32,
}

impl CapsAware for MyGameQuality {
    fn clamp_to(&mut self, caps: AdapterCaps) {
        if !caps.ray_query {
            self.ray_tracing = false;
        }
        self.shadow_distance = self.shadow_distance.clamp(10.0, 1000.0);
    }
}
```

Register it with `app.register_type::<MyGameQuality>()`. It is then loaded from
the settings file and clamped once at startup.

## Cargo features

- `dlss`: [DLSS](https://developer.nvidia.com/rtx/dlss) upscaling through
  `bevy_anti_alias`. Needs the NVIDIA DLSS SDK at build time.

## Bevy compatibility

| Bevy  | bevy_settings_plus |
| ----- | ------------------ |
| 0.20  | 0.1                |

The `main` branch follows Bevy's development branch and is not published.

## Examples

- [`basic`](examples/basic.rs): logs the detected GPU and the resolved settings.
- [`deferred_camera`](examples/deferred_camera.rs): spawns the camera after a
  state transition.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
