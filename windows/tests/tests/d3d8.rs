//! D3D8 frontend ABI and rendering, loaded in a process independent of the D3D9 suite.

#[path = "d3d8/resources.rs"]
mod resources;
#[path = "d3d8/shaders.rs"]
mod shaders;
#[path = "d3d8/state.rs"]
mod state;

use mtld3d_d3d8_types::{
    IID_IDIRECT3DBASETEXTURE8, IID_IDIRECT3DDEVICE8, IID_IDIRECT3DRESOURCE8, IID_IDIRECT3DTEXTURE8,
    IID_IUNKNOWN,
};
use mtld3d_tests::{D3D8Harness, RhwVertex, assert_pixel_eq};
use mtld3d_types::{
    D3DCULL_NONE, D3DERR_DEVICELOST, D3DERR_DEVICENOTRESET, D3DERR_INVALIDCALL, D3DFVF_DIFFUSE,
    D3DFVF_XYZRHW, D3DRS_CULLMODE, D3DRS_LIGHTING, D3DTA_DIFFUSE, D3DTOP_SELECTARG1,
    D3DTSS_ALPHAARG1, D3DTSS_ALPHAOP, D3DTSS_COLORARG1, D3DTSS_COLOROP, E_NOINTERFACE,
    IID_IDIRECT3DDEVICE9, IID_IDIRECT3DTEXTURE9,
};

#[test]
fn adapter_enumeration_includes_16_and_32_bit_modes_and_checks_boundaries() {
    let sut = D3D8Harness::new();
    let count = sut.adapter_mode_count(0);
    assert!(count > 0);
    let mut formats = std::collections::BTreeSet::new();
    for index in 0..count {
        let mode = sut.adapter_mode(0, index).expect("advertised adapter mode");
        assert!(mode.width > 0 && mode.height > 0);
        formats.insert(mode.format);
    }
    assert!(formats.contains(&mtld3d_types::D3DFMT_X8R8G8B8));
    assert!(formats.contains(&mtld3d_types::D3DFMT_R5G6B5));
    assert!(matches!(
        sut.adapter_mode(0, count),
        Err(D3DERR_INVALIDCALL)
    ));
    assert!(matches!(
        sut.adapter_mode(0, u32::MAX),
        Err(D3DERR_INVALIDCALL)
    ));
    assert_eq!(sut.adapter_mode_count(sut.adapter_count()), 0);
    assert!(matches!(
        sut.adapter_mode(sut.adapter_count(), 0),
        Err(D3DERR_INVALIDCALL)
    ));
}

#[test]
fn implicit_depth_container_and_held_reference_obey_d3d8_reset_rules() {
    let sut = D3D8Harness::new_with_depth();
    let held = sut.depth_stencil().expect("automatic depth surface");
    assert_eq!(
        held.container_is_device(&sut, &IID_IDIRECT3DDEVICE8),
        Ok(true)
    );
    assert!(held.is_same_object(&sut.depth_stencil().expect("repeated depth getter")));
    assert_eq!(sut.reset(32, 48), D3DERR_DEVICELOST);
    assert_eq!(sut.cooperative_level(), D3DERR_DEVICENOTRESET);
    drop(held);
    assert_eq!(sut.reset(32, 48), 0);
    assert_eq!(sut.cooperative_level(), 0);
    assert!(sut.depth_stencil().is_none());
}

