//! Common D3D8 interface negotiation without exposing backend identities.

use core::{ffi::c_void, ptr};

use mtld3d_shared::{InPtr, OutPtr};
use mtld3d_types::{D3D_OK, D3DERR_INVALIDCALL, E_NOINTERFACE, Guid, IID_IUNKNOWN};

/// Negotiates the wrapper's public interface and returns one owned reference.
///
/// # Safety
/// The pointers follow `IUnknown::QueryInterface`; `add_ref` belongs to `this`.
pub unsafe fn query(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
    supported: &Guid,
    add_ref: extern "system" fn(*mut c_void) -> u32,
) -> i32 {
    // SAFETY: the COM caller supplies a writable output slot or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM caller supplies a readable interface identifier or null.
    let Some(iid) = (unsafe { InPtr::<Guid>::opt(iid.cast()) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    if *iid == IID_IUNKNOWN || *iid == *supported {
        add_ref(this);
        output.write(this);
        D3D_OK
    } else {
        output.write(ptr::null_mut());
        E_NOINTERFACE
    }
}
