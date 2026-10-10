use super::*;

#[test]
fn staging_wins_over_every_other_shape() {
    // A lockable render target reads and writes its own bytes, so it never
    // takes the read-back route even though it also owns a colour texture.
    assert_eq!(
        classify_color_surface_lock(true, false),
        ColorSurfaceLock::Staging
    );
    assert_eq!(
        classify_color_surface_lock(true, true),
        ColorSurfaceLock::Staging
    );
}

#[test]
fn the_implicit_back_buffer_is_read_back() {
    assert_eq!(
        classify_color_surface_lock(false, true),
        ColorSurfaceLock::BackBufferReadback
    );
}

#[test]
fn a_non_lockable_render_target_is_rejected() {
    assert_eq!(
        classify_color_surface_lock(false, false),
        ColorSurfaceLock::Reject
    );
}

const fn region(x: u32, y: u32, w: u32, h: u32) -> StretchRegion {
    StretchRegion { x, y, w, h }
}

#[test]
fn the_offset_of_a_region_is_its_first_texel_in_the_page() {
    // A 64 x 64 BGRA8 page: 256 bytes a row, 16 KiB in all.
    let page = 64 * 256;
    assert_eq!(region_offset(region(0, 0, 64, 64), 256, 4, page), Some(0));
    assert_eq!(
        region_offset(region(16, 16, 32, 32), 256, 4, page),
        Some(16 * 256 + 16 * 4)
    );
    assert_eq!(
        region_offset(region(63, 63, 1, 1), 256, 4, page),
        Some(63 * 256 + 63 * 4)
    );
}

#[test]
fn an_offset_outside_the_page_is_refused() {
    let page = 64 * 256;
    assert_eq!(region_offset(region(0, 64, 1, 1), 256, 4, page), None);
    assert_eq!(region_offset(region(0, 0, 1, 1), 256, 4, 0), None);
    // The last texel of a page whose pitch carries padding still fits.
    assert_eq!(
        region_offset(region(0, 63, 1, 1), 256, 4, page),
        Some(63 * 256)
    );
}

#[test]
fn an_offset_that_overflows_is_refused() {
    assert_eq!(
        region_offset(
            region(u32::MAX, u32::MAX, 1, 1),
            u32::MAX,
            u32::MAX,
            usize::MAX
        ),
        None
    );
}

#[test]
fn at_full_scale_the_writeback_regions_are_the_locked_rect() {
    let locked = region(16, 16, 32, 32);
    assert_eq!(
        writeback_regions(RenderScale::IDENTITY, locked),
        (locked, locked)
    );
}

#[test]
fn a_scaled_writeback_covers_only_the_locked_rect() {
    // 75%: the 64-texel surface renders at 48, and the locked 16..48 rect
    // lands on 12..36 of the texture, not on the whole of it.
    let locked = region(16, 16, 32, 32);
    let (source, destination) = writeback_regions(RenderScale::from_percent(75), locked);
    assert_eq!(source, locked);
    assert_eq!(destination, region(12, 12, 24, 24));
}

#[test]
fn abutting_locked_rects_leave_no_seam_when_scaled() {
    let scale = RenderScale::from_percent(75);
    let (_, left) = writeback_regions(scale, region(0, 0, 5, 8));
    let (_, right) = writeback_regions(scale, region(5, 0, 5, 8));
    assert_eq!(left.x + left.w, right.x);
}
