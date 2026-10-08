//! Device-owned implicit render-target / backbuffer / depth-stencil surfaces.
//!
//! `GetRenderTarget(0)`, `GetBackBuffer(0)` and `GetDepthStencilSurface` each
//! return a single cached, device-owned object: the same pointer every call,
//! `GetRenderTarget(0) == GetBackBuffer(0)`, surviving its refcount reaching
//! zero (destroyed only at device teardown), and resolving its extent live from
//! the device so a `Reset` that recreates the backbuffer is reflected without
//! re-allocating the surface. The render-target and depth bindings a game
//! made, of those surfaces or of its own, outlive an automatic back-buffer
//! resize.

use mtld3d_tests::{Harness, HarnessConfig, PosColorVertex, RhwVertex};
use mtld3d_types::{
    D3D_OK, D3DCLEAR_TARGET, D3DCLEAR_ZBUFFER, D3DCULL_NONE, D3DERR_DEVICENOTRESET,
    D3DERR_INVALIDCALL, D3DERR_NOTFOUND, D3DFMT_A8R8G8B8, D3DFMT_A16B16G16R16F, D3DFMT_D24S8,
    D3DFMT_R5G6B5, D3DFMT_X8R8G8B8, D3DFVF_DIFFUSE, D3DFVF_XYZ, D3DFVF_XYZRHW, D3DLOCK_READONLY,
    D3DPRESENT_PARAMETERS, D3DPRESENTFLAG_LOCKABLE_BACKBUFFER, D3DPT_TRIANGLESTRIP, D3DRECT,
    D3DRS_ALPHABLENDENABLE, D3DRS_CULLMODE, D3DRS_LIGHTING, D3DRS_ZENABLE, D3DSWAPEFFECT_DISCARD,
    D3DVIEWPORT9,
};

const WM_SIZE: u32 = 0x0005;
const BLUE: u32 = 0xFF00_00FF;
const RED: u32 = 0xFFFF_0000;
const GREEN: u32 = 0xFF00_FF00;

fn assert_backbuffer_format(h: &Harness, expected: u32) {
    let chain = h.implicit_swapchain();
    for surface in [h.render_target(0), h.back_buffer(0), chain.back_buffer()] {
        let (hr, desc) = surface.desc();
        assert_eq!(hr, D3D_OK, "GetDesc");
        assert_eq!(desc.format, expected, "backbuffer format");
    }
}

#[test]
fn backbuffer_reporting_preserves_alpha_format_at_creation() {
    let h = Harness::create(&HarnessConfig {
        back_buffer_format: D3DFMT_A8R8G8B8,
        ..HarnessConfig::default()
    });
    assert_backbuffer_format(&h, D3DFMT_A8R8G8B8);
}

#[test]
fn backbuffer_reporting_tracks_same_size_format_resets() {
    let h = Harness::new();
    let chain = h.implicit_swapchain();
    let (hr, mut pp) = chain.present_parameters();
    assert_eq!(hr, D3D_OK);
    let original = h.back_buffer(0).as_ptr();
    for format in [
        D3DFMT_X8R8G8B8,
        D3DFMT_A8R8G8B8,
        D3DFMT_X8R8G8B8,
        D3DFMT_A8R8G8B8,
    ] {
        pp.back_buffer_format = format;
        assert_eq!(h.reset_params(&mut pp), D3D_OK, "same-size Reset");
        assert_eq!(
            h.back_buffer(0).as_ptr(),
            original,
            "cached surface identity"
        );
        assert_backbuffer_format(&h, format);
        let (hr, reported) = chain.present_parameters();
        assert_eq!(hr, D3D_OK);
        assert_eq!(
            reported.back_buffer_format, format,
            "cached swapchain format"
        );
    }
}

#[test]
fn backbuffer_reporting_retains_bgra8_fallback_pitch() {
    const FILL: u32 = 0xff20_4080;
    let h = Harness::create(&HarnessConfig {
        back_buffer_format: D3DFMT_R5G6B5,
        config_entries: "render.scale=1",
        ..HarnessConfig::default()
    });
    for format in [D3DFMT_R5G6B5, D3DFMT_A16B16G16R16F] {
        let (hr, mut pp) = h.implicit_swapchain().present_parameters();
        assert_eq!(hr, D3D_OK);
        pp.back_buffer_format = format;
        assert_eq!(h.reset_params(&mut pp), D3D_OK);
        assert_backbuffer_format(&h, D3DFMT_X8R8G8B8);
        assert_eq!(h.clear_target(FILL), D3D_OK);
        let surface = h.back_buffer(0);
        let locked = surface.lock_rect(D3DLOCK_READONLY);
        assert_eq!(
            locked.pitch(),
            640 * 4,
            "BGRA8 backing keeps a four-byte pitch"
        );
        assert_eq!(locked.as_u32(1)[0], FILL, "fallback readback colour");
    }
}

#[test]
fn backbuffer_reporting_resolves_additional_swapchain_unknown_format() {
    let h = Harness::create(&HarnessConfig {
        back_buffer_format: D3DFMT_A8R8G8B8,
        ..HarnessConfig::default()
    });
    let (hr, mut pp) = h.implicit_swapchain().present_parameters();
    assert_eq!(hr, D3D_OK);
    pp.back_buffer_format = 0;
    pp.back_buffer_width = 0;
    pp.back_buffer_height = 0;
    pp.back_buffer_count = 0;
    pp.device_window = 0;
    let chain = h.additional_swapchain_params(&mut pp);
    assert_eq!(
        pp.back_buffer_format, D3DFMT_X8R8G8B8,
        "resolve the desktop format"
    );
    assert_eq!((pp.back_buffer_width, pp.back_buffer_height), (640, 480));
    assert_eq!(pp.back_buffer_count, 1);
    assert_eq!(pp.device_window, 0, "the caller's window stays as supplied");
    let (hr, reported) = chain.present_parameters();
    assert_eq!(hr, D3D_OK);
    assert_eq!(reported.back_buffer_format, pp.back_buffer_format);
    assert_eq!(reported.device_window, h.hwnd());
}

