//! Volume texture levels, byte size, mapping, and retained container identity.

use core::{mem::MaybeUninit, ptr};

use mtld3d_d3d8_types::{
    D3DVOLUME_DESC8, IDirect3DVolume8Vtbl, IDirect3DVolumeTexture8Vtbl, IID_IDIRECT3DVOLUMETEXTURE8,
};
use mtld3d_types::{D3DFMT_A8R8G8B8, D3DLOCK_READONLY, D3DLOCKED_BOX, D3DPOOL_MANAGED};

use super::{D3D8Harness, D3D8Resource};
use crate::{
    check::{expect_created, expect_ok},
    resource::release_unknown,
};

impl D3D8Harness {
    /// Creates a managed volume texture with every mip level.
    ///
    /// # Panics
    /// Panics if creation fails.
    #[must_use]
    pub fn create_volume_texture8(
        &self,
        width: u32,
        height: u32,
        depth: u32,
    ) -> D3D8Resource<'_, IDirect3DVolumeTexture8Vtbl> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns the device and pointer is writable output.
        let result = unsafe {
            (self.device_vtable().create_volume_texture)(
                self.device,
                width,
                height,
                depth,
                0,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateVolumeTexture");
        // SAFETY: successful creation returned one owned volume texture reference.
        unsafe { D3D8Resource::from_owned(pointer) }
    }
}

impl<'a> D3D8Resource<'a, IDirect3DVolumeTexture8Vtbl> {
    /// Number of volume mip levels.
    #[must_use]
    pub fn level_count(&self) -> u32 {
        // SAFETY: this wrapper owns the texture reference.
        unsafe { (self.vtable().get_level_count)(self.pointer) }
    }

    /// Returns one volume mip descriptor with its full byte size.
    ///
    /// # Panics
    /// Panics if the level is invalid.
    #[must_use]
    pub fn level_desc(&self, level: u32) -> D3DVOLUME_DESC8 {
        let mut descriptor = MaybeUninit::uninit();
        // SAFETY: this wrapper owns the texture and descriptor is writable output.
        let result =
            unsafe { (self.vtable().get_level_desc)(self.pointer, level, descriptor.as_mut_ptr()) };
        expect_ok(result, "D3D8 volume texture GetLevelDesc");
        // SAFETY: successful GetLevelDesc initialized every output field.
        unsafe { descriptor.assume_init() }
    }

    /// Obtains an owned volume level independent of this texture reference.
    ///
    /// # Errors
    /// Returns the HRESULT for an invalid mip level.
    pub fn level(&self, level: u32) -> Result<D3D8Resource<'a, IDirect3DVolume8Vtbl>, i32> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the texture is owned and pointer is writable output.
        let result =
            unsafe { (self.vtable().get_volume_level)(self.pointer, level, &raw mut pointer) };
        if result < 0 {
            return Err(result);
        }
        expect_created(result, pointer, "D3D8 GetVolumeLevel");
        // SAFETY: successful GetVolumeLevel returned one owned volume interface.
        Ok(unsafe { D3D8Resource::from_owned(pointer) })
    }
}

impl D3D8Resource<'_, IDirect3DVolume8Vtbl> {
    /// Whether `GetContainer` returns the supplied frontend texture.
    ///
    /// # Panics
    /// Panics if the container query fails.
    #[must_use]
    pub fn container_matches(
        &self,
        texture: &D3D8Resource<'_, IDirect3DVolumeTexture8Vtbl>,
    ) -> bool {
        let mut pointer = ptr::null_mut();
        // SAFETY: this owned volume, IID, and writable output remain live throughout the call.
        let result = unsafe {
            (self.vtable().get_container)(
                self.pointer,
                &IID_IDIRECT3DVOLUMETEXTURE8,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 volume GetContainer");
        let matches = pointer == texture.pointer;
        // SAFETY: GetContainer supplied one owned reference.
        unsafe { release_unknown(pointer) };
        matches
    }

    /// Returns the level's dimensions and byte size.
    ///
    /// # Panics
    /// Panics if `GetDesc` fails.
    #[must_use]
    pub fn desc(&self) -> D3DVOLUME_DESC8 {
        let mut descriptor = MaybeUninit::uninit();
        // SAFETY: this wrapper owns the volume and descriptor is writable output.
        let result = unsafe { (self.vtable().get_desc)(self.pointer, descriptor.as_mut_ptr()) };
        expect_ok(result, "D3D8 volume GetDesc");
        // SAFETY: successful GetDesc initialized every output field.
        unsafe { descriptor.assume_init() }
    }

    /// Writes the first texel of a mapped A8R8G8B8 level.
    ///
    /// # Panics
    /// Panics if the format or lock is invalid.
    pub fn write_first_texel(&self, value: u32) {
        assert_eq!(self.desc().format, D3DFMT_A8R8G8B8);
        let mapping = self.lock(0);
        // SAFETY: a successful whole-level lock includes at least one writable A8R8G8B8 texel.
        unsafe { mapping.bits.cast::<u32>().write_unaligned(value) };
        self.unlock();
    }

    /// Reads the first texel of a mapped A8R8G8B8 level.
    ///
    /// # Panics
    /// Panics if the format or lock is invalid.
    #[must_use]
    pub fn read_first_texel(&self) -> u32 {
        assert_eq!(self.desc().format, D3DFMT_A8R8G8B8);
        let mapping = self.lock(D3DLOCK_READONLY);
        // SAFETY: a successful whole-level lock includes at least one readable A8R8G8B8 texel.
        let value = unsafe { mapping.bits.cast::<u32>().read_unaligned() };
        self.unlock();
        value
    }

    fn lock(&self, flags: u32) -> D3DLOCKED_BOX {
        let mut mapping = MaybeUninit::uninit();
        // SAFETY: the whole-level box is null and mapping is writable output.
        let result = unsafe {
            (self.vtable().lock_box)(self.pointer, mapping.as_mut_ptr(), ptr::null(), flags)
        };
        expect_ok(result, "D3D8 volume LockBox");
        // SAFETY: successful LockBox initialized the output.
        let mapping = unsafe { mapping.assume_init() };
        assert!(!mapping.bits.is_null());
        mapping
    }

    fn unlock(&self) {
        // SAFETY: the immediately preceding operation acquired exactly one lock.
        expect_ok(
            unsafe { (self.vtable().unlock_box)(self.pointer) },
            "D3D8 volume UnlockBox",
        );
    }
}
