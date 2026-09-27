//! Weak identity entries for resource wrappers with live public references.

use core::ffi::c_void;
use std::sync::{Mutex, MutexGuard, PoisonError};

use rustc_hash::FxHashMap;

use super::{Resource8, backend::ResourceBackend, container::ContainerRef};
use crate::device::Device8;

pub struct ResourceRegistry {
    objects: Mutex<FxHashMap<(usize, u32), *mut Resource8>>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            objects: Mutex::new(FxHashMap::default()),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, FxHashMap<(usize, u32), *mut Resource8>> {
        self.objects.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Every registry entry has at least one public reference and blocks Reset in DEFAULT pool.
    pub fn has_default_pool_resources(&self) -> bool {
        let objects = self.lock();
        let result = objects.values().any(|pointer| {
            // SAFETY: the registry lock prevents removing or finalizing these live wrappers.
            let resource = unsafe { super::object((*pointer).cast()) };
            resource.backend().pool() == Ok(mtld3d_types::D3DPOOL_DEFAULT)
        });
        drop(objects);
        result
    }

    pub fn wrap(&self, device: &Device8, backend: ResourceBackend) -> Result<*mut c_void, i32> {
        self.wrap_with_container(device, backend, None)
    }

    pub fn wrap_with_container(
        &self,
        device: &Device8,
        backend: ResourceBackend,
        container: Option<ContainerRef>,
    ) -> Result<*mut c_void, i32> {
        let identity = backend.identity()?;
        let key = (identity, backend.kind().code());
        let mut objects = self.lock();
        if let Some(pointer) = objects.get(&key) {
            // SAFETY: removal and the last decrement take this same lock; entries are live.
            let resource = unsafe { super::object((*pointer).cast()) };
            resource.retain();
            let pointer = (*pointer).cast();
            drop(objects);
            return Ok(pointer);
        }
        let pointer = Box::into_raw(Box::new(Resource8::new(
            device, backend, identity, container,
        )));
        objects.insert(key, pointer);
        drop(objects);
        Ok(pointer.cast())
    }
}
