//! Bounded shader inputs and token-handle operations for D3D8 integration tests.

use core::{ffi::c_void, ptr};

use super::D3D8Harness;
use crate::check::expect_ok;

impl D3D8Harness {
    /// Creates a shader after validating both caller-owned token streams.
    ///
    /// # Panics
    /// Panics for malformed input or failed shader creation.
    #[must_use]
    pub fn create_vertex_shader8(&self, declaration: &[u32], function: Option<&[u32]>) -> u32 {
        self.try_create_vertex_shader8(declaration, function)
            .expect("D3D8 CreateVertexShader succeeds")
    }

    /// Attempts creation from bounded token streams, returning the API's validation result.
    ///
    /// # Errors
    /// Returns the failed creation HRESULT.
    ///
    /// # Panics
    /// Panics when an input stream is incomplete or successful creation returns a null handle.
    pub fn try_create_vertex_shader8(
        &self,
        declaration: &[u32],
        function: Option<&[u32]>,
    ) -> Result<u32, i32> {
        assert!(mtld3d_core::d3d8::declaration::translate(declaration, false).is_some());
        if let Some(function) = function {
            assert!(mtld3d_core::dxso::parse(function).is_ok());
        }
        let mut handle = 0;
        // SAFETY: validation established complete terminated token streams and output is writable.
        let result = unsafe {
            (self.device_vtable().create_vertex_shader)(
                self.device,
                declaration.as_ptr(),
                function.map_or(ptr::null(), <[u32]>::as_ptr),
                &raw mut handle,
                0,
            )
        };
        if result < 0 {
            return Err(result);
        }
        assert_ne!(handle, 0);
        Ok(handle)
    }

    /// Creates a pixel shader from complete validated bytecode.
    ///
    /// # Panics
    /// Panics for malformed bytecode or failed creation.
    #[must_use]
    pub fn create_pixel_shader8(&self, function: &[u32]) -> u32 {
        assert!(mtld3d_core::dxso::parse(function).is_ok());
        let mut handle = 0;
        // SAFETY: parsing verified a complete terminated shader and output is writable.
        let result = unsafe {
            (self.device_vtable().create_pixel_shader)(
                self.device,
                function.as_ptr(),
                &raw mut handle,
            )
        };
        expect_ok(result, "D3D8 CreatePixelShader");
        assert_ne!(handle, 0);
        handle
    }

    /// Reads the active D3D8 vertex shader handle or FVF.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn vertex_shader(&self) -> u32 {
        let mut handle = 0;
        // SAFETY: the live device writes one handle into local storage.
        let result =
            unsafe { (self.device_vtable().get_vertex_shader)(self.device, &raw mut handle) };
        expect_ok(result, "D3D8 GetVertexShader");
        handle
    }

    /// Reads the active D3D8 pixel shader handle.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn pixel_shader(&self) -> u32 {
        let mut handle = 0;
        // SAFETY: the live device writes one handle into local storage.
        let result =
            unsafe { (self.device_vtable().get_pixel_shader)(self.device, &raw mut handle) };
        expect_ok(result, "D3D8 GetPixelShader");
        handle
    }

    /// Binds a pixel shader handle, or zero for fixed-function shading.
    #[must_use]
    pub fn set_pixel_shader(&self, handle: u32) -> i32 {
        // SAFETY: the live device validates this scalar handle.
        unsafe { (self.device_vtable().set_pixel_shader)(self.device, handle) }
    }

    /// Deletes a vertex shader handle and returns the API result.
    #[must_use]
    pub fn delete_vertex_shader(&self, handle: u32) -> i32 {
        // SAFETY: the live device validates this scalar handle.
        unsafe { (self.device_vtable().delete_vertex_shader)(self.device, handle) }
    }

    /// Deletes a pixel shader handle and returns the API result.
    #[must_use]
    pub fn delete_pixel_shader(&self, handle: u32) -> i32 {
        // SAFETY: the live device validates this scalar handle.
        unsafe { (self.device_vtable().delete_pixel_shader)(self.device, handle) }
    }

    /// Reads the original declaration tokens of a vertex shader handle.
    ///
    /// # Panics
    /// Panics if the handle is invalid or the byte count is malformed.
    #[must_use]
    pub fn vertex_shader_declaration(&self, handle: u32) -> Vec<u32> {
        self.shader_words(handle, self.device_vtable().get_vertex_shader_declaration)
    }

    /// Reads the original function tokens, excluding internal translation declarations.
    ///
    /// # Panics
    /// Panics if the handle is invalid or the byte count is malformed.
    #[must_use]
    pub fn vertex_shader_function(&self, handle: u32) -> Vec<u32> {
        self.shader_words(handle, self.device_vtable().get_vertex_shader_function)
    }

    /// Reads the original pixel shader bytecode.
    ///
    /// # Panics
    /// Panics if the handle is invalid or the byte count is malformed.
    #[must_use]
    pub fn pixel_shader_function(&self, handle: u32) -> Vec<u32> {
        self.shader_words(handle, self.device_vtable().get_pixel_shader_function)
    }

    fn shader_words(
        &self,
        handle: u32,
        method: unsafe extern "system" fn(*mut c_void, u32, *mut c_void, *mut u32) -> i32,
    ) -> Vec<u32> {
        let mut bytes = 0;
        // SAFETY: a null output requests the required byte count into writable local storage.
        let result = unsafe { method(self.device, handle, ptr::null_mut(), &raw mut bytes) };
        expect_ok(result, "D3D8 shader byte count");
        assert!(bytes <= 65536 * 4 && bytes.is_multiple_of(4));
        let mut words = vec![0; bytes as usize / 4];
        // SAFETY: words reserves the full reported byte count; bytes retains that capacity.
        let result = unsafe {
            method(
                self.device,
                handle,
                words.as_mut_ptr().cast(),
                &raw mut bytes,
            )
        };
        expect_ok(result, "D3D8 shader token output");
        words
    }
}

