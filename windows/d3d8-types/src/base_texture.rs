//! Frozen `IDirect3DBaseTexture8` interface layout.

use core::ffi::c_void;

use mtld3d_types::Guid;

/// The interface identifier for `IDirect3DBaseTexture8`.
pub const IID_IDIRECT3DBASETEXTURE8: Guid = Guid {
    data1: 0xB421_1CFA,
    data2: 0x51B9,
    data3: 0x4A9F,
    data4: [0xAB, 0x78, 0xDB, 0x99, 0xB2, 0xBB, 0x67, 0x8E],
};

/// The 14-entry `IDirect3DBaseTexture8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DBaseTexture8Vtbl {
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
}
