use mtld3d_tests::{D3D8Harness, RhwVertex, assert_pixel_eq};
use mtld3d_types::{D3DCULL_NONE, D3DERR_INVALIDCALL, D3DRS_CULLMODE, D3DRS_LIGHTING};

const DECLARATION: [u32; 4] = [0x2000_0000, 0x4003_0000, 0x4004_0005, u32::MAX];
const VERTEX_SHADER: [u32; 8] = [
    0xfffe_0101,
    1,
    0xc00f_0000,
    0x90e4_0000,
    1,
    0xd00f_0000,
    0x90e4_0005,
    0x0000_ffff,
];
const PIXEL_SHADER: [u32; 5] = [0xffff_0101, 1, 0x800f_0000, 0xa0e4_0000, 0x0000_ffff];

#[test]
fn vertex_shader_handles_preserve_declaration_function_and_register_semantics() {
    let sut = D3D8Harness::new();
    let shader = sut.create_vertex_shader8(&DECLARATION, Some(&VERTEX_SHADER));
    assert_eq!(sut.vertex_shader_declaration(shader), DECLARATION);
    assert_eq!(sut.vertex_shader_function(shader), VERTEX_SHADER);
    assert_eq!(sut.set_vertex_shader(shader), 0);
    assert_eq!(sut.vertex_shader(), shader);
    assert_eq!(sut.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), 0);
    assert_eq!(sut.set_render_state(D3DRS_LIGHTING, 0), 0);
    sut.clear(0xff00_0000);
    sut.draw_triangle(&triangle(0xff00_ff00));
    assert_pixel_eq(
        sut.back_buffer().read_pixel(32, 32),
        0xff00_ff00,
        "D3D8 declaration v5 color maps to shader input v5",
    );
    assert_eq!(sut.delete_vertex_shader(shader), 0);
    assert_eq!(sut.vertex_shader(), 0);
    assert_eq!(sut.set_vertex_shader(shader), D3DERR_INVALIDCALL);
    assert_eq!(sut.delete_vertex_shader(shader), D3DERR_INVALIDCALL);
}

#[test]
fn declaration_only_handles_return_an_empty_function() {
    let sut = D3D8Harness::new();
    let shader = sut.create_vertex_shader8(&DECLARATION, None);
    assert_eq!(sut.vertex_shader_declaration(shader), DECLARATION);
    assert!(sut.vertex_shader_function(shader).is_empty());
    assert_eq!(sut.delete_vertex_shader(shader), 0);
}

#[test]
fn pixel_shader_handles_constants_and_state_blocks_keep_their_d3d8_contract() {
    let sut = D3D8Harness::new();
    let shader = sut.create_pixel_shader8(&PIXEL_SHADER);
    assert_eq!(sut.pixel_shader_function(shader), PIXEL_SHADER);
    assert_eq!(sut.set_pixel_shader(shader), 0);
    assert_eq!(sut.pixel_shader(), shader);
    let value = [[0.0, 0.25, 0.5, 1.0]];
    assert_eq!(sut.set_pixel_constants(7, &value), 0);
    assert_eq!(sut.pixel_constants(7, 1), value);
    assert_eq!(sut.set_pixel_constants(8, &value), D3DERR_INVALIDCALL);
    assert_eq!(sut.set_vertex_constants(255, &value), 0);
    assert_eq!(sut.vertex_constants(255, 1), value);
    assert_eq!(sut.set_vertex_constants(256, &value), D3DERR_INVALIDCALL);
    let block = sut.create_state_block(1);
    assert_eq!(sut.set_pixel_shader(0), 0);
    assert_eq!(sut.apply_state_block(block), 0);
    assert_eq!(sut.pixel_shader(), shader);
    assert_eq!(sut.delete_state_block(block), 0);
    assert_eq!(sut.delete_pixel_shader(shader), 0);
    assert_eq!(sut.pixel_shader(), 0);
    assert_eq!(sut.set_pixel_shader(shader), D3DERR_INVALIDCALL);
    assert_eq!(sut.delete_pixel_shader(shader), D3DERR_INVALIDCALL);
}

