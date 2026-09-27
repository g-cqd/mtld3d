//! D3D8 resource creation and binding through the shared D3D9 backend.

use core::{ffi::c_void, ptr};

use mtld3d_shared::OutPtr;
use mtld3d_types::{D3D_OK, D3DERR_INVALIDCALL};

use super::{Device8, api_scope, object};
use crate::{
    backend::Backend,
    resource::{self, ResourceBackend},
};

macro_rules! create_resource {
    ($name:ident, $variant:ident, ($($argument:ident : $kind:ty),*)) => {
        pub extern "system" fn $name(this: *mut c_void, $($argument: $kind,)* output: *mut *mut c_void) -> i32 {
            // SAFETY: this COM receiver remains a live Device8 for the call.
            let _api = unsafe { api_scope(this) };
            // SAFETY: the caller supplies writable interface output storage or null.
            let Some(output) = (unsafe { OutPtr::opt(output) }) else { return D3DERR_INVALIDCALL; };
            // SAFETY: this typed vtable dispatches a live Device8 receiver.
            let device = unsafe { object(this) };
            let mut backend = ptr::null_mut();
            // SAFETY: scalar parameters retain their ABI; D3D8 has no shared-handle output.
            let status = unsafe { (device.backend().table().$name)(device.backend().pointer(), $($argument,)* &raw mut backend, ptr::null_mut()) };
            if status < 0 { output.write(ptr::null_mut()); return status; }
            // SAFETY: the successful typed factory returns one owned reference of this exact kind.
            let Some(backend) = (unsafe { Backend::adopt(backend) }) else { output.write(ptr::null_mut()); return D3DERR_INVALIDCALL; };
            write_resource(&device, ResourceBackend::$variant(backend), output)
        }
    };
}
create_resource!(create_texture, Texture, (width: u32, height: u32, levels: u32, usage: u32, format: u32, pool: u32));
create_resource!(create_cube_texture, CubeTexture, (edge: u32, levels: u32, usage: u32, format: u32, pool: u32));
create_resource!(create_volume_texture, VolumeTexture, (width: u32, height: u32, depth: u32, levels: u32, usage: u32, format: u32, pool: u32));
create_resource!(create_vertex_buffer, VertexBuffer, (length: u32, usage: u32, fvf: u32, pool: u32));
create_resource!(create_index_buffer, IndexBuffer, (length: u32, usage: u32, format: u32, pool: u32));

pub extern "system" fn set_texture(this: *mut c_void, stage: u32, texture: *mut c_void) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller supplies null or a live texture COM interface for this call.
    let texture = match unsafe { resource::input(texture, &device) } {
        Ok(texture) => texture,
        Err(status) => return status,
    };
    let pointer = match texture.as_ref().map(|texture| texture.backend()) {
        None => ptr::null_mut(),
        Some(ResourceBackend::Texture(backend)) => backend.pointer(),
        Some(ResourceBackend::CubeTexture(backend)) => backend.pointer(),
        Some(ResourceBackend::VolumeTexture(backend)) => backend.pointer(),
        _ => return D3DERR_INVALIDCALL,
    };
    // SAFETY: the validated optional backend belongs to this device and lives for the call.
    unsafe { (device.backend().table().set_texture)(device.backend().pointer(), stage, pointer) }
}

