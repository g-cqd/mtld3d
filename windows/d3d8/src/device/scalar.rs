//! Scalar device state translated through the shared backend.

use core::ffi::c_void;

use mtld3d_core::d3d8::state::{depth_bias, sampler_state};
use mtld3d_shared::OutPtr;
use mtld3d_types::{
    D3D_OK, D3DDEVICE_CREATION_PARAMETERS, D3DDISPLAYMODE, D3DERR_INVALIDCALL, D3DGAMMARAMP,
    D3DLIGHT9, D3DMATERIAL9, D3DMATRIX, D3DRS_ANTIALIASEDLINEENABLE, D3DRS_DEPTHBIAS,
    D3DRS8_EDGEANTIALIAS, D3DRS8_LINEPATTERN, D3DRS8_PATCHSEGMENTS,
    D3DRS8_SOFTWAREVERTEXPROCESSING, D3DRS8_ZBIAS, D3DRS8_ZVISIBLE, D3DVIEWPORT9,
};

use super::{
    api_scope, object,
    state::{PATCH_SEGMENTS, SOFTWARE_VP, Z_BIAS},
};

pub extern "system" fn get_display_mode(this: *mut c_void, output: *mut D3DDISPLAYMODE) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_display_mode)(backend.pointer(), 0, output.cast()) }
}

pub extern "system" fn get_creation_parameters(
    this: *mut c_void,
    output: *mut D3DDEVICE_CREATION_PARAMETERS,
) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_creation_parameters)(backend.pointer(), output.cast()) }
}

pub extern "system" fn get_raster_status(this: *mut c_void, output: *mut c_void) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_raster_status)(backend.pointer(), 0, output) }
}

pub extern "system" fn set_cursor_position(this: *mut c_void, x: u32, y: u32, flags: u32) {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe {
        (backend.table().set_cursor_position)(
            backend.pointer(),
            i32::from_ne_bytes(x.to_ne_bytes()),
            i32::from_ne_bytes(y.to_ne_bytes()),
            flags,
        );
    }
}

pub extern "system" fn show_cursor(this: *mut c_void, show: i32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().show_cursor)(backend.pointer(), show) }
}

pub extern "system" fn set_gamma_ramp(this: *mut c_void, flags: u32, ramp: *const D3DGAMMARAMP) {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_gamma_ramp)(backend.pointer(), 0, flags, ramp.cast()) }
}

pub extern "system" fn get_gamma_ramp(this: *mut c_void, ramp: *mut D3DGAMMARAMP) {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_gamma_ramp)(backend.pointer(), 0, ramp.cast()) }
}

pub extern "system" fn set_transform(
    this: *mut c_void,
    state: u32,
    matrix: *const D3DMATRIX,
) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_transform)(backend.pointer(), state, matrix.cast()) }
}

pub extern "system" fn get_transform(this: *mut c_void, state: u32, matrix: *mut D3DMATRIX) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_transform)(backend.pointer(), state, matrix.cast()) }
}

pub extern "system" fn multiply_transform(
    this: *mut c_void,
    state: u32,
    matrix: *const D3DMATRIX,
) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().multiply_transform)(backend.pointer(), state, matrix.cast()) }
}

pub extern "system" fn set_viewport(this: *mut c_void, viewport: *const D3DVIEWPORT9) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_viewport)(backend.pointer(), viewport.cast()) }
}

pub extern "system" fn get_viewport(this: *mut c_void, viewport: *mut D3DVIEWPORT9) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_viewport)(backend.pointer(), viewport.cast()) }
}

pub extern "system" fn set_material(this: *mut c_void, material: *const D3DMATERIAL9) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_material)(backend.pointer(), material.cast()) }
}

pub extern "system" fn get_material(this: *mut c_void, material: *mut D3DMATERIAL9) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_material)(backend.pointer(), material.cast()) }
}

pub extern "system" fn set_light(this: *mut c_void, index: u32, light: *const D3DLIGHT9) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_light)(backend.pointer(), index, light.cast()) }
}

pub extern "system" fn get_light(this: *mut c_void, index: u32, light: *mut D3DLIGHT9) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_light)(backend.pointer(), index, light.cast()) }
}

pub extern "system" fn light_enable(this: *mut c_void, index: u32, enabled: i32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().light_enable)(backend.pointer(), index, enabled) }
}

pub extern "system" fn get_light_enable(this: *mut c_void, index: u32, enabled: *mut i32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_light_enable)(backend.pointer(), index, enabled) }
}

pub extern "system" fn set_clip_plane(this: *mut c_void, index: u32, plane: *const f32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_clip_plane)(backend.pointer(), index, plane) }
}

pub extern "system" fn get_clip_plane(this: *mut c_void, index: u32, plane: *mut f32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_clip_plane)(backend.pointer(), index, plane) }
}

pub extern "system" fn set_clip_status(this: *mut c_void, status: *const c_void) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_clip_status)(backend.pointer(), status) }
}

pub extern "system" fn get_clip_status(this: *mut c_void, status: *mut c_void) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_clip_status)(backend.pointer(), status) }
}

pub extern "system" fn validate_device(this: *mut c_void, passes: *mut u32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().validate_device)(backend.pointer(), passes) }
}

