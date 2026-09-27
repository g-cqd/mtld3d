//! Factory identity, adapter enumeration, and D3D8-to-D3D9 creation wiring.

use core::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};

use mtld3d_d3d8_types::{
    D3DADAPTER_IDENTIFIER8, D3DCAPS8, D3DDISPLAYMODE, D3DPRESENT_PARAMETERS8, Guid, IDirect3D8Vtbl,
    IID_IDIRECT3D8, copy_d3d9_present_parameters, to_d3d8_adapter_identifier, to_d3d8_caps,
    to_d3d9_present_parameters,
};
use mtld3d_shared::{InPtr, InPtrMut, OutPtr};
use mtld3d_types::{
    D3D_OK, D3DERR_INVALIDCALL, D3DFMT_R5G6B5, D3DFMT_X1R5G5B5, D3DFMT_X8R8G8B8, IDirect3D9Vtbl,
};

use crate::{backend::Backend, device::Device8, identity};

const DISPLAY_FORMATS: [u32; 3] = [D3DFMT_X8R8G8B8, D3DFMT_R5G6B5, D3DFMT_X1R5G5B5];

static VTABLE: IDirect3D8Vtbl = IDirect3D8Vtbl {
    query_interface,
    add_ref,
    release,
    register_software_device,
    get_adapter_count,
    get_adapter_identifier,
    get_adapter_mode_count,
    enum_adapter_modes,
    get_adapter_display_mode,
    check_device_type,
    check_device_format,
    check_device_multi_sample_type,
    check_depth_stencil_match,
    get_device_caps,
    get_adapter_monitor,
    create_device,
};

#[repr(C)]
pub struct Direct3D8 {
    vtable: *const IDirect3D8Vtbl,
    references: AtomicU32,
    inner: Box<FactoryInner>,
}

struct FactoryInner {
    backend: Backend<IDirect3D9Vtbl>,
}

impl Direct3D8 {
    pub fn create(backend: Backend<IDirect3D9Vtbl>) -> *mut c_void {
        Box::into_raw(Box::new(Self {
            vtable: &raw const VTABLE,
            references: AtomicU32::new(1),
            inner: Box::new(FactoryInner { backend }),
        }))
        .cast()
    }
}

pub extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    object.references.fetch_add(1, Ordering::Relaxed) + 1
}

pub extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: the caller owns the reference being released.
    let object = unsafe { object(this) };
    let remaining = object.references.fetch_sub(1, Ordering::Release) - 1;
    if remaining == 0 {
        core::sync::atomic::fence(Ordering::Acquire);
        // SAFETY: the final COM reference uniquely owns the factory allocation.
        drop(unsafe { Box::from_raw(this.cast::<Direct3D8>()) });
    }
    remaining
}

const unsafe fn object<'a>(this: *mut c_void) -> InPtr<'a, Direct3D8> {
    // SAFETY: each caller is a factory thunk receiving a live Direct3D8 interface.
    unsafe { InPtr::new(this) }
}

extern "system" fn query_interface(
    this: *mut c_void,
    iid: *const Guid,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: this thunk receives the IUnknown parameters for the factory wrapper.
    unsafe { identity::query(this, iid, output, &IID_IDIRECT3D8, add_ref) }
}

extern "system" fn register_software_device(this: *mut c_void, initialize: *mut c_void) -> i32 {
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    // SAFETY: the backend reference and its typed vtable belong together.
    unsafe {
        (object.inner.backend.table().register_software_device)(
            object.inner.backend.pointer(),
            initialize,
        )
    }
}

extern "system" fn get_adapter_count(this: *mut c_void) -> u32 {
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    // SAFETY: the backend reference and its typed vtable belong together.
    unsafe { (object.inner.backend.table().get_adapter_count)(object.inner.backend.pointer()) }
}

extern "system" fn get_adapter_identifier(
    this: *mut c_void,
    adapter: u32,
    flags: u32,
    output: *mut D3DADAPTER_IDENTIFIER8,
) -> i32 {
    // SAFETY: the COM caller supplies writable output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    let mut identifier = MaybeUninit::uninit();
    // SAFETY: identifier provides the backend's complete output layout.
    let result = unsafe {
        (object.inner.backend.table().get_adapter_identifier)(
            object.inner.backend.pointer(),
            adapter,
            flags,
            identifier.as_mut_ptr(),
        )
    };
    if result >= 0 {
        // SAFETY: successful GetAdapterIdentifier initialized the complete output.
        let identifier = unsafe { identifier.assume_init() };
        output.write(to_d3d8_adapter_identifier(&identifier));
    }
    result
}

extern "system" fn get_adapter_mode_count(this: *mut c_void, adapter: u32) -> u32 {
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    let mut total = 0_u32;
    for format in DISPLAY_FORMATS {
        let Some(next) = total.checked_add(format_mode_count(&object, adapter, format)) else {
            return 0;
        };
        total = next;
    }
    total
}

