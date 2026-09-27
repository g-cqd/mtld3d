//! Texture metadata, level identities, and CPU lock forwarding.

use core::{ffi::c_void, mem::MaybeUninit, ptr};

use mtld3d_d3d8_types::{D3DSURFACE_DESC8, D3DVOLUME_DESC8};
use mtld3d_shared::OutPtr;
use mtld3d_types::{
    D3D_OK, D3DERR_INVALIDCALL, D3DLOCKED_BOX, D3DLOCKED_RECT, D3DSURFACE_DESC, D3DVOLUME_DESC,
};

use super::{ResourceBackend, api_scope, container::ContainerRef, object, surfaces};
use crate::{backend::Backend, device};

pub extern "system" fn set_lod(this: *mut c_void, lod: u32) -> u32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().set_lod)(backend.pointer(), lod) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().set_lod)(backend.pointer(), lod) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().set_lod)(backend.pointer(), lod) }
        }
        _ => {
            mtld3d_shared::log_once_warn!(target: "mtld3d::d3d8", "texture entry called on a non-texture wrapper");
            0
        }
    }
}

pub extern "system" fn get_lod(this: *mut c_void) -> u32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().get_lod)(backend.pointer()) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().get_lod)(backend.pointer()) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().get_lod)(backend.pointer()) }
        }
        _ => {
            mtld3d_shared::log_once_warn!(target: "mtld3d::d3d8", "texture entry called on a non-texture wrapper");
            0
        }
    }
}

pub extern "system" fn get_level_count(this: *mut c_void) -> u32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().get_level_count)(backend.pointer()) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().get_level_count)(backend.pointer()) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and has the same scalar ABI.
            unsafe { (backend.table().get_level_count)(backend.pointer()) }
        }
        _ => {
            mtld3d_shared::log_once_warn!(target: "mtld3d::d3d8", "texture entry called on a non-texture wrapper");
            0
        }
    }
}

pub extern "system" fn texture_get_level_desc(
    this: *mut c_void,
    level: u32,
    output: *mut D3DSURFACE_DESC8,
) -> i32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable descriptor storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::Texture(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    let mut description = MaybeUninit::<D3DSURFACE_DESC>::uninit();
    // SAFETY: the backend receives writable storage for its full descriptor layout.
    let status = unsafe {
        (backend.table().get_level_desc)(backend.pointer(), level, description.as_mut_ptr())
    };
    if status < 0 {
        return status;
    }
    // SAFETY: successful GetLevelDesc initialized the full descriptor.
    let description = unsafe { description.assume_init() };
    let Some(description) = surfaces::descriptor(&description) else {
        return D3DERR_INVALIDCALL;
    };
    output.write(description);
    D3D_OK
}

pub extern "system" fn cube_get_level_desc(
    this: *mut c_void,
    level: u32,
    output: *mut D3DSURFACE_DESC8,
) -> i32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable descriptor storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::CubeTexture(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    let mut description = MaybeUninit::<D3DSURFACE_DESC>::uninit();
    // SAFETY: the backend receives writable storage for its full descriptor layout.
    let status = unsafe {
        (backend.table().get_level_desc)(backend.pointer(), level, description.as_mut_ptr().cast())
    };
    if status < 0 {
        return status;
    }
    // SAFETY: successful GetLevelDesc initialized the full descriptor.
    let description = unsafe { description.assume_init() };
    let Some(description) = surfaces::descriptor(&description) else {
        return D3DERR_INVALIDCALL;
    };
    output.write(description);
    D3D_OK
}

pub extern "system" fn volume_texture_get_level_desc(
    this: *mut c_void,
    level: u32,
    output: *mut D3DVOLUME_DESC8,
) -> i32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable descriptor storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::VolumeTexture(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    let mut description = MaybeUninit::<D3DVOLUME_DESC>::uninit();
    // SAFETY: the backend receives writable storage for its full descriptor layout.
    let status = unsafe {
        (backend.table().get_level_desc)(backend.pointer(), level, description.as_mut_ptr().cast())
    };
    if status < 0 {
        return status;
    }
    // SAFETY: successful GetLevelDesc initialized the full descriptor.
    let description = unsafe { description.assume_init() };
    let Some(description) = surfaces::volume_descriptor(&description) else {
        return D3DERR_INVALIDCALL;
    };
    output.write(description);
    D3D_OK
}

