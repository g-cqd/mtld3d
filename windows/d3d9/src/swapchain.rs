//! `IDirect3DSwapChain9` — the implicit swapchain and the additional ones.
//!
//! `GetSwapChain(0)` hands out the device's implicit swapchain; the additional
//! ones are the objects returned by `CreateAdditionalSwapChain`.
//!
//! Additional chains own separate color storage and retain their cached surface
//! identities through public and binding references. Presentation selects that
//! image without changing the main back buffer; the device still has one window.

use core::ffi::c_void;

use mtld3d_shared::{InPtr, OutPtr};
use mtld3d_types::{
    D3DDISPLAYMODE, D3DERR_DEVICENOTRESET, D3DPRESENT_PARAMETERS, Guid, IDirect3DSwapChain9Vtbl,
};

use super::{D3D_OK, D3DERR_INVALIDCALL, LOG_TARGET, device::DeviceInner};
use crate::{com_ref::ComUnknown, surface::Direct3DSurface9};

pub static DIRECT3D_SWAPCHAIN9_VTBL: IDirect3DSwapChain9Vtbl = IDirect3DSwapChain9Vtbl {
    query_interface: swapchain_query_interface,
    add_ref: swapchain_add_ref,
    release: swapchain_release,
    present: swapchain_present,
    get_front_buffer_data: swapchain_get_front_buffer_data,
    get_back_buffer: swapchain_get_back_buffer,
    get_raster_status: swapchain_get_raster_status,
    get_display_mode: swapchain_get_display_mode,
    get_device: swapchain_get_device,
    get_present_parameters: swapchain_get_present_parameters,
};

#[repr(C)]
pub struct Direct3DSwapChain9 {
    vtbl: *const IDirect3DSwapChain9Vtbl,
    refcount: u32,
    private_refcount: u32,
    inner: *mut SwapChainInner,
}

impl Direct3DSwapChain9 {
    /// Installs the independently allocated back buffer before publishing this chain.
    ///
    /// # Safety
    /// `this` is a newly registered additional chain with no other references.
    pub unsafe fn initialize_backbuffer(this: *mut Self) -> i32 {
        // SAFETY: the caller owns the unpublished chain and its inner state.
        let chain = unsafe { &mut *this };
        let parameters = chain.inner().present_params;
        // SAFETY: a registered chain retains its creating device.
        let device =
            unsafe { InPtr::<crate::device::Direct3DDevice9>::new(chain.device_wrapper()) };
        let surface = match crate::device::create_swapchain_backbuffer(&device, &parameters) {
            Ok(surface) => surface,
            Err(status) => return status,
        };
        // SAFETY: only this unpublished chain can access its inner state.
        unsafe { (*chain.inner).backbuffer_surface = surface as u64 };
        // SAFETY: this chain owns the fresh surface and retains its cached storage.
        let surface = unsafe { &mut *surface };
        // SAFETY: this unpublished chain owns the newly registered surface.
        unsafe { surface.cache_for_swapchain(this) };
        D3D_OK
    }

    fn with_owner(
        device_inner: *mut DeviceInner,
        present_params: D3DPRESENT_PARAMETERS,
        owned_by_device: bool,
    ) -> Self {
        let inner = Box::into_raw(Box::new(SwapChainInner {
            device_inner,
            present_params,
            owned_by_device,
            backbuffer_surface: 0,
        }));
        Self {
            vtbl: &raw const DIRECT3D_SWAPCHAIN9_VTBL,
            // Implicit (device-owned) swapchains start at refcount 0 and forward
            // to the device on the 0↔1 boundary (D3D9 implicit-object model);
            // app-owned additional swapchains start at 1 and own their create
            // reference. `!owned_by_device`: implicit → 0, additional → 1.
            refcount: u32::from(!owned_by_device),
            private_refcount: 0,
            inner,
        }
    }

    /// An app-owned additional swapchain (`CreateAdditionalSwapChain`).
    ///
    /// `present_params` must already be normalised (dimensions resolved,
    /// back-buffer count clamped to >= 1). Freed when the app releases its last
    /// reference.
    pub fn new(device_inner: *mut DeviceInner, present_params: D3DPRESENT_PARAMETERS) -> Self {
        Self::with_owner(device_inner, present_params, false)
    }

