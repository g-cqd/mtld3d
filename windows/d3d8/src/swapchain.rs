//! Additional D3D8 swap chains with D3D8 back-buffer identities.

use core::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};

use mtld3d_d3d8_types::{IDirect3DSwapChain8Vtbl, IID_IDIRECT3DSWAPCHAIN8};
use mtld3d_shared::{InPtr, OutPtr};
use mtld3d_types::{D3DERR_INVALIDCALL, Guid, IDirect3DSwapChain9Vtbl};

use crate::{
    api_scope::ApiScope,
    backend::Backend,
    device::{self, Device8},
    identity,
};

const VTABLE: IDirect3DSwapChain8Vtbl = IDirect3DSwapChain8Vtbl {
    query_interface,
    add_ref,
    release,
    present,
    get_back_buffer,
};

#[repr(C)]
pub struct SwapChain8 {
    vtable: &'static IDirect3DSwapChain8Vtbl,
    references: AtomicU32,
    inner: Box<SwapChainInner>,
}

struct SwapChainInner {
    backend: Backend<IDirect3DSwapChain9Vtbl>,
    device: *mut c_void,
}

impl SwapChain8 {
    /// Adopts an additional backend chain and retains its D3D8 device.
    pub fn create(device: &Device8, backend: Backend<IDirect3DSwapChain9Vtbl>) -> *mut c_void {
        let parent = ptr::from_ref(device).cast_mut().cast();
        device::add_ref(parent);
        device.register_swap_chain();
        Box::into_raw(Box::new(Self {
            vtable: &VTABLE,
            references: AtomicU32::new(1),
            inner: Box::new(SwapChainInner {
                backend,
                device: parent,
            }),
        }))
        .cast()
    }
}

impl Drop for SwapChainInner {
    fn drop(&mut self) {
        // SAFETY: this chain owns its device reference until the release below.
        let device = unsafe { device::object(self.device) };
        device.unregister_swap_chain();
        device::release(self.device);
    }
}

/// Borrows a swap chain for its current COM call.
///
/// # Safety
/// `this` must identify a live `SwapChain8` throughout the returned borrow.
const unsafe fn object<'a>(this: *mut c_void) -> InPtr<'a, SwapChain8> {
    // SAFETY: the caller guarantees this receiver's concrete type and lifetime.
    unsafe { InPtr::new(this) }
}

/// Retains and acquires the owning device's API lock.
///
/// # Safety
/// `this` must identify a live `SwapChain8` for this call.
unsafe fn api_scope(this: *mut c_void) -> ApiScope {
    // SAFETY: the caller holds the swap-chain reference throughout this call.
    let chain = unsafe { object(this) };
    // SAFETY: the swap chain owns a reference to this device.
    unsafe { device::api_scope(chain.inner.device) }
}

extern "system" fn query_interface(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver retains its parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: COM dispatch supplies the receiver, readable IID and writable output.
    unsafe { identity::query(this, iid, output, &IID_IDIRECT3DSWAPCHAIN8, add_ref) }
}

extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver retains its parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live SwapChain8 for this call.
    let chain = unsafe { object(this) };
    chain.references.fetch_add(1, Ordering::Relaxed) + 1
}

extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver retains its parent device until its final release.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller owns the reference being released.
    let chain = unsafe { object(this) };
    let remaining = chain.references.fetch_sub(1, Ordering::Release) - 1;
    if remaining == 0 {
        core::sync::atomic::fence(Ordering::Acquire);
        // SAFETY: the final reference owns the allocation; _api separately retains its lock.
        drop(unsafe { Box::from_raw(this.cast::<SwapChain8>()) });
    }
    remaining
}

extern "system" fn present(
    this: *mut c_void,
    source: *const c_void,
    destination: *const c_void,
    window: usize,
    dirty: *const c_void,
) -> i32 {
    // SAFETY: the COM receiver retains its parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live SwapChain8 receiver.
    let chain = unsafe { object(this) };
    // SAFETY: D3D8's rectangles and dirty region have the D3D9 layouts; D3D8 supplies no flags.
    unsafe {
        (chain.inner.backend.table().present)(
            chain.inner.backend.pointer(),
            source,
            destination,
            window,
            dirty,
            0,
        )
    }
}

extern "system" fn get_back_buffer(
    this: *mut c_void,
    index: u32,
    _kind: u32,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver retains its parent device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this typed vtable dispatches a live SwapChain8 receiver.
    let chain = unsafe { object(this) };
    let mut surface = ptr::null_mut();
    // SAFETY: surface is writable and D3D8 ignores the back-buffer kind argument.
    let result = unsafe {
        (chain.inner.backend.table().get_back_buffer)(
            chain.inner.backend.pointer(),
            index,
            0,
            &raw mut surface,
        )
    };
    if result < 0 {
        output.write(ptr::null_mut());
        return result;
    }
    // SAFETY: successful GetBackBuffer returned one owned IDirect3DSurface9 reference.
    let Some(surface) = (unsafe { Backend::adopt(surface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this swap chain retains its creating D3D8 device.
    let device = unsafe { device::object(chain.inner.device) };
    match device::wrap_surface(&device, surface, true) {
        Ok(surface) => {
            output.write(surface);
            result
        }
        Err(status) => {
            output.write(ptr::null_mut());
            status
        }
    }
}
