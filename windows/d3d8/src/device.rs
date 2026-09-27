//! Device identity, presentation, and the implemented fixed-function entry points.

use core::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use mtld3d_core::api_lock::ApiLock;
use mtld3d_d3d8_types::{
    D3DCAPS8, D3DPRESENT_PARAMETERS8, Guid, IDirect3DDevice8Vtbl, IID_IDIRECT3DDEVICE8,
    copy_d3d9_present_parameters, to_d3d8_caps, to_d3d9_present_parameters,
};
use mtld3d_shared::{InPtr, InPtrMut, OutPtr};
use mtld3d_types::{
    D3D_OK, D3DCREATE_MULTITHREADED, D3DERR_DEVICELOST, D3DERR_DEVICENOTRESET, D3DERR_INVALIDCALL,
    IDirect3DDevice9Vtbl,
};

use crate::{api_scope::ApiScope, backend::Backend, direct3d, identity, surface::Surface8};

mod copy_rects;
mod resources;
mod surfaces;
mod swapchain;
pub use surfaces::wrap_surface;
mod scalar;
mod shader;
mod state;
mod state_block;
mod tokens;

static VTABLE: IDirect3DDevice8Vtbl = IDirect3DDevice8Vtbl {
    query_interface,
    add_ref,
    release,
    test_cooperative_level,
    get_available_texture_mem,
    resource_manager_discard_bytes,
    get_direct3d,
    get_device_caps,
    get_display_mode: scalar::get_display_mode,
    get_creation_parameters: scalar::get_creation_parameters,
    set_cursor_properties: resources::set_cursor_properties,
    set_cursor_position: scalar::set_cursor_position,
    show_cursor: scalar::show_cursor,
    create_additional_swap_chain: swapchain::create_additional_swap_chain,
    reset,
    present,
    get_back_buffer,
    get_raster_status: scalar::get_raster_status,
    set_gamma_ramp: scalar::set_gamma_ramp,
    get_gamma_ramp: scalar::get_gamma_ramp,
    create_texture: resources::create_texture,
    create_volume_texture: resources::create_volume_texture,
    create_cube_texture: resources::create_cube_texture,
    create_vertex_buffer: resources::create_vertex_buffer,
    create_index_buffer: resources::create_index_buffer,
    create_render_target: resources::create_render_target,
    create_depth_stencil_surface: resources::create_depth_stencil_surface,
    create_image_surface: resources::create_image_surface,
    copy_rects: copy_rects::copy_rects,
    update_texture: resources::update_texture,
    get_front_buffer: surfaces::get_front_buffer,
    set_render_target: surfaces::set_render_target,
    get_render_target: surfaces::get_render_target,
    get_depth_stencil_surface: surfaces::get_depth_stencil_surface,
    begin_scene,
    end_scene,
    clear,
    set_transform: scalar::set_transform,
    get_transform: scalar::get_transform,
    multiply_transform: scalar::multiply_transform,
    set_viewport: scalar::set_viewport,
    get_viewport: scalar::get_viewport,
    set_material: scalar::set_material,
    get_material: scalar::get_material,
    set_light: scalar::set_light,
    get_light: scalar::get_light,
    light_enable: scalar::light_enable,
    get_light_enable: scalar::get_light_enable,
    set_clip_plane: scalar::set_clip_plane,
    get_clip_plane: scalar::get_clip_plane,
    set_render_state: scalar::set_render_state,
    get_render_state: scalar::get_render_state,
    begin_state_block: state_block::begin_state_block,
    end_state_block: state_block::end_state_block,
    apply_state_block: state_block::apply_state_block,
    capture_state_block: state_block::capture_state_block,
    delete_state_block: state_block::delete_state_block,
    create_state_block: state_block::create_state_block,
    set_clip_status: scalar::set_clip_status,
    get_clip_status: scalar::get_clip_status,
    get_texture: resources::get_texture,
    set_texture: resources::set_texture,
    get_texture_stage_state: scalar::get_texture_stage_state,
    set_texture_stage_state: scalar::set_texture_stage_state,
    validate_device: scalar::validate_device,
    get_info: scalar::get_info,
    set_palette_entries: scalar::set_palette_entries,
    get_palette_entries: scalar::get_palette_entries,
    set_current_texture_palette: scalar::set_current_texture_palette,
    get_current_texture_palette: scalar::get_current_texture_palette,
    draw_primitive,
    draw_indexed_primitive: resources::draw_indexed_primitive,
    draw_primitive_up,
    draw_indexed_primitive_up: resources::draw_indexed_primitive_up,
    process_vertices: resources::process_vertices,
    create_vertex_shader: shader::create_vertex_shader,
    set_vertex_shader: shader::set_vertex_shader,
    get_vertex_shader: shader::get_vertex_shader,
    delete_vertex_shader: shader::delete_vertex_shader,
    set_vertex_shader_constant: shader::set_vertex_shader_constant,
    get_vertex_shader_constant: shader::get_vertex_shader_constant,
    get_vertex_shader_declaration: shader::get_vertex_shader_declaration,
    get_vertex_shader_function: shader::get_vertex_shader_function,
    set_stream_source: resources::set_stream_source,
    get_stream_source: resources::get_stream_source,
    set_indices: resources::set_indices,
    get_indices: resources::get_indices,
    create_pixel_shader: shader::create_pixel_shader,
    set_pixel_shader: shader::set_pixel_shader,
    get_pixel_shader: shader::get_pixel_shader,
    delete_pixel_shader: shader::delete_pixel_shader,
    set_pixel_shader_constant: shader::set_pixel_shader_constant,
    get_pixel_shader_constant: shader::get_pixel_shader_constant,
    get_pixel_shader_function: shader::get_pixel_shader_function,
    draw_rect_patch,
    draw_tri_patch,
    delete_patch,
};

