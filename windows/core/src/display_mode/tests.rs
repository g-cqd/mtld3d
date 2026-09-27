use super::{
    MAX_SERVED_SIZES, ModeRequest, mode_set_attempts, select_mode_sizes, served_mode_indices,
    served_mode_sizes,
};

const MBP: (u32, u32) = (3456, 2234);

#[test]
fn a_request_without_a_rate_is_one_attempt() {
    let request = ModeRequest {
        width: 1280,
        height: 720,
        refresh_hz: 0,
    };
    assert_eq!(
        mode_set_attempts(request).collect::<Vec<_>>(),
        vec![request]
    );
}

#[test]
fn a_request_with_a_rate_retries_without_it() {
    let request = ModeRequest {
        width: 1280,
        height: 720,
        refresh_hz: 60,
    };
    assert_eq!(
        mode_set_attempts(request).collect::<Vec<_>>(),
        vec![
            request,
            ModeRequest {
                refresh_hz: 0,
                ..request
            }
        ]
    );
}

#[test]
fn the_desktop_mode_comes_first_and_order_is_kept() {
    let sizes = select_mode_sizes(MBP, [(640, 480), (1920, 1200), (1280, 720)], false);
    assert_eq!(sizes, vec![MBP, (640, 480), (1920, 1200), (1280, 720)]);
}

#[test]
fn duplicates_and_the_desktop_itself_appear_once() {
    let sizes = select_mode_sizes(MBP, [(640, 480), MBP, (640, 480), (640, 480)], false);
    assert_eq!(sizes, vec![MBP, (640, 480)]);
}

#[test]
fn sizes_larger_than_the_desktop_on_either_axis_are_dropped() {
    let sizes = select_mode_sizes(
        (1728, 1117),
        [(1920, 1080), (1600, 1200), (1680, 1050)],
        false,
    );
    assert_eq!(sizes, vec![(1728, 1117), (1680, 1050)]);
}

#[test]
fn aspects_outside_the_tolerance_are_dropped() {
    // 5:4 is ~19 % off a 3:2-ish panel, 21:9 ~51 %; 4:3, 16:10 and 16:9 stay.
    let sizes = select_mode_sizes(
        MBP,
        [
            (1280, 1024),
            (2560, 1080),
            (1024, 768),
            (1920, 1200),
            (1920, 1080),
        ],
        false,
    );
    assert_eq!(sizes, vec![MBP, (1024, 768), (1920, 1200), (1920, 1080)]);
}

#[test]
fn degenerate_sizes_are_dropped() {
    let sizes = select_mode_sizes(MBP, [(0, 480), (640, 0)], false);
    assert_eq!(sizes, vec![MBP]);
}

#[test]
fn an_empty_enumeration_serves_the_desktop_mode_alone() {
    assert_eq!(select_mode_sizes(MBP, [], false), vec![MBP]);
}

#[test]
fn only_the_panels_aspect_is_served_largest_first() {
    // The desktop's aspect is 1.547; 2336x1510 and 2992x1934 share it within
    // the panel tolerance (integer rounding), the 16:10, 16:9 and 4:3 sizes
    // do not and are left to a game's own config, however large.
    let settable = [
        MBP,
        (640, 480),
        (1024, 768),
        (1920, 1080),
        (2336, 1510),
        (2560, 1600),
        (2992, 1934),
        (1920, 1280),
    ];
    assert_eq!(
        served_mode_sizes(&settable, 15, false),
        vec![MBP, (2992, 1934), (2336, 1510)]
    );
}

#[test]
fn the_bound_cuts_the_smallest_panel_sizes() {
    let settable = [MBP, (2336, 1510), (2624, 1696), (2992, 1934)];
    assert_eq!(
        served_mode_sizes(&settable, 3, false),
        vec![MBP, (2992, 1934), (2624, 1696)]
    );
}

#[test]
fn panel_sizes_of_equal_pixel_count_keep_their_enumeration_order() {
    // Two 16:9 sizes with the same pixel count on a 16:9 desktop.
    let desktop = (3840, 2160);
    let settable = [desktop, (1920, 1080), (1920, 1080)];
    assert_eq!(
        served_mode_sizes(&settable, 15, false),
        vec![desktop, (1920, 1080), (1920, 1080)]
    );
    let settable = [desktop, (1280, 720), (1600, 900), (960, 540)];
    assert_eq!(
        served_mode_sizes(&settable, 15, false),
        vec![desktop, (1600, 900), (1280, 720), (960, 540)]
    );
}

#[test]
fn a_bound_of_zero_still_serves_the_desktop() {
    assert_eq!(served_mode_sizes(&[MBP, (640, 480)], 0, false), vec![MBP]);
}

