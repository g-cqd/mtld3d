//! A child resource retains its public D3D8 texture or device container.

use core::{ffi::c_void, ptr};

use mtld3d_shared::OutPtr;
use mtld3d_types::Guid;

use super::Resource8;
use crate::device::{self, Device8};

pub enum ContainerRef {
    Resource(*mut c_void),
    Device(*mut c_void),
}

impl ContainerRef {
    pub fn retain(resource: &Resource8) -> Self {
        resource.retain();
        Self::Resource(ptr::from_ref(resource).cast_mut().cast())
    }

    pub fn device(device: &Device8) -> Self {
        let pointer = ptr::from_ref(device).cast_mut().cast();
        device::add_ref(pointer);
        Self::Device(pointer)
    }

    pub fn query(&self, iid: &Guid, output: OutPtr<'_, *mut c_void>) -> i32 {
        let mut result = ptr::null_mut();
        let status = match *self {
            Self::Resource(pointer) => super::query_interface(pointer, iid, &raw mut result),
            Self::Device(pointer) => device::query_interface(pointer, iid, &raw mut result),
        };
        output.write(result);
        status
    }
}

impl Drop for ContainerRef {
    fn drop(&mut self) {
        match *self {
            Self::Resource(pointer) => {
                super::release(pointer);
            }
            Self::Device(pointer) => {
                device::release(pointer);
            }
        }
    }
}
