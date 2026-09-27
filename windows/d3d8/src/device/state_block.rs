//! D3D8 state-block tokens backed by shared state-block objects.

use core::{ffi::c_void, ptr};

use mtld3d_shared::OutPtr;
use mtld3d_types::{
    D3D_OK, D3DERR_INVALIDCALL, E_OUTOFMEMORY, IDirect3DStateBlock9Vtbl, StateBlockType,
};

use super::{
    api_scope, object,
    state::{
        BASE_VERTEX, EXTRA_COUNT, PATCH_SEGMENTS, PIXEL_HANDLE, SOFTWARE_VP, VERTEX_HANDLE, Z_BIAS,
    },
};
use crate::backend::Backend;

/// A captured backend state block and the D3D8-only values it selects.
pub struct StateBlock {
    pub backend: Backend<IDirect3DStateBlock9Vtbl>,
    pub extra: [Option<u32>; EXTRA_COUNT],
}

pub extern "system" fn begin_state_block(this: *mut c_void) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let mut state = object.state();
    if state.recording.is_some() {
        return D3DERR_INVALIDCALL;
    }
    let backend = object.backend();
    // SAFETY: the backend owns a matching live device and vtable.
    let result = unsafe { (backend.table().begin_state_block)(backend.pointer()) };
    if result >= 0 {
        state.recording = Some([None; EXTRA_COUNT]);
    }
    result
}

pub extern "system" fn end_state_block(this: *mut c_void, output: *mut u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: output is writable handle storage or null.
    let Some(out) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let mut state = object.state();
    if state.recording.is_none() {
        return D3DERR_INVALIDCALL;
    }
    if state.blocks.try_reserve(1).is_err() {
        return E_OUTOFMEMORY;
    }
    let handle = match state.allocate_handle() {
        Ok(handle) => handle,
        Err(error) => return error,
    };
    let backend = object.backend();
    let mut pointer = ptr::null_mut();
    // SAFETY: the backend owns a recording device and output is writable.
    let result = unsafe { (backend.table().end_state_block)(backend.pointer(), &raw mut pointer) };
    if result < 0 {
        return result;
    }
    let extra = state
        .recording
        .take()
        .expect("successful EndStateBlock follows BeginStateBlock");
    // SAFETY: successful EndStateBlock returns one owned state-block reference.
    let Some(backend) = (unsafe { Backend::adopt(pointer) }) else {
        return D3DERR_INVALIDCALL;
    };
    state.blocks.insert(handle, StateBlock { backend, extra });
    drop(state);
    out.write(handle);
    D3D_OK
}

pub extern "system" fn create_state_block(this: *mut c_void, kind: u32, output: *mut u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: output is writable handle storage or null.
    let Some(out) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    out.write(0);
    let Some(filter) = StateBlockType::from_d3dsbt(kind) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let mut state = object.state();
    if state.recording.is_some() {
        return D3DERR_INVALIDCALL;
    }
    if state.blocks.try_reserve(1).is_err() {
        return E_OUTOFMEMORY;
    }
    let handle = match state.allocate_handle() {
        Ok(handle) => handle,
        Err(error) => return error,
    };
    let backend = object.backend();
    let mut pointer = ptr::null_mut();
    // SAFETY: the backend owns a live device and output is writable.
    let result =
        unsafe { (backend.table().create_state_block)(backend.pointer(), kind, &raw mut pointer) };
    if result < 0 {
        return result;
    }
    // SAFETY: successful CreateStateBlock returns one owned state-block reference.
    let Some(backend) = (unsafe { Backend::adopt(pointer) }) else {
        return D3DERR_INVALIDCALL;
    };
    let mut extra = [None; EXTRA_COUNT];
    if filter.includes_vertex_pipeline() {
        for index in [VERTEX_HANDLE, SOFTWARE_VP, PATCH_SEGMENTS] {
            extra[index] = Some(state.current[index]);
        }
    }
    if filter.includes_pixel_pipeline() {
        extra[PIXEL_HANDLE] = Some(state.current[PIXEL_HANDLE]);
        extra[Z_BIAS] = Some(state.current[Z_BIAS]);
    }
    if matches!(filter, StateBlockType::All) {
        extra[BASE_VERTEX] = Some(state.current[BASE_VERTEX]);
    }
    state.blocks.insert(handle, StateBlock { backend, extra });
    drop(state);
    // SAFETY: the original handle output remains writable during this ABI call.
    unsafe { OutPtr::write_opt(output, handle) };
    D3D_OK
}

pub extern "system" fn apply_state_block(this: *mut c_void, handle: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let mut state = object.state();
    if state.recording.is_some() {
        return D3DERR_INVALIDCALL;
    }
    let Some(block) = state.blocks.get(&handle) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the handle owns this live backend state block.
    let result = unsafe { (block.backend.table().apply)(block.backend.pointer()) };
    if result >= 0 {
        for index in 0..EXTRA_COUNT {
            if let Some(value) = state.blocks[&handle].extra[index] {
                state.current[index] = value;
            }
        }
    }
    result
}

pub extern "system" fn capture_state_block(this: *mut c_void, handle: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let mut state = object.state();
    if state.recording.is_some() {
        return D3DERR_INVALIDCALL;
    }
    let super::state::State8 {
        ref mut blocks,
        ref current,
        ..
    } = *state;
    let Some(block) = blocks.get_mut(&handle) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the handle owns this live backend state block.
    let result = unsafe { (block.backend.table().capture)(block.backend.pointer()) };
    if result >= 0 {
        for (recorded, value) in block.extra.iter_mut().zip(current) {
            if recorded.is_some() {
                *recorded = Some(*value);
            }
        }
    }
    drop(state);
    result
}

pub extern "system" fn delete_state_block(this: *mut c_void, handle: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    if object.state().blocks.remove(&handle).is_some() {
        D3D_OK
    } else {
        D3DERR_INVALIDCALL
    }
}