#[test]
fn an_empty_settable_list_serves_nothing() {
    assert!(served_mode_sizes(&[], 5, false).is_empty());
}

#[test]
fn served_indices_are_the_positions_of_served_sizes_in_list_order() {
    // user32 lists every depth and rate of a size; each occurrence keeps
    // its position, sizes not served leave gaps.
    let list = [
        (640, 480),
        (1920, 1200),
        (640, 480),
        (1280, 1024),
        (1920, 1200),
        MBP,
    ];
    assert_eq!(
        served_mode_indices(list, &[MBP, (1920, 1200)]),
        vec![1, 4, 5]
    );
}

#[test]
fn no_served_size_in_the_list_yields_no_indices() {
    assert!(served_mode_indices([(640, 480)], &[MBP]).is_empty());
    assert!(served_mode_indices([], &[MBP]).is_empty());
}

#[test]
fn legacy_four_by_three_modes_are_settable_on_a_wide_desktop() {
    let desktop = (1680, 1050);
    for (legacy, expected) in [
        (false, vec![desktop, (1280, 800)]),
        (true, vec![desktop, (1024, 768), (1400, 1050), (1280, 800)]),
    ] {
        let candidates = [
            (1024, 768),
            (1400, 1050),
            (1280, 800),
            (1024, 768),
            (1600, 1200),
            (1280, 1024),
            (0, 480),
            (640, 0),
        ];
        assert_eq!(select_mode_sizes(desktop, candidates, legacy), expected);
    }
}

#[test]
fn a_bounded_legacy_menu_keeps_large_four_by_three_and_panel_choices() {
    let desktop = (3360, 2100);
    let settable = select_mode_sizes(
        desktop,
        [
            (640, 480),
            (1920, 1440),
            (2560, 1600),
            (2560, 1920),
            (2880, 1800),
            (1920, 1200),
            (1024, 768),
        ],
        true,
    );
    let served = served_mode_sizes(&settable, 5, true);
    assert_eq!(
        served,
        vec![
            desktop,
            (2560, 1920),
            (2880, 1800),
            (1920, 1440),
            (2560, 1600)
        ]
    );
    assert!(served.iter().all(|size| settable.contains(size)));
}

#[test]
fn a_legacy_menu_reserves_the_first_optional_slot_for_four_by_three() {
    let desktop = (3360, 2100);
    let settable = [desktop, (2880, 1800), (2560, 1920), (1920, 1440)];
    for (bound, expected) in [
        (0, vec![desktop]),
        (1, vec![desktop]),
        (2, vec![desktop, (2560, 1920)]),
        (3, vec![desktop, (2560, 1920), (2880, 1800)]),
        (15, vec![desktop, (2560, 1920), (2880, 1800), (1920, 1440)]),
    ] {
        assert_eq!(served_mode_sizes(&settable, bound, true), expected);
    }
}

#[test]
fn a_four_by_three_desktop_is_not_duplicated_by_the_legacy_policy() {
    let desktop = (2048, 1536);
    let settable = select_mode_sizes(desktop, [desktop, (1600, 1200), (1024, 768)], true);
    assert_eq!(served_mode_sizes(&settable, 15, true), settable);
}

#[test]
fn the_legacy_menu_bound_survives_a_long_host_list_without_fabricating_modes() {
    let desktop = (3360, 2100);
    let candidates = (1..=20).flat_map(|step| [(step * 160, step * 100), (step * 128, step * 96)]);
    let settable = select_mode_sizes(desktop, candidates, true);
    let served = served_mode_sizes(&settable, MAX_SERVED_SIZES, true);
    assert_eq!(served.len(), MAX_SERVED_SIZES);
    assert_eq!(served[0], desktop);
    assert_eq!(served[1], (2560, 1920));
    assert_eq!(served[2], (3200, 2000));
    for (index, size) in served.iter().enumerate() {
        assert!(settable.contains(size));
        assert!(!served[..index].contains(size));
    }
    assert_eq!(served_mode_sizes(&[], MAX_SERVED_SIZES, true), vec![]);
    assert_eq!(
        served_mode_sizes(&[desktop, (2880, 1800)], MAX_SERVED_SIZES, true),
        vec![desktop, (2880, 1800)]
    );
}

#[test]
fn a_rounded_panel_aspect_does_not_hide_the_only_exact_four_by_three_mode() {
    let settable = [(1366, 1024), (1282, 960), (1024, 768)];
    assert_eq!(
        served_mode_sizes(&settable, 2, true),
        vec![(1366, 1024), (1024, 768)]
    );
}
