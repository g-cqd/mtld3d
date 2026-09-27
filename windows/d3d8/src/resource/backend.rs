//! Concrete backend interfaces retained by public D3D8 resource wrappers.

use core::ffi::c_void;

use mtld3d_types::{
    IDirect3DCubeTexture9Vtbl, IDirect3DIndexBuffer9Vtbl, IDirect3DSurface9Vtbl,
    IDirect3DTexture9Vtbl, IDirect3DVertexBuffer9Vtbl, IDirect3DVolume9Vtbl,
    IDirect3DVolumeTexture9Vtbl,
};

use super::kind::ResourceKind;
use crate::backend::Backend;

pub enum ResourceBackend {
    Texture(Backend<IDirect3DTexture9Vtbl>),
    CubeTexture(Backend<IDirect3DCubeTexture9Vtbl>),
    VolumeTexture(Backend<IDirect3DVolumeTexture9Vtbl>),
    VertexBuffer(Backend<IDirect3DVertexBuffer9Vtbl>),
    IndexBuffer(Backend<IDirect3DIndexBuffer9Vtbl>),
    Surface(Backend<IDirect3DSurface9Vtbl>),
    Volume(Backend<IDirect3DVolume9Vtbl>),
}

impl ResourceBackend {
    pub const fn kind(&self) -> ResourceKind {
        match self {
            Self::Texture(_) => ResourceKind::Texture,
            Self::CubeTexture(_) => ResourceKind::CubeTexture,
            Self::VolumeTexture(_) => ResourceKind::VolumeTexture,
            Self::VertexBuffer(_) => ResourceKind::VertexBuffer,
            Self::IndexBuffer(_) => ResourceKind::IndexBuffer,
            Self::Surface(_) => ResourceKind::Surface,
            Self::Volume(_) => ResourceKind::Volume,
        }
    }

    pub const fn pointer(&self) -> *mut c_void {
        match self {
            Self::Texture(value) => value.pointer(),
            Self::CubeTexture(value) => value.pointer(),
            Self::VolumeTexture(value) => value.pointer(),
            Self::VertexBuffer(value) => value.pointer(),
            Self::IndexBuffer(value) => value.pointer(),
            Self::Surface(value) => value.pointer(),
            Self::Volume(value) => value.pointer(),
        }
    }

    /// Returns the pool used to determine whether a live resource blocks Reset.
    pub fn pool(&self) -> Result<u32, i32> {
        match self {
            Self::Texture(backend) => {
                let mut description =
                    core::mem::MaybeUninit::<mtld3d_types::D3DSURFACE_DESC>::uninit();
                // SAFETY: the owned typed backend writes its complete resource descriptor.
                let status = unsafe {
                    (backend.table().get_level_desc)(
                        backend.pointer(),
                        0,
                        description.as_mut_ptr().cast(),
                    )
                };
                if status < 0 {
                    return Err(status);
                }
                // SAFETY: successful GetDesc initialized the complete resource descriptor.
                Ok(unsafe { description.assume_init() }.pool)
            }
            Self::CubeTexture(backend) => {
                let mut description =
                    core::mem::MaybeUninit::<mtld3d_types::D3DSURFACE_DESC>::uninit();
                // SAFETY: the owned typed backend writes its complete resource descriptor.
                let status = unsafe {
                    (backend.table().get_level_desc)(
                        backend.pointer(),
                        0,
                        description.as_mut_ptr().cast(),
                    )
                };
                if status < 0 {
                    return Err(status);
                }
                // SAFETY: successful GetDesc initialized the complete resource descriptor.
                Ok(unsafe { description.assume_init() }.pool)
            }
            Self::VolumeTexture(backend) => {
                let mut description =
                    core::mem::MaybeUninit::<mtld3d_types::D3DVOLUME_DESC>::uninit();
                // SAFETY: the owned typed backend writes its complete resource descriptor.
                let status = unsafe {
                    (backend.table().get_level_desc)(
                        backend.pointer(),
                        0,
                        description.as_mut_ptr().cast(),
                    )
                };
                if status < 0 {
                    return Err(status);
                }
                // SAFETY: successful GetDesc initialized the complete resource descriptor.
                Ok(unsafe { description.assume_init() }.pool)
            }
            Self::VertexBuffer(backend) => {
                let mut description =
                    core::mem::MaybeUninit::<mtld3d_types::D3DVERTEXBUFFER_DESC>::uninit();
                // SAFETY: the owned typed backend writes its complete resource descriptor.
                let status = unsafe {
                    (backend.table().get_desc)(backend.pointer(), description.as_mut_ptr().cast())
                };
                if status < 0 {
                    return Err(status);
                }
                // SAFETY: successful GetDesc initialized the complete resource descriptor.
                Ok(unsafe { description.assume_init() }.pool)
            }
            Self::IndexBuffer(backend) => {
                let mut description =
                    core::mem::MaybeUninit::<mtld3d_types::D3DINDEXBUFFER_DESC>::uninit();
                // SAFETY: the owned typed backend writes its complete resource descriptor.
                let status = unsafe {
                    (backend.table().get_desc)(backend.pointer(), description.as_mut_ptr().cast())
                };
                if status < 0 {
                    return Err(status);
                }
                // SAFETY: successful GetDesc initialized the complete resource descriptor.
                Ok(unsafe { description.assume_init() }.pool)
            }
            Self::Surface(backend) => {
                let mut description =
                    core::mem::MaybeUninit::<mtld3d_types::D3DSURFACE_DESC>::uninit();
                // SAFETY: the owned typed backend writes its complete resource descriptor.
                let status = unsafe {
                    (backend.table().get_desc)(backend.pointer(), description.as_mut_ptr().cast())
                };
                if status < 0 {
                    return Err(status);
                }
                // SAFETY: successful GetDesc initialized the complete resource descriptor.
                Ok(unsafe { description.assume_init() }.pool)
            }
            Self::Volume(backend) => {
                let mut description =
                    core::mem::MaybeUninit::<mtld3d_types::D3DVOLUME_DESC>::uninit();
                // SAFETY: the owned typed backend writes its complete resource descriptor.
                let status = unsafe {
                    (backend.table().get_desc)(backend.pointer(), description.as_mut_ptr().cast())
                };
                if status < 0 {
                    return Err(status);
                }
                // SAFETY: successful GetDesc initialized the complete resource descriptor.
                Ok(unsafe { description.assume_init() }.pool)
            }
        }
    }

    pub fn identity(&self) -> Result<usize, i32> {
        match self {
            Self::Texture(value) => value.identity(),
            Self::CubeTexture(value) => value.identity(),
            Self::VolumeTexture(value) => value.identity(),
            Self::VertexBuffer(value) => value.identity(),
            Self::IndexBuffer(value) => value.identity(),
            Self::Surface(value) => value.identity(),
            Self::Volume(value) => value.identity(),
        }
    }
}
