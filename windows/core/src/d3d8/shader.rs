//! D3D8 float constants and their separation from backend shader-local definitions.

use mtld3d_types::{D3D8_MAX_PIXEL_SHADER_CONSTANTS, D3D8_MAX_VERTEX_SHADER_CONSTANTS};

use crate::dxso::operand_token_count;

/// One float4 register loaded into device state when its D3D8 shader is bound.
pub struct Constant {
    register: u32,
    value: [f32; 4],
}

impl Constant {
    /// Preserves the four immediate words without numeric conversion.
    #[must_use]
    pub(super) fn new(register: u32, value: [u32; 4]) -> Self {
        Self {
            register,
            value: value.map(f32::from_bits),
        }
    }

    /// The destination register, validated against the shader stage by its parser.
    #[must_use]
    pub const fn register(&self) -> u32 {
        self.register
    }

    /// The complete float4 value, retaining all input bit patterns.
    #[must_use]
    pub const fn value(&self) -> &[f32; 4] {
        &self.value
    }
}

/// Backend bytecode and the global register writes separated from its DEF instructions.
pub struct Shader {
    words: Vec<u32>,
    constants: Vec<Constant>,
}

impl Shader {
    /// Transfers the translated bytecode and ordered constant definitions to the caller.
    #[must_use]
    pub fn into_parts(self) -> (Vec<u32>, Vec<Constant>) {
        (self.words, self.constants)
    }
}

/// Removes DEF instructions from a terminated D3D8 shader while retaining their values.
///
/// Returns `None` for an unsupported version, malformed definition, out-of-range constant,
/// incomplete payload, or missing terminator. Comments and non-DEF instructions keep their
/// original words for backend validation. Work and storage are linear in the input length,
/// capped at 65536 words. Repeated definitions retain their order so the last write wins.
#[must_use]
pub fn translate(words: &[u32]) -> Option<Shader> {
    if words.len() > 65536 {
        return None;
    }
    let &version = words.first()?;
    let limit = match version {
        0xfffe_0101 => D3D8_MAX_VERTEX_SHADER_CONSTANTS,
        0xffff_0100..=0xffff_0104 => D3D8_MAX_PIXEL_SHADER_CONSTANTS,
        _ => return None,
    };
    let mut result = Shader {
        words: Vec::with_capacity(words.len()),
        constants: Vec::new(),
    };
    result.words.push(version);
    let mut cursor = 1;
    while let Some(&token) = words.get(cursor) {
        let start = cursor;
        cursor += 1;
        match token & 0xffff {
            0xffff => {
                result.words.push(token);
                return (cursor == words.len()).then_some(result);
            }
            0xfffe => cursor = cursor.checked_add(((token >> 16) & 0x7fff) as usize)?,
            81 => {
                let payload: &[u32; 5] = words.get(cursor..cursor + 5)?.try_into().ok()?;
                let register = payload[0] & 0x7ff;
                if token != 81 || payload[0] & !0x7ff != 0xa00f_0000 || register >= limit {
                    return None;
                }
                result.constants.push(Constant::new(
                    register,
                    [payload[1], payload[2], payload[3], payload[4]],
                ));
                cursor += 5;
                continue;
            }
            47 | 48 => return None,
            _ => {
                cursor += operand_token_count(1, token, |index| words.get(cursor + index).copied());
            }
        }
        result.words.extend_from_slice(words.get(start..cursor)?);
    }
    None
}

#[cfg(test)]
mod tests;
