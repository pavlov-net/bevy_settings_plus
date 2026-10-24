//! Clamping settings to what the GPU supports.
//!
//! Implement [`CapsAware`] on a settings resource and add `CapsAware` to its
//! `#[reflect(...)]` list. [`CapsAwarePlugin`] then detects [`AdapterCaps`] and
//! clamps every such resource once, after settings are loaded from disk.
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_settings::SettingsGroup;
//! use bevy_settings_plus::prelude::*;
//!
//! #[derive(Resource, SettingsGroup, Reflect, Default, Clone, PartialEq)]
//! #[reflect(Resource, SettingsGroup, CapsAware, Default)]
//! struct MyGameQuality {
//!     ray_tracing: bool,
//! }
//!
//! impl CapsAware for MyGameQuality {
//!     fn clamp_to(&mut self, caps: AdapterCaps) {
//!         if !caps.ray_query { self.ray_tracing = false; }
//!     }
//! }
//! ```
//!
//! Register the type with `app.register_type::<MyGameQuality>()`. Adding
//! [`CapsAwarePlugin`] is only needed if you don't use
//! [`SettingsPlusPlugins`](crate::SettingsPlusPlugins).

use bevy_app::{App, Plugin};
use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::component::Mutable;
use bevy_ecs::reflect::AppTypeRegistry;
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use bevy_log::warn;
use bevy_reflect::{CreateTypeData, FromReflect, Reflect, TypePath, TypeRegistration};

use crate::caps::{AdapterCaps, detect_caps};

/// A settings resource that is clamped to the GPU's capabilities.
///
/// Add `CapsAware` to the type's `#[reflect(...)]` list so [`CapsAwarePlugin`]
/// finds it.
///
/// `Clone + PartialEq` let the resource be marked changed only when clamping
/// alters it. Otherwise the next save would write the clamped values over the
/// user's choices.
pub trait CapsAware:
    Resource<Mutability = Mutable> + Reflect + FromReflect + TypePath + Clone + PartialEq
{
    /// Adjusts `self` to fit `caps`. Called once, after settings are loaded.
    fn clamp_to(&mut self, caps: AdapterCaps);
}

/// Reflection data for [`CapsAware`], installed by adding `CapsAware` to a
/// type's `#[reflect(...)]` list.
#[derive(Clone)]
pub struct ReflectCapsAware {
    clamp_in_world: fn(&mut World, AdapterCaps),
}

impl<T: CapsAware> CreateTypeData<T> for ReflectCapsAware {
    fn create_type_data(_input: ()) -> Self {
        Self {
            clamp_in_world: |world, caps| {
                let Some(mut resource) = world.get_resource_mut::<T>() else {
                    return;
                };
                // Reading through `Deref` doesn't mark the resource changed, and
                // `set_if_neq` only does so when the clamp actually changed it.
                let mut clamped = (*resource).clone();
                clamped.clamp_to(caps);
                resource.set_if_neq(clamped);
            },
        }
    }

    // A `CapsAware` type is always a resource.
    fn insert_dependencies(type_registration: &mut TypeRegistration) {
        type_registration.register_type_data::<ReflectResource, T>();
    }
}

/// Detects [`AdapterCaps`] and clamps every [`CapsAware`] resource, once, in
/// [`Plugin::finish`].
///
/// Add it after `bevy_render::RenderPlugin` (so the render app exists) and
/// after `bevy_settings::SettingsPlugin` (so settings are loaded).
/// [`crate::SettingsPlusPlugins`] does this for you.
///
/// Without a render app (headless), nothing is detected or clamped.
pub struct CapsAwarePlugin;

impl Plugin for CapsAwarePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AdapterCaps>();
    }

    fn finish(&self, app: &mut App) {
        let Some(caps) = detect_caps(app) else {
            warn!("AdapterCaps not detected (no RenderApp); skipping clamp pass.");
            return;
        };
        app.insert_resource(caps);

        let registry = app.world().resource::<AppTypeRegistry>().clone();
        let types = registry.read();
        let world = app.world_mut();

        for ty in types.iter() {
            if let Some(rca) = ty.data::<ReflectCapsAware>() {
                (rca.clamp_in_world)(world, caps);
            }
        }
    }
}
