//! Typed access to the private D3D9 COM implementation.

use core::ffi::c_void;

use mtld3d_types::{
    Guid, IDirect3D9Vtbl, IDirect3DCubeTexture9Vtbl, IDirect3DDevice9Vtbl,
    IDirect3DIndexBuffer9Vtbl, IDirect3DPixelShader9Vtbl, IDirect3DStateBlock9Vtbl,
    IDirect3DSurface9Vtbl, IDirect3DSwapChain9Vtbl, IDirect3DTexture9Vtbl,
    IDirect3DVertexBuffer9Vtbl, IDirect3DVertexDeclaration9Vtbl, IDirect3DVertexShader9Vtbl,
    IDirect3DVolume9Vtbl, IDirect3DVolumeTexture9Vtbl,
};

/// A backend vtable with an owning `IUnknown` reference.
pub trait BackendVtable {
    fn release(&self) -> unsafe extern "system" fn(*mut c_void) -> u32;
    fn query_interface(
        &self,
    ) -> unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32;
}

macro_rules! backend_vtables {
    ($($table:ty),+ $(,)?) => {
        $(impl BackendVtable for $table {
            fn release(&self) -> unsafe extern "system" fn(*mut c_void) -> u32 { self.release }
            fn query_interface(&self) -> unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32 { self.query_interface }
        })+
    };
}
backend_vtables!(
    IDirect3D9Vtbl,
    IDirect3DDevice9Vtbl,
    IDirect3DSurface9Vtbl,
    IDirect3DTexture9Vtbl,
    IDirect3DCubeTexture9Vtbl,
    IDirect3DVolumeTexture9Vtbl,
    IDirect3DVertexBuffer9Vtbl,
    IDirect3DIndexBuffer9Vtbl,
    IDirect3DVolume9Vtbl,
    IDirect3DVertexShader9Vtbl,
    IDirect3DPixelShader9Vtbl,
    IDirect3DVertexDeclaration9Vtbl,
    IDirect3DStateBlock9Vtbl,
    IDirect3DSwapChain9Vtbl
);

/// Owns one reference to an interface whose vtable type is fixed at construction.
pub struct Backend<V: BackendVtable> {
    pointer: *mut c_void,
    table: *const V,
}

impl<V: BackendVtable> Backend<V> {
    /// Adopts the reference returned by a successful backend factory or getter.
    ///
    /// # Safety
    /// A non-null pointer must carry one owned COM reference with vtable `V`.
    pub const unsafe fn adopt(pointer: *mut c_void) -> Option<Self> {
        if pointer.is_null() {
            return None;
        }
        // SAFETY: a COM interface begins with its valid vtable pointer.
        let table = unsafe { pointer.cast::<*const V>().read_unaligned() };
        Some(Self { pointer, table })
    }

    pub const fn pointer(&self) -> *mut c_void {
        self.pointer
    }

    pub const fn table(&self) -> &V {
        // SAFETY: the owned interface keeps its vtable live until Drop.
        unsafe { &*self.table }
    }

    /// Transfers the owned reference into another ownership mechanism.
    pub const fn into_raw(self) -> *mut c_void {
        let pointer = self.pointer;
        core::mem::forget(self);
        pointer
    }
}

impl<V: BackendVtable> Drop for Backend<V> {
    fn drop(&mut self) {
        let release = self.table().release();
        // SAFETY: construction adopted this reference and Drop consumes it once.
        unsafe { release(self.pointer) };
    }
}

/// Borrows a vtable while the caller holds the corresponding interface reference.
///
/// # Safety
/// `pointer` must be a live interface with vtable `V` for the returned lifetime.
pub const unsafe fn table<'a, V>(pointer: *mut c_void) -> &'a V {
    // SAFETY: the caller holds a live COM interface reference.
    let table = unsafe { pointer.cast::<*const V>().read_unaligned() };
    // SAFETY: the caller guarantees the table's type and lifetime.
    unsafe { &*table }
}
