//! Checked D3D8 surface copies across CPU-visible and GPU-only resources.

use core::{ffi::c_void, mem::MaybeUninit, ptr};

use mtld3d_core::d3d8::copy_rects::destination_rectangle;
use mtld3d_shared::slice_from_caller;
use mtld3d_types::{
    D3D_OK, D3DERR_INVALIDCALL, D3DLOCK_READONLY, D3DPOOL_DEFAULT, D3DPOOL_SYSTEMMEM, D3DRECT,
    D3DSURFACE_DESC, D3DUSAGE_DEPTHSTENCIL, D3DUSAGE_RENDERTARGET, IDirect3DSurface9Vtbl,
    IDirect3DTexture9Vtbl, POINT,
};

use super::{Device8, api_scope, object, resources::surface_input};

mod locked_surface;
use locked_surface::LockedSurface;

use crate::backend::{self, Backend};

pub extern "system" fn copy_rects(
    this: *mut c_void,
    source: *mut c_void,
    rectangles: *const D3DRECT,
    count: u32,
    destination: *mut c_void,
    points: *const POINT,
) -> i32 {
    // SAFETY: this COM receiver remains a live Device8 for the call.
    let _api = unsafe { api_scope(this) };
    // SAFETY: this typed vtable dispatches a live Device8 receiver.
    let device = unsafe { object(this) };
    // SAFETY: the caller holds its source surface interface for the call.
    let source = match unsafe { acquire_surface(source, &device) } {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: the caller holds its destination surface interface for the call.
    let destination = match unsafe { acquire_surface(destination, &device) } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let source_desc = match description(&source) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let destination_desc = match description(&destination) {
        Ok(value) => value,
        Err(status) => return status,
    };
    if source_desc.format != destination_desc.format
        || (source_desc.usage | destination_desc.usage) & D3DUSAGE_DEPTHSTENCIL != 0
    {
        return D3DERR_INVALIDCALL;
    }
    if count == 0 && rectangles.is_null() && points.is_null() {
        let Some(rectangle) = full_rectangle(&source_desc) else {
            return D3DERR_INVALIDCALL;
        };
        return copy_region(
            &device,
            &source,
            &source_desc,
            &rectangle,
            &destination,
            &destination_desc,
            &POINT { x: 0, y: 0 },
        );
    }
    if count == 0 {
        return D3D_OK;
    }
    if rectangles.is_null() {
        return D3DERR_INVALIDCALL;
    }
    if (count as usize)
        .checked_mul(core::mem::size_of::<D3DRECT>())
        .is_none_or(|bytes| bytes > isize::MAX as usize)
    {
        return D3DERR_INVALIDCALL;
    }
    // SAFETY: the ABI promises count readable source rectangles with checked byte extent.
    let rectangles = unsafe { slice_from_caller(rectangles, count as usize) };
    let points = if points.is_null() {
        None
    } else {
        // SAFETY: the ABI supplies count readable points; their byte extent is smaller than rectangles.
        Some(unsafe { slice_from_caller(points, count as usize) })
    };
    let regions = rectangles.iter().enumerate().map(|(index, rectangle)| {
        let point = points.as_ref().map_or(
            POINT {
                x: rectangle.x1,
                y: rectangle.y1,
            },
            |points| points[index],
        );
        (rectangle, point)
    });
    for (rectangle, point) in regions.clone() {
        if destination_rectangle(rectangle, point, &source_desc, &destination_desc).is_none() {
            return D3DERR_INVALIDCALL;
        }
    }
    for (rectangle, point) in regions {
        let status = copy_region(
            &device,
            &source,
            &source_desc,
            rectangle,
            &destination,
            &destination_desc,
            &point,
        );
        if status < 0 {
            return status;
        }
    }
    D3D_OK
}

/// Acquires a temporary backend surface reference for a copy operation.
///
/// # Safety
/// A non-null input must be a live frontend surface for this call.
unsafe fn acquire_surface(
    surface: *mut c_void,
    device: &Device8,
) -> Result<Backend<IDirect3DSurface9Vtbl>, i32> {
    // SAFETY: the caller supplies a live optional frontend surface interface.
    let pointer = unsafe { surface_input(surface, device) }?;
    if pointer.is_null() {
        return Err(D3DERR_INVALIDCALL);
    }
    // SAFETY: surface_input verified the live surface's backend and parent device.
    let table = unsafe { backend::table::<IDirect3DSurface9Vtbl>(pointer) };
    // SAFETY: the input surface keeps the backend live while this acquires its own reference.
    unsafe { (table.add_ref)(pointer) };
    // SAFETY: AddRef just supplied one owned backend surface reference.
    unsafe { Backend::adopt(pointer) }.ok_or(D3DERR_INVALIDCALL)
}

fn description(surface: &Backend<IDirect3DSurface9Vtbl>) -> Result<D3DSURFACE_DESC, i32> {
    let mut description = MaybeUninit::uninit();
    // SAFETY: the typed backend writes its full descriptor into local storage.
    let status = unsafe { (surface.table().get_desc)(surface.pointer(), description.as_mut_ptr()) };
    if status < 0 {
        return Err(status);
    }
    // SAFETY: successful GetDesc initialized the full descriptor.
    Ok(unsafe { description.assume_init() })
}

fn full_rectangle(description: &D3DSURFACE_DESC) -> Option<D3DRECT> {
    Some(D3DRECT {
        x1: 0,
        y1: 0,
        x2: i32::try_from(description.width).ok()?,
        y2: i32::try_from(description.height).ok()?,
    })
}

fn copy_region(
    device: &Device8,
    source: &Backend<IDirect3DSurface9Vtbl>,
    source_desc: &D3DSURFACE_DESC,
    rectangle: &D3DRECT,
    destination: &Backend<IDirect3DSurface9Vtbl>,
    destination_desc: &D3DSURFACE_DESC,
    point: &POINT,
) -> i32 {
    let Some(destination_rect) =
        destination_rectangle(rectangle, *point, source_desc, destination_desc)
    else {
        return D3DERR_INVALIDCALL;
    };
    if source_desc.pool == D3DPOOL_SYSTEMMEM
        && destination_desc.pool == D3DPOOL_DEFAULT
        && destination_desc.usage & D3DUSAGE_RENDERTARGET == 0
    {
        // SAFETY: owned endpoints and validated regions satisfy the backend UpdateSurface ABI.
        return unsafe {
            (device.backend().table().update_surface)(
                device.backend().pointer(),
                source.pointer(),
                ptr::from_ref(rectangle).cast(),
                destination.pointer(),
                ptr::from_ref(point).cast(),
            )
        };
    }
    if source_desc.pool == D3DPOOL_DEFAULT
        && destination_desc.pool == D3DPOOL_DEFAULT
        && destination_desc.usage & D3DUSAGE_RENDERTARGET != 0
    {
        // SAFETY: owned same-format endpoints and validated equal-sized rectangles remain live.
        return unsafe {
            (device.backend().table().stretch_rect)(
                device.backend().pointer(),
                source.pointer(),
                ptr::from_ref(rectangle).cast(),
                destination.pointer(),
                ptr::from_ref(&destination_rect).cast(),
                0,
            )
        };
    }
    match copy_cpu(
        device,
        source,
        source_desc,
        rectangle,
        destination,
        destination_desc,
        &destination_rect,
    ) {
        Ok(()) => D3D_OK,
        Err(status) => status,
    }
}

fn create_image(
    device: &Device8,
    description: &D3DSURFACE_DESC,
) -> Result<Backend<IDirect3DSurface9Vtbl>, i32> {
    let mut surface = ptr::null_mut();
    // SAFETY: scalar dimensions and format come from a live surface; output is writable local storage.
    let status = unsafe {
        (device.backend().table().create_offscreen_plain_surface)(
            device.backend().pointer(),
            description.width,
            description.height,
            description.format,
            D3DPOOL_SYSTEMMEM,
            &raw mut surface,
            ptr::null_mut(),
        )
    };
    if status < 0 {
        return Err(status);
    }
    // SAFETY: successful creation returns one owned surface interface.
    unsafe { Backend::adopt(surface) }.ok_or(D3DERR_INVALIDCALL)
}

fn readback(
    device: &Device8,
    source: &Backend<IDirect3DSurface9Vtbl>,
    description: &D3DSURFACE_DESC,
) -> Result<Backend<IDirect3DSurface9Vtbl>, i32> {
    let image = create_image(device, description)?;
    // SAFETY: source and matching system-memory destination both remain owned through the copy.
    let status = unsafe {
        (device.backend().table().get_render_target_data)(
            device.backend().pointer(),
            source.pointer(),
            image.pointer(),
        )
    };
    if status < 0 {
        return Err(status);
    }
    Ok(image)
}

fn copy_cpu(
    device: &Device8,
    source: &Backend<IDirect3DSurface9Vtbl>,
    source_desc: &D3DSURFACE_DESC,
    rectangle: &D3DRECT,
    destination: &Backend<IDirect3DSurface9Vtbl>,
    destination_desc: &D3DSURFACE_DESC,
    destination_rect: &D3DRECT,
) -> Result<(), i32> {
    let staging;
    let source = if let Ok(locked) = LockedSurface::new(source, rectangle, D3DLOCK_READONLY) {
        locked
    } else {
        staging = readback(device, source, source_desc)?;
        LockedSurface::new(&staging, rectangle, D3DLOCK_READONLY)?
    };
    if destination_desc.usage & D3DUSAGE_RENDERTARGET == 0
        && let Ok(mut destination) = LockedSurface::new(destination, destination_rect, 0)
    {
        return source.copy_to(&mut destination);
    }
    let staging_destination = create_image(device, destination_desc)?;
    {
        let mut locked = LockedSurface::new(&staging_destination, destination_rect, 0)?;
        source.copy_to(&mut locked)?;
    }
    drop(source);
    upload(
        device,
        &staging_destination,
        destination,
        destination_desc,
        destination_rect,
    )
}

fn upload(
    device: &Device8,
    source: &Backend<IDirect3DSurface9Vtbl>,
    destination: &Backend<IDirect3DSurface9Vtbl>,
    description: &D3DSURFACE_DESC,
    rectangle: &D3DRECT,
) -> Result<(), i32> {
    let point = POINT {
        x: rectangle.x1,
        y: rectangle.y1,
    };
    if description.usage & D3DUSAGE_RENDERTARGET == 0 {
        // SAFETY: owned same-format endpoints and validated source/destination regions remain live.
        let status = unsafe {
            (device.backend().table().update_surface)(
                device.backend().pointer(),
                source.pointer(),
                ptr::from_ref(rectangle).cast(),
                destination.pointer(),
                ptr::from_ref(&point).cast(),
            )
        };
        return if status < 0 { Err(status) } else { Ok(()) };
    }
    let mut texture = ptr::null_mut();
    // SAFETY: scalar dimensions and format come from the target descriptor; output is local storage.
    let status = unsafe {
        (device.backend().table().create_texture)(
            device.backend().pointer(),
            description.width,
            description.height,
            1,
            0,
            description.format,
            D3DPOOL_DEFAULT,
            &raw mut texture,
            ptr::null_mut(),
        )
    };
    if status < 0 {
        return Err(status);
    }
    // SAFETY: successful creation returned one owned 2D texture interface.
    let texture =
        unsafe { Backend::<IDirect3DTexture9Vtbl>::adopt(texture) }.ok_or(D3DERR_INVALIDCALL)?;
    let mut level = ptr::null_mut();
    // SAFETY: the texture has exactly one level and level is writable local output.
    let status =
        unsafe { (texture.table().get_surface_level)(texture.pointer(), 0, &raw mut level) };
    if status < 0 {
        return Err(status);
    }
    // SAFETY: successful GetSurfaceLevel returned one owned surface reference.
    let level =
        unsafe { Backend::<IDirect3DSurface9Vtbl>::adopt(level) }.ok_or(D3DERR_INVALIDCALL)?;
    // SAFETY: owned matching surfaces and validated equal regions satisfy UpdateSurface's ABI.
    let status = unsafe {
        (device.backend().table().update_surface)(
            device.backend().pointer(),
            source.pointer(),
            ptr::from_ref(rectangle).cast(),
            level.pointer(),
            ptr::from_ref(&point).cast(),
        )
    };
    if status < 0 {
        return Err(status);
    }
    // SAFETY: the uploaded texture level and target remain owned; both rectangles are validated.
    let status = unsafe {
        (device.backend().table().stretch_rect)(
            device.backend().pointer(),
            level.pointer(),
            ptr::from_ref(rectangle).cast(),
            destination.pointer(),
            ptr::from_ref(rectangle).cast(),
            0,
        )
    };
    if status < 0 {
        return Err(status);
    }
    Ok(())
}
