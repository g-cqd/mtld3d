//! D3D8 resource identities backed by owned, concretely typed D3D9 references.

use core::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};

use mtld3d_shared::{InPtr, OutPtr};
use mtld3d_types::{D3D_OK, D3DERR_INVALIDCALL, E_NOINTERFACE, Guid};

use crate::{
    api_scope::ApiScope,
    device::{self, Device8},
};

mod backend;
mod buffers;
mod container;
mod kind;
mod registry;
mod surfaces;
mod textures;
mod vtables;

pub use backend::ResourceBackend;
pub use container::ContainerRef;
pub use registry::ResourceRegistry;

#[repr(C)]
pub struct Resource8 {
    vtable: *const c_void,
    references: AtomicU32,
    inner: Box<ResourceInner>,
}

struct ResourceInner {
    backend: ResourceBackend,
    device: *mut c_void,
    identity: usize,
    container: Option<ContainerRef>,
}

impl Resource8 {
    pub const fn backend(&self) -> &ResourceBackend {
        &self.inner.backend
    }

    fn new(
        device: &Device8,
        backend: ResourceBackend,
        identity: usize,
        container: Option<ContainerRef>,
    ) -> Self {
        let parent = ptr::from_ref(device).cast_mut().cast();
        device::add_ref(parent);
        Self {
            vtable: vtables::for_kind(&backend.kind()),
            references: AtomicU32::new(1),
            inner: Box::new(ResourceInner {
                backend,
                device: parent,
                identity,
                container,
            }),
        }
    }

    fn retain(&self) -> u32 {
        self.references.fetch_add(1, Ordering::Relaxed) + 1
    }
}

/// Borrows a resource for the caller's COM call frame.
///
/// # Safety
/// `this` must name a live `Resource8` for the returned lifetime.
const unsafe fn object<'a>(this: *mut c_void) -> InPtr<'a, Resource8> {
    // SAFETY: the caller guarantees the concrete wrapper type and lifetime.
    unsafe { InPtr::new(this) }
}

unsafe fn api_scope(this: *mut c_void) -> ApiScope {
    // SAFETY: the caller holds a reference to this resource wrapper.
    let resource = unsafe { object(this) };
    // SAFETY: every live resource wrapper owns a reference to its parent device.
    unsafe { device::api_scope(resource.inner.device) }
}

extern "system" fn query_interface(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable COM output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the caller supplies a readable interface identifier or null.
    let Some(iid) = (unsafe { InPtr::<Guid>::opt(iid.cast()) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    if resource.inner.backend.kind().supports(&iid) {
        resource.retain();
        output.write(this);
        D3D_OK
    } else {
        output.write(ptr::null_mut());
        E_NOINTERFACE
    }
}

extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    resource.retain()
}

extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller owns the resource reference being released.
    let resource = unsafe { object(this) };
    let parent = resource.inner.device;
    // SAFETY: the resource owns a reference to this parent device.
    let device = unsafe { device::object(parent) };
    let mut objects = device.resources().lock();
    let remaining = resource.references.fetch_sub(1, Ordering::Release) - 1;
    if remaining == 0 {
        core::sync::atomic::fence(Ordering::Acquire);
        objects.remove(&(
            resource.inner.identity,
            resource.inner.backend.kind().code(),
        ));
    }
    drop(objects);
    if remaining == 0 {
        // SAFETY: the last reference owns this allocation and its registry entry is gone.
        drop(unsafe { Box::from_raw(this.cast::<Resource8>()) });
        device::release(parent);
    }
    remaining
}

extern "system" fn get_device(this: *mut c_void, output: *mut *mut c_void) -> i32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable COM output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    device::add_ref(resource.inner.device);
    output.write(resource.inner.device);
    D3D_OK
}

extern "system" fn get_type(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    resource.inner.backend.kind().code()
}

extern "system" fn set_private_data(
    this: *mut c_void,
    guid: *const Guid,
    data: *const c_void,
    size: u32,
    flags: u32,
) -> i32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe {
                (backend.table().set_private_data)(backend.pointer(), guid, data, size, flags)
            }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe {
                (backend.table().set_private_data)(backend.pointer(), guid, data, size, flags)
            }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe {
                (backend.table().set_private_data)(backend.pointer(), guid, data, size, flags)
            }
        }
        ResourceBackend::VertexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe {
                (backend.table().set_private_data)(backend.pointer(), guid, data, size, flags)
            }
        }
        ResourceBackend::IndexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe {
                (backend.table().set_private_data)(backend.pointer(), guid, data, size, flags)
            }
        }
        ResourceBackend::Surface(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe {
                (backend.table().set_private_data)(backend.pointer(), guid, data, size, flags)
            }
        }
        ResourceBackend::Volume(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe {
                (backend.table().set_private_data)(backend.pointer(), guid, data, size, flags)
            }
        }
    }
}