pub extern "system" fn get_texture(this: *mut c_void, stage: u32, output: *mut *mut c_void) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    let mut texture = ptr::null_mut();
    // SAFETY: texture names writable interface output storage.
    let status = unsafe {
        (device.backend().table().get_texture)(device.backend().pointer(), stage, &raw mut texture)
    };
    if status < 0 || texture.is_null() {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: all D3D9 base texture interfaces share the GetType prefix.
    let table =
        unsafe { crate::backend::table::<mtld3d_d3d8_types::IDirect3DResource8Vtbl>(texture) };
    // SAFETY: the acquired backend texture reference remains live through this call.
    let kind = unsafe { (table.get_type)(texture) };
    let resource = match kind {
        mtld3d_types::D3DRTYPE_TEXTURE => {
            // SAFETY: GetType identified the acquired interface as a 2D texture.
            let Some(backend) = (unsafe { Backend::adopt(texture) }) else {
                return D3DERR_INVALIDCALL;
            };
            ResourceBackend::Texture(backend)
        }
        mtld3d_types::D3DRTYPE_CUBETEXTURE => {
            // SAFETY: GetType identified the acquired interface as a cube texture.
            let Some(backend) = (unsafe { Backend::adopt(texture) }) else {
                return D3DERR_INVALIDCALL;
            };
            ResourceBackend::CubeTexture(backend)
        }
        mtld3d_types::D3DRTYPE_VOLUMETEXTURE => {
            // SAFETY: GetType identified the acquired interface as a volume texture.
            let Some(backend) = (unsafe { Backend::adopt(texture) }) else {
                return D3DERR_INVALIDCALL;
            };
            ResourceBackend::VolumeTexture(backend)
        }
        _ => {
            // SAFETY: the backend returned one owned reference even if its type is unexpected.
            unsafe { (table.release)(texture) };
            mtld3d_shared::log_once_warn!(target: "mtld3d::d3d8", "GetTexture returned a non-texture backend resource");
            output.write(ptr::null_mut());
            return D3DERR_INVALIDCALL;
        }
    };
    write_resource(&device, resource, output)
}

