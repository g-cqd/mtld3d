//! Surface and volume descriptors, locking, and frontend container identities.

use core::{ffi::c_void, mem::MaybeUninit, ptr};

use mtld3d_core::format::d3d8_surface_size;
use mtld3d_d3d8_types::{D3DSURFACE_DESC8, D3DVOLUME_DESC8};
use mtld3d_shared::{InPtr, OutPtr};
use mtld3d_types::{
    D3D_OK, D3DERR_INVALIDCALL, D3DLOCKED_BOX, D3DLOCKED_RECT, D3DSURFACE_DESC, D3DVOLUME_DESC,
    Guid,
};

use super::{ResourceBackend, api_scope, object};

pub fn descriptor(description: &D3DSURFACE_DESC) -> Option<D3DSURFACE_DESC8> {
    Some(D3DSURFACE_DESC8 {
        format: description.format,
        resource_type: description.resource_type,
        usage: description.usage,
        pool: description.pool,
        size: d3d8_surface_size(description.format, description.width, description.height)?,
        multi_sample_type: description.multi_sample_type,
        width: description.width,
        height: description.height,
    })
}

pub fn volume_descriptor(description: &D3DVOLUME_DESC) -> Option<D3DVOLUME_DESC8> {
    if description.depth == 0 {
        return None;
    }
    Some(D3DVOLUME_DESC8 {
        format: description.format,
        resource_type: description.resource_type,
        usage: description.usage,
        pool: description.pool,
        size: d3d8_surface_size(description.format, description.width, description.height)?
            .checked_mul(description.depth)?,
        width: description.width,
        height: description.height,
        depth: description.depth,
    })
}

pub extern "system" fn surface_get_desc(this: *mut c_void, output: *mut D3DSURFACE_DESC8) -> i32 {
    // SAFETY: this surface owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller provides writable descriptor storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this vtable dispatches a live Resource8 surface wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::Surface(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    let mut description = MaybeUninit::uninit();
    // SAFETY: the typed backend writes its full surface descriptor into local storage.
    let status = unsafe { (backend.table().get_desc)(backend.pointer(), description.as_mut_ptr()) };
    if status < 0 {
        return status;
    }
    // SAFETY: successful GetDesc initialized the complete descriptor.
    let description = unsafe { description.assume_init() };
    let Some(description) = descriptor(&description) else {
        return D3DERR_INVALIDCALL;
    };
    output.write(description);
    D3D_OK
}

pub extern "system" fn volume_get_desc(this: *mut c_void, output: *mut D3DVOLUME_DESC8) -> i32 {
    // SAFETY: this volume owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller provides writable descriptor storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this vtable dispatches a live Resource8 volume wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::Volume(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    let mut description = MaybeUninit::uninit();
    // SAFETY: the typed backend writes its full volume descriptor into local storage.
    let status = unsafe { (backend.table().get_desc)(backend.pointer(), description.as_mut_ptr()) };
    if status < 0 {
        return status;
    }
    // SAFETY: successful GetDesc initialized the complete descriptor.
    let description = unsafe { description.assume_init() };
    let Some(description) = volume_descriptor(&description) else {
        return D3DERR_INVALIDCALL;
    };
    output.write(description);
    D3D_OK
}

pub extern "system" fn get_container(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: this surface or volume owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller provides writable interface output storage or null.
    let Some(output_slot) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this vtable dispatches a live surface or volume wrapper.
    let resource = unsafe { object(this) };
    let Some(container) = &resource.inner.container else {
        output_slot.write(ptr::null_mut());
        return mtld3d_types::E_NOINTERFACE;
    };
    // SAFETY: the caller supplies a readable interface identifier or null.
    let Some(iid) = (unsafe { InPtr::<Guid>::opt(iid.cast()) }) else {
        output_slot.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    container.query(&iid, output_slot)
}

macro_rules! surface_method {
    ($name:ident, $variant:ident, $method:ident, ($($argument:ident : $kind:ty),*)) => {
        pub extern "system" fn $name(this: *mut c_void, $($argument: $kind),*) -> i32 {
            // SAFETY: this resource owns its parent device for the call.
            let _api = unsafe { api_scope(this) };
            // SAFETY: this vtable dispatches a live wrapper of the named resource kind.
            let resource = unsafe { object(this) };
            let ResourceBackend::$variant(backend) = &resource.inner.backend else { return D3DERR_INVALIDCALL; };
            // SAFETY: these lock arguments have identical layouts in D3D8 and D3D9.
            unsafe { (backend.table().$method)(backend.pointer(), $($argument),*) }
        }
    };
}
surface_method!(surface_lock_rect, Surface, lock_rect, (output: *mut D3DLOCKED_RECT, rectangle: *const c_void, flags: u32));
surface_method!(surface_unlock_rect, Surface, unlock_rect, ());
surface_method!(volume_unlock_box, Volume, unlock_box, ());

pub extern "system" fn volume_lock_box(
    this: *mut c_void,
    output: *mut D3DLOCKED_BOX,
    region: *const mtld3d_types::D3DBOX,
    flags: u32,
) -> i32 {
    // SAFETY: this resource owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::Volume(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the converted pointer types have identical D3D8 and D3D9 memory layouts.
    unsafe { (backend.table().lock_box)(backend.pointer(), output, region.cast(), flags) }
}
