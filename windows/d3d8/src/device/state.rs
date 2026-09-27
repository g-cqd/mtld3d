//! Per-device ownership of D3D8 shader handles and state-block metadata.

use mtld3d_types::{
    D3D8_FVF_MAX, D3DCREATE_MIXED_VERTEXPROCESSING, D3DCREATE_SOFTWARE_VERTEXPROCESSING,
    E_OUTOFMEMORY,
};
use rustc_hash::FxHashMap;

use super::{
    shader::{PixelShader, VertexShader},
    state_block::StateBlock,
};

pub const VERTEX_HANDLE: usize = 0;
pub const PIXEL_HANDLE: usize = 1;
pub const BASE_VERTEX: usize = 2;
pub const Z_BIAS: usize = 3;
pub const SOFTWARE_VP: usize = 4;
pub const PATCH_SEGMENTS: usize = 5;
pub const EXTRA_COUNT: usize = 6;

/// Handle ownership and the live or recorded state unavailable through D3D9.
pub struct State8 {
    pub vertex_shaders: FxHashMap<u32, VertexShader>,
    pub pixel_shaders: FxHashMap<u32, PixelShader>,
    pub blocks: FxHashMap<u32, StateBlock>,
    pub current: [u32; EXTRA_COUNT],
    pub recording: Option<[Option<u32>; EXTRA_COUNT]>,
    next_handle: u32,
    creation_flags: u32,
}

impl State8 {
    pub fn new(flags: u32) -> Self {
        Self {
            vertex_shaders: FxHashMap::default(),
            pixel_shaders: FxHashMap::default(),
            blocks: FxHashMap::default(),
            current: Self::defaults(flags),
            recording: None,
            next_handle: D3D8_FVF_MAX + 1,
            creation_flags: flags,
        }
    }

    const fn defaults(flags: u32) -> [u32; EXTRA_COUNT] {
        [
            0,
            0,
            0,
            0,
            (flags & D3DCREATE_SOFTWARE_VERTEXPROCESSING != 0) as u32,
            1.0f32.to_bits(),
        ]
    }

    pub const fn permits_software_vp(&self, enabled: bool) -> bool {
        self.creation_flags & D3DCREATE_MIXED_VERTEXPROCESSING != 0
            || enabled == (self.creation_flags & D3DCREATE_SOFTWARE_VERTEXPROCESSING != 0)
    }

    /// Returns a fresh handle, never reusing a deleted object's identity.
    pub fn allocate_handle(&mut self) -> Result<u32, i32> {
        let Some(next) = self.next_handle.checked_add(1) else {
            return Err(E_OUTOFMEMORY);
        };
        if self.vertex_shaders.len() + self.pixel_shaders.len() + self.blocks.len() >= 65536 {
            return Err(E_OUTOFMEMORY);
        }
        let handle = self.next_handle;
        self.next_handle = next;
        Ok(handle)
    }

    pub const fn set_extra(&mut self, index: usize, value: u32) {
        if let Some(recording) = self.recording.as_mut() {
            recording[index] = Some(value);
        } else {
            self.current[index] = value;
        }
    }

    pub const fn reset_bindings(&mut self) {
        self.current = Self::defaults(self.creation_flags);
        self.recording = None;
    }

    pub const fn base_vertex(&self) -> u32 {
        self.current[BASE_VERTEX]
    }

    pub const fn set_base_vertex(&mut self, value: u32) {
        self.set_extra(BASE_VERTEX, value);
    }
}
