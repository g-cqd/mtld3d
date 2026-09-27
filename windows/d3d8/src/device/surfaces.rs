//! Surface identity and render-target state at the D3D8 boundary.

use core::{ffi::c_void, mem::MaybeUninit, ptr};

use mtld3d_shared::OutPtr;
use mtld3d_types::{
    D3D_OK, D3DERR_INVALIDCALL, D3DSURFACE_DESC, IDirect3DSurface9Vtbl, IDirect3DSwapChain9Vtbl,
    IID_IDIRECT3DCUBETEXTURE9, IID_IDIRECT3DSWAPCHAIN9, IID_IDIRECT3DTEXTURE9,
};

use super::{Device8, api_scope, object, resources::surface_input};
use crate::{
    backend::{self, Backend},
    resource::{self, ContainerRef, ResourceBackend},
    surface::Surface8,
};

/// Adopts a backend surface while preserving its frontend identity and container.
pub fn wrap_surface(
    device: &Device8,
    surface: Backend<IDirect3DSurface9Vtbl>,
    device_container: bool,
) -> Result<*mut c_void, i32> {
    let identity = surface.identity()?;
    let mut back_buffer = ptr::null_mut();
    // SAFETY: the device owns its backend; the output slot receives an owned surface reference.
    let status = unsafe {
        (device.backend().table().get_back_buffer)(
            device.backend().pointer(),
            0,
            0,
            0,
            &raw mut back_buffer,
        )
    };
    if status >= 0 {
        // SAFETY: successful GetBackBuffer returned this exact interface with an owned reference.
        if let Some(back_buffer) = unsafe { Backend::<IDirect3DSurface9Vtbl>::adopt(back_buffer) }
            && back_buffer.identity()? == identity
        {
            let mut cached = device.back_buffer();
            let wrapper = cached.get_or_insert_with(|| {
                // SAFETY: this device owns the inactive surface shell for its complete lifetime.
                Box::new(unsafe { Surface8::new(ptr::from_ref(device).cast_mut().cast()) })
            });
            let pointer = wrapper.acquire(surface);
            drop(cached);
            return Ok(pointer);
        }
    }
    let container = if device_container || device.is_implicit_depth(identity) {
        Some(ContainerRef::device(device))
    } else {
        surface_container(device, &surface)?
    };
    device
        .resources()
        .wrap_with_container(device, ResourceBackend::Surface(surface), container)
}

fn surface_container(
    device: &Device8,
    surface: &Backend<IDirect3DSurface9Vtbl>,
) -> Result<Option<ContainerRef>, i32> {
    let mut pointer = ptr::null_mut();
    // SAFETY: the queried IID names a texture interface and pointer is writable local output.
    let status = unsafe {
        (surface.table().get_container)(surface.pointer(), &IID_IDIRECT3DTEXTURE9, &raw mut pointer)
    };
    if status >= 0 {
        // SAFETY: the successful query returns one owned interface of the requested texture type.
        let texture = unsafe { Backend::adopt(pointer) }.ok_or(D3DERR_INVALIDCALL)?;
        return resource::retain_container(device, ResourceBackend::Texture(texture)).map(Some);
    }
    // SAFETY: the queried IID names a cube interface and pointer is writable local output.
    let status = unsafe {
        (surface.table().get_container)(
            surface.pointer(),
            &IID_IDIRECT3DCUBETEXTURE9,
            &raw mut pointer,
        )
    };
    if status >= 0 {
        // SAFETY: the successful query returns one owned interface of the requested cube type.
        let texture = unsafe { Backend::adopt(pointer) }.ok_or(D3DERR_INVALIDCALL)?;
        return resource::retain_container(device, ResourceBackend::CubeTexture(texture)).map(Some);
    }
    // SAFETY: the requested IID is the backend swap-chain container interface.
    let status = unsafe {
        (surface.table().get_container)(
            surface.pointer(),
            &IID_IDIRECT3DSWAPCHAIN9,
            &raw mut pointer,
        )
    };
    if status >= 0 {
        // SAFETY: successful GetContainer returned one owned reference with the requested vtable.
        let chain = unsafe { Backend::<IDirect3DSwapChain9Vtbl>::adopt(pointer) }
            .ok_or(D3DERR_INVALIDCALL)?;
        drop(chain);
        return Ok(Some(ContainerRef::device(device)));
    }
    Ok(None)
}

/// Reads the initial implicit depth identity without retaining an extra backend reference.
pub fn implicit_depth_identity(device: &Backend<mtld3d_types::IDirect3DDevice9Vtbl>) -> usize {
    let mut pointer = ptr::null_mut();
    // SAFETY: the backend owns the device; pointer is a writable local output slot.
    let status =
        unsafe { (device.table().get_depth_stencil_surface)(device.pointer(), &raw mut pointer) };
    if status < 0 {
        return 0;
    }
    // SAFETY: the successful getter returns one owned surface reference, or null if absent.
    let Some(surface) = (unsafe { Backend::<IDirect3DSurface9Vtbl>::adopt(pointer) }) else {
        return 0;
    };
    surface.identity().unwrap_or(0)
}

