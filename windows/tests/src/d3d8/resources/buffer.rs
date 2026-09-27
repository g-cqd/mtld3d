//! D3D8 buffer descriptors, byte locks, and binding identity.

use core::{mem::MaybeUninit, ptr};

use mtld3d_d3d8_types::{IDirect3DIndexBuffer8Vtbl, IDirect3DVertexBuffer8Vtbl};
use mtld3d_types::{
    D3DFMT_INDEX16, D3DINDEXBUFFER_DESC, D3DLOCK_READONLY, D3DPOOL_MANAGED, D3DVERTEXBUFFER_DESC,
};

use super::{D3D8Harness, D3D8Resource};
use crate::{
    check::{expect_created, expect_ok},
    resource::release_unknown,
};

impl D3D8Harness {
    /// Creates a managed vertex buffer with the supplied byte size and FVF.
    ///
    /// # Panics
    /// Panics if creation fails.
    #[must_use]
    pub fn create_vertex_buffer8(
        &self,
        length: u32,
        fvf: u32,
    ) -> D3D8Resource<'_, IDirect3DVertexBuffer8Vtbl> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns its device and the output is writable local storage.
        let result = unsafe {
            (self.device_vtable().create_vertex_buffer)(
                self.device,
                length,
                0,
                fvf,
                D3DPOOL_MANAGED,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateVertexBuffer");
        // SAFETY: the successful factory returned one owned vertex-buffer interface.
        unsafe { D3D8Resource::from_owned(pointer) }
    }

    /// Creates a managed 16-bit index buffer with the supplied byte size.
    ///
    /// # Panics
    /// Panics if creation fails.
    #[must_use]
    pub fn create_index_buffer8(&self, length: u32) -> D3D8Resource<'_, IDirect3DIndexBuffer8Vtbl> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns its device and the output is writable local storage.
        let result = unsafe {
            (self.device_vtable().create_index_buffer)(
                self.device,
                length,
                0,
                D3DFMT_INDEX16,
                D3DPOOL_MANAGED,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateIndexBuffer");
        // SAFETY: the successful factory returned one owned index-buffer interface.
        unsafe { D3D8Resource::from_owned(pointer) }
    }

    /// Binds an optional vertex buffer to stream zero and returns the HRESULT.
    #[must_use]
    pub fn set_vertex_buffer8(
        &self,
        buffer: Option<&D3D8Resource<'_, IDirect3DVertexBuffer8Vtbl>>,
        stride: u32,
    ) -> i32 {
        // SAFETY: the owned optional buffer remains live through the device call.
        unsafe {
            (self.device_vtable().set_stream_source)(
                self.device,
                0,
                buffer.map_or(ptr::null_mut(), |value| value.pointer),
                stride,
            )
        }
    }

    /// Returns stream zero's identity comparison and current stride.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn vertex_buffer_binding8(
        &self,
        buffer: Option<&D3D8Resource<'_, IDirect3DVertexBuffer8Vtbl>>,
    ) -> (bool, u32) {
        let mut pointer = ptr::null_mut();
        let mut stride = 0;
        // SAFETY: both output slots are writable and the harness owns the device.
        let result = unsafe {
            (self.device_vtable().get_stream_source)(
                self.device,
                0,
                &raw mut pointer,
                &raw mut stride,
            )
        };
        expect_ok(result, "D3D8 GetStreamSource");
        let same = pointer == buffer.map_or(ptr::null_mut(), |value| value.pointer);
        if !pointer.is_null() {
            // SAFETY: the getter returned one owned interface reference.
            unsafe { release_unknown(pointer) };
        }
        (same, stride)
    }

    /// Binds an optional index buffer with the D3D8 base-vertex value.
    #[must_use]
    pub fn set_index_buffer8(
        &self,
        buffer: Option<&D3D8Resource<'_, IDirect3DIndexBuffer8Vtbl>>,
        base: u32,
    ) -> i32 {
        // SAFETY: the owned optional buffer remains live through the device call.
        unsafe {
            (self.device_vtable().set_indices)(
                self.device,
                buffer.map_or(ptr::null_mut(), |value| value.pointer),
                base,
            )
        }
    }

    /// Returns the index-buffer identity comparison and base-vertex value.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn index_buffer_binding8(
        &self,
        buffer: Option<&D3D8Resource<'_, IDirect3DIndexBuffer8Vtbl>>,
    ) -> (bool, u32) {
        let mut pointer = ptr::null_mut();
        let mut base = 0;
        // SAFETY: both output slots are writable and the harness owns the device.
        let result = unsafe {
            (self.device_vtable().get_indices)(self.device, &raw mut pointer, &raw mut base)
        };
        expect_ok(result, "D3D8 GetIndices");
        let same = pointer == buffer.map_or(ptr::null_mut(), |value| value.pointer);
        if !pointer.is_null() {
            // SAFETY: the getter returned one owned interface reference.
            unsafe { release_unknown(pointer) };
        }
        (same, base)
    }
}