#[test]
fn backbuffer_reporting_tracks_auto_resize_in_present_parameters() {
    for cache_first in [false, true] {
        let h = Harness::create(&HarnessConfig {
            back_buffer_format: D3DFMT_A8R8G8B8,
            ..HarnessConfig::default()
        });
        let mut chain = cache_first.then(|| h.implicit_swapchain());
        for (width, height) in [(320_u32, 240_u32), (800, 600)] {
            let size = isize::try_from((height << 16) | width).unwrap();
            h.send_window_message(WM_SIZE, 0, size);
            let (hr, desc) = h.back_buffer(0).desc();
            assert_eq!(hr, D3D_OK);
            assert_eq!((desc.width, desc.height), (width, height));
            let sc = chain.get_or_insert_with(|| h.implicit_swapchain());
            let (hr, pp) = sc.present_parameters();
            assert_eq!(hr, D3D_OK);
            assert_eq!(
                (pp.back_buffer_width, pp.back_buffer_height),
                (width, height)
            );
            assert_eq!(pp.back_buffer_format, D3DFMT_A8R8G8B8);
            assert_eq!(pp.device_window, h.hwnd());
        }
    }
}

#[test]
fn implicit_render_target_is_cached_and_aliases_backbuffer() {
    let h = Harness::new();

    let rt1 = h.render_target(0);
    let rt2 = h.render_target(0);
    assert_eq!(
        rt1.as_ptr(),
        rt2.as_ptr(),
        "GetRenderTarget(0) must return the one cached implicit surface every call"
    );

    let bb = h.back_buffer(0);
    assert_eq!(
        rt1.as_ptr(),
        bb.as_ptr(),
        "GetRenderTarget(0) and GetBackBuffer(0) are the same device-owned object"
    );
}

#[test]
fn implicit_render_target_survives_refcount_zero() {
    let h = Harness::new();

    // Take the cached pointer, then release every reference to it.
    let cached = {
        let rt = h.render_target(0);
        rt.as_ptr()
    };

    // Device-owned: it is NOT freed at refcount 0, so re-acquiring returns the
    // very same object (D3D9 never re-allocates the implicit render target).
    let rt_again = h.render_target(0);
    assert_eq!(
        rt_again.as_ptr(),
        cached,
        "the implicit render target must persist past refcount 0"
    );

    // Still live + usable: its description resolves the current backbuffer size.
    let (hr, desc) = rt_again.desc();
    assert_eq!(hr, 0, "GetDesc on the re-acquired implicit RT");
    assert_eq!((desc.width, desc.height), (640, 480), "live extent");
}

#[test]
fn implicit_render_target_extent_tracks_reset_live() {
    let h = Harness::new();

    let before = h.render_target(0).as_ptr();

    let hr = h.reset(320, 240);
    assert_eq!(hr, 0, "Reset(320x240) failed: 0x{hr:08X}");

    // Identity is stable across Reset (the cached surface is never re-allocated),
    // while its extent resolves LIVE from the recreated backbuffer — proving the
    // surface does not snapshot a now-freed Metal handle.
    let rt = h.render_target(0);
    assert_eq!(
        rt.as_ptr(),
        before,
        "implicit RT identity must survive Reset"
    );
    let (hr, desc) = rt.desc();
    assert_eq!(hr, 0, "GetDesc after Reset");
    assert_eq!(
        (desc.width, desc.height),
        (320, 240),
        "implicit RT extent must track the post-Reset backbuffer (live resolution)"
    );
}

#[test]
fn get_dc_on_non_lockable_backbuffer_rejects_and_preserves_out() {
    let h = Harness::new();

    // The default backbuffer is non-lockable, so `GetDC` rejects with
    // `INVALIDCALL` and must leave the caller's out `HDC` untouched. Seed the
    // out slot with a sentinel and assert it survives the rejected call.
    let sentinel = 0xdead_beef_usize as *mut core::ffi::c_void;
    let (hr, out) = h.back_buffer(0).get_dc(sentinel);
    assert_eq!(
        hr, D3DERR_INVALIDCALL,
        "GetDC on a non-lockable backbuffer must return INVALIDCALL"
    );
    assert_eq!(
        out, sentinel,
        "a rejected GetDC must not write through the out HDC"
    );
}

#[test]
fn release_dc_on_a_lockable_backbuffer_reaches_the_back_buffer() {
    // The DC over a lockable back buffer wraps a read-back snapshot rather than
    // the back buffer's own pixels, so it owes the surface coherence in both
    // directions: it shows what the GPU painted before it, and what GDI draws
    // into it reaches the back buffer at `ReleaseDC`, with no Present in
    // between. Every coordinate here is the reported one, so the test also
    // stands under `make test SCALE=<n>`.
    const GREEN: u32 = 0xFF00_FF00;
    const RED: u32 = 0xFFFF_0000;
    const GREEN_COLORREF: u32 = 0x0000_FF00;
    const RED_COLORREF: u32 = 0x0000_00FF;
    let h = Harness::with_lockable_back_buffer();

    assert_eq!(h.clear_target(GREEN), D3D_OK, "clear the back buffer green");
    let bb = h.back_buffer(0);
    let dc = bb.dc();
    assert_eq!(
        dc.get_pixel(320, 240),
        GREEN_COLORREF,
        "the DC reads the colour the Clear painted",
    );
    dc.fill_block(64, RED_COLORREF);
    assert_eq!(dc.release(), D3D_OK, "ReleaseDC");

    // Alpha is masked off: GDI leaves the fourth byte at zero, but a
    // `render.scale` below 100% returns the frame through the MetalFX resolve,
    // which hands back an opaque one whatever the surface holds. The claim
    // here is about the colour GDI drew, not about the byte it did not write.
    assert_eq!(
        h.read_pixel(16, 16) | 0xFF00_0000,
        RED,
        "what GDI drew into the DC reaches the back buffer",
    );
    assert_eq!(
        h.read_pixel(320, 240),
        GREEN,
        "the pixels GDI left alone still hold the clear colour",
    );
}

