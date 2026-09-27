//! Frozen `IDirect3DVolumeTexture8` interface layout.

use core::ffi::c_void;

use mtld3d_types::{D3DBOX, D3DLOCKED_BOX, Guid};

use crate::D3DVOLUME_DESC8;

/// The interface identifier for `IDirect3DVolumeTexture8`.
pub const IID_IDIRECT3DVOLUMETEXTURE8: Guid = Guid {
    data1: 0x4B8A_AAFA,
    data2: 0x140F,
    data3: 0x42BA,
    data4: [0x91, 0x31, 0x59, 0x7E, 0xAF, 0xAA, 0x2E, 0xAD],
};

/// The 19-entry `IDirect3DVolumeTexture8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DVolumeTexture8Vtbl {
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
    pub set_lod: unsafe extern "system" fn(*mut c_void, u32) -> u32,
    pub get_lod: unsafe extern "system" fn(*mut c_void) -> u32,
    pub get_level_count: unsafe extern "system" fn(*mut c_void) -> u32,
    pub get_level_desc: unsafe extern "system" fn(*mut c_void, u32, *mut D3DVOLUME_DESC8) -> i32,
    pub get_volume_level: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> i32,
    pub lock_box:
        unsafe extern "system" fn(*mut c_void, u32, *mut D3DLOCKED_BOX, *const D3DBOX, u32) -> i32,
    pub unlock_box: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub add_dirty_box: unsafe extern "system" fn(*mut c_void, *const D3DBOX) -> i32,
}
