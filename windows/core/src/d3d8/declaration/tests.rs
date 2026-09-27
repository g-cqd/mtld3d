use super::*;

#[test]
fn streams_skips_and_semantics_preserve_register_assignment() {
    let declaration = translate(
        &[
            0x2000_0000,
            0x4002_0000,
            0x5002_0000,
            0x4004_0005,
            0x2000_0003,
            0x4001_0008,
            u32::MAX,
        ],
        false,
    )
    .unwrap();
    assert_eq!(declaration.elements.len(), 4);
    assert_eq!(
        (
            declaration.elements[0].stream,
            declaration.elements[0].offset
        ),
        (0, 0)
    );
    assert_eq!(
        (
            declaration.elements[1].offset,
            declaration.elements[1].usage
        ),
        (20, 10)
    );
    assert_eq!(
        (
            declaration.elements[2].stream,
            declaration.elements[2].offset,
            declaration.elements[2].usage_index
        ),
        (3, 0, 1)
    );
    assert_eq!(
        declaration.shader_prefix,
        [
            31,
            0x8000_0000,
            0x900f_0000,
            31,
            0x8000_000a,
            0x900f_0005,
            31,
            0x8001_0005,
            0x900f_0008,
        ]
    );
}

#[test]
fn declaration_constants_do_not_become_shader_local_definitions() {
    let declaration = translate(
        &[
            0x8200_0007,
            0xffff_ffff,
            0,
            0x3f80_0000,
            0x8000_0000,
            u32::MAX,
        ],
        false,
    )
    .unwrap();
    assert!(declaration.shader_prefix.is_empty());
    assert_eq!(declaration.constants.len(), 1);
    assert_eq!(declaration.constants[0].register(), 7);
    assert_eq!(
        declaration.constants[0].value().map(f32::to_bits),
        [u32::MAX, 0, 0x3f80_0000, 0x8000_0000]
    );
}

#[test]
fn malformed_declarations_fail_before_creating_a_layout() {
    for tokens in [
        vec![],
        vec![0x2000_0000],
        vec![0x4002_0000, u32::MAX],
        vec![0x2000_0000, 0x4002_0011, u32::MAX],
        vec![0x2000_0000, 0x4008_0000, u32::MAX],
        vec![0x2000_0000, 0x4002_0000, 0x4002_0000, u32::MAX],
        vec![0x8400_007f, 0, 0, 0, 0, u32::MAX],
        vec![0x6000_0000, u32::MAX],
    ] {
        assert!(translate(&tokens, false).is_none(), "{tokens:x?}");
    }
}

#[test]
fn fixed_function_normals_require_float3_but_programmable_normals_do_not() {
    for type_ in 0..8 {
        let words = [0x2000_0000, 0x4000_0003 | (type_ << 16), u32::MAX];
        assert_eq!(translate(&words, true).is_some(), type_ == 2);
        assert!(translate(&words, false).is_some());
    }
}