#[repr(C)]
pub struct Device8 {
    vtable: &'static IDirect3DDevice8Vtbl,
    references: AtomicU32,
    inner: Box<DeviceInner>,
}

struct DeviceInner {
    backend: Backend<IDirect3DDevice9Vtbl>,
    parent: *mut c_void,
    back_buffer: Mutex<Option<Box<Surface8>>>,
    api_lock: Option<Arc<ApiLock>>,
    reset_required: AtomicBool,
    resources: crate::resource::ResourceRegistry,
    state: Mutex<state::State8>,
    implicit_depth: AtomicUsize,
    additional_chains: AtomicU32,
}

impl Device8 {
    /// Creates a device wrapper and adopts the caller's factory reference.
    ///
    /// # Safety
    /// `parent` must be a live `Direct3D8` pointer carrying one owned reference.
    pub unsafe fn create(
        backend: Backend<IDirect3DDevice9Vtbl>,
        parent: *mut c_void,
        flags: u32,
    ) -> *mut c_void {
        let implicit_depth = surfaces::implicit_depth_identity(&backend);
        Box::into_raw(Box::new(Self {
            vtable: &VTABLE,
            references: AtomicU32::new(1),
            inner: Box::new(DeviceInner {
                backend,
                implicit_depth: AtomicUsize::new(implicit_depth),
                additional_chains: AtomicU32::new(0),
                parent,
                back_buffer: Mutex::new(None),
                api_lock: (flags & D3DCREATE_MULTITHREADED != 0).then(|| Arc::new(ApiLock::new())),
                reset_required: AtomicBool::new(false),
                resources: crate::resource::ResourceRegistry::new(),
                state: Mutex::new(state::State8::new(flags)),
            }),
        }))
        .cast()
    }

    /// Registers one live frontend additional-chain wrapper as a Reset blocker.
    pub fn register_swap_chain(&self) {
        self.inner.additional_chains.fetch_add(1, Ordering::Relaxed);
    }

