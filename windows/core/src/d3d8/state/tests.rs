use super::*;

#[test]
fn legacy_sampler_ids_map_without_reclassifying_combiner_states() {
    for (state, sampler) in [
        (13, 1),
        (14, 2),
        (25, 3),
        (15, 4),
        (16, 5),
        (17, 6),
        (18, 7),
        (19, 8),
        (20, 9),
        (21, 10),
    ] {
        assert_eq!(sampler_state(state), Some(sampler));
    }
    for state in [0, 1, 7, 11, 12, 22, 23, 24, 26, 27, 28, 29, u32::MAX] {
        assert_eq!(sampler_state(state), None);
    }
}

#[test]
fn legacy_depth_bias_is_bounded_and_moves_toward_the_viewer() {
    assert_eq!(depth_bias(0), Some(0));
    let mut previous = 0.0;
    for value in 1..=16 {
        let translated = f32::from_bits(depth_bias(value).unwrap());
        assert!(translated < previous && translated > -0.001);
        previous = translated;
    }
    for value in [17, 65536, u32::MAX] {
        assert_eq!(depth_bias(value), None);
    }
}
