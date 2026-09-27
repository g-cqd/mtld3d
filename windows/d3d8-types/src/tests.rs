use super::{D3DPRESENT_PARAMETERS8, to_d3d9_present_parameters};

#[test]
fn shader_caps_preserve_supported_constants_within_d3d8_limits() {
    // SAFETY: the ABI descriptor consists entirely of integer and floating-point fields.
    let mut backend: mtld3d_types::D3DCAPS9 = unsafe { core::mem::zeroed() };
    backend.vertex_shader_version = 0xFFFE_0300;
    backend.pixel_shader_version = 0xFFFF_0300;
    for (available, expected) in [(0, 0), (96, 96), (256, 256), (512, 256)] {
        backend.max_vertex_shader_const = available;
        let reported = super::to_d3d8_caps(&backend);
        assert_eq!(reported.vertex_shader_version, 0xFFFE_0101);
        assert_eq!(reported.pixel_shader_version, 0xFFFF_0104);
        assert_eq!(reported.max_vertex_shader_const, expected);
    }
}

#[test]
fn presentation_parameters_preserve_d3d8_fields() {
    let parameters = D3DPRESENT_PARAMETERS8 {
        back_buffer_width: 640,
        back_buffer_height: 480,
        back_buffer_format: 21,
        back_buffer_count: 1,
        multi_sample_type: 0,
        swap_effect: 1,
        device_window: 0x1234,
        windowed: 1,
        enable_auto_depth_stencil: 1,
        auto_depth_stencil_format: 75,
        flags: 1,
        full_screen_refresh_rate_in_hz: 60,
        full_screen_presentation_interval: 0x8000_0000,
    };
    let converted = to_d3d9_present_parameters(&parameters);
    assert_eq!(converted.back_buffer_width, parameters.back_buffer_width);
    assert_eq!(converted.back_buffer_height, parameters.back_buffer_height);
    assert_eq!(converted.back_buffer_format, parameters.back_buffer_format);
    assert_eq!(converted.back_buffer_count, parameters.back_buffer_count);
    assert_eq!(converted.multi_sample_type, parameters.multi_sample_type);
    assert_eq!(converted.multi_sample_quality, 0);
    assert_eq!(converted.swap_effect, parameters.swap_effect);
    assert_eq!(converted.device_window, parameters.device_window);
    assert_eq!(converted.windowed, parameters.windowed);
    assert_eq!(
        converted.enable_auto_depth_stencil,
        parameters.enable_auto_depth_stencil
    );
    assert_eq!(
        converted.auto_depth_stencil_format,
        parameters.auto_depth_stencil_format
    );
    assert_eq!(converted.flags, parameters.flags);
    assert_eq!(
        converted.full_screen_refresh_rate_in_hz,
        parameters.full_screen_refresh_rate_in_hz
    );
    assert_eq!(
        converted.presentation_interval,
        parameters.full_screen_presentation_interval
    );
}

#[test]
fn interfaces_and_descriptors_match_the_d3d8_abi() {
    use core::mem::{offset_of, size_of};

    use super::{
        D3DADAPTER_IDENTIFIER8, D3DCAPS8, D3DSURFACE_DESC8, IDirect3D8Vtbl, IDirect3DDevice8Vtbl,
        IDirect3DSurface8Vtbl,
    };

    assert_eq!(size_of::<IDirect3D8Vtbl>(), 16 * size_of::<usize>());
    assert_eq!(size_of::<IDirect3DDevice8Vtbl>(), 97 * size_of::<usize>());
    assert_eq!(size_of::<IDirect3DSurface8Vtbl>(), 11 * size_of::<usize>());
    assert_eq!(size_of::<D3DCAPS8>(), 212);
    assert_eq!(size_of::<D3DSURFACE_DESC8>(), 32);
    assert_eq!(offset_of!(D3DADAPTER_IDENTIFIER8, driver_version), 1024);
    assert_eq!(offset_of!(D3DADAPTER_IDENTIFIER8, whql_level), 1064);
    assert_eq!(
        size_of::<D3DADAPTER_IDENTIFIER8>(),
        if size_of::<usize>() == 8 { 1072 } else { 1068 }
    );
    assert_eq!(
        size_of::<D3DPRESENT_PARAMETERS8>(),
        if size_of::<usize>() == 8 { 56 } else { 52 }
    );
    assert_eq!(
        offset_of!(IDirect3DDevice8Vtbl, reset),
        14 * size_of::<usize>()
    );
    assert_eq!(
        offset_of!(IDirect3DDevice8Vtbl, set_vertex_shader),
        76 * size_of::<usize>()
    );
}

#[test]
fn resources_keep_the_d3d8_method_order_and_descriptor_size() {
    use core::mem::{offset_of, size_of};

    use super::{
        D3DVOLUME_DESC8, IDirect3DBaseTexture8Vtbl, IDirect3DCubeTexture8Vtbl,
        IDirect3DIndexBuffer8Vtbl, IDirect3DResource8Vtbl, IDirect3DSwapChain8Vtbl,
        IDirect3DTexture8Vtbl, IDirect3DVertexBuffer8Vtbl, IDirect3DVolume8Vtbl,
        IDirect3DVolumeTexture8Vtbl,
    };

    let pointer = size_of::<usize>();
    for (name, slots, bytes) in [
        ("resource", 11, size_of::<IDirect3DResource8Vtbl>()),
        ("base texture", 14, size_of::<IDirect3DBaseTexture8Vtbl>()),
        ("texture", 19, size_of::<IDirect3DTexture8Vtbl>()),
        ("cube texture", 19, size_of::<IDirect3DCubeTexture8Vtbl>()),
        (
            "volume texture",
            19,
            size_of::<IDirect3DVolumeTexture8Vtbl>(),
        ),
        ("vertex buffer", 14, size_of::<IDirect3DVertexBuffer8Vtbl>()),
        ("index buffer", 14, size_of::<IDirect3DIndexBuffer8Vtbl>()),
        ("volume", 11, size_of::<IDirect3DVolume8Vtbl>()),
        ("swap chain", 5, size_of::<IDirect3DSwapChain8Vtbl>()),
    ] {
        assert_eq!(bytes, slots * pointer, "{name}");
    }
    assert_eq!(
        offset_of!(IDirect3DTexture8Vtbl, get_level_desc),
        14 * pointer
    );
    assert_eq!(
        offset_of!(IDirect3DCubeTexture8Vtbl, get_cube_map_surface),
        15 * pointer
    );
    assert_eq!(
        offset_of!(IDirect3DVolumeTexture8Vtbl, lock_box),
        16 * pointer
    );
    assert_eq!(offset_of!(IDirect3DVertexBuffer8Vtbl, lock), 11 * pointer);
    assert_eq!(
        offset_of!(IDirect3DIndexBuffer8Vtbl, get_desc),
        13 * pointer
    );
    assert_eq!(offset_of!(IDirect3DVolume8Vtbl, get_container), 7 * pointer);
    assert_eq!(offset_of!(IDirect3DSwapChain8Vtbl, present), 3 * pointer);
    assert_eq!(size_of::<D3DVOLUME_DESC8>(), 32);
    assert_eq!(offset_of!(D3DVOLUME_DESC8, size), 16);
    assert_eq!(offset_of!(D3DVOLUME_DESC8, depth), 28);
}
