//! Maps sampler states that Direct3D 8 stores in its texture-stage namespace.

use mtld3d_types::{
    D3DSAMP_ADDRESSU, D3DSAMP_ADDRESSV, D3DSAMP_ADDRESSW, D3DSAMP_BORDERCOLOR, D3DSAMP_MAGFILTER,
    D3DSAMP_MAXANISOTROPY, D3DSAMP_MAXMIPLEVEL, D3DSAMP_MINFILTER, D3DSAMP_MIPFILTER,
    D3DSAMP_MIPMAPLODBIAS, D3DTSS8_ADDRESSU, D3DTSS8_ADDRESSV, D3DTSS8_ADDRESSW,
    D3DTSS8_BORDERCOLOR, D3DTSS8_MAGFILTER, D3DTSS8_MAXANISOTROPY, D3DTSS8_MAXMIPLEVEL,
    D3DTSS8_MINFILTER, D3DTSS8_MIPFILTER, D3DTSS8_MIPMAPLODBIAS,
};

/// Returns a shared sampler-state selector for a D3D8 texture-stage selector.
#[must_use]
pub const fn sampler_state(state: u32) -> Option<u32> {
    match state {
        D3DTSS8_ADDRESSU => Some(D3DSAMP_ADDRESSU),
        D3DTSS8_ADDRESSV => Some(D3DSAMP_ADDRESSV),
        D3DTSS8_ADDRESSW => Some(D3DSAMP_ADDRESSW),
        D3DTSS8_BORDERCOLOR => Some(D3DSAMP_BORDERCOLOR),
        D3DTSS8_MAGFILTER => Some(D3DSAMP_MAGFILTER),
        D3DTSS8_MINFILTER => Some(D3DSAMP_MINFILTER),
        D3DTSS8_MIPFILTER => Some(D3DSAMP_MIPFILTER),
        D3DTSS8_MIPMAPLODBIAS => Some(D3DSAMP_MIPMAPLODBIAS),
        D3DTSS8_MAXMIPLEVEL => Some(D3DSAMP_MAXMIPLEVEL),
        D3DTSS8_MAXANISOTROPY => Some(D3DSAMP_MAXANISOTROPY),
        _ => None,
    }
}

/// Converts a documented legacy Z-bias value to the shared normalized depth offset.
///
/// The compatibility scale uses one 16-bit depth unit per step. D3D8 specifies
/// the direction and the 0..=16 range; the exact hardware offset is driver-dependent.
#[must_use]
pub fn depth_bias(value: u32) -> Option<u32> {
    if value > 16 {
        return None;
    }
    let value = u16::try_from(value).ok()?;
    Some(if value == 0 {
        0
    } else {
        (-f32::from(value) / 65535.0).to_bits()
    })
}

#[cfg(test)]
mod tests;