#[test]
fn release_dc_on_a_lockable_backbuffer_resamples_under_a_render_scale() {
    // `render.scale` rasterizes the back buffer smaller than the extent `GetDC`
    // hands the DIB out at, so the write-back has to resample on the way in.
    // Pinning the key here runs that path in every test run rather than only in
    // the scaled sweep; a machine without MetalFX holds the scale at 1.0 and
    // takes the direct upload, which the same assertions cover.
    const GREEN: u32 = 0xFF00_FF00;
    const RED: u32 = 0xFFFF_0000;
    const RED_COLORREF: u32 = 0x0000_00FF;
    let h = Harness::create(&HarnessConfig {
        present_flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
        config_entries: "render.scale=0.75",
        ..HarnessConfig::default()
    });

    assert_eq!(h.clear_target(GREEN), D3D_OK, "clear the back buffer green");
    let bb = h.back_buffer(0);
    let dc = bb.dc();
    dc.fill_block(64, RED_COLORREF);
    assert_eq!(dc.release(), D3D_OK, "ReleaseDC");

    // Deep inside the block on both sides of the round trip, so the linear
    // downscale and the resolve back up both read only red neighbours.
    assert_eq!(
        h.read_pixel(16, 16) | 0xFF00_0000,
        RED,
        "the write-back resamples GDI's drawing into the scaled back buffer",
    );
    assert_eq!(
        h.read_pixel(320, 240),
        GREEN,
        "the pixels GDI left alone still hold the clear colour",
    );
}

#[test]
fn read_only_lock_rect_on_a_non_lockable_backbuffer_reads_the_rendered_pixels() {
    // D3D9 gives a backbuffer created without `D3DPRESENTFLAG_LOCKABLE_BACKBUFFER`
    // no CPU access at all and rejects every `LockRect` of it. A read-only lock
    // is accepted here and served by a GPU read-back instead, because that is
    // the shape of the screenshot and character-portrait paths titles drive
    // through the backbuffer. A lock that asks to write is still rejected, and
    // so is a lock of any other non-lockable render target.
    const WIDTH: u32 = 640;
    const FILL: u32 = 0xFF20_4080;
    let h = Harness::new();
    assert_eq!(h.clear_target(FILL), 0, "clear the backbuffer");
    let backbuffer = h.back_buffer(0);

    let (hr, bits_null) = backbuffer.lock_rect_probe(0);
    assert_eq!(
        hr, D3DERR_INVALIDCALL,
        "a writable lock of a non-lockable backbuffer must return INVALIDCALL"
    );
    assert!(
        !bits_null,
        "a rejected LockRect must leave the caller's D3DLOCKED_RECT untouched"
    );
    assert_eq!(
        backbuffer.unlock_rect(),
        D3DERR_INVALIDCALL,
        "UnlockRect without a lock held must return INVALIDCALL"
    );

    let locked = backbuffer.lock_rect(D3DLOCK_READONLY);
    assert_eq!(
        locked.pitch().cast_unsigned(),
        WIDTH * 4,
        "the read-back page steps by the backbuffer format's row pitch"
    );
    assert_eq!(
        locked.as_u32(1)[0],
        FILL,
        "the read-back must show the cleared backbuffer"
    );
}

/// A back-buffer lock is recorded, so a second lock and a `GetDC` wait for its unlock.
///
/// The lock maps a read-back page, and a second lock or a `GetDC` that
/// succeeded would replace that page under the pointer the first lock handed
/// out. D3D9 refuses both while the surface is mapped, for the read-only lock
/// of a back buffer that is not lockable as for one that is.
#[test]
fn back_buffer_lock_rect_refuses_a_second_lock_and_get_dc() {
    const FILL: u32 = 0xFF20_4080;
    for lockable in [false, true] {
        let h = if lockable {
            Harness::with_lockable_back_buffer()
        } else {
            Harness::new()
        };
        assert_eq!(h.clear_target(FILL), D3D_OK, "clear the back buffer");
        let back_buffer = h.back_buffer(0);
        let locked = back_buffer.lock_rect(D3DLOCK_READONLY);
        let (hr, bits_null) = back_buffer.lock_rect_probe(D3DLOCK_READONLY);
        assert_eq!(
            hr, D3DERR_INVALIDCALL,
            "a second LockRect of a locked back buffer (lockable={lockable})"
        );
        assert!(
            !bits_null,
            "a refused LockRect leaves the caller's D3DLOCKED_RECT untouched"
        );
        let sentinel = 0xdead_beef_usize as *mut core::ffi::c_void;
        let (hr, out) = back_buffer.get_dc(sentinel);
        assert_eq!(
            hr, D3DERR_INVALIDCALL,
            "GetDC on a locked back buffer (lockable={lockable})"
        );
        assert_eq!(out, sentinel, "a refused GetDC leaves the out HDC alone");
        assert_eq!(
            locked.as_u32(1)[0],
            FILL,
            "the first lock's page still holds the read-back"
        );
        drop(locked);
        assert_eq!(
            back_buffer.unlock_rect(),
            D3DERR_INVALIDCALL,
            "a second UnlockRect finds no lock"
        );
    }
}

