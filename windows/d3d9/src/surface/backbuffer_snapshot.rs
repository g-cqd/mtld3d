//! CPU ownership and logical extent of one backbuffer mapping.

use mtld3d_core::{held_pages::HeldPages, stretch_rect::StretchRegion};

/// A live CPU mapping whose page is released with its lock or device context.
pub struct BackbufferSnapshot {
    /// Full-surface storage for writable mappings; region-only storage for read-only locks.
    pub page: HeldPages,
    /// The logical rectangle exposed by the lock or device context.
    pub region: StretchRegion,
}
