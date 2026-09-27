//! D3D8 shader handles, declaration conversion, and constant-register access.

use core::{ffi::c_void, ptr};

use mtld3d_core::d3d8::{
    declaration::translate,
    shader::{self, Constant},
};
use mtld3d_shared::OutPtr;
use mtld3d_types::{
    D3D_OK, D3D8_FVF_MAX, D3D8_MAX_PIXEL_SHADER_CONSTANTS, D3D8_MAX_VERTEX_SHADER_CONSTANTS,
    D3DERR_INVALIDCALL, D3DUSAGE_SOFTWAREPROCESSING, E_OUTOFMEMORY, IDirect3DDevice9Vtbl,
    IDirect3DPixelShader9Vtbl, IDirect3DVertexDeclaration9Vtbl, IDirect3DVertexShader9Vtbl,
};

use super::{
    api_scope, object,
    state::{PIXEL_HANDLE, VERTEX_HANDLE},
    tokens::{Tokens, copy_words},
};
use crate::backend::Backend;

/// Owns a translated shader and the unmodified bytes returned to its D3D8 caller.
pub struct VertexShader {
    pub declaration: Backend<IDirect3DVertexDeclaration9Vtbl>,
    pub shader: Option<Backend<IDirect3DVertexShader9Vtbl>>,
    pub declaration_words: Vec<u32>,
    pub function_words: Vec<u32>,
    pub constants: Vec<Constant>,
}

/// Owns backend bytecode separately from the unmodified function and bind-time constants.
pub struct PixelShader {
    pub shader: Backend<IDirect3DPixelShader9Vtbl>,
    pub function_words: Vec<u32>,
    pub constants: Vec<Constant>,
}

