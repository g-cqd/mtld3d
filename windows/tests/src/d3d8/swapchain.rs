//! Additional D3D8 swap chains and their owned back-buffer references.

use core::{ffi::c_void, marker::PhantomData, ptr};

use mtld3d_d3d8_types::IDirect3DSwapChain8Vtbl;

use super::{D3D8Harness, D3D8Surface, presentation_parameters};
use crate::{check::expect_created, vtbl::deref_vtbl};

/// An additional swap chain tied to its creating harness.
pub struct D3D8SwapChain<'a> {
    pointer: *mut c_void,
    owner: PhantomData<&'a D3D8Harness>,
}

impl D3D8Harness {
    /// Creates a windowed, lockable additional swap chain.
    ///
    /// # Panics
    /// Panics if creation fails.
    #[must_use]
    pub fn create_swap_chain(&self, width: u32, height: u32) -> D3D8SwapChain<'_> {
        let mut parameters = presentation_parameters(self.window, width, height);
        let mut pointer = ptr::null_mut();
        // SAFETY: the device is live and both output arguments are writable.
        let result = unsafe {
            (self.device_vtable().create_additional_swap_chain)(
                self.device,
                &raw mut parameters,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateAdditionalSwapChain");
        D3D8SwapChain {
            pointer,
            owner: PhantomData,
        }
    }
}

impl<'a> D3D8SwapChain<'a> {
    /// Obtains a back buffer without extending this wrapper's borrow.
    ///
    /// # Errors
    /// Returns the HRESULT if the index is unavailable.
    ///
    /// # Panics
    /// Panics if success and output disagree.
    pub fn back_buffer(&self, index: u32, kind: u32) -> Result<D3D8Surface<'a>, i32> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the chain is live and pointer is writable.
        let result =
            unsafe { (self.vtable().get_back_buffer)(self.pointer, index, kind, &raw mut pointer) };
        if result < 0 {
            assert!(
                pointer.is_null(),
                "failed swap-chain GetBackBuffer retained an output"
            );
            return Err(result);
        }
        expect_created(result, pointer, "D3D8 swap-chain GetBackBuffer");
        // SAFETY: the getter returned an independently owned D3D8 surface reference.
        Ok(unsafe { D3D8Surface::from_owned(pointer) })
    }

    /// Presents the whole additional back buffer and returns its HRESULT.
    #[must_use]
    pub fn present(&self) -> i32 {
        // SAFETY: null rectangles select the whole buffer and zero selects the original window.
        unsafe { (self.vtable().present)(self.pointer, ptr::null(), ptr::null(), 0, ptr::null()) }
    }

    fn vtable(&self) -> &'static IDirect3DSwapChain8Vtbl {
        // SAFETY: this wrapper owns a live swap-chain reference.
        unsafe { deref_vtbl(self.pointer) }
    }
}

impl Drop for D3D8SwapChain<'_> {
    fn drop(&mut self) {
        // SAFETY: creation supplied this wrapper's owned reference.
        unsafe { (self.vtable().release)(self.pointer) };
    }
}