macro_rules! buffer_methods {
    ($vtable:ty, $descriptor:ty) => {
        impl D3D8Resource<'_, $vtable> {
            /// Returns the buffer's creation descriptor.
            ///
            /// # Panics
            /// Panics if `GetDesc` fails.
            #[must_use]
            pub fn desc(&self) -> $descriptor {
                let mut descriptor = MaybeUninit::uninit();
                let result =
                    // SAFETY: the owned buffer writes a complete descriptor into local storage.
                    unsafe { (self.vtable().get_desc)(self.pointer, descriptor.as_mut_ptr()) };
                expect_ok(result, "D3D8 buffer GetDesc");
                // SAFETY: successful GetDesc initialized all descriptor fields.
                unsafe { descriptor.assume_init() }
            }

            /// Copies bytes into a checked buffer range.
            ///
            /// # Panics
            /// Panics if the range or lock is invalid.
            pub fn write(&self, offset: u32, bytes: &[u8]) {
                let size = u32::try_from(bytes.len()).expect("buffer write length");
                assert!(!bytes.is_empty());
                let mut pointer = ptr::null_mut();
                // SAFETY: the backend validates the range; pointer is writable lock output.
                let result = unsafe {
                    (self.vtable().lock)(self.pointer, offset, size, &raw mut pointer, 0)
                };
                expect_created(result, pointer.cast(), "D3D8 buffer Lock");
                // SAFETY: a successful lock grants writable size bytes; input is a distinct caller slice.
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
                expect_ok(
                    // SAFETY: this method owns exactly one successful buffer lock.
                    unsafe { (self.vtable().unlock)(self.pointer) },
                    "D3D8 buffer Unlock",
                );
            }

            /// Reads a checked buffer range into caller storage.
            ///
            /// # Panics
            /// Panics if the range or lock is invalid.
            pub fn read(&self, offset: u32, bytes: &mut [u8]) {
                let size = u32::try_from(bytes.len()).expect("buffer read length");
                assert!(!bytes.is_empty());
                let mut pointer = ptr::null_mut();
                // SAFETY: the backend validates the range; pointer is writable lock output.
                let result = unsafe {
                    (self.vtable().lock)(
                        self.pointer,
                        offset,
                        size,
                        &raw mut pointer,
                        D3DLOCK_READONLY,
                    )
                };
                expect_created(result, pointer.cast(), "D3D8 buffer Lock READONLY");
                // SAFETY: a successful lock grants readable size bytes; output is a distinct caller slice.
                unsafe { ptr::copy_nonoverlapping(pointer, bytes.as_mut_ptr(), bytes.len()) };
                expect_ok(
                    // SAFETY: this method owns exactly one successful buffer lock.
                    unsafe { (self.vtable().unlock)(self.pointer) },
                    "D3D8 buffer Unlock",
                );
            }
        }
    };
}
buffer_methods!(IDirect3DVertexBuffer8Vtbl, D3DVERTEXBUFFER_DESC);
buffer_methods!(IDirect3DIndexBuffer8Vtbl, D3DINDEXBUFFER_DESC);
