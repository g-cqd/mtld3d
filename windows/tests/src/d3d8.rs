//! D3D8 COM calls used by the separate frontend integration binary.

use core::ffi::c_void;

use mtld3d_d3d8_types::{
    D3DPRESENT_PARAMETERS8, D3DSDK_VERSION8, IDirect3D8Vtbl, IDirect3DDevice8Vtbl, IID_IDIRECT3D8,
};
use mtld3d_types::{
    D3DCLEAR_TARGET, D3DCREATE_HARDWARE_VERTEXPROCESSING, D3DDEVTYPE_HAL, D3DFMT_A8R8G8B8,
    D3DPRESENTFLAG_LOCKABLE_BACKBUFFER, D3DPT_TRIANGLELIST, D3DSWAPEFFECT_DISCARD,
};

use crate::{
    RhwVertex,
    check::{expect_created, expect_ok},
    resource::release_unknown,
    vtbl::deref_vtbl,
    win32,
};

mod adapter;
pub mod resources;
mod shaders;
mod state;
mod surface;
mod swapchain;
mod targets;
mod texture;

pub use surface::D3D8Surface;
pub use swapchain::D3D8SwapChain;
pub use texture::D3D8Texture;

#[cfg_attr(
    target_arch = "x86",
    link(name = "d3d8", kind = "raw-dylib", import_name_type = "undecorated")
)]
#[cfg_attr(target_arch = "x86_64", link(name = "d3d8", kind = "raw-dylib"))]
unsafe extern "system" {
    fn Direct3DCreate8(sdk_version: u32) -> *mut c_void;
}

/// A D3D8 factory and a 64 by 64 device with a lockable back buffer.
pub struct D3D8Harness {
    factory: *mut c_void,
    device: *mut c_void,
    window: usize,
}

impl D3D8Harness {
    /// Creates the frontend through its exported entry point.
    ///
    /// # Panics
    /// Panics if the window, factory, or device cannot be created.
    #[must_use]
    pub fn new() -> Self {
        Self::create(false, D3DSDK_VERSION8).expect("D3D8 SDK is supported")
    }

    /// Creates the harness with an automatic D24S8 depth surface.
    ///
    /// # Panics
    /// Panics if the window, factory, or device cannot be created.
    #[must_use]
    pub fn new_with_depth() -> Self {
        Self::create(true, D3DSDK_VERSION8).expect("D3D8 SDK is supported")
    }

    /// Creates a device for the requested SDK, or returns `None` when the factory rejects it.
    ///
    /// # Panics
    /// Panics if an accepted factory cannot create the window or device.
    #[must_use]
    pub fn with_sdk(sdk_version: u32) -> Option<Self> {
        Self::create(false, sdk_version)
    }

    fn create(auto_depth: bool, sdk_version: u32) -> Option<Self> {
        crate::in_flight::announce();
        win32::install_failure_exit_hook();
        // SAFETY: the exported factory has no pointer arguments.
        let factory = unsafe { Direct3DCreate8(sdk_version) };
        if factory.is_null() {
            return None;
        }
        let window = win32::create_styled_window(64, 64, false, &win32::WindowStyle::Borderless);
        let mut parameters = presentation_parameters(window, 64, 64);
        parameters.enable_auto_depth_stencil = u32::from(auto_depth);
        parameters.auto_depth_stencil_format = mtld3d_types::D3DFMT_D24S8;
        let mut device = core::ptr::null_mut();
        // SAFETY: the non-null factory was returned by Direct3DCreate8.
        let vtable = unsafe { deref_vtbl::<IDirect3D8Vtbl>(factory) };
        // SAFETY: live factory and window; both output arguments are writable.
        let result = unsafe {
            (vtable.create_device)(
                factory,
                0,
                D3DDEVTYPE_HAL,
                window,
                D3DCREATE_HARDWARE_VERTEXPROCESSING,
                &raw mut parameters,
                &raw mut device,
            )
        };
        expect_created(result, device, "D3D8 CreateDevice");
        Some(Self {
            factory,
            device,
            window,
        })
    }

    /// The number of adapters reported through the D3D8 interface.
    #[must_use]
    pub fn adapter_count(&self) -> u32 {
        // SAFETY: the harness owns the factory reference.
        unsafe { (self.factory_vtable().get_adapter_count)(self.factory) }
    }

    /// Checks that factory queries and the device's parent retain COM identity.
    ///
    /// # Panics
    /// Panics if either query fails.
    #[must_use]
    pub fn factory_identity_is_preserved(&self) -> bool {
        let mut queried = core::ptr::null_mut();
        let iid = IID_IDIRECT3D8;
        // SAFETY: the factory and IID are live and queried is writable.
        let result = unsafe {
            (self.factory_vtable().query_interface)(self.factory, &raw const iid, &raw mut queried)
        };
        expect_created(result, queried, "D3D8 QueryInterface");
        let same_query = queried == self.factory;
        // SAFETY: QueryInterface returned an owned reference to this factory.
        unsafe { release_unknown(queried) };
        let mut parent = core::ptr::null_mut();
        // SAFETY: the device is live and parent is writable.
        let result = unsafe { (self.device_vtable().get_direct3d)(self.device, &raw mut parent) };
        expect_created(result, parent, "D3D8 GetDirect3D");
        let same_parent = parent == self.factory;
        // SAFETY: GetDirect3D returned an owned reference to this factory.
        unsafe { release_unknown(parent) };
        same_query && same_parent
    }

    /// Applies a D3D8 render-state write and returns its HRESULT.
    #[must_use]
    pub fn set_render_state(&self, state: u32, value: u32) -> i32 {
        // SAFETY: the device is live; the arguments are ABI values.
        unsafe { (self.device_vtable().set_render_state)(self.device, state, value) }
    }

