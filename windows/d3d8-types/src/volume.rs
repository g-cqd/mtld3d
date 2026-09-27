//! Frozen `IDirect3DVolume8` interface layout.

use core::ffi::c_void;

use mtld3d_types::{D3DBOX, D3DLOCKED_BOX, Guid};

use crate::D3DVOLUME_DESC8;

/// The interface identifier for `IDirect3DVolume8`.
pub const IID_IDIRECT3DVOLUME8: Guid = Guid {
    data1: 0xBD73_49F5,
    data2: 0x14F1,
    data3: 0x42E4,
    data4: [0x9C, 0x79, 0x97, 0x23, 0x80, 0xDB, 0x40, 0xC0],
};

/// The 11-entry `IDirect3DVolume8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DVolume8Vtbl {
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
    pub get_container: unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    pub get_desc: unsafe extern "system" fn(*mut c_void, *mut D3DVOLUME_DESC8) -> i32,
    pub lock_box:
        unsafe extern "system" fn(*mut c_void, *mut D3DLOCKED_BOX, *const D3DBOX, u32) -> i32,
    pub unlock_box: unsafe extern "system" fn(*mut c_void) -> i32,
}