/// An `UnlockRect` with no lock under a back-buffer DC leaves the DC's page in place.
///
/// The DC wraps the read-back page a lock would map, and `UnlockRect` of a
/// surface that is not mapped while a DC is out is a no-op `D3D_OK`. What GDI
/// draws after it still reaches the back buffer at `ReleaseDC`, and a
/// `LockRect` while the DC is out is refused.
#[test]
fn back_buffer_unlock_rect_under_a_dc_keeps_what_gdi_draws() {
    const GREEN: u32 = 0xFF00_FF00;
    const RED: u32 = 0xFFFF_0000;
    const RED_COLORREF: u32 = 0x0000_00FF;
    let h = Harness::with_lockable_back_buffer();
    assert_eq!(h.clear_target(GREEN), D3D_OK, "clear the back buffer green");
    let back_buffer = h.back_buffer(0);
    let dc = back_buffer.dc();
    let (hr, bits_null) = back_buffer.lock_rect_probe(D3DLOCK_READONLY);
    assert_eq!(hr, D3DERR_INVALIDCALL, "LockRect while a GetDC is out");
    assert!(
        !bits_null,
        "a refused LockRect leaves the caller's D3DLOCKED_RECT untouched"
    );
    assert_eq!(
        back_buffer.unlock_rect(),
        D3D_OK,
        "UnlockRect of an unmapped surface under a DC is a no-op"
    );
    dc.fill_block(64, RED_COLORREF);
    assert_eq!(dc.release(), D3D_OK, "ReleaseDC");
    // Alpha is masked off as in the write-back test above: GDI leaves the
    // fourth byte at zero.
    assert_eq!(
        h.read_pixel(16, 16) | 0xFF00_0000,
        RED,
        "what GDI drew after the UnlockRect reaches the back buffer"
    );
    assert_eq!(
        h.read_pixel(320, 240),
        GREEN,
        "the pixels GDI left alone still hold the clear colour"
    );
}

#[test]
fn lockable_backbuffer_writable_lock_uploads_the_complete_surface() {
    const FILL: u32 = 0xff20_4080;
    let h = Harness::create(&HarnessConfig {
        width: 64,
        height: 64,
        present_flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
        config_entries: "render.scale=1",
        ..HarnessConfig::default()
    });
    assert_eq!(h.clear_target(0xff00_0000), D3D_OK);
    let backbuffer = h.back_buffer(0);
    {
        let mut locked = backbuffer.lock_rect(0);
        locked.write_u32_rect(64, 64, &[FILL; 64 * 64]);
    }
    for (x, y) in [(0, 0), (31, 32), (63, 63)] {
        assert_eq!(h.read_pixel(x, y), FILL, "uploaded pixel ({x}, {y})");
    }
    assert_eq!(backbuffer.unlock_rect(), D3DERR_INVALIDCALL);
}

#[test]
fn unlock_rect_preserves_cpu_pixels_across_a_partial_draw() {
    assert_cpu_pixels_survive_partial_draw(false);
}

#[test]
fn release_dc_preserves_cpu_pixels_across_a_partial_draw() {
    assert_cpu_pixels_survive_partial_draw(true);
}

#[test]
fn discard_preservation_keeps_scene_pixels_under_transition_ui() {
    let h = Harness::create(&HarnessConfig {
        width: 64,
        height: 64,
        depth_format: Some(mtld3d_types::D3DFMT_D24S8),
        config_entries: "render.scale=1;shader.asyncCompile=false;render.preserveDiscardBackbuffer=true",
        ..HarnessConfig::default()
    });
    assert_transition_frames_preserve_color(&h, 64);
    let (hr, mut pp) = h.implicit_swapchain().present_parameters();
    assert_eq!(hr, D3D_OK);
    pp.back_buffer_width = 80;
    pp.back_buffer_height = 80;
    assert_eq!(h.reset_params(&mut pp), D3D_OK);
    assert_transition_frames_preserve_color(&h, 80);
}

fn assert_transition_frames_preserve_color(h: &Harness, width: u16) {
    use mtld3d_types::{
        D3DBLEND_INVSRCALPHA, D3DBLEND_SRCALPHA, D3DCLEAR_STENCIL, D3DCLEAR_ZBUFFER,
        D3DRS_DESTBLEND, D3DRS_SRCBLEND,
    };

    const BACKGROUND: u32 = 0xff20_4080;
    let size = usize::from(width);
    let (hr, pp) = h.implicit_swapchain().present_parameters();
    assert_eq!(hr, D3D_OK);
    assert_eq!(pp.swap_effect, D3DSWAPEFFECT_DISCARD);
    assert_eq!(h.set_render_state(D3DRS_LIGHTING, 0), D3D_OK);
    assert_eq!(h.set_render_state(D3DRS_ZENABLE, 0), D3D_OK);
    assert_eq!(h.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), D3D_OK);
    assert_eq!(h.set_render_state(D3DRS_ALPHABLENDENABLE, 1), D3D_OK);
    assert_eq!(
        h.set_render_state(D3DRS_SRCBLEND, D3DBLEND_SRCALPHA),
        D3D_OK
    );
    assert_eq!(
        h.set_render_state(D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA),
        D3D_OK
    );
    assert_eq!(h.clear_texture(0), D3D_OK);
    h.select_diffuse_stage(0);
    assert_eq!(h.set_fvf(D3DFVF_XYZRHW | D3DFVF_DIFFUSE), D3D_OK);
    assert_eq!(h.clear_target(BACKGROUND), D3D_OK);
    assert_eq!(h.present(), D3D_OK);

    let low = f32::from(width) * 0.25;
    let high = f32::from(width) * 0.75;
    let quad = [(low, low), (high, low), (low, high), (high, high)].map(|(x, y)| RhwVertex {
        x,
        y,
        z: 0.5,
        rhw: 1.0,
        color: 0x8000_ff00,
    });
    let backbuffer = h.back_buffer(0);
    for expected_rgb in [[16_u32, 160, 64], [8, 208, 32], [4, 232, 16]] {
        // Morrowind's captured door frames clear only depth/stencil, then
        // alpha-blend UI while relying on the preceding frame's color.
        assert_eq!(
            h.clear(D3DCLEAR_ZBUFFER | D3DCLEAR_STENCIL, 0, 1.0, 0),
            D3D_OK
        );
        assert_eq!(h.begin_scene(), D3D_OK);
        assert_eq!(h.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &quad), D3D_OK);
        assert_eq!(h.end_scene(), D3D_OK);
        {
            // This readback does not force Load as GetRenderTargetData can.
            let locked = backbuffer.lock_rect(D3DLOCK_READONLY);
            assert_eq!(locked.pitch(), i32::from(width) * 4);
            let pixels = locked.as_u32(size * size);
            for (x, y) in [(4, 4), (size - 4, 4), (4, size - 4), (size - 4, size - 4)] {
                assert_eq!(pixels[y * size + x] & 0x00ff_ffff, BACKGROUND & 0x00ff_ffff);
            }
            let center = pixels[(size / 2) * size + size / 2];
            for (shift, expected) in [16, 8, 0].into_iter().zip(expected_rgb) {
                let actual = (center >> shift) & 0xff;
                assert!(
                    actual.abs_diff(expected) <= 2,
                    "blended channel {shift}: {actual}, expected {expected}"
                );
            }
        }
        assert_eq!(h.present(), D3D_OK);
    }
}

