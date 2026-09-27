//! Owned D3D8 surface references and lock-based readback.

use core::{ffi::c_void, marker::PhantomData};

use mtld3d_d3d8_types::{D3DSURFACE_DESC8, IDirect3DSurface8Vtbl};
use mtld3d_types::{D3DLOCK_READONLY, D3DLOCKED_RECT, Guid};

use super::{D3D8Harness, D3D8Texture};
use crate::{
    check::{expect_created, expect_ok},
    resource::release_unknown,
    vtbl::deref_vtbl,
};

/// A surface reference whose harness outlives every call and its release.
pub struct D3D8Surface<'a> {
    pointer: *mut c_void,
    owner: PhantomData<&'a D3D8Harness>,
}

impl D3D8Surface<'_> {
    pub(super) const fn pointer(&self) -> *mut c_void {
        self.pointer
    }

    /// Adopts the reference returned by a successful D3D8 surface getter.
    ///
    /// # Safety
    /// The pointer must own one live `IDirect3DSurface8` reference.
    pub const unsafe fn from_owned(pointer: *mut c_void) -> Self {
        Self {
            pointer,
            owner: PhantomData,
        }
    }

    /// Whether both getters returned the same COM surface identity.
    #[must_use]
    pub fn is_same_object(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }

    /// Queries the container and compares its identity to the creating device.
    ///
    /// # Errors
    /// Returns the HRESULT if the container does not expose the requested interface.
    ///
    /// # Panics
    /// Panics if a successful query returns no interface.
    pub fn container_is_device(&self, owner: &D3D8Harness, iid: &Guid) -> Result<bool, i32> {
        self.container_matches(owner.device, iid)
    }

    /// Queries the container and compares it to the creating texture.
    ///
    /// # Errors
    /// Returns the HRESULT if the requested container interface is unavailable.
    ///
    /// # Panics
    /// Panics if query success and output disagree.
    pub fn container_is_texture(&self, owner: &D3D8Texture<'_>, iid: &Guid) -> Result<bool, i32> {
        self.container_matches(owner.pointer, iid)
    }

    fn container_matches(&self, expected: *mut c_void, iid: &Guid) -> Result<bool, i32> {
        let mut pointer = core::ptr::null_mut();
        // SAFETY: the surface, IID and output remain valid throughout the call.
        let result = unsafe { (self.vtable().get_container)(self.pointer, iid, &raw mut pointer) };
        if result < 0 {
            assert!(pointer.is_null(), "failed GetContainer retained an output");
            return Err(result);
        }
        expect_created(result, pointer, "D3D8 surface GetContainer");
        let matches = pointer == expected;
        // SAFETY: successful GetContainer returned an owned COM reference.
        unsafe { release_unknown(pointer) };
        Ok(matches)
    }

    /// Returns the D3D8 descriptor, including its byte-size field.
    ///
    /// # Panics
    /// Panics if `GetDesc` fails.
    #[must_use]
    pub fn desc(&self) -> D3DSURFACE_DESC8 {
        let mut desc = core::mem::MaybeUninit::uninit();
        // SAFETY: the reference is live and desc can hold one descriptor.
        let result = unsafe { (self.vtable().get_desc)(self.pointer, desc.as_mut_ptr()) };
        expect_ok(result, "D3D8 surface GetDesc");
        // SAFETY: successful GetDesc initialized every descriptor field.
        unsafe { desc.assume_init() }
    }

    /// Reads one A8R8G8B8 texel from a lockable surface.
    ///
    /// # Panics
    /// Panics if the format or coordinates are invalid, or locking fails.
    #[must_use]
    pub fn read_pixel(&self, x: u32, y: u32) -> u32 {
        let desc = self.desc();
        assert_eq!(desc.format, mtld3d_types::D3DFMT_A8R8G8B8);
        assert!(x < desc.width && y < desc.height);
        let mut locked = D3DLOCKED_RECT {
            pitch: 0,
            bits: core::ptr::null_mut(),
        };
        // SAFETY: the surface is live and locked is writable; null rect locks the whole surface.
        let result = unsafe {
            (self.vtable().lock_rect)(
                self.pointer,
                &raw mut locked,
                core::ptr::null(),
                D3DLOCK_READONLY,
            )
        };
        expect_ok(result, "D3D8 surface LockRect");
        let pitch = usize::try_from(locked.pitch).expect("positive surface pitch");
        let offset = usize::try_from(y).expect("coordinate fits usize") * pitch
            + usize::try_from(x).expect("coordinate fits usize") * size_of::<u32>();
        // SAFETY: x/y are inside the descriptor; the successful lock covers every row at pitch.
        let address = unsafe { locked.bits.cast::<u8>().add(offset) };
        // SAFETY: the in-bounds A8R8G8B8 pixel occupies four bytes; read_unaligned imposes no alignment.
        let pixel = unsafe { address.cast::<u32>().read_unaligned() };
        // SAFETY: this method holds the successful lock and releases it once.
        expect_ok(
            unsafe { (self.vtable().unlock_rect)(self.pointer) },
            "D3D8 surface UnlockRect",
        );
        pixel
    }

    fn vtable(&self) -> &'static IDirect3DSurface8Vtbl {
        // SAFETY: this wrapper owns a live IDirect3DSurface8 reference.
        unsafe { deref_vtbl(self.pointer) }
    }
}

impl Drop for D3D8Surface<'_> {
    fn drop(&mut self) {
        // SAFETY: release the one reference adopted by from_owned.
        unsafe { (self.vtable().release)(self.pointer) };
    }
}