extern "system" fn get_private_data(
    this: *mut c_void,
    guid: *const Guid,
    data: *mut c_void,
    size: *mut u32,
) -> i32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_private_data)(backend.pointer(), guid, data, size) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_private_data)(backend.pointer(), guid, data, size) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_private_data)(backend.pointer(), guid, data, size) }
        }
        ResourceBackend::VertexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_private_data)(backend.pointer(), guid, data, size) }
        }
        ResourceBackend::IndexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_private_data)(backend.pointer(), guid, data, size) }
        }
        ResourceBackend::Surface(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_private_data)(backend.pointer(), guid, data, size) }
        }
        ResourceBackend::Volume(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_private_data)(backend.pointer(), guid, data, size) }
        }
    }
}

extern "system" fn free_private_data(this: *mut c_void, guid: *const Guid) -> i32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().free_private_data)(backend.pointer(), guid) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().free_private_data)(backend.pointer(), guid) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().free_private_data)(backend.pointer(), guid) }
        }
        ResourceBackend::VertexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().free_private_data)(backend.pointer(), guid) }
        }
        ResourceBackend::IndexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().free_private_data)(backend.pointer(), guid) }
        }
        ResourceBackend::Surface(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().free_private_data)(backend.pointer(), guid) }
        }
        ResourceBackend::Volume(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().free_private_data)(backend.pointer(), guid) }
        }
    }
}

extern "system" fn set_priority(this: *mut c_void, priority: u32) -> u32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().set_priority)(backend.pointer(), priority) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().set_priority)(backend.pointer(), priority) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().set_priority)(backend.pointer(), priority) }
        }
        ResourceBackend::VertexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().set_priority)(backend.pointer(), priority) }
        }
        ResourceBackend::IndexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().set_priority)(backend.pointer(), priority) }
        }
        ResourceBackend::Surface(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().set_priority)(backend.pointer(), priority) }
        }
        ResourceBackend::Volume(_) => {
            unreachable!("volume vtables do not expose resource priority methods")
        }
    }
}

extern "system" fn get_priority(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_priority)(backend.pointer()) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_priority)(backend.pointer()) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_priority)(backend.pointer()) }
        }
        ResourceBackend::VertexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_priority)(backend.pointer()) }
        }
        ResourceBackend::IndexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_priority)(backend.pointer()) }
        }
        ResourceBackend::Surface(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().get_priority)(backend.pointer()) }
        }
        ResourceBackend::Volume(_) => {
            unreachable!("volume vtables do not expose resource priority methods")
        }
    }
}

extern "system" fn pre_load(this: *mut c_void) {
    // SAFETY: the COM receiver owns the parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Resource8 for this call.
    let resource = unsafe { object(this) };
    match &resource.inner.backend {
        ResourceBackend::Texture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().pre_load)(backend.pointer()) }
        }
        ResourceBackend::CubeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().pre_load)(backend.pointer()) }
        }
        ResourceBackend::VolumeTexture(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().pre_load)(backend.pointer()) }
        }
        ResourceBackend::VertexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().pre_load)(backend.pointer()) }
        }
        ResourceBackend::IndexBuffer(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().pre_load)(backend.pointer()) }
        }
        ResourceBackend::Surface(backend) => {
            // SAFETY: the typed backend owns its receiver and shares this method's ABI.
            unsafe { (backend.table().pre_load)(backend.pointer()) }
        }
        ResourceBackend::Volume(_) => {
            unreachable!("volume vtables do not expose resource priority methods")
        }
    }
}

/// Validates that an input interface is one of this device's resource wrappers.
///
/// # Safety
/// A non-null pointer must name a live COM interface for the returned lifetime.
pub unsafe fn input<'a>(
    pointer: *mut c_void,
    device: &Device8,
) -> Result<Option<InPtr<'a, Resource8>>, i32> {
    if pointer.is_null() {
        return Ok(None);
    }
    // SAFETY: a live COM interface starts with a readable vtable pointer.
    let table = unsafe { pointer.cast::<*const c_void>().read_unaligned() };
    if !vtables::is_known(table) {
        return Err(D3DERR_INVALIDCALL);
    }
    // SAFETY: only Resource8 constructors install a vtable accepted above.
    let resource = unsafe { object(pointer) };
    if resource.inner.device != ptr::from_ref(device).cast_mut().cast() {
        return Err(D3DERR_INVALIDCALL);
    }
    Ok(Some(resource))
}

/// Retains a texture container returned by a backend surface or volume getter.
///
/// The typed backend owns the reference until it transfers into the parent wrapper.
pub fn retain_container(device: &Device8, backend: ResourceBackend) -> Result<ContainerRef, i32> {
    let pointer = device.resources().wrap(device, backend)?;
    // SAFETY: wrap returned one owned Resource8 reference of the typed texture backend.
    let resource = unsafe { object(pointer) };
    let container = ContainerRef::retain(&resource);
    release(pointer);
    Ok(container)
}
