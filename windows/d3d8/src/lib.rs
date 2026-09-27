//! Direct3D 8 frontend backed by mtld3d's shared D3D9 implementation.

use core::{ffi::c_void, ptr};

use mtld3d_d3d8_types::D3DSDK_VERSION8;

mod api_scope;
mod backend;
mod backend_unknown;
mod device;
mod direct3d;
mod identity;
mod resource;
mod surface;
mod swapchain;

const D3DSDK_VERSION8_0: u32 = 120;

#[cfg_attr(
    target_arch = "x86",
    link(name = "d3d9", kind = "raw-dylib", import_name_type = "undecorated")
)]
#[cfg_attr(not(target_arch = "x86"), link(name = "d3d9", kind = "raw-dylib"))]
unsafe extern "system" {
    fn Direct3DCreate9(sdk_version: u32) -> *mut c_void;
}

/// Creates an `IDirect3D8` interface with one owned reference, or null for an unsupported SDK.
///
/// Accepts the Direct3D 8.0 and 8.1 SDK versions.
#[unsafe(no_mangle)]
pub extern "system" fn Direct3DCreate8(sdk_version: u32) -> *mut c_void {
    if !matches!(sdk_version, D3DSDK_VERSION8_0 | D3DSDK_VERSION8) {
        return ptr::null_mut();
    }
    // SAFETY: the private backend's factory accepts the fixed D3D9 SDK version.
    let backend = unsafe { Direct3DCreate9(mtld3d_types::D3DSDK_VERSION) };
    // SAFETY: the factory returns null or one owned IDirect3D9 reference.
    let Some(backend) = (unsafe { backend::Backend::adopt(backend) }) else {
        return ptr::null_mut();
    };
    log::debug!(target: "mtld3d::d3d8", "d3d8.dll {} created", mtld3d_shared::identity::BUILD);
    direct3d::Direct3D8::create(backend)
}
