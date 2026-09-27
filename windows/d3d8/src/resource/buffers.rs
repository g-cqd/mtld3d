//! Vertex and index buffer methods with the D3D8 ABI.

use core::ffi::c_void;

use mtld3d_types::{D3DERR_INVALIDCALL, D3DINDEXBUFFER_DESC, D3DVERTEXBUFFER_DESC};

use super::{ResourceBackend, api_scope, object};

macro_rules! buffer_method {
    ($name:ident, $variant:ident, $method:ident, ($($argument:ident : $kind:ty),*)) => {
        pub extern "system" fn $name(this: *mut c_void, $($argument: $kind),*) -> i32 {
            // SAFETY: the COM receiver owns the parent device for this call.
            let _api = unsafe { api_scope(this) };
            // SAFETY: this typed vtable dispatches a live Resource8 receiver.
            let resource = unsafe { object(this) };
            let ResourceBackend::$variant(backend) = &resource.inner.backend else { return D3DERR_INVALIDCALL; };
            // SAFETY: buffer data and descriptor layouts are identical in D3D8 and D3D9.
            unsafe { (backend.table().$method)(backend.pointer(), $($argument),*) }
        }
    };
}
buffer_method!(vertex_unlock, VertexBuffer, unlock, ());
buffer_method!(vertex_get_desc, VertexBuffer, get_desc, (output: *mut D3DVERTEXBUFFER_DESC));
buffer_method!(index_unlock, IndexBuffer, unlock, ());
buffer_method!(index_get_desc, IndexBuffer, get_desc, (output: *mut D3DINDEXBUFFER_DESC));

pub extern "system" fn vertex_lock(
    this: *mut c_void,
    offset: u32,
    size: u32,
    output: *mut *mut u8,
    flags: u32,
) -> i32 {
    // SAFETY: this resource owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::VertexBuffer(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the converted pointer types have identical D3D8 and D3D9 memory layouts.
    unsafe { (backend.table().lock)(backend.pointer(), offset, size, output.cast(), flags) }
}

pub extern "system" fn index_lock(
    this: *mut c_void,
    offset: u32,
    size: u32,
    output: *mut *mut u8,
    flags: u32,
) -> i32 {
    // SAFETY: this resource owns its parent device for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live resource wrapper.
    let resource = unsafe { object(this) };
    let ResourceBackend::IndexBuffer(backend) = &resource.inner.backend else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the converted pointer types have identical D3D8 and D3D9 memory layouts.
    unsafe { (backend.table().lock)(backend.pointer(), offset, size, output.cast(), flags) }
}