fn assert_cpu_pixels_survive_partial_draw(use_dc: bool) {
    const GREEN: u32 = 0xff00_ff00;
    let background = if use_dc { 0xffff_0000 } else { 0xff00_00ff };
    let h = Harness::create(&HarnessConfig {
        width: 64,
        height: 64,
        present_flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
        config_entries: "render.scale=1;shader.asyncCompile=false",
        ..HarnessConfig::default()
    });
    let (hr, pp) = h.implicit_swapchain().present_parameters();
    assert_eq!(hr, D3D_OK);
    assert_eq!(pp.swap_effect, D3DSWAPEFFECT_DISCARD);
    assert_eq!(
        h.set_viewport(&D3DVIEWPORT9 {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
            min_z: 0.0,
            max_z: 1.0,
        }),
        D3D_OK
    );
    assert_eq!(h.set_render_state(D3DRS_LIGHTING, 0), D3D_OK);
    assert_eq!(h.set_render_state(D3DRS_ZENABLE, 0), D3D_OK);
    assert_eq!(h.set_render_state(D3DRS_ALPHABLENDENABLE, 0), D3D_OK);
    assert_eq!(h.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), D3D_OK);
    assert_eq!(h.clear_texture(0), D3D_OK);
    h.select_diffuse_stage(0);
    assert_eq!(h.set_fvf(D3DFVF_XYZRHW | D3DFVF_DIFFUSE), D3D_OK);

    // Start a fresh DISCARD frame. None of its expected pixels come from
    // this clear: the CPU explicitly initializes every pixel afterwards.
    assert_eq!(h.clear_target(0xff00_0000), D3D_OK);
    assert_eq!(h.present(), D3D_OK);
    let backbuffer = h.back_buffer(0);
    if use_dc {
        let dc = backbuffer.dc();
        dc.fill_block(64, 0x0000_00ff);
        assert_eq!(dc.release(), D3D_OK);
    } else {
        let mut locked = backbuffer.lock_rect(0);
        locked.write_u32_rect(64, 64, &[background; 64 * 64]);
    }

    let quad = [(16.0, 16.0), (48.0, 16.0), (16.0, 48.0), (48.0, 48.0)].map(|(x, y)| RhwVertex {
        x,
        y,
        z: 0.5,
        rhw: 1.0,
        color: GREEN,
    });
    assert_eq!(h.begin_scene(), D3D_OK);
    assert_eq!(h.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &quad), D3D_OK);
    assert_eq!(h.end_scene(), D3D_OK);
    // GetRenderTargetData marks the target read before submission, which
    // can force Load and hide a missing CPU-upload dependency. Read through
    // the backbuffer's own lock API instead.
    let locked = backbuffer.lock_rect(D3DLOCK_READONLY);
    assert_eq!(locked.pitch(), 64 * 4);
    let pixels = locked.as_u32(64 * 64);
    assert_eq!(
        pixels[32 * 64 + 32],
        GREEN,
        "the partial draw reaches the GPU"
    );
    for (x, y) in [(4, 4), (60, 4), (4, 60), (60, 60)] {
        // X8R8G8B8 has no alpha contract, and GDI writes zero in that byte.
        assert_eq!(
            pixels[y * 64 + x] & 0x00ff_ffff,
            background & 0x00ff_ffff,
            "CPU-written pixel ({x}, {y}) outside the draw, use_dc={use_dc}"
        );
    }
}

#[test]
fn lockable_backbuffer_writable_subrect_preserves_surrounding_pixels() {
    const BACKGROUND: u32 = 0xff20_4080;
    const FILL: u32 = 0xff80_4020;
    for scale in ["render.scale=1", "render.scale=0.75"] {
        let h = Harness::create(&HarnessConfig {
            width: 64,
            height: 64,
            present_flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
            config_entries: scale,
            ..HarnessConfig::default()
        });
        assert_eq!(h.clear_target(BACKGROUND), D3D_OK);
        assert_eq!(
            h.clear_target_rects(
                0xffff_ffff,
                &[D3DRECT {
                    x1: 0,
                    y1: 24,
                    x2: 8,
                    y2: 40
                }]
            ),
            D3D_OK
        );
        let untouched_edge = [6, 7, 8, 9].map(|x| h.read_pixel(x, 32));
        let backbuffer = h.back_buffer(0);
        {
            let mut locked = backbuffer.lock_rect_partial(&[16, 16, 48, 48], 0);
            assert_eq!(locked.as_u32(1)[0], BACKGROUND, "read before write");
            locked.write_u32_rect(32, 32, &[FILL; 32 * 32]);
        }
        assert_eq!(h.read_pixel(32, 32), FILL, "subrectangle center, {scale}");
        for (x, before) in [6, 7, 8, 9].into_iter().zip(untouched_edge) {
            assert_eq!(
                h.read_pixel(x, 32),
                before,
                "edge outside the written rectangle at x={x}, {scale}"
            );
        }
        for (x, y) in [(4, 4), (60, 4), (4, 60), (60, 60)] {
            assert_eq!(
                h.read_pixel(x, y),
                BACKGROUND,
                "unmodified pixel ({x}, {y}), {scale}"
            );
        }
    }
}

