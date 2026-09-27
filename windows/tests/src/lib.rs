//! Shared support library for the mtld3d end-to-end test suite.
//!
//! Integration tests drive the real frontend DLLs through [`Harness`] or
//! [`D3D8Harness`]: one factory + window + device, with safe wrappers around the
//! COM vtables and RAII `resource` handles. All FFI and `unsafe` live here so
//! the test files read as plain D3D9 call sequences with pixel/`HRESULT`
//! assertions. D3D9 constants come from `mtld3d_types`; nothing is restated.

mod check;
mod d3d8;
mod ffi;
mod harness;
mod in_flight;
mod pixel;
mod reread;
mod resource;
mod shared;
mod vertex;
mod vtbl;
mod win32;

/// Typed D3D8 resource wrappers for frontend integration tests.
pub use d3d8::resources;
pub use d3d8::{D3D8Harness, D3D8Surface, D3D8SwapChain, D3D8Texture};
pub use harness::{
    DrawIndexedUpParams, Harness, HarnessConfig, config_value, config_var,
    render_scale_is_identity, run_child,
};
pub use in_flight::spawn_scoped;
pub use pixel::{Rgba8, assert_pixel_approx, assert_pixel_eq};
pub use reread::{Reading, assert_or_reread};
pub use resource::{
    BufferLock, CubeTexture, IndexBuffer, LockedRect, PixelShader, Query, StateBlock, Surface,
    SurfaceDc, SwapChain, Texture, VertexBuffer, VertexDeclaration, VertexShader, Volume,
    VolumeTexture,
};
pub use shared::{SharedDevice, SharedQuery, SharedVertexBuffer};
pub use vertex::{
    LitVertex, PosColorVertex, PosVertex, RhwVertex, SpecularVertex, TexturedVertex, Vertex,
    VolumeVertex,
};
pub use win32::{
    MemorySample, Rect, WM_ACTIVATEAPP, WS_CAPTION, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
    WindowStyle, create_window, cursor_is_live, cursor_mask_bits, destroy_window,
    enumerate_display_sizes, post_quit_message, send_message, set_window_pos, window_rect,
};