    /// Releases the Reset blocker when an additional-chain wrapper is destroyed.
    pub fn unregister_swap_chain(&self) {
        let previous = self.inner.additional_chains.fetch_sub(1, Ordering::Relaxed);
        debug_assert!(previous != 0);
    }

    pub fn is_implicit_depth(&self, identity: usize) -> bool {
        identity != 0 && self.inner.implicit_depth.load(Ordering::Relaxed) == identity
    }

    pub fn state(&self) -> MutexGuard<'_, state::State8> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub const fn resources(&self) -> &crate::resource::ResourceRegistry {
        &self.inner.resources
    }

    pub const fn backend(&self) -> &Backend<IDirect3DDevice9Vtbl> {
        &self.inner.backend
    }

    pub fn back_buffer(&self) -> MutexGuard<'_, Option<Box<Surface8>>> {
        self.inner
            .back_buffer
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Device8 {
    fn drop(&mut self) {
        direct3d::release(self.inner.parent);
    }
}

/// Locks a device while retaining the lock through its final release.
///
/// # Safety
/// `this` must name a live `Device8` for this call.
pub unsafe fn api_scope(this: *mut c_void) -> ApiScope {
    // SAFETY: the caller guarantees the device receiver's type and lifetime.
    let object = unsafe { object(this) };
    ApiScope::enter(object.inner.api_lock.as_ref())
}

/// Borrows a device for one ABI call.
///
/// # Safety
/// `this` must name a live `Device8` for the entire returned lifetime.
pub const unsafe fn object<'a>(this: *mut c_void) -> InPtr<'a, Device8> {
    // SAFETY: callers dispatch a live D3D8 device or hold its child-owned reference.
    unsafe { InPtr::new(this) }
}

pub extern "system" fn query_interface(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM caller supplies IUnknown parameters for this device.
    unsafe { identity::query(this, iid, output, &IID_IDIRECT3DDEVICE8, add_ref) }
}

pub extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
    object.references.fetch_add(1, Ordering::Relaxed) + 1
}

pub extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller owns the reference being released.
    let object = unsafe { object(this) };
    let remaining = object.references.fetch_sub(1, Ordering::Release) - 1;
    if remaining == 0 {
        core::sync::atomic::fence(Ordering::Acquire);
        // SAFETY: the final reference owns the allocation; _api retains its lock separately.
        drop(unsafe { Box::from_raw(this.cast::<Device8>()) });
    }
    remaining
}

extern "system" fn get_direct3d(this: *mut c_void, output: *mut *mut c_void) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this COM receiver owns its factory reference.
    let object = unsafe { object(this) };
    let parent = object.inner.parent;
    direct3d::add_ref(parent);
    output.write(parent);
    D3D_OK
}

extern "system" fn get_device_caps(this: *mut c_void, output: *mut D3DCAPS8) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable capability storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
    let mut caps = MaybeUninit::uninit();
    // SAFETY: caps has the backend's complete capability output layout.
    let result = unsafe {
        (object.inner.backend.table().get_device_caps)(
            object.inner.backend.pointer(),
            caps.as_mut_ptr(),
        )
    };
    if result >= 0 {
        // SAFETY: successful GetDeviceCaps initialized its output.
        let caps = unsafe { caps.assume_init() };
        output.write(to_d3d8_caps(&caps));
    }
    result
}

extern "system" fn reset(this: *mut c_void, parameters: *mut D3DPRESENT_PARAMETERS8) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: parameters is an exclusive in-out argument of Reset.
    let Some(mut parameters) =
        (unsafe { InPtrMut::<D3DPRESENT_PARAMETERS8>::opt(parameters.cast()) })
    else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
    let cached = object.back_buffer();
    if cached
        .as_ref()
        .is_some_and(|surface| surface.has_references())
        || object.resources().has_default_pool_resources()
        || object.inner.additional_chains.load(Ordering::Relaxed) != 0
    {
        object.inner.reset_required.store(true, Ordering::Relaxed);
        return D3DERR_DEVICELOST;
    }
    drop(cached);
    let mut converted = to_d3d9_present_parameters(&parameters);
    // SAFETY: converted provides the backend's complete presentation-parameter layout.
    let result = unsafe {
        (object.inner.backend.table().reset)(
            object.inner.backend.pointer(),
            (&raw mut converted).cast(),
        )
    };
    if result >= 0 {
        *object.back_buffer() = None;
        object.inner.reset_required.store(false, Ordering::Relaxed);
        object.state().reset_bindings();
        object.inner.implicit_depth.store(
            surfaces::implicit_depth_identity(object.backend()),
            Ordering::Relaxed,
        );
        copy_d3d9_present_parameters(&mut parameters, &converted);
    }
    result
}