extern "system" fn enum_adapter_modes(
    this: *mut c_void,
    adapter: u32,
    mut index: u32,
    output: *mut D3DDISPLAYMODE,
) -> i32 {
    if output.is_null() {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    for format in DISPLAY_FORMATS {
        let count = format_mode_count(&object, adapter, format);
        if index < count {
            // SAFETY: both APIs share the display-mode layout; the caller owns output.
            return unsafe {
                (object.inner.backend.table().enum_adapter_modes)(
                    object.inner.backend.pointer(),
                    adapter,
                    format,
                    index,
                    output.cast(),
                )
            };
        }
        index -= count;
    }
    D3DERR_INVALIDCALL
}

fn format_mode_count(object: &Direct3D8, adapter: u32, format: u32) -> u32 {
    // SAFETY: the typed backend is live and rejects unsupported formats or adapters.
    unsafe {
        (object.inner.backend.table().get_adapter_mode_count)(
            object.inner.backend.pointer(),
            adapter,
            format,
        )
    }
}

extern "system" fn get_adapter_display_mode(
    this: *mut c_void,
    adapter: u32,
    output: *mut D3DDISPLAYMODE,
) -> i32 {
    if output.is_null() {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    // SAFETY: D3D8 and D3D9 share the display-mode layout; the caller owns output.
    unsafe {
        (object.inner.backend.table().get_adapter_display_mode)(
            object.inner.backend.pointer(),
            adapter,
            output.cast(),
        )
    }
}

macro_rules! forward_check {
    ($name:ident, ($($argument:ident : $kind:ty),*)) => {
        extern "system" fn $name(this: *mut c_void, $($argument: $kind),*) -> i32 {
            // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
            // SAFETY: scalar arguments and the live backend receiver have the same ABI.
            unsafe { (object.inner.backend.table().$name)(object.inner.backend.pointer(), $($argument),*) }
        }
    };
}
forward_check!(check_device_type, (adapter: u32, kind: u32, display: u32, back_buffer: u32, windowed: i32));
forward_check!(check_device_format, (adapter: u32, kind: u32, display: u32, usage: u32, resource: u32, format: u32));
forward_check!(check_depth_stencil_match, (adapter: u32, kind: u32, display: u32, render_target: u32, depth_stencil: u32));

extern "system" fn check_device_multi_sample_type(
    this: *mut c_void,
    adapter: u32,
    kind: u32,
    format: u32,
    windowed: i32,
    sample: u32,
) -> i32 {
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    // SAFETY: D3D8 has no quality output and D3D9 accepts null for that optional output.
    unsafe {
        (object.inner.backend.table().check_device_multi_sample_type)(
            object.inner.backend.pointer(),
            adapter,
            kind,
            format,
            windowed,
            sample,
            ptr::null_mut(),
        )
    }
}

extern "system" fn get_device_caps(
    this: *mut c_void,
    adapter: u32,
    kind: u32,
    output: *mut D3DCAPS8,
) -> i32 {
    // SAFETY: the COM caller supplies writable output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    let mut caps = MaybeUninit::uninit();
    // SAFETY: caps names writable storage for the backend's full capability structure.
    let result = unsafe {
        (object.inner.backend.table().get_device_caps)(
            object.inner.backend.pointer(),
            adapter,
            kind,
            caps.as_mut_ptr(),
        )
    };
    if result >= 0 {
        // SAFETY: successful GetDeviceCaps initialized its output.
        let caps = unsafe { caps.assume_init() };
        output.write(to_d3d8_caps(&caps));
    }
    result
}

extern "system" fn get_adapter_monitor(this: *mut c_void, adapter: u32) -> *mut c_void {
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    // SAFETY: the live backend receives the identical adapter argument.
    unsafe {
        (object.inner.backend.table().get_adapter_monitor)(object.inner.backend.pointer(), adapter)
    }
}

extern "system" fn create_device(
    this: *mut c_void,
    adapter: u32,
    kind: u32,
    window: usize,
    flags: u32,
    parameters: *mut D3DPRESENT_PARAMETERS8,
    output: *mut *mut c_void,
) -> i32 {
    // SAFETY: the COM caller supplies writable output storage or null.
    let Some(output) = (unsafe { OutPtr::opt(output) }) else {
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: presentation parameters are an exclusive in-out parameter of CreateDevice.
    let Some(mut parameters) =
        (unsafe { InPtrMut::<D3DPRESENT_PARAMETERS8>::opt(parameters.cast()) })
    else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    // SAFETY: the COM receiver is a live Direct3D8 for this call.
    let object = unsafe { object(this) };
    let mut converted = to_d3d9_present_parameters(&parameters);
    let mut backend = ptr::null_mut();
    // SAFETY: converted and backend are valid writable backend ABI storage.
    let result = unsafe {
        (object.inner.backend.table().create_device)(
            object.inner.backend.pointer(),
            adapter,
            kind,
            window as *mut c_void,
            flags,
            (&raw mut converted).cast(),
            &raw mut backend,
        )
    };
    if result < 0 {
        output.write(ptr::null_mut());
        return result;
    }
    // SAFETY: successful CreateDevice returned one owned IDirect3DDevice9 reference.
    let Some(backend) = (unsafe { Backend::adopt(backend) }) else {
        output.write(ptr::null_mut());
        return D3DERR_INVALIDCALL;
    };
    add_ref(this);
    copy_d3d9_present_parameters(&mut parameters, &converted);
    // SAFETY: add_ref above transferred one factory reference into the new wrapper.
    let device = unsafe { Device8::create(backend, this, flags) };
    output.write(device);
    D3D_OK
}