pub extern "system" fn get_render_target(this: *mut c_void, output: *mut *mut c_void) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    let mut surface = ptr::null_mut();
    // SAFETY: the backend owns this device and the output slot is writable.
    let status = unsafe {
        (device.backend().table().get_render_target)(
            device.backend().pointer(),
            0,
            &raw mut surface,
        )
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: a successful getter returns one owned surface interface.
    let Some(surface) = (unsafe { Backend::adopt(surface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    write_surface(&device, surface, output)
}

pub extern "system" fn get_depth_stencil_surface(
    this: *mut c_void,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    let mut surface = ptr::null_mut();
    // SAFETY: the backend owns this device and the output slot is writable.
    let status = unsafe {
        (device.backend().table().get_depth_stencil_surface)(
            device.backend().pointer(),
            &raw mut surface,
        )
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: a successful getter returns one owned surface interface.
    let Some(surface) = (unsafe { Backend::adopt(surface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    write_surface(&device, surface, output)
}

fn write_surface(
    device: &Device8,
    surface: Backend<IDirect3DSurface9Vtbl>,
    output: OutPtr<'_, *mut c_void>,
) -> i32 {
    match wrap_surface(device, surface, false) {
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

/// Reads a borrowed backend surface descriptor.
///
/// # Safety
/// The pointer must name a live backend surface throughout this call.
unsafe fn description(surface: *mut c_void) -> Result<D3DSURFACE_DESC, i32> {
    // SAFETY: the caller guarantees this backend surface's type and lifetime.
    let table = unsafe { backend::table::<IDirect3DSurface9Vtbl>(surface) };
    let mut description = MaybeUninit::uninit();
    // SAFETY: the live typed backend writes the complete descriptor to local storage.
    let status = unsafe { (table.get_desc)(surface, description.as_mut_ptr()) };
    if status < 0 {
        return Err(status);
    }
    // SAFETY: successful GetDesc initialized the full descriptor.
    Ok(unsafe { description.assume_init() })
}

pub extern "system" fn set_render_target(
    this: *mut c_void,
    target: *mut c_void,
    depth: *mut c_void,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller holds the optional frontend surface for this call.
    let target = match unsafe { surface_input(target, &device) } {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: the caller holds the optional frontend depth surface for this call.
    let depth = match unsafe { surface_input(depth, &device) } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let backend = device.backend();
    if !depth.is_null() {
        let mut effective_target = ptr::null_mut();
        // SAFETY: the backend owns this device and writes one owned surface into local storage.
        let status = unsafe {
            (backend.table().get_render_target)(backend.pointer(), 0, &raw mut effective_target)
        };
        if status < 0 {
            return status;
        }
        // SAFETY: the successful getter supplied one owned render-target reference.
        let effective_target = unsafe { Backend::<IDirect3DSurface9Vtbl>::adopt(effective_target) };
        let target = if target.is_null() {
            effective_target
                .as_ref()
                .map_or(ptr::null_mut(), Backend::pointer)
        } else {
            target
        };
        if target.is_null() {
            return D3DERR_INVALIDCALL;
        }
        // SAFETY: either the caller or effective_target owns the validated backend surface.
        let target_desc = match unsafe { description(target) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        // SAFETY: the caller owns the validated backend depth surface for this call.
        let depth_desc = match unsafe { description(depth) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        if depth_desc.width < target_desc.width
            || depth_desc.height < target_desc.height
            || depth_desc.multi_sample_type != target_desc.multi_sample_type
            || depth_desc.multi_sample_quality != target_desc.multi_sample_quality
        {
            return D3DERR_INVALIDCALL;
        }
    }
    let mut old_depth = ptr::null_mut();
    // SAFETY: the device is live and old_depth is writable local output.
    let old_status = unsafe {
        (backend.table().get_depth_stencil_surface)(backend.pointer(), &raw mut old_depth)
    };
    if old_status < 0 && old_status != mtld3d_types::D3DERR_NOTFOUND {
        return old_status;
    }
    // SAFETY: successful GetDepthStencilSurface returns an owned reference; no surface is null.
    let old_depth = unsafe { Backend::<IDirect3DSurface9Vtbl>::adopt(old_depth) };
    // SAFETY: the validated optional depth surface belongs to this device and remains live.
    let status = unsafe { (backend.table().set_depth_stencil_surface)(backend.pointer(), depth) };
    if status < 0 || target.is_null() {
        return status;
    }
    // SAFETY: the validated target belongs to this device and remains live through the call.
    let status = unsafe { (backend.table().set_render_target)(backend.pointer(), 0, target) };
    if status < 0 {
        // SAFETY: old_depth retains the exact previous binding until the transaction ends.
        let restored = unsafe {
            (backend.table().set_depth_stencil_surface)(
                backend.pointer(),
                old_depth.as_ref().map_or(ptr::null_mut(), Backend::pointer),
            )
        };
        if restored < 0 {
            log::error!("D3D8 SetRenderTarget failed to restore depth binding: {restored:#x}");
        }
    }
    status
}

pub extern "system" fn get_front_buffer(this: *mut c_void, destination: *mut c_void) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller holds this destination surface for the call.
    let destination = match unsafe { surface_input(destination, &device) } {
        Ok(value) if !value.is_null() => value,
        _ => return D3DERR_INVALIDCALL,
    };
    // SAFETY: the validated destination belongs to the backend device and is live.
    unsafe {
        (device.backend().table().get_front_buffer_data)(device.backend().pointer(), 0, destination)
    }
}
