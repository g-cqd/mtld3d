//! Converts legacy vertex declarations into shared vertex layouts and shader declarations.

use mtld3d_types::{
    D3DDECL_END, D3DDECLUSAGE_BLENDINDICES, D3DDECLUSAGE_BLENDWEIGHT, D3DDECLUSAGE_COLOR,
    D3DDECLUSAGE_NORMAL, D3DDECLUSAGE_POSITION, D3DDECLUSAGE_PSIZE, D3DDECLUSAGE_TEXCOORD,
    D3DVERTEXELEMENT9, D3DVSD_END, D3DVSD_TOKEN_CONSTMEM, D3DVSD_TOKEN_EXT, D3DVSD_TOKEN_NOP,
    D3DVSD_TOKEN_STREAM, D3DVSD_TOKEN_STREAMDATA, D3DVSD_TOKENTYPESHIFT,
};

use super::shader::Constant;

/// A vertex layout, shader input declarations, and constants loaded on binding.
pub struct Declaration {
    /// The shared layout, including its terminal element.
    pub elements: Vec<D3DVERTEXELEMENT9>,
    /// Input-register declarations before shader instructions.
    pub shader_prefix: Vec<u32>,
    /// Ordered global-register writes performed when the D3D8 handle is bound.
    pub constants: Vec<Constant>,
}

/// Number of words occupied by one declaration token and its payload.
///
/// Returns `None` for an unknown discriminator. The caller validates payload bounds.
#[must_use]
pub const fn token_words(token: u32) -> Option<usize> {
    match token >> D3DVSD_TOKENTYPESHIFT {
        D3DVSD_TOKEN_CONSTMEM => Some(1 + (((token >> 25) & 15) as usize) * 4),
        D3DVSD_TOKEN_EXT => Some(1 + (((token >> 24) & 31) as usize)),
        0..=3 | 7 => Some(1),
        _ => None,
    }
}

/// Converts a bounded, terminated D3D8 declaration into the shared D3D9 representation.
///
/// Invalid tokens, duplicate input registers, unsupported tessellation, and missing
/// payloads return `None`. Fixed-function declarations require FLOAT3 for the normal input.
/// Work and storage are linear in the token count, capped at 4096.
#[must_use]
pub fn translate(tokens: &[u32], fixed_function: bool) -> Option<Declaration> {
    if tokens.len() > 4096 {
        return None;
    }
    let mut result = Declaration {
        elements: Vec::new(),
        shader_prefix: Vec::new(),
        constants: Vec::new(),
    };
    let mut stream = None;
    let mut offset = 0u16;
    let mut registers = 0u32;
    let mut cursor = 0;
    while let Some(&token) = tokens.get(cursor) {
        if token == D3DVSD_END {
            result.elements.push(D3DDECL_END);
            return Some(result);
        }
        let words = token_words(token)?;
        let payload = tokens.get(cursor + 1..cursor.checked_add(words)?)?;
        match token >> D3DVSD_TOKENTYPESHIFT {
            D3DVSD_TOKEN_NOP if token == 0 => {}
            D3DVSD_TOKEN_STREAM if token & 0x1fff_fff0 == 0 => {
                stream = Some(u16::try_from(token & 15).ok()?);
                offset = 0;
            }
            D3DVSD_TOKEN_STREAMDATA => {
                let stream = stream?;
                if token & 0x1000_0000 != 0 {
                    if token & 0x0ff0_ffff != 0 {
                        return None;
                    }
                    offset = offset.checked_add(u16::try_from((token >> 16) & 15).ok()? * 4)?;
                } else {
                    if token & 0x0ff0_ffe0 != 0 {
                        return None;
                    }
                    let register = token & 31;
                    if registers & (1 << register) != 0 {
                        return None;
                    }
                    registers |= 1 << register;
                    let (usage, usage_index) = register_semantic(register)?;
                    let type_ = u8::try_from((token >> 16) & 15).ok()?;
                    if fixed_function && register == 3 && type_ != 2 {
                        return None;
                    }
                    let size = [4u16, 8, 12, 16, 4, 4, 4, 8].get(usize::from(type_))?;
                    result.elements.push(D3DVERTEXELEMENT9 {
                        stream,
                        offset,
                        type_,
                        method: 0,
                        usage,
                        usage_index,
                    });
                    offset = offset.checked_add(*size)?;
                    result.shader_prefix.extend_from_slice(&[
                        31,
                        0x8000_0000 | (u32::from(usage_index) << 16) | u32::from(usage),
                        0x900f_0000 | register,
                    ]);
                }
            }
            D3DVSD_TOKEN_CONSTMEM => {
                let first = token & 127;
                let count = (token >> 25) & 15;
                if token & 0x01ff_ff80 != 0 || first + count > 256 {
                    return None;
                }
                for (index, value) in payload.as_chunks::<4>().0.iter().enumerate() {
                    result
                        .constants
                        .push(Constant::new(first + u32::try_from(index).ok()?, *value));
                }
            }
            D3DVSD_TOKEN_EXT => {}
            _ => return None,
        }
        cursor += words;
    }
    None
}

fn register_semantic(register: u32) -> Option<(u8, u8)> {
    match register {
        0 => Some((D3DDECLUSAGE_POSITION, 0)),
        1 => Some((D3DDECLUSAGE_BLENDWEIGHT, 0)),
        2 => Some((D3DDECLUSAGE_BLENDINDICES, 0)),
        3 => Some((D3DDECLUSAGE_NORMAL, 0)),
        4 => Some((D3DDECLUSAGE_PSIZE, 0)),
        5 => Some((D3DDECLUSAGE_COLOR, 0)),
        6 => Some((D3DDECLUSAGE_COLOR, 1)),
        7..=14 => Some((
            D3DDECLUSAGE_TEXCOORD,
            u8::try_from(register - 7).expect("texture register is between 7 and 14"),
        )),
        15 => Some((D3DDECLUSAGE_POSITION, 1)),
        16 => Some((D3DDECLUSAGE_NORMAL, 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
