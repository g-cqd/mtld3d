use super::*;

#[test]
fn definitions_keep_register_order_and_all_immediate_bits() {
    let input = [
        0xfffe_0101,
        81,
        0xa00f_00ff,
        u32::MAX,
        0,
        0x8000_0000,
        0x7fc0_0001,
        1,
        0xc00f_0000,
        0x90e4_0000,
        81,
        0xa00f_00ff,
        1,
        2,
        3,
        4,
        0xffff,
    ];
    let (code, constants) = translate(&input).unwrap().into_parts();
    assert_eq!(code, [0xfffe_0101, 1, 0xc00f_0000, 0x90e4_0000, 0xffff]);
    assert_eq!(constants.len(), 2);
    assert_eq!(constants[0].register(), 255);
    assert_eq!(
        constants[0].value().map(f32::to_bits),
        [u32::MAX, 0, 0x8000_0000, 0x7fc0_0001]
    );
    assert_eq!(constants[1].register(), 255);
    assert_eq!(constants[1].value().map(f32::to_bits), [1, 2, 3, 4]);
}

#[test]
fn comments_do_not_expose_payload_as_definitions_or_terminators() {
    let input = [
        0xffff_0104,
        0x0006_fffe,
        81,
        0xa00f_0008,
        0xffff,
        2,
        3,
        4,
        0xffff,
    ];
    let (code, constants) = translate(&input).unwrap().into_parts();
    assert_eq!(code, input);
    assert!(constants.is_empty());
}

#[test]
fn invalid_definition_destinations_and_payload_bounds_fail() {
    for (version, limit) in [(0xfffe_0101, 256), (0xffff_0100, 8), (0xffff_0104, 8)] {
        let mut input = [version, 81, 0xa00f_0000 | (limit - 1), 0, 1, 2, 3, 0xffff];
        assert!(translate(&input).is_some());
        for destination in [0xa00f_0000 | limit, 0x800f_0000, 0xa001_0000, 0xa00f_2000] {
            input[2] = destination;
            assert!(translate(&input).is_none());
        }
        input[2] = 0xa00f_0000;
        for end in 0..input.len() {
            assert!(translate(&input[..end]).is_none());
        }
    }
}

#[test]
fn invalid_versions_missing_ends_and_oversized_inputs_fail() {
    for input in [
        vec![0xfffe_0200, 0xffff],
        vec![0xffff_0105, 0xffff],
        vec![0xffff_0101, 0x0002_fffe, 0xffff],
        vec![0xfffe_0101, 47, 0xe00f_0000, 1, 0xffff],
        vec![0xfffe_0101, 48, 0xf00f_0000, 1, 2, 3, 4, 0xffff],
        vec![0xfffe_0101, 0xffff, 0],
    ] {
        assert!(translate(&input).is_none());
    }
    let mut input = vec![0; 65537];
    input[0] = 0xfffe_0101;
    input[65536] = 0xffff;
    assert!(translate(&input).is_none());
}
