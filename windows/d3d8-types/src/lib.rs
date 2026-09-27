//! Direct3D 8's frozen COM ABI and lossless conversions to the internal D3D9 ABI.
//!
//! D3D8 and D3D9 have different interface layouts.  D3D9 objects consequently
//! never cross this crate's public boundary.

use core::ffi::c_void;

mod base_texture;
mod cube_texture;
mod device;
mod index_buffer;
mod resource;
mod swapchain;
mod texture;
mod vertex_buffer;
mod volume;
mod volume_desc;
mod volume_texture;

pub use base_texture::{IDirect3DBaseTexture8Vtbl, IID_IDIRECT3DBASETEXTURE8};
pub use cube_texture::{IDirect3DCubeTexture8Vtbl, IID_IDIRECT3DCUBETEXTURE8};
pub use device::IDirect3DDevice8Vtbl;
pub use index_buffer::{IDirect3DIndexBuffer8Vtbl, IID_IDIRECT3DINDEXBUFFER8};
pub use mtld3d_types::{
    D3DDEVICE_CREATION_PARAMETERS, D3DDISPLAYMODE, D3DGAMMARAMP, D3DLOCKED_RECT, D3DRECT,
    D3DSURFACE_DESC as D3DSURFACE_DESC9, Guid, IID_IUNKNOWN,
};
pub use resource::{IDirect3DResource8Vtbl, IID_IDIRECT3DRESOURCE8};
pub use swapchain::{IDirect3DSwapChain8Vtbl, IID_IDIRECT3DSWAPCHAIN8};
pub use texture::{IDirect3DTexture8Vtbl, IID_IDIRECT3DTEXTURE8};
pub use vertex_buffer::{IDirect3DVertexBuffer8Vtbl, IID_IDIRECT3DVERTEXBUFFER8};
pub use volume::{IDirect3DVolume8Vtbl, IID_IDIRECT3DVOLUME8};
pub use volume_desc::D3DVOLUME_DESC8;
pub use volume_texture::{IDirect3DVolumeTexture8Vtbl, IID_IDIRECT3DVOLUMETEXTURE8};

/// D3D8's `D3D_SDK_VERSION`.
pub const D3DSDK_VERSION8: u32 = 220;

/// `IID_IDirect3D8`.
pub const IID_IDIRECT3D8: Guid = Guid {
    data1: 0x1DD9_E8DA,
    data2: 0x1C77,
    data3: 0x4D40,
    data4: [0xB0, 0xCF, 0x98, 0xFE, 0xFD, 0xFF, 0x95, 0x12],
};

/// `IID_IDirect3DDevice8`.
pub const IID_IDIRECT3DDEVICE8: Guid = Guid {
    data1: 0x7385_E5DF,
    data2: 0x8FE8,
    data3: 0x41D5,
    data4: [0x86, 0xB6, 0xD7, 0xB4, 0x85, 0x47, 0xB6, 0xCF],
};

/// `IID_IDirect3DSurface8`.
pub const IID_IDIRECT3DSURFACE8: Guid = Guid {
    data1: 0xB96E_EBCA,
    data2: 0xB326,
    data3: 0x4EA5,
    data4: [0x88, 0x2F, 0x2F, 0xF5, 0xBA, 0xE0, 0x21, 0xDD],
};

/// D3D8 presentation parameters.  D3D8 does not expose D3D9's multisample quality.
#[repr(C)]
pub struct D3DPRESENT_PARAMETERS8 {
    pub back_buffer_width: u32,
    pub back_buffer_height: u32,
    pub back_buffer_format: u32,
    pub back_buffer_count: u32,
    pub multi_sample_type: u32,
    pub swap_effect: u32,
    pub device_window: usize,
    pub windowed: u32,
    pub enable_auto_depth_stencil: u32,
    pub auto_depth_stencil_format: u32,
    pub flags: u32,
    pub full_screen_refresh_rate_in_hz: u32,
    pub full_screen_presentation_interval: u32,
}