#[test]
fn surface_copy_and_target_binding_preserve_identity_and_pixels() {
    let sut = D3D8Harness::new();
    let back_buffer = sut.back_buffer();
    assert!(sut.render_target().is_same_object(&back_buffer));
    let target = sut.create_render_target(64, 64);
    let image = sut.create_image_surface(64, 64);
    assert_eq!(
        image.container_is_device(&sut, &IID_IUNKNOWN),
        Err(E_NOINTERFACE)
    );
    assert_eq!(
        target.container_is_device(&sut, &IID_IUNKNOWN),
        Err(E_NOINTERFACE)
    );
    assert_eq!(sut.set_render_target(Some(&target), None), 0);
    assert!(sut.render_target().is_same_object(&target));
    sut.clear(0xFF12_3456);
    assert_eq!(sut.set_render_target(None, None), 0);
    assert!(sut.render_target().is_same_object(&target));
    sut.copy_surface(&target, &image);
    assert_pixel_eq(
        image.read_pixel(32, 32),
        0xFF12_3456,
        "D3D8 render-target readback",
    );
    assert_eq!(sut.set_render_target(Some(&back_buffer), None), 0);
    sut.copy_surface(&image, &back_buffer);
    assert_pixel_eq(
        back_buffer.read_pixel(32, 32),
        0xFF12_3456,
        "D3D8 image upload",
    );
}

#[test]
fn copy_rects_without_destination_points_preserves_source_offsets() {
    let sut = D3D8Harness::new();
    let source = sut.create_image_surface(64, 64);
    let destination = sut.create_image_surface(64, 64);
    let target = sut.create_render_target(64, 64);
    assert_eq!(sut.set_render_target(Some(&target), None), 0);
    sut.clear(0xFF12_3456);
    sut.copy_surface(&target, &source);
    sut.clear(0xFFAB_CDEF);
    sut.copy_surface(&target, &destination);

    let rectangles = [
        mtld3d_types::D3DRECT {
            x1: 7,
            y1: 9,
            x2: 13,
            y2: 17,
        },
        mtld3d_types::D3DRECT {
            x1: 20,
            y1: 3,
            x2: 25,
            y2: 8,
        },
    ];
    assert_eq!(sut.copy_rects(&source, &rectangles, &destination, None), 0);
    for (x, y) in [(7, 9), (12, 16), (20, 3), (24, 7)] {
        assert_pixel_eq(
            destination.read_pixel(x, y),
            0xFF12_3456,
            "source offset retained",
        );
    }
    for (x, y) in [(0, 0), (6, 9), (13, 16), (19, 3), (25, 7), (63, 63)] {
        assert_pixel_eq(
            destination.read_pixel(x, y),
            0xFFAB_CDEF,
            "outside copy unchanged",
        );
    }
}

#[test]
fn bound_additional_back_buffer_retains_chain_after_public_references_are_released() {
    let sut = D3D8Harness::new();
    let main = sut.back_buffer();
    let chain = sut.create_swap_chain(32, 16);
    let surface = chain.back_buffer(0, 0).expect("additional back buffer");
    assert_eq!(sut.set_render_target(Some(&surface), None), 0);
    sut.clear(0xFF12_3456);
    drop(surface);
    drop(chain);
    let retained = sut.render_target();
    assert_eq!((retained.desc().width, retained.desc().height), (32, 16));
    assert_eq!(
        retained.container_is_device(&sut, &IID_IDIRECT3DDEVICE8),
        Ok(true)
    );
    assert_pixel_eq(
        retained.read_pixel(8, 8),
        0xFF12_3456,
        "bound buffer survives",
    );
    assert_eq!(sut.set_render_target(Some(&main), None), 0);
}

