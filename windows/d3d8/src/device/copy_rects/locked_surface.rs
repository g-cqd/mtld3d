//! A checked surface mapping whose copy extent comes from the mapped region.

use core::{mem::MaybeUninit, ptr};

use mtld3d_core::format::map_d3d_format;
use mtld3d_types::{
    D3DERR_INVALIDCALL, D3DLOCK_READONLY, D3DLOCKED_RECT, D3DRECT, IDirect3DSurface9Vtbl,
};

use crate::backend::Backend;

/// The backend keeps exactly this mapped region valid until the guard unlocks it.
pub struct LockedSurface<'a> {
    surface: &'a Backend<IDirect3DSurface9Vtbl>,
    mapping: D3DLOCKED_RECT,
    row_bytes: usize,
    rows: usize,
    writable: bool,
}

impl<'a> LockedSurface<'a> {
    pub fn new(
        surface: &'a Backend<IDirect3DSurface9Vtbl>,
        rectangle: &D3DRECT,
        flags: u32,
    ) -> Result<Self, i32> {
        let description = super::description(surface)?;
        let format = map_d3d_format(description.format).ok_or(D3DERR_INVALIDCALL)?;
        let width = rectangle
            .x2
            .checked_sub(rectangle.x1)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(D3DERR_INVALIDCALL)?;
        let height = rectangle
            .y2
            .checked_sub(rectangle.y1)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(D3DERR_INVALIDCALL)?;
        if width == 0 || height == 0 {
            return Err(D3DERR_INVALIDCALL);
        }
        let (row_bytes, rows) = if format.is_compressed() {
            (
                width
                    .div_ceil(format.block_width())
                    .checked_mul(format.block_bytes())
                    .ok_or(D3DERR_INVALIDCALL)?,
                height.div_ceil(format.block_height()),
            )
        } else {
            (
                width
                    .checked_mul(format.bytes_per_pixel())
                    .ok_or(D3DERR_INVALIDCALL)?,
                height,
            )
        };
        let mut mapping = MaybeUninit::uninit();
        // SAFETY: the owned backend validates the region and writes a complete lock descriptor.
        let status = unsafe {
            (surface.table().lock_rect)(
                surface.pointer(),
                mapping.as_mut_ptr(),
                ptr::from_ref(rectangle).cast(),
                flags,
            )
        };
        if status < 0 {
            return Err(status);
        }
        // SAFETY: successful LockRect initialized the descriptor and retained the validated region.
        let mapping = unsafe { mapping.assume_init() };
        let locked = Self {
            surface,
            mapping,
            row_bytes: row_bytes as usize,
            rows: rows as usize,
            writable: flags & D3DLOCK_READONLY == 0,
        };
        let pitch = locked.pitch()?;
        if pitch < locked.row_bytes
            || locked.mapping.bits.is_null()
            || locked
                .rows
                .checked_mul(pitch)
                .is_none_or(|length| length > isize::MAX as usize)
        {
            return Err(D3DERR_INVALIDCALL);
        }
        Ok(locked)
    }

    pub fn copy_to(&self, destination: &mut Self) -> Result<(), i32> {
        if !destination.writable
            || self.row_bytes != destination.row_bytes
            || self.rows != destination.rows
        {
            return Err(D3DERR_INVALIDCALL);
        }
        let source_pitch = self.pitch()?;
        let destination_pitch = destination.pitch()?;
        for row in 0..self.rows {
            // SAFETY: construction checked the mapping's row count, pitch, and complete byte extent.
            let source = unsafe { self.mapping.bits.cast::<u8>().add(row * source_pitch) };
            // SAFETY: the same checks bound this row inside the destination's writable lock region.
            let destination = unsafe {
                destination
                    .mapping
                    .bits
                    .cast::<u8>()
                    .add(row * destination_pitch)
            };
            // SAFETY: both rows contain row_bytes bytes; the destination guard guarantees writability.
            unsafe { ptr::copy(source, destination, self.row_bytes) };
        }
        Ok(())
    }

    fn pitch(&self) -> Result<usize, i32> {
        usize::try_from(self.mapping.pitch).map_err(|_| D3DERR_INVALIDCALL)
    }
}

impl Drop for LockedSurface<'_> {
    fn drop(&mut self) {
        // SAFETY: construction acquired exactly one lock on this still-owned surface.
        let status = unsafe { (self.surface.table().unlock_rect)(self.surface.pointer()) };
        if status < 0 {
            log::error!("D3D8 CopyRects failed to unlock surface: {status:#x}");
        }
    }
}