    /// Applies a D3D8 texture-stage-state write and returns its HRESULT.
    #[must_use]
    pub fn set_texture_stage_state(&self, index: u32, state: u32, value: u32) -> i32 {
        // SAFETY: the device is live; the arguments are ABI values.
        unsafe { (self.device_vtable().set_texture_stage_state)(self.device, index, state, value) }
    }

    /// Selects an FVF or shader handle through the D3D8 shader entry point.
    #[must_use]
    pub fn set_vertex_shader(&self, shader: u32) -> i32 {
        // SAFETY: the device is live; shader is a D3D8 handle or FVF.
        unsafe { (self.device_vtable().set_vertex_shader)(self.device, shader) }
    }

    /// Clears the back buffer without presenting or discarding its contents.
    ///
    /// # Panics
    /// Panics when the clear fails.
    pub fn clear(&self, color: u32) {
        // SAFETY: a zero rectangle count permits a null rectangle pointer.
        let result = unsafe {
            (self.device_vtable().clear)(
                self.device,
                0,
                core::ptr::null(),
                D3DCLEAR_TARGET,
                color,
                1.0,
                0,
            )
        };
        expect_ok(result, "D3D8 Clear");
    }

    /// Draws one pretransformed triangle, preserving its result for readback.
    ///
    /// # Panics
    /// Panics if scene entry, drawing, or scene exit fails.
    pub fn draw_triangle(&self, vertices: &[RhwVertex; 3]) {
        // SAFETY: the harness owns this device and no scene is open.
        expect_ok(
            unsafe { (self.device_vtable().begin_scene)(self.device) },
            "D3D8 BeginScene",
        );
        // SAFETY: three contiguous vertices satisfy one triangle; stride matches their layout.
        let result = unsafe {
            (self.device_vtable().draw_primitive_up)(
                self.device,
                D3DPT_TRIANGLELIST,
                1,
                vertices.as_ptr().cast(),
                u32::try_from(size_of::<RhwVertex>()).expect("vertex stride fits u32"),
            )
        };
        expect_ok(result, "D3D8 DrawPrimitiveUP");
        // SAFETY: BeginScene succeeded on this device.
        expect_ok(
            unsafe { (self.device_vtable().end_scene)(self.device) },
            "D3D8 EndScene",
        );
    }

    /// Obtains an owned reference to the first back buffer.
    ///
    /// # Panics
    /// Panics if the back buffer cannot be obtained.
    #[must_use]
    pub fn back_buffer(&self) -> D3D8Surface<'_> {
        self.try_back_buffer(0, 0).expect("D3D8 GetBackBuffer")
    }

    /// Queries a back buffer, preserving an error for contract assertions.
    ///
    /// # Errors
    /// Returns the HRESULT when the getter fails.
    ///
    /// # Panics
    /// Panics if the getter returns an output inconsistent with its HRESULT.
    pub fn try_back_buffer(&self, index: u32, kind: u32) -> Result<D3D8Surface<'_>, i32> {
        let mut surface = core::ptr::null_mut();
        // SAFETY: live device and writable output; the getter validates the index.
        let result = unsafe {
            (self.device_vtable().get_back_buffer)(self.device, index, kind, &raw mut surface)
        };
        if result < 0 {
            assert!(surface.is_null(), "failed GetBackBuffer retained an output");
            return Err(result);
        }
        expect_created(result, surface, "D3D8 GetBackBuffer");
        // SAFETY: GetBackBuffer returned an owned IDirect3DSurface8 reference.
        Ok(unsafe { D3D8Surface::from_owned(surface) })
    }

    /// Resizes the windowed back buffer and returns the reset HRESULT.
    #[must_use]
    pub fn reset(&self, width: u32, height: u32) -> i32 {
        let mut parameters = presentation_parameters(self.window, width, height);
        // SAFETY: the device is live and parameters is readable and writable.
        unsafe { (self.device_vtable().reset)(self.device, &raw mut parameters) }
    }

    /// Returns the device's cooperative status after a failed or successful reset.
    #[must_use]
    pub fn cooperative_level(&self) -> i32 {
        // SAFETY: the harness owns the device reference.
        unsafe { (self.device_vtable().test_cooperative_level)(self.device) }
    }

    fn factory_vtable(&self) -> &'static IDirect3D8Vtbl {
        // SAFETY: the harness retains this factory until Drop.
        unsafe { deref_vtbl(self.factory) }
    }

    fn device_vtable(&self) -> &'static IDirect3DDevice8Vtbl {
        // SAFETY: the harness retains this device until Drop.
        unsafe { deref_vtbl(self.device) }
    }
}

const fn presentation_parameters(window: usize, width: u32, height: u32) -> D3DPRESENT_PARAMETERS8 {
    D3DPRESENT_PARAMETERS8 {
        back_buffer_width: width,
        back_buffer_height: height,
        back_buffer_format: D3DFMT_A8R8G8B8,
        back_buffer_count: 1,
        multi_sample_type: 0,
        swap_effect: D3DSWAPEFFECT_DISCARD,
        device_window: window,
        windowed: 1,
        enable_auto_depth_stencil: 0,
        auto_depth_stencil_format: 0,
        flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
        full_screen_refresh_rate_in_hz: 0,
        full_screen_presentation_interval: 0,
    }
}

impl Default for D3D8Harness {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for D3D8Harness {
    fn drop(&mut self) {
        // SAFETY: release the one device reference owned by this harness.
        unsafe { (self.device_vtable().release)(self.device) };
        // SAFETY: release the one factory reference owned by this harness.
        unsafe { (self.factory_vtable().release)(self.factory) };
        win32::destroy_window(self.window);
    }
}