    /// The device's implicit swapchain (`GetSwapChain(0)`).
    ///
    /// It is owned by the device: `Release` never frees it, matching the D3D9
    /// contract where the implicit swapchain outlives an app `Release` and is
    /// destroyed with the device. The small shell is leaked at teardown, like
    /// the device wrapper.
    pub fn new_implicit(
        device_inner: *mut DeviceInner,
        present_params: D3DPRESENT_PARAMETERS,
    ) -> Self {
        Self::with_owner(device_inner, present_params, true)
    }

    /// Overwrite the present parameters this swapchain reports.
    ///
    /// The device keeps its cached implicit swapchain in lockstep after Reset
    /// or an automatic resize, so `GetSwapChain(0).GetPresentParameters`
    /// reflects the live geometry rather than the values captured at first
    /// hand-out.
    pub fn set_present_params(&mut self, present_params: D3DPRESENT_PARAMETERS) {
        // SAFETY: `self.inner` is a live `Box::into_raw` (see `inner()`), valid
        // for every live wrapper reference.
        unsafe { (*self.inner).present_params = present_params };
    }

    /// The owning device's `Direct3DDevice9`* wrapper, or null if the device pointer is unset.
    fn device_wrapper(&self) -> *mut c_void {
        let device_inner = self.inner().device_inner;
        if device_inner.is_null() {
            return core::ptr::null_mut();
        }
        // SAFETY: `device_inner` is the live owning device (see `SwapChainInner`),
        // alive past its child swapchains per D3D9 lifetime rules.
        unsafe { (*device_inner).device_wrapper() }
    }

    fn inner(&self) -> &SwapChainInner {
        // SAFETY: `self.inner` was installed by a constructor as a
        // `Box::into_raw` and is dropped only in `swapchain_release` at
        // refcount zero, so it stays live for every live wrapper reference.
        unsafe { &*self.inner }
    }
}

struct SwapChainInner {
    /// The owning device.
    ///
    /// Borrowed (not `AddRef`'d) — D3D9 lifetime rules keep the device alive
    /// past its child swapchains.
    device_inner: *mut DeviceInner,
    present_params: D3DPRESENT_PARAMETERS,
    /// `true` for the implicit swapchain: `Release` never frees it (the device owns it).
    ///
    /// `false` for additional swapchains, freed at refcount zero.
    owned_by_device: bool,
    /// This (app-owned) swapchain's cached backbuffer surface.
    ///
    /// Initialized before publication; unused on the implicit chain. Its
    /// public references retain this chain on their 0-to-1 edge, and each
    /// binding reference retains this chain privately. Only chain teardown
    /// frees the cached surface, preserving repeated getter identity.
    backbuffer_surface: u64,
}

extern "system" fn swapchain_query_interface(
    this: *mut c_void,
    riid: *const Guid,
    ppv: *mut *mut c_void,
) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    // SAFETY: vtable thunk; `this`, `riid` and `ppv` are the caller's per the
    // IUnknown::QueryInterface ABI.
    unsafe {
        crate::com_ref::com_query_interface(
            this,
            riid,
            ppv,
            &[
                mtld3d_types::IID_IUNKNOWN,
                mtld3d_types::IID_IDIRECT3DSWAPCHAIN9,
            ],
            swapchain_add_ref,
            "IDirect3DSwapChain9",
        )
    }
}

extern "system" fn swapchain_add_ref(this: *mut c_void) -> u32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    // SAFETY: IDirect3DSwapChain9 IUnknown AddRef thunk; the D3D9 ABI guarantees
    // `this` is the live wrapper for the call. The engine forwards the device
    // reference for the device-owned implicit swapchain on its 0→1 transition.
    unsafe { crate::com_ref::com_add_ref::<Direct3DSwapChain9>(this) }
}