extern "system" fn get_back_buffer(
    this: *mut c_void,
    index: u32,
    _kind: u32,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the caller supplies writable interface output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
    let mut surface = ptr::null_mut();
    // SAFETY: the output slot is writable. D3D8 ignores the back-buffer type argument.
    let result = unsafe {
        (object.inner.backend.table().get_back_buffer)(
            object.inner.backend.pointer(),
            0,
            index,
            0,
            &raw mut surface,
        )
    };
    if result < 0 {
        output.write(ptr::null_mut());
        return result;
    }
    // SAFETY: successful GetBackBuffer returns one owned IDirect3DSurface9 reference.
    let Some(surface) = (unsafe { Backend::adopt(surface) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    let mut cached = object.back_buffer();
    let wrapper = cached.get_or_insert_with(|| {
        // SAFETY: this device owns the cache and keeps its inactive shell alive.
        Box::new(unsafe { Surface8::new(this) })
    });
    let pointer = wrapper.acquire(surface);
    drop(cached);
    output.write(pointer);
    D3D_OK
}

extern "system" fn test_cooperative_level(this: *mut c_void) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
    if object.inner.reset_required.load(Ordering::Relaxed) {
        return D3DERR_DEVICENOTRESET;
    }
    // SAFETY: the backend reference and vtable belong together.
    unsafe { (object.inner.backend.table().test_cooperative_level)(object.inner.backend.pointer()) }
}

extern "system" fn get_available_texture_mem(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
    // SAFETY: the backend reference and vtable belong together.
    unsafe {
        (object.inner.backend.table().get_available_texture_mem)(object.inner.backend.pointer())
    }
}

extern "system" fn resource_manager_discard_bytes(this: *mut c_void, _bytes: u32) -> i32 {
    // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
    // SAFETY: D3D9's eviction drops all managed allocations, satisfying any byte minimum.
    unsafe {
        (object.inner.backend.table().evict_managed_resources)(object.inner.backend.pointer())
    }
}

macro_rules! forward {
    ($name:ident, ($($argument:ident : $kind:ty),*)) => {
        extern "system" fn $name(this: *mut c_void, $($argument: $kind),*) -> i32 {
            // SAFETY: the COM receiver and its owning device remain live for this call.
    let _api = unsafe { api_scope(this) };
            // SAFETY: the COM receiver is a live Device8 for this call.
    let object = unsafe { object(this) };
            // SAFETY: this method shares its parameter ABI with D3D9 and owns its receiver.
            unsafe { (object.inner.backend.table().$name)(object.inner.backend.pointer(), $($argument),*) }
        }
    };
}
forward!(present, (source: *const c_void, destination: *const c_void, window: *mut c_void, dirty: *const c_void));
forward!(draw_rect_patch, (handle: u32, segments: *const f32, info: *const c_void));
forward!(draw_tri_patch, (handle: u32, segments: *const f32, info: *const c_void));
forward!(delete_patch, (handle: u32));
forward!(begin_scene, ());
forward!(end_scene, ());
forward!(clear, (count: u32, rectangles: *const c_void, flags: u32, color: u32, depth: f32, stencil: u32));
forward!(draw_primitive, (primitive: u32, start: u32, count: u32));
forward!(draw_primitive_up, (primitive: u32, count: u32, data: *const c_void, stride: u32));
