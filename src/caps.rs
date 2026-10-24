//! GPU capabilities and platform detection.

use bevy_app::App;
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use bevy_render::RenderApp;
use bevy_render::renderer::{RenderAdapterInfo, RenderDevice};

/// Summary of the active GPU adapter.
///
/// Inserted as a resource by [`crate::caps_aware::CapsAwarePlugin`].
#[derive(Resource, Reflect, Debug, Clone, Copy)]
#[reflect(Resource)]
pub struct AdapterCaps {
    /// Graphics backend in use.
    pub backend: Backend,
    /// GPU vendor.
    pub vendor: GpuVendor,
    /// Largest supported 2D texture dimension.
    pub max_texture_dim_2d: u32,
    /// Ray queries are supported.
    pub ray_query: bool,
    /// Timestamp queries (GPU profiling) are supported.
    pub timestamp_query: bool,
    /// The adapter can run DLSS (Vulkan on an NVIDIA GPU). The `dlss` cargo
    /// feature must also be enabled for DLSS to be applied.
    pub supports_dlss: bool,
}

/// Graphics backend.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Backend {
    Vulkan,
    Dx12,
    Metal,
    Gl,
    BrowserWebGpu,
    Other,
}

impl Backend {
    fn from_wgpu(b: wgpu_types::Backend) -> Self {
        match b {
            wgpu_types::Backend::Vulkan => Self::Vulkan,
            wgpu_types::Backend::Dx12 => Self::Dx12,
            wgpu_types::Backend::Metal => Self::Metal,
            wgpu_types::Backend::Gl => Self::Gl,
            wgpu_types::Backend::BrowserWebGpu => Self::BrowserWebGpu,
            _ => Self::Other,
        }
    }
}

/// GPU vendor, from the PCI vendor id.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    Arm,
    Other,
}

impl GpuVendor {
    fn from_pci_id(vendor: u32) -> Self {
        match vendor {
            0x10DE => Self::Nvidia,
            0x1002 | 0x1022 => Self::Amd,
            0x8086 => Self::Intel,
            0x106B => Self::Apple,
            0x5143 => Self::Qualcomm,
            0x13B5 => Self::Arm,
            _ => Self::Other,
        }
    }
}

/// Where the application is running.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlatformTarget {
    Wasm,
    WindowsDesktop,
    LinuxDesktop,
    MacDesktop,
    Other,
}

impl PlatformTarget {
    /// The platform this build targets.
    pub const fn detect() -> Self {
        if cfg!(target_family = "wasm") {
            Self::Wasm
        } else if cfg!(target_os = "windows") {
            Self::WindowsDesktop
        } else if cfg!(target_os = "linux") {
            Self::LinuxDesktop
        } else if cfg!(target_os = "macos") {
            Self::MacDesktop
        } else {
            Self::Other
        }
    }
}

/// Reads the adapter from the [`RenderApp`] sub-app. Returns `None` when there
/// is no render app (headless).
///
/// Call it from [`bevy_app::Plugin::finish`]; the render app is not ready
/// during `build`.
pub fn detect_caps(app: &mut App) -> Option<AdapterCaps> {
    let render_app = app.get_sub_app(RenderApp)?;
    let info = render_app.world().get_resource::<RenderAdapterInfo>()?;
    let device = render_app.world().get_resource::<RenderDevice>()?;

    let features = device.features();
    let limits = device.limits();

    let backend = Backend::from_wgpu(info.backend);
    let vendor = GpuVendor::from_pci_id(info.vendor);

    Some(AdapterCaps {
        backend,
        vendor,
        max_texture_dim_2d: limits.max_texture_dimension_2d,
        ray_query: features.contains(wgpu_types::Features::EXPERIMENTAL_RAY_QUERY),
        timestamp_query: features.contains(wgpu_types::Features::TIMESTAMP_QUERY),
        supports_dlss: matches!((backend, vendor), (Backend::Vulkan, GpuVendor::Nvidia)),
    })
}