extern "system" fn swapchain_release(this: *mut c_void) -> u32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    // SAFETY: IDirect3DSwapChain9 IUnknown Release thunk; the D3D9 ABI guarantees
    // `this` is the live wrapper for the call. The engine frees an app-owned
    // additional swapchain on its 1→0 transition and forwards the device release
    // for the device-owned implicit swapchain (which is never freed here).
    unsafe { crate::com_ref::com_release::<Direct3DSwapChain9>(this) }
}

/// Destroy an app-owned `Direct3DSwapChain9` wrapper once both refcounts reach zero.
///
/// Its back buffer goes with it, so this runs only once nothing references
/// the back buffer either: the last public `Release`, or the last binding
/// reference's private release after it. The device-owned implicit swapchain
/// is never finalized (its shell is leaked at device teardown).
///
/// # Safety
/// `this` must point to a live, app-owned `Direct3DSwapChain9` wrapper at
/// public and private refcounts zero; caller must not access the wrapper afterwards.
unsafe fn finalize_swapchain(this: *mut Direct3DSwapChain9) {
    // SAFETY: refcount reached zero on an app-owned swapchain; `(*this).inner`
    // is the original `Box::into_raw(SwapChainInner)` and no other reference can
    // survive a zero refcount.
    let inner = unsafe { (*this).inner };
    // Finalize the swapchain-owned cached backbuffer surface (a `Backbuffer`-kind
    // surface never freed by its own `Release` — destroyed with its owner here,
    // mirroring how `device_release` finalizes the implicit RT/DS surfaces).
    // Its references retain this swap chain, so nothing references it now.
    // SAFETY: `inner` is live (sole owner); read the cached pointer before the
    // box is freed.
    let backbuffer_surface = unsafe { (*inner).backbuffer_surface };
    if backbuffer_surface != 0 {
        // SAFETY: a non-zero `backbuffer_surface` is a live `Backbuffer`-kind
        // surface created by this swapchain; finalized exactly once here.
        unsafe { crate::surface::finalize_implicit_surface(backbuffer_surface) };
    }
    // SAFETY: as above — sole owner of the inner allocation.
    drop(unsafe { Box::from_raw(inner) });
    // SAFETY: refcount reached zero; `this` is the original
    // `Box::into_raw(Direct3DSwapChain9)` allocation.
    drop(unsafe { Box::from_raw(this) });
}

// SAFETY: `refcount_mut` exposes this wrapper's own counter and
// `private_refcount` the binding references to its back buffer; `finalize` frees an app-owned
// swapchain exactly once when both are zero. The device-owned implicit
// swapchain forwards its refcount to the device and is never finalized here.
unsafe impl crate::com_ref::ComChild for Direct3DSwapChain9 {
    fn refcount_mut(&mut self) -> &mut u32 {
        &mut self.refcount
    }
    fn private_refcount(&self) -> u32 {
        self.private_refcount
    }
    fn owning_device(&self) -> *mut c_void {
        // Both the device-owned implicit swapchain (forwards on its 0→1 edge)
        // and an app-owned additional swapchain (registered at creation, forwards
        // its release at teardown) hold one reference on the device.
        self.device_wrapper()
    }
    fn finalizes_on_zero(&self) -> bool {
        !self.inner().owned_by_device
    }
    fn blocks_reset_while_referenced(&self) -> bool {
        !self.inner().owned_by_device
    }
    unsafe fn finalize(this: *mut Self) {
        // SAFETY: forwarded from the engine — refcount is zero and the swapchain
        // is app-owned (`finalizes_on_zero()` true).
        unsafe { finalize_swapchain(this) };
    }
}

impl ComUnknown for Direct3DSwapChain9 {
    fn private_refcount_inc(&mut self) {
        self.private_refcount += 1;
    }

    unsafe fn private_refcount_dec_maybe_finalize(this: *mut Self) {
        let finalize = {
            // SAFETY: the caller owns a live private reference to this chain.
            let chain = unsafe { &mut *this };
            chain.private_refcount -= 1;
            chain.private_refcount == 0 && chain.refcount == 0 && !chain.inner().owned_by_device
        };
        if finalize {
            // SAFETY: both counters are zero and the device does not own the chain.
            unsafe { finalize_swapchain(this) };
        }
    }
}

