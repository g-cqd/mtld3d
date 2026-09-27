//! D3D8 surface copy bounds and compressed-block alignment.

use mtld3d_types::{D3DRECT, D3DSURFACE_DESC, POINT};

use crate::format::map_d3d_format;

/// Translates a source region to a destination point while preserving its pixel dimensions.
///
/// Returns `None` for empty or out-of-bounds regions, signed coordinate overflow, unmapped
/// source formats, or compressed regions that split a block away from a surface edge.
/// The caller verifies that source and destination formats match before copying pixels.
#[must_use]
pub fn destination_rectangle(
    source: &D3DRECT,
    point: POINT,
    source_desc: &D3DSURFACE_DESC,
    destination_desc: &D3DSURFACE_DESC,
) -> Option<D3DRECT> {
    if source.x1 < 0 || source.y1 < 0 || point.x < 0 || point.y < 0 {
        return None;
    }
    let width = source.x2.checked_sub(source.x1)?;
    let height = source.y2.checked_sub(source.y1)?;
    if width <= 0
        || height <= 0
        || u32::try_from(source.x2).ok()? > source_desc.width
        || u32::try_from(source.y2).ok()? > source_desc.height
    {
        return None;
    }
    let destination = D3DRECT {
        x1: point.x,
        y1: point.y,
        x2: point.x.checked_add(width)?,
        y2: point.y.checked_add(height)?,
    };
    if u32::try_from(destination.x2).ok()? > destination_desc.width
        || u32::try_from(destination.y2).ok()? > destination_desc.height
    {
        return None;
    }
    let mapping = map_d3d_format(source_desc.format)?;
    if mapping.is_compressed() {
        let block_width = i32::try_from(mapping.block_width()).ok()?;
        let block_height = i32::try_from(mapping.block_height()).ok()?;
        if source.x1 % block_width != 0
            || destination.x1 % block_width != 0
            || source.y1 % block_height != 0
            || destination.y1 % block_height != 0
            || (source.x2 % block_width != 0 && u32::try_from(source.x2).ok()? != source_desc.width)
            || (source.y2 % block_height != 0
                && u32::try_from(source.y2).ok()? != source_desc.height)
            || (destination.x2 % block_width != 0
                && u32::try_from(destination.x2).ok()? != destination_desc.width)
            || (destination.y2 % block_height != 0
                && u32::try_from(destination.y2).ok()? != destination_desc.height)
        {
            return None;
        }
    }
    Some(destination)
}

#[cfg(test)]
mod tests;