#[test]
fn additional_swap_chain_back_buffer_survives_releasing_the_chain() {
    let sut = D3D8Harness::new();
    let chain = sut.create_swap_chain(32, 16);
    let first = chain
        .back_buffer(0, 0)
        .expect("first additional back buffer");
    let second = chain
        .back_buffer(0, u32::MAX)
        .expect("D3D8 ignores back-buffer kind");
    assert!(first.is_same_object(&second));
    assert!(matches!(chain.back_buffer(1, 0), Err(D3DERR_INVALIDCALL)));
    assert_eq!(
        first.container_is_device(&sut, &IID_IDIRECT3DDEVICE8),
        Ok(true)
    );
    assert_eq!((first.desc().width, first.desc().height), (32, 16));
    let main = sut.back_buffer();
    sut.clear(0xFFAA_5500);
    assert_eq!(sut.set_render_target(Some(&first), None), 0);
    sut.clear(0xFF12_3456);
    assert_eq!(sut.set_render_target(Some(&main), None), 0);
    assert_pixel_eq(
        main.read_pixel(8, 8),
        0xFFAA_5500,
        "independent main buffer",
    );
    assert_eq!(chain.present(), 0);
    assert_pixel_eq(
        main.read_pixel(8, 8),
        0xFFAA_5500,
        "Present preserves main buffer",
    );
    drop(chain);
    drop(second);
    let descriptor = first.desc();
    assert_eq!((descriptor.width, descriptor.height), (32, 16));
    assert_eq!(descriptor.size, 32 * 16 * 4);
    assert_pixel_eq(
        first.read_pixel(8, 8),
        0xFF12_3456,
        "retained additional buffer",
    );
}

#[test]
fn factory_and_back_buffer_preserve_com_identity() {
    let sut = D3D8Harness::new();
    assert!(sut.adapter_count() > 0);
    assert!(sut.factory_identity_is_preserved());
    let first = sut.back_buffer();
    let second = sut.back_buffer();
    assert!(first.is_same_object(&second));
    let descriptor = first.desc();
    assert_eq!((descriptor.width, descriptor.height), (64, 64));
    assert_eq!(descriptor.size, 64 * 64 * 4);
}

#[test]
fn texture_levels_and_bindings_preserve_d3d8_resource_identity() {
    let sut = D3D8Harness::new();
    let texture = sut.create_texture(16, 8);
    assert_eq!(texture.level_count(), 5);
    for (level, width, height) in [(0, 16, 8), (1, 8, 4), (4, 1, 1)] {
        let descriptor = texture.level_desc(level);
        assert_eq!((descriptor.width, descriptor.height), (width, height));
        assert_eq!(descriptor.size, width * height * 4);
    }
    for iid in [
        IID_IUNKNOWN,
        IID_IDIRECT3DRESOURCE8,
        IID_IDIRECT3DBASETEXTURE8,
        IID_IDIRECT3DTEXTURE8,
    ] {
        assert_eq!(texture.query_preserves_identity(&iid), Ok(true));
    }
    assert_eq!(
        texture.query_preserves_identity(&IID_IDIRECT3DTEXTURE9),
        Err(E_NOINTERFACE)
    );
    assert!(texture.belongs_to(&sut));
    let first = texture.surface(0);
    let second = texture.surface(0);
    assert!(first.is_same_object(&second));
    assert_eq!(
        first.container_is_texture(&texture, &IID_IDIRECT3DTEXTURE8),
        Ok(true)
    );
    assert_eq!(
        first.container_is_texture(&texture, &IID_IDIRECT3DTEXTURE9),
        Err(E_NOINTERFACE)
    );
    assert_eq!(sut.set_texture(0, Some(&texture)), 0);
    assert!(sut.texture_binding_matches(0, Some(&texture)));
    assert_eq!(sut.set_texture(0, None), 0);
    assert!(sut.texture_binding_matches(0, None));
    drop(first);
    drop(second);
    let retained_level = texture.surface(2);
    drop(texture);
    assert_eq!(retained_level.desc().size, 4 * 2 * 4);
}

#[test]
fn back_buffer_container_exposes_the_d3d8_device() {
    let sut = D3D8Harness::new();
    let surface = sut.back_buffer();
    assert_eq!(
        surface.container_is_device(&sut, &IID_IDIRECT3DDEVICE8),
        Ok(true)
    );
    assert_eq!(surface.container_is_device(&sut, &IID_IUNKNOWN), Ok(true));
    assert_eq!(
        surface.container_is_device(&sut, &IID_IDIRECT3DDEVICE9),
        Err(E_NOINTERFACE)
    );
}