#[test]
fn declaration_constants_load_global_registers_on_each_bind() {
    let sut = D3D8Harness::new();
    let declaration = declaration_with_constant();
    let mut function = VERTEX_SHADER;
    function[6] = 0xa0e4_0000;
    let shader = sut.create_vertex_shader8(&declaration, Some(&function));
    check_vertex_constant_binding(&sut, shader);
    assert_eq!(sut.delete_vertex_shader(shader), 0);
}

#[test]
fn vertex_def_constants_load_global_registers_on_each_bind() {
    let sut = D3D8Harness::new();
    let mut function = with_green_def(&VERTEX_SHADER);
    function[12] = 0xa0e4_0000;
    let shader = sut.create_vertex_shader8(&DECLARATION, Some(&function));
    assert_eq!(sut.vertex_shader_function(shader), function);
    check_vertex_constant_binding(&sut, shader);
    assert_eq!(sut.delete_vertex_shader(shader), 0);
}

#[test]
fn pixel_def_constants_load_global_registers_on_each_bind() {
    let sut = D3D8Harness::new();
    let vertex = sut.create_vertex_shader8(&DECLARATION, Some(&VERTEX_SHADER));
    assert_eq!(sut.set_vertex_shader(vertex), 0);
    let function = with_green_def(&PIXEL_SHADER);
    let shader = sut.create_pixel_shader8(&function);
    assert_eq!(sut.pixel_shader_function(shader), function);
    assert_eq!(sut.set_pixel_constants(0, &[[1.0, 0.0, 0.0, 1.0]]), 0);
    assert_eq!(sut.set_pixel_shader(shader), 0);
    assert_eq!(sut.pixel_constants(0, 1), [[0.0, 1.0, 0.0, 1.0]]);
    check_constant_pixel(&sut, 0xff00_ff00);
    assert_eq!(sut.set_pixel_constants(0, &[[1.0, 0.0, 0.0, 1.0]]), 0);
    assert_eq!(sut.pixel_constants(0, 1), [[1.0, 0.0, 0.0, 1.0]]);
    check_constant_pixel(&sut, 0xffff_0000);
    assert_eq!(sut.set_pixel_shader(shader), 0);
    assert_eq!(sut.pixel_constants(0, 1), [[0.0, 1.0, 0.0, 1.0]]);
    check_constant_pixel(&sut, 0xff00_ff00);
    assert_eq!(sut.delete_pixel_shader(shader), 0);
    assert_eq!(sut.delete_vertex_shader(vertex), 0);
}

#[test]
fn recorded_shader_bindings_load_constants_only_when_applied() {
    let sut = D3D8Harness::new();
    let vertex = sut.create_vertex_shader8(&declaration_with_constant(), None);
    let pixel = sut.create_pixel_shader8(&with_green_def(&PIXEL_SHADER));
    let red = [[1.0, 0.0, 0.0, 1.0]];
    assert_eq!(sut.set_vertex_constants(0, &red), 0);
    assert_eq!(sut.set_pixel_constants(0, &red), 0);
    assert_eq!(sut.begin_state_block(), 0);
    assert_eq!(sut.set_vertex_shader(vertex), 0);
    assert_eq!(sut.set_pixel_shader(pixel), 0);
    let block = sut.end_state_block();
    assert_eq!(sut.vertex_constants(0, 1), red);
    assert_eq!(sut.pixel_constants(0, 1), red);
    assert_eq!(sut.vertex_shader(), 0);
    assert_eq!(sut.pixel_shader(), 0);
    assert_eq!(sut.apply_state_block(block), 0);
    assert_eq!(sut.vertex_constants(0, 1), [[0.0, 1.0, 0.0, 1.0]]);
    assert_eq!(sut.pixel_constants(0, 1), [[0.0, 1.0, 0.0, 1.0]]);
    assert_eq!(sut.vertex_shader(), vertex);
    assert_eq!(sut.pixel_shader(), pixel);
    assert_eq!(sut.set_vertex_constants(0, &red), 0);
    assert_eq!(sut.set_pixel_constants(0, &red), 0);
    assert_eq!(sut.capture_state_block(block), 0);
    assert_eq!(sut.set_vertex_shader(vertex), 0);
    assert_eq!(sut.set_pixel_shader(pixel), 0);
    assert_eq!(sut.apply_state_block(block), 0);
    assert_eq!(sut.vertex_constants(0, 1), red);
    assert_eq!(sut.pixel_constants(0, 1), red);
    assert_eq!(sut.delete_state_block(block), 0);
    assert_eq!(sut.delete_pixel_shader(pixel), 0);
    assert_eq!(sut.delete_vertex_shader(vertex), 0);
}