#[test]
fn lockable_backbuffer_readonly_locks_do_not_resample_the_surface() {
    let h = Harness::create(&HarnessConfig {
        width: 64,
        height: 64,
        present_flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
        config_entries: "render.scale=0.75",
        ..HarnessConfig::default()
    });
    assert_eq!(h.clear_target(0xff00_0000), D3D_OK);
    assert_eq!(
        h.clear_target_rects(
            0xffff_ffff,
            &[D3DRECT {
                x1: 0,
                y1: 0,
                x2: 31,
                y2: 64
            }]
        ),
        D3D_OK
    );
    let backbuffer = h.back_buffer(0);
    let before = {
        let locked = backbuffer.lock_rect(D3DLOCK_READONLY);
        assert_eq!(locked.pitch(), 64 * 4);
        locked.as_u32(64 * 64).to_vec()
    };
    for _ in 0..2 {
        let locked = backbuffer.lock_rect(D3DLOCK_READONLY);
        assert_eq!(
            locked.as_u32(64 * 64),
            before,
            "a read-only unlock must not upload and resample the edge"
        );
    }
}

#[test]
fn lockable_backbuffer_rejects_a_second_lock_without_replacing_the_mapping() {
    const FILL: u32 = 0xff20_4080;
    let h = Harness::create(&HarnessConfig {
        width: 64,
        height: 64,
        present_flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
        config_entries: "render.scale=1",
        ..HarnessConfig::default()
    });
    assert_eq!(h.clear_target(FILL), D3D_OK);
    let backbuffer = h.back_buffer(0);
    let locked = backbuffer.lock_rect(D3DLOCK_READONLY);
    let (result, bits_null) = backbuffer.lock_rect_probe(D3DLOCK_READONLY);
    assert_eq!(result, D3DERR_INVALIDCALL, "a second lock is rejected");
    assert!(
        !bits_null,
        "the rejected lock does not clear its output pointer"
    );
    assert_eq!(locked.as_u32(1)[0], FILL, "the first mapping remains live");
}

#[test]
fn lockable_backbuffer_lock_and_dc_keep_exclusive_snapshot_ownership() {
    const FILL: u32 = 0xff20_4080;
    let h = Harness::create(&HarnessConfig {
        width: 64,
        height: 64,
        present_flags: D3DPRESENTFLAG_LOCKABLE_BACKBUFFER,
        config_entries: "render.scale=1",
        ..HarnessConfig::default()
    });
    assert_eq!(h.clear_target(FILL), D3D_OK);
    let backbuffer = h.back_buffer(0);
    {
        let locked = backbuffer.lock_rect(D3DLOCK_READONLY);
        let sentinel = core::ptr::without_provenance_mut(0xdead_beef);
        let (result, output) = backbuffer.get_dc(sentinel);
        assert_eq!(result, D3DERR_INVALIDCALL, "GetDC during LockRect");
        assert_eq!(output, sentinel, "rejected GetDC preserves its output");
        assert_eq!(locked.as_u32(1)[0], FILL, "LockRect still owns its page");
    }
    let dc = backbuffer.dc();
    assert_eq!(
        backbuffer.unlock_rect(),
        D3D_OK,
        "UnlockRect during GetDC is a no-op"
    );
    assert_eq!(
        dc.get_pixel(32, 32),
        0x0080_4020,
        "GetDC still owns its page"
    );
    assert_eq!(dc.release(), D3D_OK);
    assert_eq!(h.read_pixel(32, 32), FILL);
}

#[test]
fn implicit_depth_stencil_is_cached() {
    let h = Harness::with_depth();

    let ds1 = h
        .depth_stencil_surface()
        .expect("auto depth-stencil present");
    let ds2 = h
        .depth_stencil_surface()
        .expect("auto depth-stencil present");
    assert_eq!(
        ds1.as_ptr(),
        ds2.as_ptr(),
        "GetDepthStencilSurface must return the one cached implicit surface"
    );
}

// ── Bindings across an automatic back-buffer resize ──

/// A `WM_SIZE` lparam: the client height in the high word, the width in the low.
///
/// The two words fill 32 bits, which go through `i32` because an i686 lparam
/// has no more: a height past 32767 sets its sign bit there.
fn client_size(width: u16, height: u16) -> isize {
    let words = (u32::from(height) << 16) | u32::from(width);
    isize::try_from(words.cast_signed()).expect("an i32 fits an lparam")
}

/// A clip-space quad over the whole viewport in `color`, at depth 0.5.
fn full_quad(color: u32) -> [PosColorVertex; 4] {
    [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)].map(|(x, y)| PosColorVertex {
        x,
        y,
        z: 0.5,
        color,
    })
}

/// Lighting off and the diffuse colour on stage 0, for [`full_quad`].
fn arm_diffuse_draws(h: &Harness) {
    assert_eq!(h.set_render_state(D3DRS_LIGHTING, 0), D3D_OK);
    assert_eq!(h.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), D3D_OK);
    assert_eq!(h.set_fvf(D3DFVF_XYZ | D3DFVF_DIFFUSE), D3D_OK);
    h.select_diffuse_stage(0);
}