/// D3D8 adapter identifier without the D3D9 device-name field.
#[repr(C)]
#[cfg_attr(target_pointer_width = "64", repr(align(8)))]
pub struct D3DADAPTER_IDENTIFIER8 {
    pub driver: [u8; 512],
    pub description: [u8; 512],
    /// Win32 `LARGE_INTEGER`, represented as two aligned 32-bit halves on both PE ABIs.
    pub driver_version: [u32; 2],
    pub vendor_id: u32,
    pub device_id: u32,
    pub sub_sys_id: u32,
    pub revision: u32,
    pub device_identifier: [u8; 16],
    pub whql_level: u32,
}

/// Drops D3D9's device-name field while preserving the D3D8 identifier data.
#[must_use]
pub const fn to_d3d8_adapter_identifier(
    identifier: &mtld3d_types::D3DADAPTER_IDENTIFIER9,
) -> D3DADAPTER_IDENTIFIER8 {
    D3DADAPTER_IDENTIFIER8 {
        driver: identifier.driver,
        description: identifier.description,
        driver_version: identifier.driver_version,
        vendor_id: identifier.vendor_id,
        device_id: identifier.device_id,
        sub_sys_id: identifier.sub_sys_id,
        revision: identifier.revision,
        device_identifier: identifier.device_identifier,
        whql_level: identifier.whql_level,
    }
}

/// Converts D3D8 presentation parameters to their D3D9 equivalent.
///
/// D3D8 has no quality field, so the result always requests quality zero.
#[must_use]
pub const fn to_d3d9_present_parameters(
    parameters: &D3DPRESENT_PARAMETERS8,
) -> mtld3d_types::D3DPRESENT_PARAMETERS {
    mtld3d_types::D3DPRESENT_PARAMETERS {
        back_buffer_width: parameters.back_buffer_width,
        back_buffer_height: parameters.back_buffer_height,
        back_buffer_format: parameters.back_buffer_format,
        back_buffer_count: parameters.back_buffer_count,
        multi_sample_type: parameters.multi_sample_type,
        multi_sample_quality: 0,
        swap_effect: parameters.swap_effect,
        device_window: parameters.device_window,
        windowed: parameters.windowed,
        enable_auto_depth_stencil: parameters.enable_auto_depth_stencil,
        auto_depth_stencil_format: parameters.auto_depth_stencil_format,
        flags: parameters.flags,
        full_screen_refresh_rate_in_hz: parameters.full_screen_refresh_rate_in_hz,
        presentation_interval: parameters.full_screen_presentation_interval,
    }
}

/// Copies the D3D8-visible fields from mutated D3D9 presentation parameters.
pub const fn copy_d3d9_present_parameters(
    destination: &mut D3DPRESENT_PARAMETERS8,
    source: &mtld3d_types::D3DPRESENT_PARAMETERS,
) {
    destination.back_buffer_width = source.back_buffer_width;
    destination.back_buffer_height = source.back_buffer_height;
    destination.back_buffer_format = source.back_buffer_format;
    destination.back_buffer_count = source.back_buffer_count;
    destination.multi_sample_type = source.multi_sample_type;
    destination.swap_effect = source.swap_effect;
    destination.device_window = source.device_window;
    destination.windowed = source.windowed;
    destination.enable_auto_depth_stencil = source.enable_auto_depth_stencil;
    destination.auto_depth_stencil_format = source.auto_depth_stencil_format;
    destination.flags = source.flags;
    destination.full_screen_refresh_rate_in_hz = source.full_screen_refresh_rate_in_hz;
    destination.full_screen_presentation_interval = source.presentation_interval;
}

/// D3D8's surface descriptor, which omits D3D9's multisample quality.
#[repr(C)]
pub struct D3DSURFACE_DESC8 {
    pub format: u32,
    pub resource_type: u32,
    pub usage: u32,
    pub pool: u32,
    pub size: u32,
    pub multi_sample_type: u32,
    pub width: u32,
    pub height: u32,
}

