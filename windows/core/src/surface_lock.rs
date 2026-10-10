//! Which store `LockRect` maps for a standalone colour surface.
//!
//! A colour surface with no parent texture reaches `LockRect` in one of three
//! shapes. `CreateRenderTarget` with `Lockable == TRUE` carries a CPU staging
//! buffer alongside its colour texture; the device's implicit back buffer
//! carries no persistent CPU store but is read back on demand; and a
//! `Lockable == FALSE` render target has no CPU-visible store at all, which
//! D3D9 answers with `D3DERR_INVALIDCALL`. The surface plumbing lives in
//! `windows/d3d9`; the routing decision is here, with the geometry of the
//! page a writable back-buffer lock hands out and writes back.

use crate::{render_scale::RenderScale, stretch_rect::StretchRegion};

/// The store a standalone colour surface's `LockRect` hands back.
#[derive(Debug, PartialEq, Eq)]
pub enum ColorSurfaceLock {
    /// The surface's own CPU staging buffer, uploaded back at `UnlockRect`.
    Staging,
    /// A one-shot read-back page, blitted out of the colour texture.
    ///
    /// Held until `UnlockRect` drops it. The implicit back buffer keeps no
    /// persistent CPU store, so this is the only way its pixels reach the
    /// application.
    BackBufferReadback,
    /// No CPU-visible store: `D3DERR_INVALIDCALL`.
    Reject,
}

/// Route a standalone colour surface's `LockRect` to the store that serves it.
///
/// `has_staging` marks a lockable render target, the one standalone colour
/// surface that owns CPU bytes. `is_back_buffer` marks the device's implicit
/// back buffer, whose read-back serves the screenshot and portrait paths an
/// application drives through a read-only lock. Anything else is a
/// `Lockable == FALSE` render target: D3D9 gives it no lock, so neither do we.
#[must_use]
pub const fn classify_color_surface_lock(
    has_staging: bool,
    is_back_buffer: bool,
) -> ColorSurfaceLock {
    if has_staging {
        ColorSurfaceLock::Staging
    } else if is_back_buffer {
        ColorSurfaceLock::BackBufferReadback
    } else {
        ColorSurfaceLock::Reject
    }
}

/// Byte offset of `region`'s first texel inside a page laid out at `pitch`.
///
/// The page a writable back-buffer lock maps holds the whole surface, and
/// `bits` points at the locked rect's first texel inside it. `None` when the
/// offset overflows or does not lie inside a page of `page_len` bytes, which
/// the caller answers with `D3DERR_INVALIDCALL` rather than hand out a pointer
/// past the page.
#[must_use]
pub fn region_offset(
    region: StretchRegion,
    pitch: u32,
    bytes_per_pixel: u32,
    page_len: usize,
) -> Option<usize> {
    let row = (region.y as usize).checked_mul(pitch as usize)?;
    let column = (region.x as usize).checked_mul(bytes_per_pixel as usize)?;
    row.checked_add(column).filter(|&offset| offset < page_len)
}

/// The source and destination regions of the upload that writes `region` of a page back.
///
/// The source is `region` in the page's logical space. The destination is the
/// same rect in the render-resolution texture, scaled by its edges so two
/// abutting rects never gap or overlap. At 100% they are equal. Below that,
/// only the rect the application locked is resampled, so the texels around it
/// keep what the render path drew instead of taking a round trip through the
/// downscale.
#[must_use]
pub fn writeback_regions(
    scale: RenderScale,
    region: StretchRegion,
) -> (StretchRegion, StretchRegion) {
    let (x, y, w, h) = scale.rect(region.x, region.y, region.w, region.h);
    (region, StretchRegion { x, y, w, h })
}

#[cfg(test)]
mod tests;
