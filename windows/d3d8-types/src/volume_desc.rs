//! D3D8 volume resource metadata.

/// Volume descriptor including the D3D8 byte-size field absent from D3D9.
#[repr(C)]
pub struct D3DVOLUME_DESC8 {
    pub format: u32,
    pub resource_type: u32,
    pub usage: u32,
    pub pool: u32,
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}