#[test]
fn explicitly_bound_implicit_surfaces_follow_an_auto_resize() {
    // The back buffer and the implicit depth surface bound by hand are the
    // surfaces the resize replaces. The frame after the resize and the one
    // after its Present must both draw into the new pair, depth-tested
    // against the new depth surface.
    let h = Harness::with_depth();
    arm_diffuse_draws(&h);
    {
        let back_buffer = h.back_buffer(0);
        let depth = h.depth_stencil_surface().expect("implicit depth surface");
        assert_eq!(h.set_render_target(0, &back_buffer), D3D_OK);
        assert_eq!(h.set_depth_stencil_surface(&depth), D3D_OK);
    }
    let _ = h.send_window_message(WM_SIZE, 0, client_size(320, 240));
    let (hr, desc) = h.back_buffer(0).desc();
    assert_eq!(hr, D3D_OK);
    assert_eq!((desc.width, desc.height), (320, 240), "the resize took");
    let vp = h.viewport();
    assert_eq!(
        (vp.width, vp.height),
        (320, 240),
        "the viewport follows the back buffer bound as render target 0"
    );
    for (color, frame) in [
        (RED, "the frame after the resize"),
        (GREEN, "the next frame"),
    ] {
        let quad = full_quad(color);
        h.render_once(BLUE, |d| {
            assert_eq!(
                d.clear(D3DCLEAR_ZBUFFER, 0, 1.0, 0),
                D3D_OK,
                "{frame}: clear depth"
            );
            assert_eq!(d.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &quad), D3D_OK);
        });
        assert_eq!(
            h.read_pixel(160, 120),
            color,
            "{frame}: the quad reaches the back buffer"
        );
    }
}

#[test]
fn offscreen_render_target_and_depth_bindings_survive_a_mid_frame_auto_resize() {
    // A resize between a Clear and a draw of one frame flushes the frame. The
    // draw after it still goes to the render target and the depth surface the
    // game bound, not to the new back buffer.
    let h = Harness::with_depth();
    arm_diffuse_draws(&h);
    let target = h.create_render_target(64, 64, D3DFMT_A8R8G8B8);
    let depth = h.create_depth_stencil_surface(64, 64, D3DFMT_D24S8);
    assert_eq!(h.set_render_target(0, &target), D3D_OK);
    assert_eq!(h.set_depth_stencil_surface(&depth), D3D_OK);
    let quad = full_quad(GREEN);
    assert!(h.pump(), "WM_QUIT before render");
    assert_eq!(h.begin_scene(), D3D_OK);
    assert_eq!(
        h.clear(D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER, BLUE, 1.0, 0),
        D3D_OK,
        "clear the target and its depth"
    );
    let _ = h.send_window_message(WM_SIZE, 0, client_size(320, 240));
    // The viewport and scissor `SetRenderTarget` gave the 64x64 target stay:
    // the resize changed the back buffer, not the target being drawn.
    let vp = h.viewport();
    assert_eq!(
        (vp.x, vp.y, vp.width, vp.height),
        (0, 0, 64, 64),
        "the viewport keeps the target's extent"
    );
    let scissor = h.scissor_rect();
    assert_eq!(
        (scissor.x1, scissor.y1, scissor.x2, scissor.y2),
        (0, 0, 64, 64),
        "the scissor keeps the target's extent"
    );
    assert_eq!(h.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &quad), D3D_OK);
    assert_eq!(h.end_scene(), D3D_OK);
    assert_eq!(h.present(), D3D_OK);
    assert_eq!(
        h.render_target(0).as_ptr(),
        target.as_ptr(),
        "the game's target is still bound"
    );
    assert_eq!(
        h.read_pixel(32, 32),
        GREEN,
        "the draw after the resize reached the target"
    );
}

#[test]
fn a_failed_auto_resize_leaves_no_destroyed_texture_bound() {
    // A back buffer Metal refuses (65535 texels a side) leaves the device
    // requiring Reset, with the old back buffer and depth texture already
    // destroyed. A Clear recorded meanwhile reaches the GPU with the flush
    // the Reset starts with, and must find no destroyed texture bound,
    // neither the default target nor the back buffer and depth surface the
    // game bound by hand.
    const OVERSIZE: u16 = 0xffff;
    let h = Harness::with_depth();
    arm_diffuse_draws(&h);
    {
        let back_buffer = h.back_buffer(0);
        let depth = h.depth_stencil_surface().expect("implicit depth surface");
        assert_eq!(h.set_render_target(0, &back_buffer), D3D_OK);
        assert_eq!(h.set_depth_stencil_surface(&depth), D3D_OK);
    }
    let _ = h.send_window_message(WM_SIZE, 0, client_size(OVERSIZE, OVERSIZE));
    assert_eq!(
        h.test_cooperative_level(),
        D3DERR_DEVICENOTRESET,
        "the refused back buffer requires Reset"
    );
    assert_eq!(
        h.clear(D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER, RED, 1.0, 0),
        D3D_OK,
        "a Clear before the Reset"
    );
    assert_eq!(h.reset(640, 480), D3D_OK, "Reset rebuilds the back buffer");
    // Reset returned every state to its default, the FVF included.
    arm_diffuse_draws(&h);
    let quad = full_quad(GREEN);
    h.render_once(BLUE, |d| {
        assert_eq!(d.clear(D3DCLEAR_ZBUFFER, 0, 1.0, 0), D3D_OK);
        assert_eq!(d.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &quad), D3D_OK);
    });
    assert_eq!(
        h.read_pixel(320, 240),
        GREEN,
        "the device draws after the Reset"
    );
}

/// Bind the back buffer and the implicit depth surface by hand, as saved bindings.
fn bind_implicit_surfaces_by_hand(h: &Harness) {
    let back_buffer = h.back_buffer(0);
    let depth = h.depth_stencil_surface().expect("implicit depth surface");
    assert_eq!(h.set_render_target(0, &back_buffer), D3D_OK);
    assert_eq!(h.set_depth_stencil_surface(&depth), D3D_OK);
}

