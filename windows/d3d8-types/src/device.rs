//! Frozen `IDirect3DDevice8` method ordering and signatures.

use core::ffi::c_void;

use mtld3d_types::{
    D3DDEVICE_CREATION_PARAMETERS, D3DDISPLAYMODE, D3DGAMMARAMP, D3DLIGHT9, D3DMATERIAL9,
    D3DMATRIX, D3DRECT, D3DVIEWPORT9, Guid, POINT,
};

use crate::{D3DCAPS8, D3DPRESENT_PARAMETERS8};

/// The 97-entry `IDirect3DDevice8` vtable, including `IUnknown`.
#[repr(C)]
pub struct IDirect3DDevice8Vtbl {
    pub query_interface:
        unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    pub add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub test_cooperative_level: unsafe extern "system" fn(*mut c_void) -> i32,
    pub get_available_texture_mem: unsafe extern "system" fn(*mut c_void) -> u32,
    pub resource_manager_discard_bytes: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub get_direct3d: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32,
    pub get_device_caps: unsafe extern "system" fn(*mut c_void, *mut D3DCAPS8) -> i32,
    pub get_display_mode: unsafe extern "system" fn(*mut c_void, *mut D3DDISPLAYMODE) -> i32,
    pub get_creation_parameters:
        unsafe extern "system" fn(*mut c_void, *mut D3DDEVICE_CREATION_PARAMETERS) -> i32,
    pub set_cursor_properties: unsafe extern "system" fn(*mut c_void, u32, u32, *mut c_void) -> i32,
    pub set_cursor_position: unsafe extern "system" fn(*mut c_void, u32, u32, u32),
    pub show_cursor: unsafe extern "system" fn(*mut c_void, i32) -> i32,
    pub create_additional_swap_chain: unsafe extern "system" fn(
        *mut c_void,
        *mut D3DPRESENT_PARAMETERS8,
        *mut *mut c_void,
    ) -> i32,
    pub reset: unsafe extern "system" fn(*mut c_void, *mut D3DPRESENT_PARAMETERS8) -> i32,
    pub present: unsafe extern "system" fn(
        *mut c_void,
        *const c_void,
        *const c_void,
        *mut c_void,
        *const c_void,
    ) -> i32,
    pub get_back_buffer: unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void) -> i32,
    pub get_raster_status: unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32,
    pub set_gamma_ramp: unsafe extern "system" fn(*mut c_void, u32, *const D3DGAMMARAMP),
    pub get_gamma_ramp: unsafe extern "system" fn(*mut c_void, *mut D3DGAMMARAMP),
    pub create_texture: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        *mut *mut c_void,
    ) -> i32,
    pub create_volume_texture: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        *mut *mut c_void,
    ) -> i32,
    pub create_cube_texture:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, u32, *mut *mut c_void) -> i32,
    pub create_vertex_buffer:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, *mut *mut c_void) -> i32,
    pub create_index_buffer:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, *mut *mut c_void) -> i32,
    pub create_render_target:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, i32, *mut *mut c_void) -> i32,
    pub create_depth_stencil_surface:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, *mut *mut c_void) -> i32,
    pub create_image_surface:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, *mut *mut c_void) -> i32,
    pub copy_rects: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const D3DRECT,
        u32,
        *mut c_void,
        *const POINT,
    ) -> i32,
    pub update_texture: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> i32,
    pub get_front_buffer: unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32,
    pub set_render_target: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> i32,
    pub get_render_target: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32,
    pub get_depth_stencil_surface: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32,
    pub begin_scene: unsafe extern "system" fn(*mut c_void) -> i32,
    pub end_scene: unsafe extern "system" fn(*mut c_void) -> i32,
    pub clear:
        unsafe extern "system" fn(*mut c_void, u32, *const c_void, u32, u32, f32, u32) -> i32,
    pub set_transform: unsafe extern "system" fn(*mut c_void, u32, *const D3DMATRIX) -> i32,
    pub get_transform: unsafe extern "system" fn(*mut c_void, u32, *mut D3DMATRIX) -> i32,
    pub multiply_transform: unsafe extern "system" fn(*mut c_void, u32, *const D3DMATRIX) -> i32,
    pub set_viewport: unsafe extern "system" fn(*mut c_void, *const D3DVIEWPORT9) -> i32,
    pub get_viewport: unsafe extern "system" fn(*mut c_void, *mut D3DVIEWPORT9) -> i32,
    pub set_material: unsafe extern "system" fn(*mut c_void, *const D3DMATERIAL9) -> i32,
    pub get_material: unsafe extern "system" fn(*mut c_void, *mut D3DMATERIAL9) -> i32,
    pub set_light: unsafe extern "system" fn(*mut c_void, u32, *const D3DLIGHT9) -> i32,
    pub get_light: unsafe extern "system" fn(*mut c_void, u32, *mut D3DLIGHT9) -> i32,
    pub light_enable: unsafe extern "system" fn(*mut c_void, u32, i32) -> i32,
    pub get_light_enable: unsafe extern "system" fn(*mut c_void, u32, *mut i32) -> i32,
    pub set_clip_plane: unsafe extern "system" fn(*mut c_void, u32, *const f32) -> i32,
    pub get_clip_plane: unsafe extern "system" fn(*mut c_void, u32, *mut f32) -> i32,
    pub set_render_state: unsafe extern "system" fn(*mut c_void, u32, u32) -> i32,
    pub get_render_state: unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> i32,
    pub begin_state_block: unsafe extern "system" fn(*mut c_void) -> i32,
    pub end_state_block: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    pub apply_state_block: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub capture_state_block: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub delete_state_block: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub create_state_block: unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> i32,
    pub set_clip_status: unsafe extern "system" fn(*mut c_void, *const c_void) -> i32,
    pub get_clip_status: unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32,
    pub get_texture: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> i32,
    pub set_texture: unsafe extern "system" fn(*mut c_void, u32, *mut c_void) -> i32,
    pub get_texture_stage_state: unsafe extern "system" fn(*mut c_void, u32, u32, *mut u32) -> i32,
    pub set_texture_stage_state: unsafe extern "system" fn(*mut c_void, u32, u32, u32) -> i32,
    pub validate_device: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    pub get_info: unsafe extern "system" fn(*mut c_void, u32, *mut c_void, u32) -> i32,
    pub set_palette_entries: unsafe extern "system" fn(*mut c_void, u32, *const c_void) -> i32,
    pub get_palette_entries: unsafe extern "system" fn(*mut c_void, u32, *mut c_void) -> i32,
    pub set_current_texture_palette: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub get_current_texture_palette: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    pub draw_primitive: unsafe extern "system" fn(*mut c_void, u32, u32, u32) -> i32,
    pub draw_indexed_primitive:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, u32, u32) -> i32,
    pub draw_primitive_up:
        unsafe extern "system" fn(*mut c_void, u32, u32, *const c_void, u32) -> i32,
    pub draw_indexed_primitive_up: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        u32,
        u32,
        *const c_void,
        u32,
        *const c_void,
        u32,
    ) -> i32,
    pub process_vertices:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, *mut c_void, u32) -> i32,
    pub create_vertex_shader:
        unsafe extern "system" fn(*mut c_void, *const u32, *const u32, *mut u32, u32) -> i32,
    pub set_vertex_shader: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub get_vertex_shader: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    pub delete_vertex_shader: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub set_vertex_shader_constant:
        unsafe extern "system" fn(*mut c_void, u32, *const c_void, u32) -> i32,
    pub get_vertex_shader_constant:
        unsafe extern "system" fn(*mut c_void, u32, *mut c_void, u32) -> i32,
    pub get_vertex_shader_declaration:
        unsafe extern "system" fn(*mut c_void, u32, *mut c_void, *mut u32) -> i32,
    pub get_vertex_shader_function:
        unsafe extern "system" fn(*mut c_void, u32, *mut c_void, *mut u32) -> i32,
    pub set_stream_source: unsafe extern "system" fn(*mut c_void, u32, *mut c_void, u32) -> i32,
    pub get_stream_source:
        unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void, *mut u32) -> i32,
    pub set_indices: unsafe extern "system" fn(*mut c_void, *mut c_void, u32) -> i32,
    pub get_indices: unsafe extern "system" fn(*mut c_void, *mut *mut c_void, *mut u32) -> i32,
    pub create_pixel_shader: unsafe extern "system" fn(*mut c_void, *const u32, *mut u32) -> i32,
    pub set_pixel_shader: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub get_pixel_shader: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32,
    pub delete_pixel_shader: unsafe extern "system" fn(*mut c_void, u32) -> i32,
    pub set_pixel_shader_constant:
        unsafe extern "system" fn(*mut c_void, u32, *const c_void, u32) -> i32,
    pub get_pixel_shader_constant:
        unsafe extern "system" fn(*mut c_void, u32, *mut c_void, u32) -> i32,
    pub get_pixel_shader_function:
        unsafe extern "system" fn(*mut c_void, u32, *mut c_void, *mut u32) -> i32,
    pub draw_rect_patch:
        unsafe extern "system" fn(*mut c_void, u32, *const f32, *const c_void) -> i32,
    pub draw_tri_patch:
        unsafe extern "system" fn(*mut c_void, u32, *const f32, *const c_void) -> i32,
    pub delete_patch: unsafe extern "system" fn(*mut c_void, u32) -> i32,
}
