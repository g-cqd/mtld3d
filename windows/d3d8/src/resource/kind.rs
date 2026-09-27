//! Concrete resource kinds and the interfaces each D3D8 wrapper exposes.

use mtld3d_d3d8_types::{
    IID_IDIRECT3DBASETEXTURE8, IID_IDIRECT3DCUBETEXTURE8, IID_IDIRECT3DINDEXBUFFER8,
    IID_IDIRECT3DRESOURCE8, IID_IDIRECT3DSURFACE8, IID_IDIRECT3DTEXTURE8,
    IID_IDIRECT3DVERTEXBUFFER8, IID_IDIRECT3DVOLUME8, IID_IDIRECT3DVOLUMETEXTURE8,
};
use mtld3d_types::{
    D3DRTYPE_CUBETEXTURE, D3DRTYPE_INDEXBUFFER, D3DRTYPE_SURFACE, D3DRTYPE_TEXTURE,
    D3DRTYPE_VERTEXBUFFER, D3DRTYPE_VOLUME, D3DRTYPE_VOLUMETEXTURE, Guid, IID_IUNKNOWN,
};

pub enum ResourceKind {
    Surface,
    Volume,
    Texture,
    VolumeTexture,
    CubeTexture,
    VertexBuffer,
    IndexBuffer,
}

impl ResourceKind {
    pub const fn code(&self) -> u32 {
        match self {
            Self::Surface => D3DRTYPE_SURFACE,
            Self::Volume => D3DRTYPE_VOLUME,
            Self::Texture => D3DRTYPE_TEXTURE,
            Self::VolumeTexture => D3DRTYPE_VOLUMETEXTURE,
            Self::CubeTexture => D3DRTYPE_CUBETEXTURE,
            Self::VertexBuffer => D3DRTYPE_VERTEXBUFFER,
            Self::IndexBuffer => D3DRTYPE_INDEXBUFFER,
        }
    }

    pub fn supports(&self, iid: &Guid) -> bool {
        *iid == IID_IUNKNOWN
            || *iid == *self.iid()
            || (!matches!(self, Self::Surface | Self::Volume) && *iid == IID_IDIRECT3DRESOURCE8)
            || (matches!(
                self,
                Self::Texture | Self::VolumeTexture | Self::CubeTexture
            ) && *iid == IID_IDIRECT3DBASETEXTURE8)
    }

    const fn iid(&self) -> &Guid {
        match self {
            Self::Surface => &IID_IDIRECT3DSURFACE8,
            Self::Volume => &IID_IDIRECT3DVOLUME8,
            Self::Texture => &IID_IDIRECT3DTEXTURE8,
            Self::VolumeTexture => &IID_IDIRECT3DVOLUMETEXTURE8,
            Self::CubeTexture => &IID_IDIRECT3DCUBETEXTURE8,
            Self::VertexBuffer => &IID_IDIRECT3DVERTEXBUFFER8,
            Self::IndexBuffer => &IID_IDIRECT3DINDEXBUFFER8,
        }
    }
}
