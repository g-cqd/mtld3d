//! Owning backend COM references whose concrete resource type is tracked by their wrapper.

use core::{ffi::c_void, ptr};

use mtld3d_types::{D3DERR_INVALIDCALL, Guid, IID_IUNKNOWN};

use crate::backend::{Backend, BackendVtable};

/// The common three-slot prefix present in every COM interface vtable.
#[repr(C)]
pub struct UnknownVtable {
    query_interface: unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}

impl BackendVtable for UnknownVtable {
    fn release(&self) -> unsafe extern "system" fn(*mut c_void) -> u32 {
        self.release
    }
    fn query_interface(
        &self,
    ) -> unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32 {
        self.query_interface
    }
}

impl<V: BackendVtable> Backend<V> {
    /// Returns the canonical `IUnknown` address while retaining the original interface.
    ///
    /// # Errors
    /// Returns the backend HRESULT when interface negotiation fails.
    pub fn identity(&self) -> Result<usize, i32> {
        let identity = self.query(&IID_IUNKNOWN)?;
        Ok(identity.pointer() as usize)
    }

    /// Returns an owned backend interface matching `iid`.
    ///
    /// # Errors
    /// Returns the backend's failure, or `INVALIDCALL` for a null successful output.
    pub fn query(&self, iid: &Guid) -> Result<Backend<UnknownVtable>, i32> {
        let mut result = ptr::null_mut();
        // SAFETY: the owned COM interface receives a valid identifier and output slot.
        let status = unsafe {
            (self.table().query_interface())(self.pointer(), ptr::from_ref(iid), &raw mut result)
        };
        if status < 0 {
            return Err(status);
        }
        // SAFETY: successful QueryInterface transfers one owned reference of the requested type.
        unsafe { Backend::adopt(result) }.ok_or(D3DERR_INVALIDCALL)
    }
}
