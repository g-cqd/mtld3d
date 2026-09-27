//! Typed owned resources for D3D8 resource-lifetime integration tests.

use core::{ffi::c_void, marker::PhantomData};

use super::D3D8Harness;
use crate::{resource::release_unknown, vtbl::deref_vtbl};

mod buffer;
mod cube;
mod volume;

/// Owns an interface with vtable V while its creating harness remains alive.
pub struct D3D8Resource<'a, V> {
    pointer: *mut c_void,
    owner: PhantomData<(&'a D3D8Harness, V)>,
}

impl<V: 'static> D3D8Resource<'_, V> {
    /// Adopts exactly one typed COM reference returned by a successful getter.
    ///
    /// # Safety
    /// The pointer must carry one owned reference with vtable V.
    const unsafe fn from_owned(pointer: *mut c_void) -> Self {
        Self {
            pointer,
            owner: PhantomData,
        }
    }

    fn vtable(&self) -> &V {
        // SAFETY: construction fixed V and the owned reference keeps the interface live.
        unsafe { deref_vtbl(self.pointer) }
    }

    /// Whether two references designate the same frontend interface.
    #[must_use]
    pub fn is_same_object(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
}

impl<V> Drop for D3D8Resource<'_, V> {
    fn drop(&mut self) {
        // SAFETY: construction adopted exactly one owned COM reference, consumed here.
        unsafe { release_unknown(self.pointer) };
    }
}