/// The part of `D3DCAPS9` D3D8 defines.
#[repr(C)]
pub struct D3DCAPS8 {
    pub device_type: u32,
    pub adapter_ordinal: u32,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub presentation_intervals: u32,
    pub cursor_caps: u32,
    pub dev_caps: u32,
    pub primitive_misc_caps: u32,
    pub raster_caps: u32,
    pub z_cmp_caps: u32,
    pub src_blend_caps: u32,
    pub dest_blend_caps: u32,
    pub alpha_cmp_caps: u32,
    pub shade_caps: u32,
    pub texture_caps: u32,
    pub texture_filter_caps: u32,
    pub cube_texture_filter_caps: u32,
    pub volume_texture_filter_caps: u32,
    pub texture_address_caps: u32,
    pub volume_texture_address_caps: u32,
    pub line_caps: u32,
    pub max_texture_width: u32,
    pub max_texture_height: u32,
    pub max_volume_extent: u32,
    pub max_texture_repeat: u32,
    pub max_texture_aspect_ratio: u32,
    pub max_anisotropy: u32,
    pub max_vertex_w: f32,
    pub guard_band_left: f32,
    pub guard_band_top: f32,
    pub guard_band_right: f32,
    pub guard_band_bottom: f32,
    pub extents_adjust: f32,
    pub stencil_caps: u32,
    pub fvf_caps: u32,
    pub texture_op_caps: u32,
    pub max_texture_blend_stages: u32,
    pub max_simultaneous_textures: u32,
    pub vertex_processing_caps: u32,
    pub max_active_lights: u32,
    pub max_user_clip_planes: u32,
    pub max_vertex_blend_matrices: u32,
    pub max_vertex_blend_matrix_index: u32,
    pub max_point_size: f32,
    pub max_primitive_count: u32,
    pub max_vertex_index: u32,
    pub max_streams: u32,
    pub max_stream_stride: u32,
    pub vertex_shader_version: u32,
    pub max_vertex_shader_const: u32,
    pub pixel_shader_version: u32,
    pub max_pixel_shader_value: f32,
}

/// Converts a D3D9 capability report to D3D8's smaller contract.
#[must_use]
pub const fn to_d3d8_caps(caps: &mtld3d_types::D3DCAPS9) -> D3DCAPS8 {
    D3DCAPS8 {
        device_type: caps.device_type,
        adapter_ordinal: caps.adapter_ordinal,
        caps: caps.caps,
        caps2: caps.caps2,
        caps3: caps.caps3,
        presentation_intervals: caps.presentation_intervals,
        cursor_caps: caps.cursor_caps,
        dev_caps: caps.dev_caps,
        primitive_misc_caps: caps.primitive_misc_caps,
        raster_caps: caps.raster_caps,
        z_cmp_caps: caps.z_cmp_caps,
        src_blend_caps: caps.src_blend_caps,
        dest_blend_caps: caps.dest_blend_caps,
        alpha_cmp_caps: caps.alpha_cmp_caps,
        shade_caps: caps.shade_caps,
        texture_caps: caps.texture_caps,
        texture_filter_caps: caps.texture_filter_caps,
        cube_texture_filter_caps: caps.cube_texture_filter_caps,
        volume_texture_filter_caps: caps.volume_texture_filter_caps,
        texture_address_caps: caps.texture_address_caps,
        volume_texture_address_caps: caps.volume_texture_address_caps,
        line_caps: caps.line_caps,
        max_texture_width: caps.max_texture_width,
        max_texture_height: caps.max_texture_height,
        max_volume_extent: caps.max_volume_extent,
        max_texture_repeat: caps.max_texture_repeat,
        max_texture_aspect_ratio: caps.max_texture_aspect_ratio,
        max_anisotropy: caps.max_anisotropy,
        max_vertex_w: caps.max_vertex_w,
        guard_band_left: caps.guard_band_left,
        guard_band_top: caps.guard_band_top,
        guard_band_right: caps.guard_band_right,
        guard_band_bottom: caps.guard_band_bottom,
        extents_adjust: caps.extents_adjust,
        stencil_caps: caps.stencil_caps,
        fvf_caps: caps.fvf_caps,
        texture_op_caps: caps.texture_op_caps,
        max_texture_blend_stages: caps.max_texture_blend_stages,
        max_simultaneous_textures: caps.max_simultaneous_textures,
        vertex_processing_caps: caps.vertex_processing_caps,
        max_active_lights: caps.max_active_lights,
        max_user_clip_planes: caps.max_user_clip_planes,
        max_vertex_blend_matrices: caps.max_vertex_blend_matrices,
        max_vertex_blend_matrix_index: caps.max_vertex_blend_matrix_index,
        max_point_size: caps.max_point_size,
        max_primitive_count: caps.max_primitive_count,
        max_vertex_index: caps.max_vertex_index,
        max_streams: caps.max_streams,
        max_stream_stride: caps.max_stream_stride,
        vertex_shader_version: min_u32(caps.vertex_shader_version, 0xfffe_0101),
        max_vertex_shader_const: min_u32(
            caps.max_vertex_shader_const,
            mtld3d_types::D3D8_MAX_VERTEX_SHADER_CONSTANTS,
        ),
        pixel_shader_version: min_u32(caps.pixel_shader_version, 0xffff_0104),
        max_pixel_shader_value: caps.pixel_shader_1x_max_value,
    }
}

