//! Cube texture face identity and mip descriptors.

use core::{mem::MaybeUninit, ptr};

use mtld3d_d3d8_types::{D3DSURFACE_DESC8, IDirect3DCubeTexture8Vtbl};
use mtld3d_types::{D3DFMT_A8R8G8B8, D3DPOOL_MANAGED};

use super::{D3D8Harness, D3D8Resource};
use crate::{
    check::{expect_created, expect_ok},
    d3d8::D3D8Surface,
};

impl D3D8Harness {
    /// Creates a managed cube texture with every mip level.
    ///
    /// # Panics
    /// Panics if creation fails.
    #[must_use]
    pub fn create_cube_texture8(&self, edge: u32) -> D3D8Resource<'_, IDirect3DCubeTexture8Vtbl> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns the device and pointer is writable output.
        let result = unsafe {
            (self.device_vtable().create_cube_texture)(
                self.device,
                edge,
                0,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateCubeTexture");
        // SAFETY: successful creation returned one owned cube texture reference.
        unsafe { D3D8Resource::from_owned(pointer) }
    }
}

impl<'a> D3D8Resource<'a, IDirect3DCubeTexture8Vtbl> {
    /// Number of levels in each face.
    #[must_use]
    pub fn level_count(&self) -> u32 {
        // SAFETY: this wrapper owns the cube texture reference.
        unsafe { (self.vtable().get_level_count)(self.pointer) }
    }

    /// Returns one level's descriptor with the D3D8 size field.
    ///
    /// # Panics
    /// Panics if the level is invalid.
    #[must_use]
    pub fn level_desc(&self, level: u32) -> D3DSURFACE_DESC8 {
        let mut descriptor = MaybeUninit::uninit();
        // SAFETY: this wrapper owns the texture and descriptor is writable output.
        let result =
            unsafe { (self.vtable().get_level_desc)(self.pointer, level, descriptor.as_mut_ptr()) };
        expect_ok(result, "D3D8 cube GetLevelDesc");
        // SAFETY: the successful getter initialized all descriptor fields.
        unsafe { descriptor.assume_init() }
    }

    /// Obtains an owned face surface whose lifetime is independent of this reference.
    ///
    /// # Errors
    /// Returns the HRESULT for an invalid face or mip level.
    pub fn face(&self, face: u32, level: u32) -> Result<D3D8Surface<'a>, i32> {
        let mut pointer = ptr::null_mut();
        // SAFETY: this wrapper owns the texture and pointer is writable output.
        let result = unsafe {
            (self.vtable().get_cube_map_surface)(self.pointer, face, level, &raw mut pointer)
        };
        if result < 0 {
            return Err(result);
        }
        expect_created(result, pointer, "D3D8 GetCubeMapSurface");
        // SAFETY: successful retrieval returned one owned D3D8 surface reference.
        Ok(unsafe { D3D8Surface::from_owned(pointer) })
    }
}
