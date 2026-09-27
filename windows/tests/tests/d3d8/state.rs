//! D3D8 state conversion and state-block token behavior.

use mtld3d_tests::D3D8Harness;
use mtld3d_types::{
    D3DCOLORVALUE, D3DCULL_CW, D3DCULL_NONE, D3DERR_INVALIDCALL, D3DLIGHT_POINT, D3DLIGHT9,
    D3DMATERIAL9, D3DMATRIX, D3DRS_CULLMODE, D3DRS_LIGHTING, D3DRS8_ZBIAS, D3DSBT_VERTEXSTATE,
    D3DTADDRESS_CLAMP, D3DTADDRESS_WRAP, D3DTEXF_LINEAR, D3DTEXF_POINT, D3DTS_VIEW,
    D3DTSS8_ADDRESSU, D3DTSS8_MINFILTER, D3DVECTOR, D3DVIEWPORT9,
};

#[test]
fn material_and_light_use_the_shared_layout_without_losing_fields() {
    let sut = D3D8Harness::new();
    let color = D3DCOLORVALUE {
        r: 0.125,
        g: 0.25,
        b: 0.5,
        a: 0.75,
    };
    let material = D3DMATERIAL9 {
        diffuse: color,
        ambient: D3DCOLORVALUE { r: 0.5, ..color },
        emissive: D3DCOLORVALUE { a: 0.25, ..color },
        specular: D3DCOLORVALUE { b: 0.75, ..color },
        power: 17.0,
    };
    assert_eq!(sut.set_material(&material), 0);
    let returned = sut.material();
    let components = |value: D3DCOLORVALUE| [value.r, value.g, value.b, value.a].map(f32::to_bits);
    assert_eq!(components(returned.diffuse), components(material.diffuse));
    assert_eq!(components(returned.ambient), components(material.ambient));
    assert_eq!(components(returned.emissive), components(material.emissive));
    assert_eq!(components(returned.specular), components(material.specular));
    assert_eq!(returned.power.to_bits(), material.power.to_bits());
    let light = D3DLIGHT9 {
        type_: D3DLIGHT_POINT,
        diffuse: color,
        position: D3DVECTOR {
            x: 2.0,
            y: 3.0,
            z: 4.0,
        },
        range: 20.0,
        attenuation0: 0.5,
        attenuation1: 0.25,
        attenuation2: 0.125,
        ..D3DLIGHT9::default()
    };
    assert_eq!(sut.set_light(3, &light), 0);
    let returned = sut.light(3);
    assert_eq!(returned.type_, D3DLIGHT_POINT);
    assert_eq!(components(returned.diffuse), components(light.diffuse));
    assert_eq!(
        [
            returned.position.x,
            returned.position.y,
            returned.position.z
        ]
        .map(f32::to_bits),
        [2.0, 3.0, 4.0].map(f32::to_bits)
    );
    assert_eq!(
        [
            returned.range,
            returned.attenuation0,
            returned.attenuation1,
            returned.attenuation2
        ]
        .map(f32::to_bits),
        [20.0, 0.5, 0.25, 0.125].map(f32::to_bits)
    );
}

#[test]
fn d3d8_sampler_selectors_and_integer_bias_round_trip() {
    let sut = D3D8Harness::new();
    for (selector, first, second) in [
        (D3DTSS8_ADDRESSU, D3DTADDRESS_CLAMP, D3DTADDRESS_WRAP),
        (D3DTSS8_MINFILTER, D3DTEXF_LINEAR, D3DTEXF_POINT),
    ] {
        assert_eq!(sut.set_texture_stage_state(0, selector, first), 0);
        assert_eq!(sut.set_texture_stage_state(1, selector, second), 0);
        assert_eq!(sut.texture_stage_state(0, selector), first);
        assert_eq!(sut.texture_stage_state(1, selector), second);
    }
    for bias in [0, 1, 16] {
        assert_eq!(sut.set_render_state(D3DRS8_ZBIAS, bias), 0);
        assert_eq!(sut.render_state(D3DRS8_ZBIAS), bias);
    }
}