pub extern "system" fn set_palette_entries(
    this: *mut c_void,
    palette: u32,
    entries: *const c_void,
) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_palette_entries)(backend.pointer(), palette, entries) }
}

pub extern "system" fn get_palette_entries(
    this: *mut c_void,
    palette: u32,
    entries: *mut c_void,
) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_palette_entries)(backend.pointer(), palette, entries) }
}

pub extern "system" fn set_current_texture_palette(this: *mut c_void, palette: u32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().set_current_texture_palette)(backend.pointer(), palette) }
}

pub extern "system" fn get_current_texture_palette(this: *mut c_void, palette: *mut u32) -> i32 {
    // SAFETY: the COM receiver names a live D3D8 device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the device remains live under its caller-owned COM reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    // SAFETY: these scalar arguments share the D3D9 ABI and retain their caller storage.
    unsafe { (backend.table().get_current_texture_palette)(backend.pointer(), palette) }
}

pub extern "system" fn set_texture_stage_state(
    this: *mut c_void,
    stage: u32,
    selector: u32,
    value: u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    if stage >= 8 {
        return D3DERR_INVALIDCALL;
    }
    let (method, mapped) = sampler_state(selector).map_or_else(
        || (backend.table().set_texture_stage_state, selector),
        |sampler| (backend.table().set_sampler_state, sampler),
    );
    // SAFETY: the selected method belongs to this backend and receives scalar arguments.
    unsafe { method(backend.pointer(), stage, mapped, value) }
}

pub extern "system" fn get_texture_stage_state(
    this: *mut c_void,
    stage: u32,
    selector: u32,
    output: *mut u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    if stage >= 8 {
        return D3DERR_INVALIDCALL;
    }
    let (method, mapped) = sampler_state(selector).map_or_else(
        || (backend.table().get_texture_stage_state, selector),
        |sampler| (backend.table().get_sampler_state, sampler),
    );
    // SAFETY: output retains the caller's DWORD storage contract through forwarding.
    unsafe { method(backend.pointer(), stage, mapped, output) }
}

pub extern "system" fn set_render_state(this: *mut c_void, state: u32, value: u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    let (selector, translated) = match state {
        D3DRS8_EDGEANTIALIAS => (D3DRS_ANTIALIASEDLINEENABLE, value),
        D3DRS8_ZBIAS => {
            let Some(translated) = depth_bias(value) else {
                return D3DERR_INVALIDCALL;
            };
            (D3DRS_DEPTHBIAS, translated)
        }
        D3DRS8_SOFTWAREVERTEXPROCESSING => {
            let mut state = object.state();
            if !state.permits_software_vp(value != 0) {
                return D3DERR_INVALIDCALL;
            }
            state.set_extra(SOFTWARE_VP, u32::from(value != 0));
            drop(state);
            return D3D_OK;
        }
        D3DRS8_PATCHSEGMENTS => {
            let segments = f32::from_bits(value);
            if !segments.is_finite() || !(0.0..=1.0).contains(&segments) {
                return D3DERR_INVALIDCALL;
            }
            object.state().set_extra(PATCH_SEGMENTS, value);
            return D3D_OK;
        }
        D3DRS8_LINEPATTERN | D3DRS8_ZVISIBLE => {
            return if value == 0 {
                D3D_OK
            } else {
                D3DERR_INVALIDCALL
            };
        }
        _ => (state, value),
    };
    // SAFETY: translated selectors and values use the backend's render-state ABI.
    let result =
        unsafe { (backend.table().set_render_state)(backend.pointer(), selector, translated) };
    if result >= 0 && state == D3DRS8_ZBIAS {
        object.state().set_extra(Z_BIAS, value);
    }
    result
}

pub extern "system" fn get_render_state(this: *mut c_void, state: u32, output: *mut u32) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: output is writable DWORD storage or null.
    let Some(out) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the receiver remains live under its caller-owned reference.
    let object = unsafe { object(this) };
    let backend = object.backend();
    let value = match state {
        D3DRS8_ZBIAS => Some(object.state().current[Z_BIAS]),
        D3DRS8_LINEPATTERN | D3DRS8_ZVISIBLE => Some(0),
        D3DRS8_SOFTWAREVERTEXPROCESSING => Some(object.state().current[SOFTWARE_VP]),
        D3DRS8_PATCHSEGMENTS => Some(object.state().current[PATCH_SEGMENTS]),
        _ => None,
    };
    if let Some(value) = value {
        out.write(value);
        return D3D_OK;
    }
    let selector = if state == D3DRS8_EDGEANTIALIAS {
        D3DRS_ANTIALIASEDLINEENABLE
    } else {
        state
    };
    // SAFETY: output is validated above and retains its writable DWORD storage.
    unsafe { (backend.table().get_render_state)(backend.pointer(), selector, output) }
}

pub extern "system" fn get_info(
    this: *mut c_void,
    identifier: u32,
    _output: *mut c_void,
    _size: u32,
) -> i32 {
    // SAFETY: the COM receiver names a live device for this call.
    let _api = unsafe { api_scope(this) };
    mtld3d_shared::log_once_warn!(target: "mtld3d::d3d8", "GetInfo: driver diagnostic data is unavailable");
    if identifier < 4 {
        mtld3d_types::E_FAIL
    } else {
        mtld3d_types::S_FALSE
    }
}
