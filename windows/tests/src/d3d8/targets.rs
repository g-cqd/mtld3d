//! D3D8 image surfaces, render targets, and whole-surface copies.

use core::ptr;

use mtld3d_types::{D3DFMT_A8R8G8B8, D3DRECT, POINT};

use super::{D3D8Harness, D3D8Surface};
use crate::check::{expect_created, expect_ok};

impl D3D8Harness {
    /// Obtains the current depth-stencil surface when one is bound.
    ///
    /// # Panics
    /// Panics if the getter fails for a reason other than an absent binding.
    #[must_use]
    pub fn depth_stencil(&self) -> Option<D3D8Surface<'_>> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns its device and pointer is writable.
        let result = unsafe {
            (self.device_vtable().get_depth_stencil_surface)(self.device, &raw mut pointer)
        };
        if result == mtld3d_types::D3DERR_NOTFOUND {
            assert!(pointer.is_null(), "absent depth surface retained an output");
            return None;
        }
        expect_created(result, pointer, "D3D8 GetDepthStencilSurface");
        // SAFETY: the getter returned one owned D3D8 surface reference.
        Some(unsafe { D3D8Surface::from_owned(pointer) })
    }

    /// Obtains the current render target.
    ///
    /// # Panics
    /// Panics if the getter fails.
    #[must_use]
    pub fn render_target(&self) -> D3D8Surface<'_> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns its device and pointer is writable.
        let result =
            unsafe { (self.device_vtable().get_render_target)(self.device, &raw mut pointer) };
        expect_created(result, pointer, "D3D8 GetRenderTarget");
        // SAFETY: GetRenderTarget returned one owned D3D8 surface reference.
        unsafe { D3D8Surface::from_owned(pointer) }
    }

    /// Creates an A8R8G8B8 image surface in system memory.
    ///
    /// # Panics
    /// Panics if creation fails.
    #[must_use]
    pub fn create_image_surface(&self, width: u32, height: u32) -> D3D8Surface<'_> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns its device and pointer is writable.
        let result = unsafe {
            (self.device_vtable().create_image_surface)(
                self.device,
                width,
                height,
                D3DFMT_A8R8G8B8,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateImageSurface");
        // SAFETY: CreateImageSurface returned one owned D3D8 surface reference.
        unsafe { D3D8Surface::from_owned(pointer) }
    }

    /// Creates a lockable A8R8G8B8 render target without multisampling.
    ///
    /// # Panics
    /// Panics if creation fails.
    #[must_use]
    pub fn create_render_target(&self, width: u32, height: u32) -> D3D8Surface<'_> {
        let mut pointer = ptr::null_mut();
        // SAFETY: the harness owns its device and pointer is writable.
        let result = unsafe {
            (self.device_vtable().create_render_target)(
                self.device,
                width,
                height,
                D3DFMT_A8R8G8B8,
                0,
                1,
                &raw mut pointer,
            )
        };
        expect_created(result, pointer, "D3D8 CreateRenderTarget");
        // SAFETY: CreateRenderTarget returned one owned D3D8 surface reference.
        unsafe { D3D8Surface::from_owned(pointer) }
    }

    /// Sets the target and depth surface, preserving the target when it is absent.
    #[must_use]
    pub fn set_render_target(
        &self,
        color: Option<&D3D8Surface<'_>>,
        depth: Option<&D3D8Surface<'_>>,
    ) -> i32 {
        let color = color.map_or(ptr::null_mut(), D3D8Surface::pointer);
        let depth = depth.map_or(ptr::null_mut(), D3D8Surface::pointer);
        // SAFETY: the device and both optional surface references remain live.
        unsafe { (self.device_vtable().set_render_target)(self.device, color, depth) }
    }

    /// Copies the whole source to an equally sized destination.
    ///
    /// # Panics
    /// Panics if copying fails.
    pub fn copy_surface(&self, source: &D3D8Surface<'_>, destination: &D3D8Surface<'_>) {
        // SAFETY: both owned surfaces are live; zero count and null rectangles select the whole source.
        let result = unsafe {
            (self.device_vtable().copy_rects)(
                self.device,
                source.pointer(),
                ptr::null(),
                0,
                destination.pointer(),
                ptr::null(),
            )
        };
        expect_ok(result, "D3D8 CopyRects");
    }

    /// Copies source rectangles to explicit positions, or to their original offsets.
    ///
    /// # Panics
    /// Panics if the rectangle count exceeds the ABI or the point count differs.
    #[must_use]
    pub fn copy_rects(
        &self,
        source: &D3D8Surface<'_>,
        rectangles: &[D3DRECT],
        destination: &D3D8Surface<'_>,
        points: Option<&[POINT]>,
    ) -> i32 {
        let count = u32::try_from(rectangles.len()).expect("rectangle count fits the ABI");
        let points = points.map_or(ptr::null(), |points| {
            assert_eq!(points.len(), rectangles.len());
            points.as_ptr()
        });
        // SAFETY: owned surfaces and both count-matched slices remain live through the call.
        unsafe {
            (self.device_vtable().copy_rects)(
                self.device,
                source.pointer(),
                rectangles.as_ptr(),
                count,
                destination.pointer(),
                points,
            )
        }
    }
}