pub extern "system" fn create_vertex_shader(
    this: *mut c_void,
    declaration: *const u32,
    function: *const u32,
    output: *mut u32,
    usage: u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: output is writable handle storage or null per the ABI.
    let Some(out) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    out.write(0);
    if usage & !D3DUSAGE_SOFTWAREPROCESSING != 0 {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the declaration argument is terminated and includes its declared payloads.
    let Some(declaration) = (unsafe { Tokens::new(declaration) }) else {
        return D3DERR_INVALIDCALL;
    };
    let Some(declaration_words) = declaration.declaration() else {
        return D3DERR_INVALIDCALL;
    };
    let Some(converted) = translate(&declaration_words, function.is_null()) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: a non-null function argument is a terminated shader with complete payloads.
    let function = unsafe { Tokens::new(function) };
    let function_words = match function {
        Some(function) => match function.shader(true) {
            Some(words) => words,
            None => return D3DERR_INVALIDCALL,
        },
        None => Vec::new(),
    };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    let mut state = object.state();
    if state.vertex_shaders.try_reserve(1).is_err() {
        return E_OUTOFMEMORY;
    }
    let handle = match state.allocate_handle() {
        Ok(handle) => handle,
        Err(error) => return error,
    };
    let mut declaration = ptr::null_mut();
    // SAFETY: conversion emits a valid terminated element list and writable interface output.
    let result = unsafe {
        (backend.table().create_vertex_declaration)(
            backend.pointer(),
            converted.elements.as_ptr().cast(),
            &raw mut declaration,
        )
    };
    if result < 0 {
        return result;
    }
    // SAFETY: successful creation returns one owned vertex declaration reference.
    let Some(declaration) = (unsafe { Backend::adopt(declaration) }) else {
        return D3DERR_INVALIDCALL;
    };
    let mut constants = converted.constants;
    let shader = if function_words.is_empty() {
        None
    } else {
        let Some(function) = shader::translate(&function_words) else {
            return D3DERR_INVALIDCALL;
        };
        let (function, function_constants) = function.into_parts();
        constants.extend(function_constants);
        let mut translated = Vec::with_capacity(function.len() + converted.shader_prefix.len());
        translated.push(function[0]);
        translated.extend_from_slice(&converted.shader_prefix);
        translated.extend_from_slice(&function[1..]);
        let mut pointer = ptr::null_mut();
        // SAFETY: translated contains a terminated shader and output is writable.
        let result = unsafe {
            (backend.table().create_vertex_shader)(
                backend.pointer(),
                translated.as_ptr(),
                &raw mut pointer,
            )
        };
        if result < 0 {
            return result;
        }
        // SAFETY: successful creation returns one owned vertex shader reference.
        let Some(shader) = (unsafe { Backend::adopt(pointer) }) else {
            return D3DERR_INVALIDCALL;
        };
        Some(shader)
    };
    state.vertex_shaders.insert(
        handle,
        VertexShader {
            declaration,
            shader,
            declaration_words,
            function_words,
            constants,
        },
    );
    drop(state);
    // SAFETY: the original ABI output remains writable after successful creation.
    unsafe { OutPtr::write_opt(output, handle) };
    D3D_OK
}

pub extern "system" fn set_vertex_shader(this: *mut c_void, handle: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    let mut state = object.state();
    let result = if handle <= D3D8_FVF_MAX {
        // SAFETY: the FVF is validated by the shared backend before mutation.
        let result = unsafe { (backend.table().set_fvf)(backend.pointer(), handle) };
        if result < 0 {
            return result;
        }
        // SAFETY: null disables the programmable vertex shader.
        unsafe { (backend.table().set_vertex_shader)(backend.pointer(), ptr::null_mut()) }
    } else {
        let Some(shader) = state.vertex_shaders.get(&handle) else {
            return D3DERR_INVALIDCALL;
        };
        // SAFETY: this device owns the live translated declaration.
        let result = unsafe {
            (backend.table().set_vertex_declaration)(
                backend.pointer(),
                shader.declaration.pointer(),
            )
        };
        if result < 0 {
            return result;
        }
        let pointer = shader
            .shader
            .as_ref()
            .map_or(ptr::null_mut(), Backend::pointer);
        // SAFETY: the shader is null or a live backend vertex shader owned by this handle.
        let result = unsafe { (backend.table().set_vertex_shader)(backend.pointer(), pointer) };
        if result < 0 {
            return result;
        }
        load_constants(
            backend,
            &shader.constants,
            backend.table().set_vertex_shader_constant_f,
        )
    };
    if result >= 0 {
        state.set_extra(VERTEX_HANDLE, handle);
    }
    result
}

pub extern "system" fn get_vertex_shader(this: *mut c_void, output: *mut u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: output is writable handle storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    output.write(object.state().current[VERTEX_HANDLE]);
    D3D_OK
}

pub extern "system" fn delete_vertex_shader(this: *mut c_void, handle: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let mut state = object.state();
    if !state.vertex_shaders.contains_key(&handle) {
        return D3DERR_INVALIDCALL;
    }
    if state.current[VERTEX_HANDLE] == handle {
        drop(state);
        let result = set_vertex_shader(this, 0);
        if result < 0 {
            return result;
        }
        state = object.state();
    }
    state.vertex_shaders.remove(&handle);
    D3D_OK
}

pub extern "system" fn get_vertex_shader_declaration(
    this: *mut c_void,
    handle: u32,
    output: *mut c_void,
    size: *mut u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let state = object.state();
    let Some(shader) = state.vertex_shaders.get(&handle) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this ABI supplies writable output bytes and an in-out byte count.
    let result = unsafe { copy_words(&shader.declaration_words, output, size) };
    drop(state);
    result
}

pub extern "system" fn get_vertex_shader_function(
    this: *mut c_void,
    handle: u32,
    output: *mut c_void,
    size: *mut u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let state = object.state();
    let Some(shader) = state.vertex_shaders.get(&handle) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this ABI supplies writable output bytes and an in-out byte count.
    let result = unsafe { copy_words(&shader.function_words, output, size) };
    drop(state);
    result
}

pub extern "system" fn create_pixel_shader(
    this: *mut c_void,
    function: *const u32,
    output: *mut u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: output is writable handle storage or null.
    let Some(out) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    out.write(0);
    // SAFETY: the function argument is a terminated shader with complete payloads.
    let Some(function) = (unsafe { Tokens::new(function) }) else {
        return D3DERR_INVALIDCALL;
    };
    let Some(function) = function.shader(false) else {
        return D3DERR_INVALIDCALL;
    };
    let Some(translated) = shader::translate(&function) else {
        return D3DERR_INVALIDCALL;
    };
    let (translated, constants) = translated.into_parts();
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    let mut state = object.state();
    if state.pixel_shaders.try_reserve(1).is_err() {
        return E_OUTOFMEMORY;
    }
    let handle = match state.allocate_handle() {
        Ok(handle) => handle,
        Err(error) => return error,
    };
    let mut pointer = ptr::null_mut();
    // SAFETY: function is terminated shader bytecode and output is writable.
    let result = unsafe {
        (backend.table().create_pixel_shader)(
            backend.pointer(),
            translated.as_ptr(),
            &raw mut pointer,
        )
    };
    if result < 0 {
        return result;
    }
    // SAFETY: successful creation returns one owned backend pixel shader reference.
    let Some(shader) = (unsafe { Backend::adopt(pointer) }) else {
        return D3DERR_INVALIDCALL;
    };
    state.pixel_shaders.insert(
        handle,
        PixelShader {
            shader,
            function_words: function,
            constants,
        },
    );
    drop(state);
    // SAFETY: the original handle output remains writable for this call.
    unsafe { OutPtr::write_opt(output, handle) };
    D3D_OK
}

pub extern "system" fn set_pixel_shader(this: *mut c_void, handle: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    let mut state = object.state();
    let pointer = if handle == 0 {
        ptr::null_mut()
    } else {
        let Some(shader) = state.pixel_shaders.get(&handle) else {
            return D3DERR_INVALIDCALL;
        };
        shader.shader.pointer()
    };
    // SAFETY: the shader is null or a live pixel shader owned by this device's handle table.
    let result = unsafe { (backend.table().set_pixel_shader)(backend.pointer(), pointer) };
    if result < 0 {
        return result;
    }
    if let Some(shader) = state.pixel_shaders.get(&handle) {
        let result = load_constants(
            backend,
            &shader.constants,
            backend.table().set_pixel_shader_constant_f,
        );
        if result < 0 {
            return result;
        }
    }
    state.set_extra(PIXEL_HANDLE, handle);
    D3D_OK
}

pub extern "system" fn get_pixel_shader(this: *mut c_void, output: *mut u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: output is writable handle storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    output.write(object.state().current[PIXEL_HANDLE]);
    D3D_OK
}

pub extern "system" fn delete_pixel_shader(this: *mut c_void, handle: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let mut state = object.state();
    if !state.pixel_shaders.contains_key(&handle) {
        return D3DERR_INVALIDCALL;
    }
    if state.current[PIXEL_HANDLE] == handle {
        drop(state);
        let result = set_pixel_shader(this, 0);
        if result < 0 {
            return result;
        }
        state = object.state();
    }
    state.pixel_shaders.remove(&handle);
    D3D_OK
}

pub extern "system" fn get_pixel_shader_function(
    this: *mut c_void,
    handle: u32,
    output: *mut c_void,
    size: *mut u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let state = object.state();
    let Some(shader) = state.pixel_shaders.get(&handle) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: this ABI supplies writable output bytes and an in-out byte count.
    let result = unsafe { copy_words(&shader.function_words, output, size) };
    drop(state);
    result
}

pub extern "system" fn set_vertex_shader_constant(
    this: *mut c_void,
    start: u32,
    values: *const c_void,
    count: u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    if start > D3D8_MAX_VERTEX_SHADER_CONSTANTS || count > D3D8_MAX_VERTEX_SHADER_CONSTANTS - start
    {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: the ABI supplies count float4 registers in the caller's input or output storage.
    unsafe {
        (backend.table().set_vertex_shader_constant_f)(
            backend.pointer(),
            start,
            values.cast(),
            count,
        )
    }
}

pub extern "system" fn get_vertex_shader_constant(
    this: *mut c_void,
    start: u32,
    values: *mut c_void,
    count: u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    if start > D3D8_MAX_VERTEX_SHADER_CONSTANTS || count > D3D8_MAX_VERTEX_SHADER_CONSTANTS - start
    {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: the ABI supplies count float4 registers in the caller's input or output storage.
    unsafe {
        (backend.table().get_vertex_shader_constant_f)(
            backend.pointer(),
            start,
            values.cast(),
            count,
        )
    }
}

pub extern "system" fn set_pixel_shader_constant(
    this: *mut c_void,
    start: u32,
    values: *const c_void,
    count: u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    if start > D3D8_MAX_PIXEL_SHADER_CONSTANTS || count > D3D8_MAX_PIXEL_SHADER_CONSTANTS - start {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: the ABI supplies count float4 registers in the caller's input or output storage.
    unsafe {
        (backend.table().set_pixel_shader_constant_f)(
            backend.pointer(),
            start,
            values.cast(),
            count,
        )
    }
}

pub extern "system" fn get_pixel_shader_constant(
    this: *mut c_void,
    start: u32,
    values: *mut c_void,
    count: u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    if start > D3D8_MAX_PIXEL_SHADER_CONSTANTS || count > D3D8_MAX_PIXEL_SHADER_CONSTANTS - start {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the receiver remains live under the caller's COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: the ABI supplies count float4 registers in the caller's input or output storage.
    unsafe {
        (backend.table().get_pixel_shader_constant_f)(
            backend.pointer(),
            start,
            values.cast(),
            count,
        )
    }
}

fn load_constants(
    backend: &Backend<IDirect3DDevice9Vtbl>,
    constants: &[Constant],
    setter: unsafe extern "system" fn(*mut c_void, u32, *const f32, u32) -> i32,
) -> i32 {
    for constant in constants {
        // SAFETY: creation validated the destination; the owned value contains one float4.
        let result = unsafe {
            setter(
                backend.pointer(),
                constant.register(),
                constant.value().as_ptr(),
                1,
            )
        };
        if result < 0 {
            return result;
        }
    }
    D3D_OK
}
