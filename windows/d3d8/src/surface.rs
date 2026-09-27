//! Stable D3D8 back-buffer identity with external-reference ownership of the backend.

use core::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use mtld3d_core::format::d3d8_surface_size;
use mtld3d_d3d8_types::{
    D3DLOCKED_RECT, D3DSURFACE_DESC8, Guid, IDirect3DSurface8Vtbl, IID_IDIRECT3DSURFACE8,
};
use mtld3d_shared::{InPtr, OutPtr};
use mtld3d_types::{D3D_OK, D3DERR_INVALIDCALL, IDirect3DSurface9Vtbl};

use crate::{
    api_scope::ApiScope,
    backend::{self, Backend},
    device, identity,
};

static VTABLE: IDirect3DSurface8Vtbl = IDirect3DSurface8Vtbl {
    query_interface,
    add_ref,
    release,
    get_device,
    set_private_data,
    get_private_data,
    free_private_data,
    get_container,
    get_desc,
    lock_rect,
    unlock_rect,
};

/// The device owns the shell; a nonzero public count owns the backend and device references.
#[repr(C)]
pub struct Surface8 {
    vtable: *const IDirect3DSurface8Vtbl,
    references: AtomicU32,
    inner: Box<SurfaceInner>,
}

struct SurfaceInner {
    backend: AtomicPtr<c_void>,
    device: *mut c_void,
}

impl Surface8 {
    /// Creates an inactive shell owned by the supplied device's back-buffer cache.
    ///
    /// # Safety
    /// `device` must remain live while this shell is cached or externally referenced.
    pub unsafe fn new(device: *mut c_void) -> Self {
        Self {
            vtable: &raw const VTABLE,
            references: AtomicU32::new(0),
            inner: Box::new(SurfaceInner {
                backend: AtomicPtr::new(ptr::null_mut()),
                device,
            }),
        }
    }

    /// The caller holds the owning device's back-buffer cache lock.
    pub fn acquire(&self, backend: Backend<IDirect3DSurface9Vtbl>) -> *mut c_void {
        if self.references.fetch_add(1, Ordering::Relaxed) == 0 {
            device::add_ref(self.inner.device);
            self.inner
                .backend
                .store(backend.into_raw(), Ordering::Release);
        }
        ptr::from_ref(self).cast_mut().cast()
    }

    pub fn has_references(&self) -> bool {
        self.references.load(Ordering::Relaxed) != 0
    }

    fn backend(&self) -> (*mut c_void, &IDirect3DSurface9Vtbl) {
        let pointer = self.inner.backend.load(Ordering::Acquire);
        // SAFETY: every caller holds an external reference, which retains this backend.
        let table = unsafe { backend::table(pointer) };
        (pointer, table)
    }
}

const unsafe fn object<'a>(this: *mut c_void) -> InPtr<'a, Surface8> {
    // SAFETY: every caller dispatches a live D3D8 surface interface.
    unsafe { InPtr::new(this) }
}

unsafe fn api_scope(this: *mut c_void) -> ApiScope {
    // SAFETY: the caller guarantees a live surface COM receiver.
    let object = unsafe { object(this) };
    // SAFETY: an externally referenced surface owns a reference to its device.
    unsafe { device::api_scope(object.inner.device) }
}

extern "system" fn query_interface(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM caller supplies IUnknown parameters for the surface wrapper.
    unsafe { identity::query(this, iid, output, &IID_IDIRECT3DSURFACE8, add_ref) }
}

extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Surface8 for this call.
    let object = unsafe { object(this) };
    object.references.fetch_add(1, Ordering::Relaxed) + 1
}

extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Surface8 for this call.
    let surface = unsafe { object(this) };
    let parent = surface.inner.device;
    // SAFETY: this active surface owns a reference to its parent device.
    let device = unsafe { device::object(parent) };
    let cached = device.back_buffer();
    let remaining = surface.references.fetch_sub(1, Ordering::Release) - 1;
    let backend = if remaining == 0 {
        core::sync::atomic::fence(Ordering::Acquire);
        surface
            .inner
            .backend
            .swap(ptr::null_mut(), Ordering::AcqRel)
    } else {
        ptr::null_mut()
    };
    drop(cached);
    if remaining == 0 {
        // SAFETY: the transition to zero transfers the sole owned backend reference.
        drop(unsafe { Backend::<IDirect3DSurface9Vtbl>::adopt(backend) });
        // The final child reference may destroy the device and this surface shell.
        device::release(parent);
    }
    remaining
}

extern "system" fn get_device(this: *mut c_void, output: *mut *mut c_void) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies a writable interface output slot or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this COM receiver owns its device reference.
    let object = unsafe { object(this) };
    let parent = object.inner.device;
    device::add_ref(parent);
    output.write(parent);
    D3D_OK
}

extern "system" fn get_container(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM caller holds the surface and its parent device alive.
    let object = unsafe { object(this) };
    device::query_interface(object.inner.device, iid, output)
}

extern "system" fn get_desc(this: *mut c_void, output: *mut D3DSURFACE_DESC8) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable descriptor storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Surface8 for this call.
    let object = unsafe { object(this) };
    let (pointer, table) = object.backend();
    let mut description = MaybeUninit::uninit();
    // SAFETY: description has the backend's full descriptor layout.
    let result = unsafe { (table.get_desc)(pointer, description.as_mut_ptr()) };
    if result >= 0 {
        // SAFETY: successful GetDesc initialized the output.
        let description = unsafe { description.assume_init() };
        let Some(size) =
            d3d8_surface_size(description.format, description.width, description.height)
        else {
            return D3DERR_INVALIDCALL;
        };
        output.write(D3DSURFACE_DESC8 {
            format: description.format,
            resource_type: description.resource_type,
            usage: description.usage,
            pool: description.pool,
            size,
            multi_sample_type: description.multi_sample_type,
            width: description.width,
            height: description.height,
        });
    }
    result
}

macro_rules! forward {
    ($name:ident, ($($argument:ident : $kind:ty),*)) => {
        extern "system" fn $name(this: *mut c_void, $($argument: $kind),*) -> i32 {
            // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
            // SAFETY: the COM receiver is a live Surface8 for this call.
    let object = unsafe { object(this) };
            let (pointer, table) = object.backend();
            // SAFETY: the surface retains its backend and this method shares D3D9's ABI.
            unsafe { (table.$name)(pointer, $($argument),*) }
        }
    };
}
forward!(set_private_data, (guid: *const Guid, data: *const c_void, size: u32, flags: u32));
forward!(get_private_data, (guid: *const Guid, data: *mut c_void, size: *mut u32));
forward!(free_private_data, (guid: *const Guid));
forward!(lock_rect, (locked: *mut D3DLOCKED_RECT, rectangle: *const c_void, flags: u32));
forward!(unlock_rect, ());

/// Validates an implicit surface input and returns its borrowed backend pointer.
///
/// # Safety
/// A non-null pointer must name a live COM interface for the call duration.
pub unsafe fn input_backend(
    pointer: *mut c_void,
    device: &crate::device::Device8,
) -> Option<*mut c_void> {
    if pointer.is_null() {
        return None;
    }
    // SAFETY: a live COM interface starts with its readable vtable pointer.
    let table = unsafe { pointer.cast::<*const c_void>().read_unaligned() };
    if table != (&raw const VTABLE).cast() {
        return None;
    }
    // SAFETY: only Surface8 constructors install the vtable accepted above.
    let surface = unsafe { object(pointer) };
    if surface.inner.device != ptr::from_ref(device).cast_mut().cast() {
        return None;
    }
    Some(surface.inner.backend.load(Ordering::Acquire))
}