/// After a `Reset` failed past its destroy, a `Clear` and the next `Reset` find nothing destroyed.
///
/// A back buffer Metal refuses (65535 texels a side) fails `Reset` after the
/// old back buffer and depth texture are destroyed. The same request again
/// fails the same way rather than taking the same-size path, as no back
/// buffer of that size exists. A `Clear` recorded before the `Reset` the
/// device then requires reaches the GPU with the flush that `Reset` starts
/// with, and must find neither the destroyed textures nor the saved bindings
/// of them by hand; the device then draws.
#[test]
fn a_reset_whose_back_buffer_is_refused_leaves_no_destroyed_texture_bound() {
    const OVERSIZE: u32 = 0xffff;
    let h = Harness::with_depth();
    arm_diffuse_draws(&h);
    bind_implicit_surfaces_by_hand(&h);
    assert_eq!(
        h.reset(OVERSIZE, OVERSIZE),
        D3DERR_INVALIDCALL,
        "a back buffer Metal refuses fails the Reset"
    );
    assert_eq!(h.test_cooperative_level(), D3DERR_DEVICENOTRESET);
    assert_eq!(
        h.reset(OVERSIZE, OVERSIZE),
        D3DERR_INVALIDCALL,
        "the same refused size fails the Reset again"
    );
    assert_eq!(h.test_cooperative_level(), D3DERR_DEVICENOTRESET);
    assert_eq!(
        h.clear(D3DCLEAR_TARGET, RED, 1.0, 0),
        D3D_OK,
        "a Clear before the next Reset"
    );
    assert_eq!(h.reset(640, 480), D3D_OK, "Reset rebuilds the back buffer");
    arm_diffuse_draws(&h);
    let quad = full_quad(GREEN);
    h.render_once(BLUE, |d| {
        assert_eq!(d.clear(D3DCLEAR_ZBUFFER, 0, 1.0, 0), D3D_OK);
        assert_eq!(d.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &quad), D3D_OK);
    });
    assert_eq!(
        h.read_pixel(320, 240),
        GREEN,
        "the device draws after the Reset"
    );
}

/// A `Reset` whose new depth format is no depth format leaves no destroyed depth bound.
///
/// The `Reset` refuses the `AutoDepthStencilFormat` before it touches the
/// implicit surfaces, on a same-size request and on one that also resizes
/// the back buffer, so the implicit depth surface is still the one reported,
/// as after any rejected `Reset`. A `Clear` recorded before the next `Reset`
/// reaches the GPU with that `Reset`'s flush against a depth texture that
/// still exists, and the next `Reset` keeps the depth surface so the device
/// draws depth-tested again.
#[test]
fn a_reset_with_an_unusable_depth_format_leaves_no_destroyed_depth_bound() {
    let h = Harness::with_depth();
    let (width, height) = h.dims();
    for (request, case) in [((width, height), "same size"), ((320, 240), "resizing")] {
        arm_diffuse_draws(&h);
        bind_implicit_surfaces_by_hand(&h);
        let mut pp = D3DPRESENT_PARAMETERS {
            back_buffer_width: request.0,
            back_buffer_height: request.1,
            back_buffer_format: D3DFMT_X8R8G8B8,
            back_buffer_count: 1,
            multi_sample_type: 0,
            multi_sample_quality: 0,
            swap_effect: D3DSWAPEFFECT_DISCARD,
            device_window: h.hwnd(),
            windowed: 1,
            enable_auto_depth_stencil: 1,
            auto_depth_stencil_format: D3DFMT_A8R8G8B8,
            flags: 0,
            full_screen_refresh_rate_in_hz: 0,
            presentation_interval: 0,
        };
        assert_eq!(
            h.reset_params(&mut pp),
            D3DERR_INVALIDCALL,
            "{case}: a colour format as the auto depth format fails the Reset"
        );
        assert_eq!(h.test_cooperative_level(), D3DERR_DEVICENOTRESET);
        let (hr, depth) = h.depth_stencil_surface_hr();
        assert_eq!(
            hr, D3D_OK,
            "{case}: the implicit depth surface outlives the refused Reset"
        );
        // The reference `GetDepthStencilSurface` handed out would hold the
        // next `Reset` off, so it goes before that `Reset`.
        assert!(depth.is_some());
        drop(depth);
        assert_eq!(
            h.clear(D3DCLEAR_TARGET, RED, 1.0, 0),
            D3D_OK,
            "{case}: a Clear before the next Reset"
        );
        assert_eq!(
            h.reset(width, height),
            D3D_OK,
            "{case}: Reset restores the depth"
        );
        assert!(
            h.depth_stencil_surface().is_some(),
            "{case}: the implicit depth surface is back"
        );
        arm_diffuse_draws(&h);
        let near = full_quad(GREEN).map(|v| PosColorVertex { z: 0.25, ..v });
        let far = full_quad(RED);
        h.render_once(BLUE, |d| {
            assert_eq!(d.clear(D3DCLEAR_ZBUFFER, 0, 1.0, 0), D3D_OK);
            assert_eq!(d.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &near), D3D_OK);
            assert_eq!(d.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &far), D3D_OK);
        });
        assert_eq!(
            h.read_pixel(width / 2, height / 2),
            GREEN,
            "{case}: the farther quad is depth-tested away"
        );
    }
}

#[test]
fn an_unbound_depth_surface_stays_unbound_across_an_auto_resize() {
    // `SetDepthStencilSurface(NULL)` is the game's choice and the resize does
    // not undo it: no depth surface is reported, and draws in the frames
    // after the resize run without one.
    let h = Harness::with_depth();
    arm_diffuse_draws(&h);
    assert_eq!(h.clear_depth_stencil_surface(), D3D_OK, "unbind depth");
    let _ = h.send_window_message(WM_SIZE, 0, client_size(320, 240));
    let (hr, surface) = h.depth_stencil_surface_hr();
    assert_eq!(hr, D3DERR_NOTFOUND, "no depth surface after the resize");
    assert!(surface.is_none(), "a null out-pointer with NOTFOUND");
    for (color, frame) in [
        (RED, "the frame after the resize"),
        (GREEN, "the next frame"),
    ] {
        let quad = full_quad(color);
        h.render_once(BLUE, |d| {
            assert_eq!(d.draw_primitive_up(D3DPT_TRIANGLESTRIP, 2, &quad), D3D_OK);
        });
        assert_eq!(h.read_pixel(160, 120), color, "{frame}: the draw lands");
    }
}