impl D3D8Harness {
    /// Writes consecutive float4 vertex constant registers.
    ///
    /// # Panics
    /// Panics if the input length cannot fit the ABI count.
    #[must_use]
    pub fn set_vertex_constants(&self, start: u32, values: &[[f32; 4]]) -> i32 {
        let count = u32::try_from(values.len()).expect("register count fits u32");
        // SAFETY: values contains count contiguous float4 registers for this call.
        unsafe {
            (self.device_vtable().set_vertex_shader_constant)(
                self.device,
                start,
                values.as_ptr().cast(),
                count,
            )
        }
    }

    /// Reads consecutive float4 vertex constant registers.
    ///
    /// # Panics
    /// Panics for an excessive register count or failed API call.
    #[must_use]
    pub fn vertex_constants(&self, start: u32, count: u32) -> Vec<[f32; 4]> {
        assert!(count <= 256);
        let mut values = vec![[0.0; 4]; count as usize];
        // SAFETY: values contains count writable contiguous float4 registers.
        let result = unsafe {
            (self.device_vtable().get_vertex_shader_constant)(
                self.device,
                start,
                values.as_mut_ptr().cast(),
                count,
            )
        };
        expect_ok(result, "D3D8 vertex shader constants");
        values
    }

    /// Writes consecutive float4 pixel constant registers.
    ///
    /// # Panics
    /// Panics if the input length cannot fit the ABI count.
    #[must_use]
    pub fn set_pixel_constants(&self, start: u32, values: &[[f32; 4]]) -> i32 {
        let count = u32::try_from(values.len()).expect("register count fits u32");
        // SAFETY: values contains count contiguous float4 registers for this call.
        unsafe {
            (self.device_vtable().set_pixel_shader_constant)(
                self.device,
                start,
                values.as_ptr().cast(),
                count,
            )
        }
    }

    /// Reads consecutive float4 pixel constant registers.
    ///
    /// # Panics
    /// Panics for an excessive register count or failed API call.
    #[must_use]
    pub fn pixel_constants(&self, start: u32, count: u32) -> Vec<[f32; 4]> {
        assert!(count <= 256);
        let mut values = vec![[0.0; 4]; count as usize];
        // SAFETY: values contains count writable contiguous float4 registers.
        let result = unsafe {
            (self.device_vtable().get_pixel_shader_constant)(
                self.device,
                start,
                values.as_mut_ptr().cast(),
                count,
            )
        };
        expect_ok(result, "D3D8 pixel shader constants");
        values
    }
}
