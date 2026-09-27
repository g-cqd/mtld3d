//! Frozen `IDirect3DCubeTexture8` interface layout.

use core::ffi::c_void;

use mtld3d_types::{D3DLOCKED_RECT, Guid};

use crate::D3DSURFACE_DESC8;

/// The interface identifier for `IDirect3DCubeTexture8`.
pub const IID_IDIRECT3DCUBETEXTURE8: Guid = Guid {
    data1: 0x3EE5_B968,
    data2: 0x2ACA,
    data3: 0x4C34,
    data4: [0x8B, 0xB5, 0x7E, 0x0C, 0x3D, 0x19, 0xB7, 0x50],
};

/// The 19-entry `IDirect3DCubeTexture8` vtable, including its inherited methods.
#[repr(C)]
pub struct IDirect3DCubeTexture8Vtbl {
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
    pub get_level_desc: unsafe extern "system" fn(*mut c_void, u32, *mut D3DSURFACE_DESC8) -> i32,
    pub get_cube_map_surface:
        unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void) -> i32,
    pub lock_rect: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        *mut D3DLOCKED_RECT,
        *const c_void,
        u32,
    ) -> i32,
    pub unlock_rect: unsafe extern "system" fn(*mut c_void, u32, u32) -> i32,
    pub add_dirty_rect: unsafe extern "system" fn(*mut c_void, u32, *const c_void) -> i32,
}