#[test]
fn matrix_and_viewport_keep_all_d3d8_fields() {
    let sut = D3D8Harness::new();
    let matrix = D3DMATRIX {
        m: [
            2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 5.0, 6.0, 7.0, 1.0,
        ],
    };
    assert_eq!(sut.set_transform(D3DTS_VIEW, &matrix), 0);
    assert_eq!(
        sut.transform(D3DTS_VIEW).m.map(f32::to_bits),
        matrix.m.map(f32::to_bits)
    );
    let viewport = D3DVIEWPORT9 {
        x: 3,
        y: 4,
        width: 48,
        height: 32,
        min_z: 0.25,
        max_z: 0.75,
    };
    assert_eq!(sut.set_viewport(&viewport), 0);
    let returned = sut.viewport();
    assert_eq!(
        (returned.x, returned.y, returned.width, returned.height),
        (3, 4, 48, 32)
    );
    assert_eq!(returned.min_z.to_bits(), 0.25_f32.to_bits());
    assert_eq!(returned.max_z.to_bits(), 0.75_f32.to_bits());
}

#[test]
fn end_state_block_without_begin_preserves_output() {
    let sut = D3D8Harness::new();
    let mut token = 0x1234_5678;
    assert_eq!(sut.end_state_block_into(&mut token), D3DERR_INVALIDCALL);
    assert_eq!(token, 0x1234_5678);
}

#[test]
fn recorded_state_blocks_preserve_live_state_until_applied() {
    let sut = D3D8Harness::new();
    assert_eq!(sut.set_render_state(D3DRS_CULLMODE, D3DCULL_CW), 0);
    assert_eq!(sut.begin_state_block(), 0);
    assert_eq!(sut.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), 0);
    assert_eq!(sut.render_state(D3DRS_CULLMODE), D3DCULL_CW);
    let block = sut.end_state_block();
    assert_ne!(block, 0);
    assert_eq!(sut.render_state(D3DRS_CULLMODE), D3DCULL_CW);
    assert_eq!(sut.apply_state_block(block), 0);
    assert_eq!(sut.render_state(D3DRS_CULLMODE), D3DCULL_NONE);
    assert_eq!(sut.set_render_state(D3DRS_CULLMODE, D3DCULL_CW), 0);
    assert_eq!(sut.capture_state_block(block), 0);
    assert_eq!(sut.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), 0);
    assert_eq!(sut.apply_state_block(block), 0);
    assert_eq!(sut.render_state(D3DRS_CULLMODE), D3DCULL_CW);
    assert_eq!(sut.delete_state_block(block), 0);
    assert_eq!(sut.apply_state_block(block), D3DERR_INVALIDCALL);
    assert_eq!(sut.delete_state_block(block), D3DERR_INVALIDCALL);
}

#[test]
fn vertex_state_block_restores_lighting_without_pixel_sampler_state() {
    let sut = D3D8Harness::new();
    assert_eq!(sut.set_render_state(D3DRS_LIGHTING, 0), 0);
    assert_eq!(
        sut.set_texture_stage_state(0, D3DTSS8_ADDRESSU, D3DTADDRESS_CLAMP),
        0
    );
    let block = sut.create_state_block(D3DSBT_VERTEXSTATE);
    assert_eq!(sut.set_render_state(D3DRS_LIGHTING, 1), 0);
    assert_eq!(
        sut.set_texture_stage_state(0, D3DTSS8_ADDRESSU, D3DTADDRESS_WRAP),
        0
    );
    assert_eq!(sut.apply_state_block(block), 0);
    assert_eq!(sut.render_state(D3DRS_LIGHTING), 0);
    assert_eq!(
        sut.texture_stage_state(0, D3DTSS8_ADDRESSU),
        D3DTADDRESS_WRAP
    );
    assert_eq!(sut.delete_state_block(block), 0);
}