extern "system" fn swapchain_present(
    this: *mut c_void,
    source_rect: *const c_void,
    dest_rect: *const c_void,
    dest_window_override: usize,
    dirty_region: *const c_void,
    _flags: u32,
) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    // SAFETY: vtable thunk; `this` is *mut Direct3DSwapChain9 per the ABI.
    let Some(obj) = (unsafe { InPtr::<Direct3DSwapChain9>::opt(this) }) else {
        return D3DERR_INVALIDCALL;
    };
    crate::device::warn_ignored_present_arguments(
        source_rect,
        dest_rect,
        dest_window_override != 0,
        dirty_region,
    );
    if !obj.inner().owned_by_device {
        // SAFETY: the live chain retains its device and owns its cached back buffer.
        let device = unsafe { InPtr::<crate::device::Direct3DDevice9>::new(obj.device_wrapper()) };
        // SAFETY: successful additional-chain creation initialized the cached surface.
        let surface = unsafe {
            InPtr::<Direct3DSurface9>::new(obj.inner().backbuffer_surface as *mut c_void)
        };
        return crate::device::present_swapchain_backbuffer(&device, &surface);
    }
    let device_inner = obj.inner().device_inner;
    // SAFETY: `device_inner` was stamped from a live `DeviceInner` that
    // outlives its swapchains per D3D9 lifetime rules. There is one drawable,
    // so presenting any swapchain presents the device frame.
    let dev = unsafe { &mut *device_inner };
    if dev.needs_reset() {
        return D3DERR_DEVICENOTRESET;
    }
    dev.present()
}

extern "system" fn swapchain_get_front_buffer_data(this: *mut c_void, surface: *mut c_void) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    // SAFETY: vtable thunk; `this` is a live swapchain pointer or null.
    let Some(obj) = (unsafe { InPtr::<Direct3DSwapChain9>::opt(this) }) else {
        mtld3d_shared::log_once_warn!(target: LOG_TARGET,
            "reject swapchain GetFrontBufferData: null swapchain → INVALIDCALL");
        return D3DERR_INVALIDCALL;
    };
    if !obj.inner().owned_by_device {
        mtld3d_shared::log_once_warn!(target: LOG_TARGET,
            "reject swapchain GetFrontBufferData: additional swapchain has no front image → INVALIDCALL");
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the destination is a caller-owned IDirect3DSurface9 or null.
    let Some(dst) = (unsafe { InPtr::<Direct3DSurface9>::opt(surface) }) else {
        mtld3d_shared::log_once_warn!(target: LOG_TARGET,
            "reject swapchain GetFrontBufferData: null destination → INVALIDCALL");
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: a referenced implicit swapchain pins its owning device. The API
    // lock above serializes access; an unset owner is rejected below.
    let Some(dev) = (unsafe { obj.inner().device_inner.as_mut() }) else {
        mtld3d_shared::log_once_warn!(target: LOG_TARGET,
            "reject swapchain GetFrontBufferData: no owning device → INVALIDCALL");
        return D3DERR_INVALIDCALL;
    };
    dev.read_front_buffer(&dst)
}

extern "system" fn swapchain_get_back_buffer(
    this: *mut c_void,
    i_back_buffer: u32,
    _type: u32,
    back_buffer: *mut *mut c_void,
) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    if back_buffer.is_null() {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: vtable thunk; `this` is *mut Direct3DSwapChain9 per the ABI.
    let Some(obj) = (unsafe { InPtr::<Direct3DSwapChain9>::opt(this) }) else {
        return D3DERR_INVALIDCALL;
    };
    // Out-of-range index fails and leaves the caller's out-param untouched, per
    // the D3D9 contract for a failed interface-returning call.
    if i_back_buffer >= obj.inner().present_params.back_buffer_count {
        return D3DERR_INVALIDCALL;
    }
    // The implicit swapchain's backbuffer IS the device's implicit render target:
    // return the same device-owned cached surface so `GetSwapChain(0)
    // .GetBackBuffer(0)`, `GetRenderTarget(0)` and `GetBackBuffer(0)` share one
    // identity (the `pRenderTarget == pBackBuffer` invariant the suite checks).
    if obj.inner().owned_by_device {
        // SAFETY: `device_inner` is the live owning device (see `SwapChainInner`).
        let dev = unsafe { &mut *obj.inner().device_inner };
        let surf = dev.get_or_create_implicit_render_target();
        // SAFETY: `surf` is the live cached implicit RT surface.
        let add_ref = unsafe { (*surf).vtbl().add_ref };
        // SAFETY: calling the surface AddRef thunk; D3D9 mandates AddRef on return.
        unsafe { add_ref(surf.cast::<c_void>()) };
        // SAFETY: vtable out-param; `back_buffer` is *mut *mut c_void per the ABI.
        unsafe { *back_buffer = surf.cast::<c_void>() };
        return D3D_OK;
    }
    // The chain owns one stable, independently allocated surface. Its public
    // and binding references retain this chain until its last user releases it.
    let surf = obj.inner().backbuffer_surface as *mut Direct3DSurface9;
    // SAFETY: `surf` is the live cached backbuffer surface.
    let add_ref = unsafe { (*surf).vtbl().add_ref };
    // SAFETY: calling the surface AddRef thunk; D3D9 mandates AddRef on return —
    // the engine forwards the device refcount on the backbuffer's 0→1 edge.
    unsafe { add_ref(surf.cast::<c_void>()) };
    // SAFETY: vtable out-param; `back_buffer` is *mut *mut c_void per the ABI.
    unsafe { OutPtr::write_opt(back_buffer, surf.cast::<c_void>()) };
    D3D_OK
}

extern "system" fn swapchain_get_raster_status(this: *mut c_void, _status: *mut c_void) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    mtld3d_shared::log_once_warn!(target: LOG_TARGET, "stub IDirect3DSwapChain9::GetRasterStatus → INVALIDCALL");
    D3DERR_INVALIDCALL
}

