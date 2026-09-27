//! Keeps the optional reentrant device lock alive through final COM release.

use std::sync::Arc;

use mtld3d_core::api_lock::{ApiGuard, ApiLock};

/// Field order releases the guard before its owning lock reference.
pub struct ApiScope {
    _guard: ApiGuard,
    _lock: Option<Arc<ApiLock>>,
}

impl ApiScope {
    pub fn enter(lock: Option<&Arc<ApiLock>>) -> Self {
        let lock = lock.cloned();
        let guard = lock.as_ref().map_or(ApiGuard::NOOP, |lock| {
            // SAFETY: the owning Arc is retained beside the guard and dropped after it.
            unsafe { lock.enter() }
        });
        Self {
            _guard: guard,
            _lock: lock,
        }
    }
}
