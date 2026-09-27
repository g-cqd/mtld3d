//! Immutable D3D8 resource vtables selected by concrete backend type.

use core::ffi::c_void;

use mtld3d_d3d8_types::{
    IDirect3DCubeTexture8Vtbl, IDirect3DIndexBuffer8Vtbl, IDirect3DSurface8Vtbl,
    IDirect3DTexture8Vtbl, IDirect3DVertexBuffer8Vtbl, IDirect3DVolume8Vtbl,
    IDirect3DVolumeTexture8Vtbl,
};

use super::{
    add_ref, buffers, free_private_data, get_device, get_priority, get_private_data, get_type,
    kind::ResourceKind, pre_load, query_interface, release, set_priority, set_private_data,
    surfaces, textures,
};

static TEXTURE: IDirect3DTexture8Vtbl = IDirect3DTexture8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    set_priority,
    get_priority,
    pre_load,
    get_type,
    set_lod: textures::set_lod,
    get_lod: textures::get_lod,
    get_level_count: textures::get_level_count,
    get_level_desc: textures::texture_get_level_desc,
    get_surface_level: textures::texture_get_surface_level,
    lock_rect: textures::texture_lock_rect,
    unlock_rect: textures::texture_unlock_rect,
    add_dirty_rect: textures::texture_add_dirty_rect,
};

static CUBE_TEXTURE: IDirect3DCubeTexture8Vtbl = IDirect3DCubeTexture8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    set_priority,
    get_priority,
    pre_load,
    get_type,
    set_lod: textures::set_lod,
    get_lod: textures::get_lod,
    get_level_count: textures::get_level_count,
    get_level_desc: textures::cube_get_level_desc,
    get_cube_map_surface: textures::cube_get_cube_map_surface,
    lock_rect: textures::cube_lock_rect,
    unlock_rect: textures::cube_unlock_rect,
    add_dirty_rect: textures::cube_add_dirty_rect,
};

static VOLUME_TEXTURE: IDirect3DVolumeTexture8Vtbl = IDirect3DVolumeTexture8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    set_priority,
    get_priority,
    pre_load,
    get_type,
    set_lod: textures::set_lod,
    get_lod: textures::get_lod,
    get_level_count: textures::get_level_count,
    get_level_desc: textures::volume_texture_get_level_desc,
    get_volume_level: textures::volume_texture_get_volume_level,
    lock_box: textures::volume_texture_lock_box,
    unlock_box: textures::volume_texture_unlock_box,
    add_dirty_box: textures::volume_texture_add_dirty_box,
};

static VERTEX_BUFFER: IDirect3DVertexBuffer8Vtbl = IDirect3DVertexBuffer8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    set_priority,
    get_priority,
    pre_load,
    get_type,
    lock: buffers::vertex_lock,
    unlock: buffers::vertex_unlock,
    get_desc: buffers::vertex_get_desc,
};

static INDEX_BUFFER: IDirect3DIndexBuffer8Vtbl = IDirect3DIndexBuffer8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    set_priority,
    get_priority,
    pre_load,
    get_type,
    lock: buffers::index_lock,
    unlock: buffers::index_unlock,
    get_desc: buffers::index_get_desc,
};

static VOLUME: IDirect3DVolume8Vtbl = IDirect3DVolume8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    get_container: surfaces::get_container,
    get_desc: surfaces::volume_get_desc,
    lock_box: surfaces::volume_lock_box,
    unlock_box: surfaces::volume_unlock_box,
};

static SURFACE: IDirect3DSurface8Vtbl = IDirect3DSurface8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    get_container: surfaces::get_container,
    get_desc: surfaces::surface_get_desc,
    lock_rect: surfaces::surface_lock_rect,
    unlock_rect: surfaces::surface_unlock_rect,
};

pub const fn for_kind(kind: &ResourceKind) -> *const c_void {
    match kind {
        ResourceKind::Texture => (&raw const TEXTURE).cast(),
        ResourceKind::CubeTexture => (&raw const CUBE_TEXTURE).cast(),
        ResourceKind::VolumeTexture => (&raw const VOLUME_TEXTURE).cast(),
        ResourceKind::VertexBuffer => (&raw const VERTEX_BUFFER).cast(),
        ResourceKind::IndexBuffer => (&raw const INDEX_BUFFER).cast(),
        ResourceKind::Volume => (&raw const VOLUME).cast(),
        ResourceKind::Surface => (&raw const SURFACE).cast(),
    }
}

pub fn is_known(table: *const c_void) -> bool {
    table == (&raw const TEXTURE).cast()
        || table == (&raw const CUBE_TEXTURE).cast()
        || table == (&raw const VOLUME_TEXTURE).cast()
        || table == (&raw const VERTEX_BUFFER).cast()
        || table == (&raw const INDEX_BUFFER).cast()
        || table == (&raw const VOLUME).cast()
        || table == (&raw const SURFACE).cast()
}
