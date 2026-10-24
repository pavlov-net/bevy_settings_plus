//! Caps clamping for each settings type. Saving and loading is tested in
//! `bevy_settings`.

use bevy_settings_plus::prelude::*;

fn nvidia_vulkan_caps() -> AdapterCaps {
    AdapterCaps {
        backend: Backend::Vulkan,
        vendor: GpuVendor::Nvidia,
        max_texture_dim_2d: 16384,
        ray_query: true,
        timestamp_query: true,
        supports_dlss: true,
    }
}

fn integrated_intel_caps() -> AdapterCaps {
    AdapterCaps {
        backend: Backend::Vulkan,
        vendor: GpuVendor::Intel,
        max_texture_dim_2d: 8192,
        ray_query: false,
        timestamp_query: true,
        supports_dlss: false,
    }
}

#[test]
fn render_platform_default_wasm_is_conservative() {
    let r = RenderSettings::platform_default(PlatformTarget::Wasm);
    assert!(matches!(r.anti_alias, AntiAlias::None));
    assert!(matches!(r.upscaler, Upscaler::Native));
    assert!(matches!(r.msaa, MsaaLevel::Off));
    assert!(!r.ray_tracing);
}

#[test]
fn render_platform_default_desktop_uses_taa() {
    let r = RenderSettings::platform_default(PlatformTarget::LinuxDesktop);
    assert!(matches!(r.anti_alias, AntiAlias::Taa));
}

#[test]
fn render_clamp_strips_ray_tracing_when_caps_lack_ray_query() {
    let mut r = RenderSettings::platform_default(PlatformTarget::LinuxDesktop);
    r.ray_tracing = true;
    r.clamp_to(integrated_intel_caps());
    assert!(!r.ray_tracing);
}

#[test]
fn render_clamp_keeps_ray_tracing_when_caps_support_it() {
    let mut r = RenderSettings::platform_default(PlatformTarget::LinuxDesktop);
    r.ray_tracing = true;
    r.clamp_to(nvidia_vulkan_caps());
    assert!(r.ray_tracing);
}

#[test]
fn render_clamp_falls_back_to_native_when_dlss_unavailable() {
    let mut r = RenderSettings::platform_default(PlatformTarget::LinuxDesktop);
    r.upscaler = Upscaler::Dlss(UpscalerPreset::Quality);
    r.clamp_to(integrated_intel_caps());
    assert!(matches!(r.upscaler, Upscaler::Native));
}

#[test]
fn render_clamp_keeps_dlss_when_supported() {
    let mut r = RenderSettings::platform_default(PlatformTarget::LinuxDesktop);
    r.upscaler = Upscaler::Dlss(UpscalerPreset::Balanced);
    r.clamp_to(nvidia_vulkan_caps());
    assert!(matches!(r.upscaler, Upscaler::Dlss(_)));
}

#[test]
fn render_clamp_clamps_render_scale_and_sharpness() {
    let mut r = RenderSettings::platform_default(PlatformTarget::LinuxDesktop);
    r.render_scale = 1.5;
    r.sharpness = -0.2;
    r.clamp_to(nvidia_vulkan_caps());
    assert!((0.25..=1.0).contains(&r.render_scale));
    assert!((0.0..=1.0).contains(&r.sharpness));
}

#[test]
fn accessibility_clamp_clamps_subtitle_scale() {
    let mut a = AccessibilitySettings::platform_default(PlatformTarget::LinuxDesktop);
    a.subtitle_scale = 5.0;
    a.clamp_to(nvidia_vulkan_caps());
    assert!((0.5..=3.0).contains(&a.subtitle_scale));
}

#[test]
fn default_resolves_to_platform_default() {
    let d = DisplaySettings::default();
    let p = DisplaySettings::platform_default(PlatformTarget::detect());
    assert_eq!(d, p);
}