#[test]
fn cached_back_buffer_checks_the_index_and_ignores_the_type() {
    let sut = D3D8Harness::new();
    let first = sut.back_buffer();
    assert!(matches!(sut.try_back_buffer(1, 0), Err(D3DERR_INVALIDCALL)));
    for kind in [1, 2, 0xDEAD_BEEF] {
        let other = sut
            .try_back_buffer(0, kind)
            .expect("D3D8 ignores back-buffer type");
        assert!(first.is_same_object(&other));
    }
    drop(first);
    sut.clear(0xFF12_3456);
    assert_pixel_eq(
        sut.back_buffer().read_pixel(32, 32),
        0xFF12_3456,
        "reacquired back buffer",
    );
}

#[test]
fn held_back_buffer_blocks_reset_until_the_reference_is_released() {
    let sut = D3D8Harness::new();
    let held = sut.back_buffer();
    assert_eq!(sut.reset(32, 48), D3DERR_DEVICELOST);
    assert_eq!(sut.cooperative_level(), D3DERR_DEVICENOTRESET);
    let before = held.desc();
    assert_eq!((before.width, before.height), (64, 64));
    drop(held);
    assert_eq!(sut.reset(32, 48), 0);
    assert_eq!(sut.cooperative_level(), 0);
    let after = sut.back_buffer().desc();
    assert_eq!((after.width, after.height), (32, 48));
    sut.clear(0xFF12_3456);
    assert_pixel_eq(
        sut.back_buffer().read_pixel(16, 24),
        0xFF12_3456,
        "reset back buffer",
    );
}

#[test]
fn sdk_versions_create_devices_and_render_fvf_triangles() {
    const BLUE: u32 = 0xFF00_00FF;
    const GREEN: u32 = 0xFF00_FF00;
    for sdk in [120, 220] {
        let sut = D3D8Harness::with_sdk(sdk).expect("supported D3D8 SDK creates a device");
        assert_eq!(sut.set_render_state(D3DRS_LIGHTING, 0), 0);
        assert_eq!(sut.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), 0);
        assert_eq!(sut.set_vertex_shader(D3DFVF_XYZRHW | D3DFVF_DIFFUSE), 0);
        for (state, value) in [
            (D3DTSS_COLOROP, D3DTOP_SELECTARG1),
            (D3DTSS_COLORARG1, D3DTA_DIFFUSE),
            (D3DTSS_ALPHAOP, D3DTOP_SELECTARG1),
            (D3DTSS_ALPHAARG1, D3DTA_DIFFUSE),
        ] {
            assert_eq!(sut.set_texture_stage_state(0, state, value), 0);
        }
        sut.clear(BLUE);
        assert_pixel_eq(sut.back_buffer().read_pixel(32, 32), BLUE, "D3D8 clear");
        let vertices = [
            RhwVertex {
                x: 8.0,
                y: 8.0,
                z: 0.5,
                rhw: 1.0,
                color: GREEN,
            },
            RhwVertex {
                x: 56.0,
                y: 8.0,
                z: 0.5,
                rhw: 1.0,
                color: GREEN,
            },
            RhwVertex {
                x: 32.0,
                y: 56.0,
                z: 0.5,
                rhw: 1.0,
                color: GREEN,
            },
        ];
        sut.draw_triangle(&vertices);
        let surface = sut.back_buffer();
        assert_pixel_eq(surface.read_pixel(32, 24), GREEN, "D3D8 FVF triangle");
        assert_pixel_eq(surface.read_pixel(2, 2), BLUE, "D3D8 triangle background");
    }
}

#[test]
fn unsupported_sdk_versions_are_rejected() {
    for sdk in [0, 119, 121, 219, 221, u32::MAX] {
        assert!(D3D8Harness::with_sdk(sdk).is_none(), "SDK {sdk}");
    }
}