fn write_resource(
    device: &Device8,
    backend: ResourceBackend,
    output: OutPtr<'_, *mut c_void>,
) -> i32 {
    match device.resources().wrap(device, backend) {
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

pub extern "system" fn create_render_target(
    this: *mut c_void,
    width: u32,
    height: u32,
    format: u32,
    samples: u32,
    lockable: i32,
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
    // SAFETY: scalar arguments retain their ABI; D3D8 has no quality/shared-handle parameters.
    let status = unsafe {
        (device.backend().table().create_render_target)(
            device.backend().pointer(),
            width,
            height,
            format,
            samples,
            0,
            lockable,
            &raw mut surface,
            ptr::null_mut(),
        )
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: successful CreateRenderTarget returned an owned IDirect3DSurface9 reference.
    let Some(surface) = (unsafe { Backend::adopt(surface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    write_resource(&device, ResourceBackend::Surface(surface), output)
}

pub extern "system" fn create_depth_stencil_surface(
    this: *mut c_void,
    width: u32,
    height: u32,
    format: u32,
    samples: u32,
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
    // SAFETY: quality zero and preserve-depth match the D3D8 creation contract.
    let status = unsafe {
        (device.backend().table().create_depth_stencil_surface)(
            device.backend().pointer(),
            width,
            height,
            format,
            samples,
            0,
            0,
            &raw mut surface,
            ptr::null_mut(),
        )
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: successful creation returned an owned IDirect3DSurface9 reference.
    let Some(surface) = (unsafe { Backend::adopt(surface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    write_resource(&device, ResourceBackend::Surface(surface), output)
}

pub extern "system" fn create_image_surface(
    this: *mut c_void,
    width: u32,
    height: u32,
    format: u32,
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
    // SAFETY: D3D8 image surfaces use SYSTEMMEM and have no shared-handle output.
    let status = unsafe {
        (device.backend().table().create_offscreen_plain_surface)(
            device.backend().pointer(),
            width,
            height,
            format,
            mtld3d_types::D3DPOOL_SYSTEMMEM,
            &raw mut surface,
            ptr::null_mut(),
        )
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    // SAFETY: successful creation returned an owned IDirect3DSurface9 reference.
    let Some(surface) = (unsafe { Backend::adopt(surface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    write_resource(&device, ResourceBackend::Surface(surface), output)
}

/// Borrows a validated surface backend from either D3D8 surface wrapper family.
///
/// # Safety
/// A non-null pointer must be a live COM interface for the duration of the call.
pub unsafe fn surface_input(pointer: *mut c_void, device: &Device8) -> Result<*mut c_void, i32> {
    if pointer.is_null() {
        return Ok(ptr::null_mut());
    }
    // SAFETY: the caller guarantees a readable COM interface for this call.
    if let Some(backend) = unsafe { crate::surface::input_backend(pointer, device) } {
        return Ok(backend);
    }
    // SAFETY: the same COM pointer contract allows checked resource-wrapper recognition.
    let Some(resource) = unsafe { resource::input(pointer, device) }? else {
        return Ok(ptr::null_mut());
    };
    match resource.backend() {
        ResourceBackend::Surface(backend) => Ok(backend.pointer()),
        _ => Err(D3DERR_INVALIDCALL),
    }
}

pub extern "system" fn set_cursor_properties(
    this: *mut c_void,
    x: u32,
    y: u32,
    surface: *mut c_void,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller supplies a live surface interface or null.
    let surface = match unsafe { surface_input(surface, &device) } {
        Ok(surface) => surface,
        Err(status) => return status,
    };
    // SAFETY: the validated backend surface remains live through its caller-owned frontend reference.
    unsafe {
        (device.backend().table().set_cursor_properties)(device.backend().pointer(), x, y, surface)
    }
}

pub extern "system" fn set_stream_source(
    this: *mut c_void,
    stream: u32,
    buffer: *mut c_void,
    stride: u32,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller supplies null or a live vertex-buffer interface for this call.
    let buffer = match unsafe { resource::input(buffer, &device) } {
        Ok(buffer) => buffer,
        Err(status) => return status,
    };
    let pointer = match buffer.as_ref().map(|buffer| buffer.backend()) {
        None => ptr::null_mut(),
        Some(ResourceBackend::VertexBuffer(backend)) => backend.pointer(),
        _ => return D3DERR_INVALIDCALL,
    };
    // SAFETY: D3D8 has no byte offset; the validated buffer remains live through this call.
    unsafe {
        (device.backend().table().set_stream_source)(
            device.backend().pointer(),
            stream,
            pointer,
            0,
            stride,
        )
    }
}

pub extern "system" fn get_stream_source(
    this: *mut c_void,
    stream: u32,
    output: *mut *mut c_void,
    stride: *mut u32,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the caller supplies writable stride output storage or null.
    let Some(stride) = (unsafe { OutPtr::opt(stride) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    let mut buffer = ptr::null_mut();
    let mut offset = 0;
    let mut current_stride = 0;
    // SAFETY: all backend output slots are valid local storage.
    let status = unsafe {
        (device.backend().table().get_stream_source)(
            device.backend().pointer(),
            stream,
            &raw mut buffer,
            &raw mut offset,
            &raw mut current_stride,
        )
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    stride.write(current_stride);
    // SAFETY: successful GetStreamSource returns null or an owned vertex-buffer reference.
    let Some(buffer) = (unsafe { Backend::adopt(buffer) }) else {
        output.write(ptr::null_mut());
        return D3D_OK;
    };
    write_resource(&device, ResourceBackend::VertexBuffer(buffer), output)
}

pub extern "system" fn set_indices(
    this: *mut c_void,
    buffer: *mut c_void,
    base_vertex: u32,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller supplies null or a live index-buffer interface for this call.
    let buffer = match unsafe { resource::input(buffer, &device) } {
        Ok(buffer) => buffer,
        Err(status) => return status,
    };
    let pointer = match buffer.as_ref().map(|buffer| buffer.backend()) {
        None => ptr::null_mut(),
        Some(ResourceBackend::IndexBuffer(backend)) => backend.pointer(),
        _ => return D3DERR_INVALIDCALL,
    };
    // SAFETY: the validated backend belongs to this device and remains live for this call.
    let status =
        unsafe { (device.backend().table().set_indices)(device.backend().pointer(), pointer) };
    if status >= 0 {
        device.state().set_base_vertex(base_vertex);
    }
    status
}

pub extern "system" fn get_indices(
    this: *mut c_void,
    output: *mut *mut c_void,
    base_vertex: *mut u32,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the caller supplies writable base-vertex output storage or null.
    let Some(base_vertex) = (unsafe { OutPtr::opt(base_vertex) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    let mut buffer = ptr::null_mut();
    // SAFETY: buffer is writable interface output storage.
    let status = unsafe {
        (device.backend().table().get_indices)(device.backend().pointer(), &raw mut buffer)
    };
    if status < 0 {
        output.write(ptr::null_mut());
        return status;
    }
    base_vertex.write(device.state().base_vertex());
    // SAFETY: successful GetIndices returns null or an owned index-buffer reference.
    let Some(buffer) = (unsafe { Backend::adopt(buffer) }) else {
        output.write(ptr::null_mut());
        return D3D_OK;
    };
    write_resource(&device, ResourceBackend::IndexBuffer(buffer), output)
}

pub extern "system" fn draw_indexed_primitive(
    this: *mut c_void,
    kind: u32,
    minimum: u32,
    vertices: u32,
    start: u32,
    count: u32,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    let base_vertex = i32::from_ne_bytes(device.state().base_vertex().to_ne_bytes());
    // SAFETY: D3D8's stored base-vertex bits are the signed offset supplied per draw in D3D9.
    unsafe {
        (device.backend().table().draw_indexed_primitive)(
            device.backend().pointer(),
            kind,
            base_vertex,
            minimum,
            vertices,
            start,
            count,
        )
    }
}

pub extern "system" fn draw_indexed_primitive_up(
    this: *mut c_void,
    kind: u32,
    minimum: u32,
    vertices: u32,
    count: u32,
    indices: *const c_void,
    format: u32,
    data: *const c_void,
    stride: u32,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller supplies the same index and vertex data contract in D3D8 and D3D9.
    unsafe {
        (device.backend().table().draw_indexed_primitive_up)(
            device.backend().pointer(),
            kind,
            minimum,
            vertices,
            count,
            indices,
            format,
            data,
            stride,
        )
    }
}

pub extern "system" fn process_vertices(
    this: *mut c_void,
    source_start: u32,
    destination_index: u32,
    count: u32,
    destination: *mut c_void,
    flags: u32,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller supplies a live destination buffer interface for this call.
    let buffer = match unsafe { resource::input(destination, &device) } {
        Ok(Some(buffer)) => buffer,
        Ok(None) => return D3DERR_INVALIDCALL,
        Err(status) => return status,
    };
    let ResourceBackend::VertexBuffer(buffer) = buffer.backend() else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: a null declaration selects the destination buffer's FVF, matching D3D8.
    unsafe {
        (device.backend().table().process_vertices)(
            device.backend().pointer(),
            source_start,
            destination_index,
            count,
            buffer.pointer(),
            ptr::null_mut(),
            flags,
        )
    }
}

pub extern "system" fn update_texture(
    this: *mut c_void,
    source: *mut c_void,
    destination: *mut c_void,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller holds the source texture for this call.
    let Ok(Some(source)) = (unsafe { resource::input(source, &device) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the caller holds the destination texture for this call.
    let Ok(Some(destination)) = (unsafe { resource::input(destination, &device) }) else {
        return D3DERR_INVALIDCALL;
    };
    if !matches!(
        (source.backend(), destination.backend()),
        (ResourceBackend::Texture(_), ResourceBackend::Texture(_))
            | (
                ResourceBackend::CubeTexture(_),
                ResourceBackend::CubeTexture(_)
            )
            | (
                ResourceBackend::VolumeTexture(_),
                ResourceBackend::VolumeTexture(_)
            )
    ) {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: both validated textures have the same kind and belong to this backend device.
    unsafe {
        (device.backend().table().update_texture)(
            device.backend().pointer(),
            source.backend().pointer(),
            destination.backend().pointer(),
        )
    }
}
