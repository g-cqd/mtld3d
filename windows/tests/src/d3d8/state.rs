//! D3D8 scalar state and token-based state-block calls.

use core::mem::MaybeUninit;

use mtld3d_types::{D3DLIGHT9, D3DMATERIAL9, D3DMATRIX, D3DVIEWPORT9};

use super::D3D8Harness;
use crate::check::expect_ok;

impl D3D8Harness {
    /// Writes the complete material state.
    #[must_use]
    pub fn set_material(&self, value: &D3DMATERIAL9) -> i32 {
        // SAFETY: the device and complete input remain live throughout the call.
        unsafe { (self.device_vtable().set_material)(self.device, value) }
    }

    /// Reads the complete material state.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn material(&self) -> D3DMATERIAL9 {
        let mut value = MaybeUninit::uninit();
        // SAFETY: the device is live and value holds a complete output.
        let result =
            unsafe { (self.device_vtable().get_material)(self.device, value.as_mut_ptr()) };
        expect_ok(result, "D3D8 get_material");
        // SAFETY: successful retrieval initialized every field.
        unsafe { value.assume_init() }
    }

    /// Writes the complete light state.
    #[must_use]
    pub fn set_light(&self, index: u32, value: &D3DLIGHT9) -> i32 {
        // SAFETY: the device and complete input remain live throughout the call.
        unsafe { (self.device_vtable().set_light)(self.device, index, value) }
    }

    /// Reads the complete light state.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn light(&self, index: u32) -> D3DLIGHT9 {
        let mut value = MaybeUninit::uninit();
        // SAFETY: the device is live and value holds a complete output.
        let result =
            unsafe { (self.device_vtable().get_light)(self.device, index, value.as_mut_ptr()) };
        expect_ok(result, "D3D8 get_light");
        // SAFETY: successful retrieval initialized every field.
        unsafe { value.assume_init() }
    }

    /// Reads one render state through the D3D8 device.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn render_state(&self, state: u32) -> u32 {
        let mut value = 0;
        // SAFETY: the harness owns its device and value is writable.
        let result =
            unsafe { (self.device_vtable().get_render_state)(self.device, state, &raw mut value) };
        expect_ok(result, "D3D8 GetRenderState");
        value
    }

    /// Reads a texture-stage state, including D3D8's sampler selectors.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn texture_stage_state(&self, stage: u32, selector: u32) -> u32 {
        let mut value = 0;
        // SAFETY: the harness owns its device and value is writable.
        let result = unsafe {
            (self.device_vtable().get_texture_stage_state)(
                self.device,
                stage,
                selector,
                &raw mut value,
            )
        };
        expect_ok(result, "D3D8 GetTextureStageState");
        value
    }

    /// Writes a complete transform matrix.
    #[must_use]
    pub fn set_transform(&self, state: u32, matrix: &D3DMATRIX) -> i32 {
        // SAFETY: the device and complete matrix remain live for the call.
        unsafe {
            (self.device_vtable().set_transform)(self.device, state, (&raw const *matrix).cast())
        }
    }

    /// Reads a complete transform matrix.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn transform(&self, state: u32) -> D3DMATRIX {
        let mut matrix = MaybeUninit::<D3DMATRIX>::uninit();
        // SAFETY: the device is live and matrix holds a complete output.
        let result = unsafe {
            (self.device_vtable().get_transform)(self.device, state, matrix.as_mut_ptr().cast())
        };
        expect_ok(result, "D3D8 GetTransform");
        // SAFETY: the successful getter initialized the complete matrix.
        unsafe { matrix.assume_init() }
    }

    /// Applies a viewport using the layout shared by D3D8 and D3D9.
    #[must_use]
    pub fn set_viewport(&self, viewport: &D3DVIEWPORT9) -> i32 {
        // SAFETY: device and viewport remain live for the call.
        unsafe { (self.device_vtable().set_viewport)(self.device, viewport) }
    }

    /// Reads the current viewport.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn viewport(&self) -> D3DVIEWPORT9 {
        let mut viewport = MaybeUninit::uninit();
        // SAFETY: the device is live and viewport holds a complete output.
        let result =
            unsafe { (self.device_vtable().get_viewport)(self.device, viewport.as_mut_ptr()) };
        expect_ok(result, "D3D8 GetViewport");
        // SAFETY: the successful getter initialized every viewport field.
        unsafe { viewport.assume_init() }
    }

    /// Starts recording a state block and returns the HRESULT.
    #[must_use]
    pub fn begin_state_block(&self) -> i32 {
        // SAFETY: the harness owns its device.
        unsafe { (self.device_vtable().begin_state_block)(self.device) }
    }

    /// Finishes recording and returns the owned block token.
    ///
    /// # Panics
    /// Panics if recording cannot be finished.
    #[must_use]
    pub fn end_state_block(&self) -> u32 {
        let mut token = 0;
        let result = self.end_state_block_into(&mut token);
        expect_ok(result, "D3D8 EndStateBlock");
        token
    }

    /// Finishes recording into caller-owned storage and returns the HRESULT.
    #[must_use]
    pub fn end_state_block_into(&self, token: &mut u32) -> i32 {
        // SAFETY: the device is live and token is writable.
        unsafe { (self.device_vtable().end_state_block)(self.device, token) }
    }

    /// Captures current state in a newly owned block token.
    ///
    /// # Panics
    /// Panics if the requested block cannot be created.
    #[must_use]
    pub fn create_state_block(&self, kind: u32) -> u32 {
        let mut token = 0;
        // SAFETY: the device is live and token is writable.
        let result =
            unsafe { (self.device_vtable().create_state_block)(self.device, kind, &raw mut token) };
        expect_ok(result, "D3D8 CreateStateBlock");
        token
    }

    /// Applies the selected block and returns the HRESULT.
    #[must_use]
    pub fn apply_state_block(&self, token: u32) -> i32 {
        // SAFETY: the harness owns its device; the implementation validates the token.
        unsafe { (self.device_vtable().apply_state_block)(self.device, token) }
    }

    /// Refreshes an existing block's captured state and returns the HRESULT.
    #[must_use]
    pub fn capture_state_block(&self, token: u32) -> i32 {
        // SAFETY: the harness owns its device; the implementation validates the token.
        unsafe { (self.device_vtable().capture_state_block)(self.device, token) }
    }

    /// Releases an owned state-block token and returns the HRESULT.
    #[must_use]
    pub fn delete_state_block(&self, token: u32) -> i32 {
        // SAFETY: the harness owns its device; the implementation validates the token.
        unsafe { (self.device_vtable().delete_state_block)(self.device, token) }
    }
}