#[test]
fn declaration_only_normals_require_float3() {
    let sut = D3D8Harness::new();
    for type_ in 0..8 {
        let declaration = [
            0x2000_0000,
            0x4003_0000,
            0x4000_0003 | (type_ << 16),
            u32::MAX,
        ];
        let fixed = sut.try_create_vertex_shader8(&declaration, None);
        if type_ == 2 {
            assert_eq!(
                sut.delete_vertex_shader(fixed.expect("FLOAT3 normal is valid")),
                0
            );
        } else {
            assert_eq!(fixed, Err(D3DERR_INVALIDCALL), "type {type_}");
        }
        let programmable = sut.create_vertex_shader8(&declaration, Some(&VERTEX_SHADER));
        assert_eq!(sut.delete_vertex_shader(programmable), 0);
    }
}

fn declaration_with_constant() -> Vec<u32> {
    let mut declaration = DECLARATION[..3].to_vec();
    declaration.extend_from_slice(&[
        0x8200_0000,
        0,
        1.0f32.to_bits(),
        0,
        1.0f32.to_bits(),
        u32::MAX,
    ]);
    declaration
}

fn with_green_def(function: &[u32]) -> Vec<u32> {
    let mut result = vec![
        function[0],
        81,
        0xa00f_0000,
        0,
        1.0f32.to_bits(),
        0,
        1.0f32.to_bits(),
    ];
    result.extend_from_slice(&function[1..]);
    result
}

fn check_vertex_constant_binding(sut: &D3D8Harness, shader: u32) {
    let red = [[1.0, 0.0, 0.0, 1.0]];
    let green = [[0.0, 1.0, 0.0, 1.0]];
    assert_eq!(sut.set_vertex_constants(0, &red), 0);
    assert_eq!(sut.set_vertex_shader(shader), 0);
    assert_eq!(sut.vertex_constants(0, 1), green);
    check_constant_pixel(sut, 0xff00_ff00);
    assert_eq!(sut.set_vertex_constants(0, &red), 0);
    assert_eq!(sut.vertex_constants(0, 1), red);
    check_constant_pixel(sut, 0xffff_0000);
    assert_eq!(sut.set_vertex_shader(shader), 0);
    assert_eq!(sut.vertex_constants(0, 1), green);
    check_constant_pixel(sut, 0xff00_ff00);
}

fn check_constant_pixel(sut: &D3D8Harness, expected: u32) {
    assert_eq!(sut.set_render_state(D3DRS_CULLMODE, D3DCULL_NONE), 0);
    sut.clear(0xff00_0000);
    sut.draw_triangle(&triangle(0xffff_0000));
    assert_pixel_eq(
        sut.back_buffer().read_pixel(32, 32),
        expected,
        "bound D3D8 constant",
    );
}

fn triangle(color: u32) -> [RhwVertex; 3] {
    [(-1.0, -1.0), (0.0, 1.0), (1.0, -1.0)].map(|(x, y)| RhwVertex {
        x,
        y,
        z: 0.5,
        rhw: 1.0,
        color,
    })
}
