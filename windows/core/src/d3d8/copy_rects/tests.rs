//! Bounds and compressed-block alignment at the D3D8 `CopyRects` boundary.

use mtld3d_types::{D3DFMT_A8R8G8B8, D3DFMT_DXT1, D3DRECT, D3DSURFACE_DESC, POINT};

use super::destination_rectangle;

fn surface(width: u32, height: u32, format: u32) -> D3DSURFACE_DESC {
    D3DSURFACE_DESC {
        format,
        resource_type: 1,
        usage: 0,
        pool: 2,
        multi_sample_type: 0,
        multi_sample_quality: 0,
        width,
        height,
    }
}

fn rect(x1: i32, y1: i32, x2: i32, y2: i32) -> D3DRECT {
    D3DRECT { x1, y1, x2, y2 }
}

#[test]
fn translates_equal_sized_regions_and_checks_every_edge() {
    let description = surface(8, 8, D3DFMT_A8R8G8B8);
    let output = destination_rectangle(
        &rect(1, 2, 4, 6),
        POINT { x: 2, y: 1 },
        &description,
        &description,
    )
    .expect("in-bounds copy");
    assert_eq!((output.x1, output.y1, output.x2, output.y2), (2, 1, 5, 5));
    for (source, point) in [
        (rect(-1, 0, 1, 1), POINT { x: 0, y: 0 }),
        (rect(0, -1, 1, 1), POINT { x: 0, y: 0 }),
        (rect(0, 0, 0, 1), POINT { x: 0, y: 0 }),
        (rect(0, 1, 1, 0), POINT { x: 0, y: 0 }),
        (rect(0, 0, 9, 1), POINT { x: 0, y: 0 }),
        (rect(0, 0, 1, 9), POINT { x: 0, y: 0 }),
        (rect(0, 0, 1, 1), POINT { x: -1, y: 0 }),
        (rect(0, 0, 1, 1), POINT { x: 0, y: -1 }),
        (rect(0, 0, 2, 1), POINT { x: 7, y: 0 }),
        (rect(0, 0, 1, 2), POINT { x: 0, y: 7 }),
    ] {
        assert!(destination_rectangle(&source, point, &description, &description).is_none());
    }
}

#[test]
fn rejects_signed_overflow_without_wrapping_into_the_target() {
    let description = surface(u32::MAX, u32::MAX, D3DFMT_A8R8G8B8);
    for point in [POINT { x: i32::MAX, y: 0 }, POINT { x: 0, y: i32::MAX }] {
        assert!(
            destination_rectangle(&rect(0, 0, 2, 2), point, &description, &description).is_none()
        );
    }
    assert!(
        destination_rectangle(
            &rect(i32::MIN, 0, i32::MAX, 1),
            POINT { x: 0, y: 0 },
            &description,
            &description
        )
        .is_none()
    );
}

#[test]
fn compressed_regions_require_block_alignment_except_at_matching_edges() {
    let partial = surface(5, 7, D3DFMT_DXT1);
    let full = surface(8, 8, D3DFMT_DXT1);
    assert!(
        destination_rectangle(&rect(0, 0, 5, 7), POINT { x: 0, y: 0 }, &partial, &partial)
            .is_some()
    );
    assert!(destination_rectangle(&rect(0, 0, 4, 4), POINT { x: 4, y: 4 }, &full, &full).is_some());
    for (source, point) in [
        (rect(1, 0, 4, 4), POINT { x: 0, y: 0 }),
        (rect(0, 1, 4, 4), POINT { x: 0, y: 0 }),
        (rect(0, 0, 4, 4), POINT { x: 1, y: 0 }),
        (rect(0, 0, 4, 4), POINT { x: 0, y: 1 }),
        (rect(0, 0, 3, 4), POINT { x: 0, y: 0 }),
        (rect(0, 0, 4, 3), POINT { x: 0, y: 0 }),
    ] {
        assert!(destination_rectangle(&source, point, &full, &full).is_none());
    }
    assert!(
        destination_rectangle(&rect(0, 0, 5, 7), POINT { x: 0, y: 0 }, &partial, &full).is_none()
    );
}

#[test]
fn unmapped_formats_cannot_define_a_copy_extent() {
    let description = surface(8, 8, 0xFFFF_FFFF);
    assert!(
        destination_rectangle(
            &rect(0, 0, 8, 8),
            POINT { x: 0, y: 0 },
            &description,
            &description
        )
        .is_none()
    );
}
