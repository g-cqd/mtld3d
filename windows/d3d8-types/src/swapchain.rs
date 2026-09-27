//! Frozen `IDirect3DSwapChain8` interface layout.

use core::ffi::c_void;

use mtld3d_types::Guid;

/// The interface identifier for `IDirect3DSwapChain8`.
pub const IID_IDIRECT3DSWAPCHAIN8: Guid = Guid {
    data1: 0x928C_088B,
    data2: 0x76B9,
    data3: 0x4C6B,
    data4: [0xA5, 0x36, 0xA5, 0x90, 0x85, 0x38, 0x76, 0xCD],
};

/// The 5-entry `IDirect3DSwapChain8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DSwapChain8Vtbl {
    pub query_interface:
        unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    pub add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub present: unsafe extern "system" fn(
        *mut c_void,
        *const c_void,
        *const c_void,
        usize,
        *const c_void,
    ) -> i32,
    pub get_back_buffer: unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void) -> i32,
}
