//! Selectors and token encodings specific to the Direct3D 8 ABI.

/// D3D8 line-pattern render-state selector.
pub const D3DRS8_LINEPATTERN: u32 = 10;

/// D3D8 depth-visibility render-state selector.
pub const D3DRS8_ZVISIBLE: u32 = 30;

/// D3D8 edge-antialias render-state selector.
pub const D3DRS8_EDGEANTIALIAS: u32 = 40;

/// D3D8 integer depth-bias render-state selector.
pub const D3DRS8_ZBIAS: u32 = 47;

/// D3D8 software vertex-processing render-state selector.
pub const D3DRS8_SOFTWAREVERTEXPROCESSING: u32 = 153;

/// D3D8 patch-segment render-state selector.
pub const D3DRS8_PATCHSEGMENTS: u32 = 164;

/// D3D8 `ADDRESSU` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_ADDRESSU: u32 = 13;

/// D3D8 `ADDRESSV` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_ADDRESSV: u32 = 14;

/// D3D8 `BORDERCOLOR` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_BORDERCOLOR: u32 = 15;

/// D3D8 `MAGFILTER` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_MAGFILTER: u32 = 16;

/// D3D8 `MINFILTER` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_MINFILTER: u32 = 17;

/// D3D8 `MIPFILTER` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_MIPFILTER: u32 = 18;

/// D3D8 `MIPMAPLODBIAS` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_MIPMAPLODBIAS: u32 = 19;

/// D3D8 `MAXMIPLEVEL` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_MAXMIPLEVEL: u32 = 20;

/// D3D8 `MAXANISOTROPY` texture-stage selector; D3D9 stores it as sampler state.
pub const D3DTSS8_MAXANISOTROPY: u32 = 21;

/// D3D8 W-addressing texture-stage selector.
pub const D3DTSS8_ADDRESSW: u32 = 25;

/// Upper bound reserved for fixed-function vertex-format values.
pub const D3D8_FVF_MAX: u32 = 0xF000_0000;

/// D3D8 declaration-token discriminator for `NOP`.
pub const D3DVSD_TOKEN_NOP: u32 = 0;

/// D3D8 declaration-token discriminator for `STREAM`.
pub const D3DVSD_TOKEN_STREAM: u32 = 1;

/// D3D8 declaration-token discriminator for `STREAMDATA`.
pub const D3DVSD_TOKEN_STREAMDATA: u32 = 2;

/// D3D8 declaration-token discriminator for `TESSELLATOR`.
pub const D3DVSD_TOKEN_TESSELLATOR: u32 = 3;

/// D3D8 declaration-token discriminator for `CONSTMEM`.
pub const D3DVSD_TOKEN_CONSTMEM: u32 = 4;

/// D3D8 declaration-token discriminator for `EXT`.
pub const D3DVSD_TOKEN_EXT: u32 = 5;

/// D3D8 declaration-token discriminator for `END`.
pub const D3DVSD_TOKEN_END: u32 = 7;

/// The D3D8 vertex-declaration terminator.
pub const D3DVSD_END: u32 = u32::MAX;

/// Bit offset of the D3D8 declaration-token discriminator.
pub const D3DVSD_TOKENTYPESHIFT: u32 = 29;

/// Number of addressable D3D8 vertex float-constant registers.
pub const D3D8_MAX_VERTEX_SHADER_CONSTANTS: u32 = 256;

/// Number of addressable D3D8 pixel float-constant registers.
pub const D3D8_MAX_PIXEL_SHADER_CONSTANTS: u32 = 8;
