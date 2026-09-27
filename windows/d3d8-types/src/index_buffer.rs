//! Frozen `IDirect3DIndexBuffer8` interface layout.

use core::ffi::c_void;

use mtld3d_types::{D3DINDEXBUFFER_DESC, Guid};

/// The interface identifier for `IDirect3DIndexBuffer8`.
pub const IID_IDIRECT3DINDEXBUFFER8: Guid = Guid {
    data1: 0x0E68_9C9A,
    data2: 0x053D,
    data3: 0x44A0,
    data4: [0x9D, 0x92, 0xDB, 0x0E, 0x3D, 0x75, 0x0F, 0x86],
};

/// The 14-entry `IDirect3DIndexBuffer8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DIndexBuffer8Vtbl {
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
    pub get_desc: unsafe extern "system" fn(*mut c_void, *mut D3DINDEXBUFFER_DESC) -> i32,
}
