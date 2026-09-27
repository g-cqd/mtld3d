//! D3D8 adapter mode enumeration through the factory interface.

use core::mem::MaybeUninit;

use mtld3d_types::D3DDISPLAYMODE;

use super::D3D8Harness;

impl D3D8Harness {
    /// Returns the number of modes across all supported display formats.
    #[must_use]
    pub fn adapter_mode_count(&self, adapter: u32) -> u32 {
        // SAFETY: this harness retains its factory.
        unsafe { (self.factory_vtable().get_adapter_mode_count)(self.factory, adapter) }
    }

    /// Reads one mode from the D3D8 factory's format-independent index.
    ///
    /// # Errors
    /// Returns the HRESULT for an invalid adapter or mode index.
    pub fn adapter_mode(&self, adapter: u32, index: u32) -> Result<D3DDISPLAYMODE, i32> {
        let mut mode = MaybeUninit::uninit();
        // SAFETY: the factory is live and mode provides writable ABI storage.
        let result = unsafe {
            (self.factory_vtable().enum_adapter_modes)(
                self.factory,
                adapter,
                index,
                mode.as_mut_ptr(),
            )
        };
        if result < 0 {
            Err(result)
        } else {
            // SAFETY: a successful enumeration initializes the complete mode.
            Ok(unsafe { mode.assume_init() })
        }
    }
}
