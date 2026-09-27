//! Owned D3D8 texture references and their level surfaces.

use core::{ffi::c_void, marker::PhantomData};

use mtld3d_d3d8_types::{D3DSURFACE_DESC8, IDirect3DTexture8Vtbl};
use mtld3d_types::{D3DFMT_A8R8G8B8, D3DPOOL_MANAGED, Guid};

use super::{D3D8Harness, D3D8Surface};
use crate::{
    check::{expect_created, expect_ok},
    resource::release_unknown,
    vtbl::deref_vtbl,
};

/// A texture reference whose creating harness remains alive.
pub struct D3D8Texture<'a> {
    pub(super) pointer: *mut c_void,
    owner: PhantomData<&'a D3D8Harness>,
}

impl D3D8Harness {
    /// Creates a managed A8R8G8B8 texture with a complete mip chain.
    ///
    /// # Panics
    /// Panics if texture creation fails.
    #[must_use]
    pub fn create_texture(&self, width: u32, height: u32) -> D3D8Texture<'_> {
        let mut pointer = core::ptr::null_mut();
        // SAFETY: this harness owns its device and the output slot is writable.
        let result = unsafe {
            (self.device_vtable().create_texture)(
                self.device,
                width,
                height,
                0,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateTexture");
        D3D8Texture {
            pointer,
            owner: PhantomData,
        }
    }

    /// Binds a texture, or unbinds the selected stage.
    #[must_use]
    pub fn set_texture(&self, stage: u32, texture: Option<&D3D8Texture<'_>>) -> i32 {
        let pointer = texture.map_or(core::ptr::null_mut(), |texture| texture.pointer);
        // SAFETY: device and optional texture references remain live throughout the call.
        unsafe { (self.device_vtable().set_texture)(self.device, stage, pointer) }
    }

    /// Checks a stage's binding against an owned texture reference.
    ///
    /// # Panics
    /// Panics if the binding query fails.
    #[must_use]
    pub fn texture_binding_matches(&self, stage: u32, texture: Option<&D3D8Texture<'_>>) -> bool {
        let mut pointer = core::ptr::null_mut();
        // SAFETY: device is live and pointer is writable.
        let result =
            unsafe { (self.device_vtable().get_texture)(self.device, stage, &raw mut pointer) };
        expect_ok(result, "D3D8 GetTexture");
        let same = pointer == texture.map_or(core::ptr::null_mut(), |texture| texture.pointer);
        if !pointer.is_null() {
            // SAFETY: GetTexture returned one owned COM reference.
            unsafe { release_unknown(pointer) };
        }
        same
    }
}

impl<'a> D3D8Texture<'a> {
    /// Number of mip levels in this texture.
    #[must_use]
    pub fn level_count(&self) -> u32 {
        // SAFETY: this wrapper owns the texture reference.
        unsafe { (self.vtable().get_level_count)(self.pointer) }
    }

    /// Returns a mip descriptor with the D3D8 byte-size field.
    ///
    /// # Panics
    /// Panics if the descriptor query fails.
    #[must_use]
    pub fn level_desc(&self, level: u32) -> D3DSURFACE_DESC8 {
        let mut descriptor = core::mem::MaybeUninit::uninit();
        // SAFETY: the reference is live and descriptor holds a complete output.
        let result =
            unsafe { (self.vtable().get_level_desc)(self.pointer, level, descriptor.as_mut_ptr()) };
        expect_ok(result, "D3D8 texture GetLevelDesc");
        // SAFETY: a successful descriptor query initialized every field.
        unsafe { descriptor.assume_init() }
    }

    /// Obtains an independently owned level surface.
    ///
    /// # Panics
    /// Panics if the level query fails.
    #[must_use]
    pub fn surface(&self, level: u32) -> D3D8Surface<'a> {
        let mut pointer = core::ptr::null_mut();
        // SAFETY: the texture is live and pointer is writable.
        let result =
            unsafe { (self.vtable().get_surface_level)(self.pointer, level, &raw mut pointer) };
        expect_created(result, pointer, "D3D8 GetSurfaceLevel");
        // SAFETY: the getter returned an owned D3D8 surface reference.
        unsafe { D3D8Surface::from_owned(pointer) }
    }

    /// Queries an interface and checks its canonical texture identity.
    ///
    /// # Errors
    /// Returns the query's HRESULT for unsupported interfaces.
    ///
    /// # Panics
    /// Panics if query success and output disagree.
    pub fn query_preserves_identity(&self, iid: &Guid) -> Result<bool, i32> {
        let mut pointer = core::ptr::null_mut();
        // SAFETY: texture and IID are live, and pointer is writable.
        let result =
            unsafe { (self.vtable().query_interface)(self.pointer, iid, &raw mut pointer) };
        if result < 0 {
            assert!(
                pointer.is_null(),
                "failed QueryInterface retained an output"
            );
            return Err(result);
        }
        expect_created(result, pointer, "D3D8 texture QueryInterface");
        let same = pointer == self.pointer;
        // SAFETY: QueryInterface returned one owned COM reference.
        unsafe { release_unknown(pointer) };
        Ok(same)
    }

    /// Checks the resource's device identity through its D3D8 parent getter.
    ///
    /// # Panics
    /// Panics if the parent getter fails.
    #[must_use]
    pub fn belongs_to(&self, harness: &D3D8Harness) -> bool {
        let mut pointer = core::ptr::null_mut();
        // SAFETY: the resource is live and pointer is writable.
        let result = unsafe { (self.vtable().get_device)(self.pointer, &raw mut pointer) };
        expect_created(result, pointer, "D3D8 texture GetDevice");
        let same = pointer == harness.device;
        // SAFETY: GetDevice returned one owned COM reference.
        unsafe { release_unknown(pointer) };
        same
    }

    fn vtable(&self) -> &'static IDirect3DTexture8Vtbl {
        // SAFETY: this wrapper owns a live IDirect3DTexture8 reference.
        unsafe { deref_vtbl(self.pointer) }
    }
}

impl Drop for D3D8Texture<'_> {
    fn drop(&mut self) {
        // SAFETY: release the reference acquired by CreateTexture.
        unsafe { (self.vtable().release)(self.pointer) };
    }
}