pub extern "system" fn texture_get_surface_level(
    this: *mut c_void,
    level: u32,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::Texture(backend) = &resource.inner.backend else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    let mut level_interface = ptr::null_mut();
    // SAFETY: the backend receives valid scalar arguments and writable interface output storage.
    let status = unsafe {
        (backend.table().get_surface_level)(backend.pointer(), level, &raw mut level_interface)
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: the successful typed getter returns one owned reference for this resource kind.
    let Some(level) = (unsafe { Backend::adopt(level_interface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the texture wrapper owns this device reference.
    let device = unsafe { device::object(resource.inner.device) };
    let container = ContainerRef::retain(&resource);
    match device.resources().wrap_with_container(
        &device,
        ResourceBackend::Surface(level),
        Some(container),
    ) {
        Ok(pointer) => {
            output.write(pointer);
            D3D_OK
        }
        Err(status) => {
            output.write(ptr::null_mut());
            status
        }
    }
}

pub extern "system" fn cube_get_cube_map_surface(
    this: *mut c_void,
    face: u32,
    level: u32,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::CubeTexture(backend) = &resource.inner.backend else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    let mut level_interface = ptr::null_mut();
    // SAFETY: the backend receives valid scalar arguments and writable interface output storage.
    let status = unsafe {
        (backend.table().get_cube_map_surface)(
            backend.pointer(),
            face,
            level,
            &raw mut level_interface,
        )
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: the successful typed getter returns one owned reference for this resource kind.
    let Some(level) = (unsafe { Backend::adopt(level_interface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the texture wrapper owns this device reference.
    let device = unsafe { device::object(resource.inner.device) };
    let container = ContainerRef::retain(&resource);
    match device.resources().wrap_with_container(
        &device,
        ResourceBackend::Surface(level),
        Some(container),
    ) {
        Ok(pointer) => {
            output.write(pointer);
            D3D_OK
        }
        Err(status) => {
            output.write(ptr::null_mut());
            status
        }
    }
}

pub extern "system" fn volume_texture_get_volume_level(
    this: *mut c_void,
    level: u32,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: this texture owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::VolumeTexture(backend) = &resource.inner.backend else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    let mut level_interface = ptr::null_mut();
    // SAFETY: the backend receives valid scalar arguments and writable interface output storage.
    let status = unsafe {
        (backend.table().get_volume_level)(backend.pointer(), level, &raw mut level_interface)
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: the successful typed getter returns one owned reference for this resource kind.
    let Some(level) = (unsafe { Backend::adopt(level_interface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the texture wrapper owns this device reference.
    let device = unsafe { device::object(resource.inner.device) };
    let container = ContainerRef::retain(&resource);
    match device.resources().wrap_with_container(
        &device,
        ResourceBackend::Volume(level),
        Some(container),
    ) {
        Ok(pointer) => {
            output.write(pointer);
            D3D_OK
        }
        Err(status) => {
            output.write(ptr::null_mut());
            status
        }
    }
}

macro_rules! texture_method {
    ($name:ident, $variant:ident, $method:ident, ($($argument:ident : $kind:ty),*)) => {
        pub extern "system" fn $name(this: *mut c_void, $($argument: $kind),*) -> i32 {
            // SAFETY: this resource owns its parent device for the call.
            let _api = unsafe { api_scope(this) };
            // SAFETY: this typed vtable dispatches a live resource wrapper.
            let resource = unsafe { object(this) };
            let ResourceBackend::$variant(backend) = &resource.inner.backend else { return D3DERR_INVALIDCALL; };
            // SAFETY: D3D8 and D3D9 use identical lock and dirty-region layouts.
            unsafe { (backend.table().$method)(backend.pointer(), $($argument),*) }
        }
    };
}
texture_method!(texture_lock_rect, Texture, lock_rect, (level: u32, output: *mut D3DLOCKED_RECT, rectangle: *const c_void, flags: u32));
texture_method!(texture_unlock_rect, Texture, unlock_rect, (level: u32));
texture_method!(texture_add_dirty_rect, Texture, add_dirty_rect, (rectangle: *const c_void));
texture_method!(cube_lock_rect, CubeTexture, lock_rect, (face: u32, level: u32, output: *mut D3DLOCKED_RECT, rectangle: *const c_void, flags: u32));
texture_method!(cube_unlock_rect, CubeTexture, unlock_rect, (face: u32, level: u32));
texture_method!(cube_add_dirty_rect, CubeTexture, add_dirty_rect, (face: u32, rectangle: *const c_void));
texture_method!(volume_texture_unlock_box, VolumeTexture, unlock_box, (level: u32));

pub extern "system" fn volume_texture_lock_box(
    this: *mut c_void,
    level: u32,
    output: *mut D3DLOCKED_BOX,
    region: *const mtld3d_types::D3DBOX,
    flags: u32,
) -> i32 {
    // SAFETY: this resource owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::VolumeTexture(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the converted pointer types have identical D3D8 and D3D9 memory layouts.
    unsafe { (backend.table().lock_box)(backend.pointer(), level, output, region.cast(), flags) }
}

pub extern "system" fn volume_texture_add_dirty_box(
    this: *mut c_void,
    region: *const mtld3d_types::D3DBOX,
) -> i32 {
    // SAFETY: this resource owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::VolumeTexture(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the converted pointer types have identical D3D8 and D3D9 memory layouts.
    unsafe { (backend.table().add_dirty_box)(backend.pointer(), region.cast()) }
}