extern "system" fn swapchain_get_display_mode(this: *mut c_void, mode: *mut c_void) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    if mode.is_null() {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: vtable thunk; `this` is *mut Direct3DSwapChain9 per the ABI.
    let Some(obj) = (unsafe { InPtr::<Direct3DSwapChain9>::opt(this) }) else {
        return D3DERR_INVALIDCALL;
    };
    // Mirror `device_get_display_mode`: fullscreen reports the honored mode,
    // windowed reports the desktop's. `present_params` is normalised at
    // creation and refreshed on Reset (`DeviceInner::set_present_params`), so the
    // implicit swapchain stays in sync with the device.
    let pp = obj.inner().present_params;
    // SAFETY: `mode` is non-null (checked) and per the D3D9 ABI points to a
    // writable `D3DDISPLAYMODE` slot owned by the caller.
    unsafe {
        *mode.cast::<D3DDISPLAYMODE>() = crate::direct3d9::reported_display_mode(&pp);
    }
    D3D_OK
}

extern "system" fn swapchain_get_device(this: *mut c_void, device: *mut *mut c_void) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    // SAFETY: vtable thunk; `this` is *mut Direct3DSwapChain9 per the ABI.
    unsafe { crate::com_ref::com_get_device::<Direct3DSwapChain9>(this, device) }
}

extern "system" fn swapchain_get_present_parameters(
    this: *mut c_void,
    parameters: *mut c_void,
) -> i32 {
    let _api = crate::com_ref::com_api_lock::<Direct3DSwapChain9>(this);
    if parameters.is_null() {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: vtable thunk; `this` is *mut Direct3DSwapChain9 per the ABI.
    let Some(obj) = (unsafe { InPtr::<Direct3DSwapChain9>::opt(this) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: `parameters` is non-null (checked) and points to a writable
    // `D3DPRESENT_PARAMETERS` per the D3D9 ABI.
    unsafe { *parameters.cast::<D3DPRESENT_PARAMETERS>() = obj.inner().present_params };
    D3D_OK
}
