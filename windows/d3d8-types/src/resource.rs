//! Frozen `IDirect3DResource8` interface layout.

use core::ffi::c_void;

use mtld3d_types::Guid;

/// The interface identifier for `IDirect3DResource8`.
pub const IID_IDIRECT3DRESOURCE8: Guid = Guid {
    data1: 0x1B36_BB7B,
    data2: 0x09B7,
    data3: 0x410A,
    data4: [0xB4, 0x45, 0x7D, 0x14, 0x30, 0xD7, 0xB3, 0x3F],
};

/// The 11-entry `IDirect3DResource8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DResource8Vtbl {
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
}
