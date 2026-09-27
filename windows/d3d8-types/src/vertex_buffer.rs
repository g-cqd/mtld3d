//! Frozen `IDirect3DVertexBuffer8` interface layout.

use core::ffi::c_void;

use mtld3d_types::{D3DVERTEXBUFFER_DESC, Guid};

/// The interface identifier for `IDirect3DVertexBuffer8`.
pub const IID_IDIRECT3DVERTEXBUFFER8: Guid = Guid {
    data1: 0x8AEE_EAC7,
    data2: 0x05F9,
    data3: 0x44D4,
    data4: [0xB5, 0x91, 0x00, 0x0B, 0x0D, 0xF1, 0xCB, 0x95],
};

/// The 14-entry `IDirect3DVertexBuffer8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DVertexBuffer8Vtbl {
    pub query_interface:
        unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    pub add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub get_device: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32,
    pub set_private_data:
        unsafe extern "system" fn(*mut c_void, *const Guid, *const c_void, u32, u32) -> i32,
    pub get_private_data:
        unsafe extern "system" fn(*mut c_void, *const Guid, *mut c_void, *mut u32) -> i32,
    pub free_private_data: unsafe extern "system" fn(*mut c_void, *const Guid) -> i32,
    pub set_priority: unsafe extern "system" fn(*mut c_void, u32) -> u32,
    pub get_priority: unsafe extern "system" fn(*mut c_void) -> u32,
    pub pre_load: unsafe extern "system" fn(*mut c_void),
    pub get_type: unsafe extern "system" fn(*mut c_void) -> u32,
    pub lock: unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut u8, u32) -> i32,
    pub unlock: unsafe extern "system" fn(*mut c_void) -> i32,
    pub get_desc: unsafe extern "system" fn(*mut c_void, *mut D3DVERTEXBUFFER_DESC) -> i32,
}
