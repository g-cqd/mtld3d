//! Additional swap-chain creation with D3D8 presentation parameters.

use core::{ffi::c_void, ptr};

use mtld3d_d3d8_types::{
    D3DPRESENT_PARAMETERS8, copy_d3d9_present_parameters, to_d3d9_present_parameters,
};
use mtld3d_shared::{InPtrMut, OutPtr};
use mtld3d_types::D3DERR_INVALIDCALL;

use super::{api_scope, object};
use crate::{backend::Backend, swapchain::SwapChain8};

pub extern "system" fn create_additional_swap_chain(
    this: *mut c_void,
    parameters: *mut D3DPRESENT_PARAMETERS8,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: presentation parameters are a readable and writable ABI argument.
    let Some(mut parameters) =
        (unsafe { InPtrMut::<D3DPRESENT_PARAMETERS8>::opt(parameters.cast()) })
    else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let device = unsafe { object(this) };
    let mut converted = to_d3d9_present_parameters(&parameters);
    let mut backend = ptr::null_mut();
    // SAFETY: both backend outputs are writable and converted has the D3D9 parameter layout.
    let result = unsafe {
        (device.backend().table().create_additional_swap_chain)(
            device.backend().pointer(),
            (&raw mut converted).cast(),
            &raw mut backend,
        )
    };
    if result < 0 {
        output.write(ptr::null_mut());
        return result;
    }
    // SAFETY: successful creation returned one owned IDirect3DSwapChain9 reference.
    let Some(backend) = (unsafe { Backend::adopt(backend) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    let chain = SwapChain8::create(&device, backend);
    copy_d3d9_present_parameters(&mut parameters, &converted);
    output.write(chain);
    result
}