const fn min_u32(value: u32, maximum: u32) -> u32 {
    if value > maximum { maximum } else { value }
}

/// Vtable for `IDirect3D8` in Wine's `d3d8.h` order.
#[repr(C)]
pub struct IDirect3D8Vtbl {
    pub query_interface:
        unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    pub add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub register_software_device: unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32,
    pub get_adapter_count: unsafe extern "system" fn(*mut c_void) -> u32,
    pub get_adapter_identifier:
        unsafe extern "system" fn(*mut c_void, u32, u32, *mut D3DADAPTER_IDENTIFIER8) -> i32,
    pub get_adapter_mode_count: unsafe extern "system" fn(*mut c_void, u32) -> u32,
    pub enum_adapter_modes:
        unsafe extern "system" fn(*mut c_void, u32, u32, *mut D3DDISPLAYMODE) -> i32,
    pub get_adapter_display_mode:
        unsafe extern "system" fn(*mut c_void, u32, *mut D3DDISPLAYMODE) -> i32,
    pub check_device_type: unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, i32) -> i32,
    pub check_device_format:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, u32, u32) -> i32,
    pub check_device_multi_sample_type:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, i32, u32) -> i32,
    pub check_depth_stencil_match:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, u32) -> i32,
    pub get_device_caps: unsafe extern "system" fn(*mut c_void, u32, u32, *mut D3DCAPS8) -> i32,
    pub get_adapter_monitor: unsafe extern "system" fn(*mut c_void, u32) -> *mut c_void,
    pub create_device: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        usize,
        u32,
        *mut D3DPRESENT_PARAMETERS8,
        *mut *mut c_void,
    ) -> i32,
}

/// Vtable for `IDirect3DSurface8`; device-facing resource calls are supplied by the frontend.
#[repr(C)]
pub struct IDirect3DSurface8Vtbl {
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
    pub get_desc: unsafe extern "system" fn(*mut c_void, *mut D3DSURFACE_DESC8) -> i32,
    pub lock_rect:
        unsafe extern "system" fn(*mut c_void, *mut D3DLOCKED_RECT, *const c_void, u32) -> i32,
    pub unlock_rect: unsafe extern "system" fn(*mut c_void) -> i32,
}

#[cfg(test)]
mod tests;
