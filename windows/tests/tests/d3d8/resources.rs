//! D3D8 buffer and non-2D texture resource behavior.

use mtld3d_tests::D3D8Harness;
use mtld3d_types::{
    D3DERR_INVALIDCALL, D3DFMT_INDEX16, D3DFMT_VERTEXDATA, D3DFVF_XYZ, D3DPOOL_MANAGED,
};

#[test]
fn buffers_preserve_bytes_bindings_and_unsigned_base_vertex() {
    let sut = D3D8Harness::new();
    let vertex = sut.create_vertex_buffer8(64, D3DFVF_XYZ);
    let index = sut.create_index_buffer8(16);
    let vertex_desc = vertex.desc();
    assert_eq!(
        (
            vertex_desc.size,
            vertex_desc.format,
            vertex_desc.fvf,
            vertex_desc.pool
        ),
        (64, D3DFMT_VERTEXDATA, D3DFVF_XYZ, D3DPOOL_MANAGED)
    );
    let index_desc = index.desc();
    assert_eq!(
        (index_desc.size, index_desc.format, index_desc.pool),
        (16, D3DFMT_INDEX16, D3DPOOL_MANAGED)
    );
    vertex.write(7, &[3, 1, 4, 1, 5, 9]);
    index.write(2, &[0, 0, 1, 0, 2, 0]);
    let mut bytes = [0; 6];
    vertex.read(7, &mut bytes);
    assert_eq!(bytes, [3, 1, 4, 1, 5, 9]);
    index.read(2, &mut bytes);
    assert_eq!(bytes, [0, 0, 1, 0, 2, 0]);
    assert_eq!(sut.set_vertex_buffer8(Some(&vertex), 12), 0);
    assert_eq!(sut.vertex_buffer_binding8(Some(&vertex)), (true, 12));
    assert_eq!(sut.set_index_buffer8(Some(&index), u32::MAX - 1), 0);
    assert_eq!(
        sut.index_buffer_binding8(Some(&index)),
        (true, u32::MAX - 1)
    );
    assert_eq!(sut.set_index_buffer8(None, 9), 0);
    assert_eq!(sut.index_buffer_binding8(None), (true, 9));
    assert_eq!(sut.set_vertex_buffer8(None, 0), 0);
    assert!(sut.vertex_buffer_binding8(None).0);
}

#[test]
fn cube_faces_have_distinct_identity_and_survive_the_texture_reference() {
    let sut = D3D8Harness::new();
    let cube = sut.create_cube_texture8(8);
    assert_eq!(cube.level_count(), 4);
    for level in 0..4 {
        let description = cube.level_desc(level);
        let edge = 8 >> level;
        assert_eq!(
            (description.width, description.height, description.size),
            (edge, edge, edge * edge * 4)
        );
    }
    let face = cube.face(0, 1).expect("positive X mip one");
    let repeated = cube.face(0, 1).expect("same face and mip");
    let other = cube.face(1, 1).expect("negative X mip one");
    assert!(face.is_same_object(&repeated));
    assert!(!face.is_same_object(&other));
    assert!(matches!(cube.face(6, 0), Err(D3DERR_INVALIDCALL)));
    assert!(matches!(cube.face(0, 4), Err(D3DERR_INVALIDCALL)));
    drop(cube);
    assert_eq!(face.desc().size, 4 * 4 * 4);
}

#[test]
fn volume_levels_retain_their_container_and_map_the_correct_storage() {
    let sut = D3D8Harness::new();
    let texture = sut.create_volume_texture8(8, 4, 2);
    assert_eq!(texture.level_count(), 4);
    for level in 0..4 {
        let description = texture.level_desc(level);
        let width = (8 >> level).max(1);
        let height = (4 >> level).max(1);
        let depth = (2 >> level).max(1);
        assert_eq!(
            (
                description.width,
                description.height,
                description.depth,
                description.size
            ),
            (width, height, depth, width * height * depth * 4)
        );
    }
    let level = texture.level(1).expect("volume mip one");
    assert!(level.is_same_object(&texture.level(1).expect("same volume mip")));
    assert!(level.container_matches(&texture));
    assert!(matches!(texture.level(4), Err(D3DERR_INVALIDCALL)));
    level.write_first_texel(0xA134_56EF);
    assert_eq!(level.read_first_texel(), 0xA134_56EF);
    drop(texture);
    assert_eq!(level.read_first_texel(), 0xA134_56EF);
    let description = level.desc();
    assert_eq!(
        (description.width, description.height, description.depth),
        (4, 2, 1)
    );
}

#[test]
fn an_additional_chain_without_a_buffer_reference_blocks_reset_until_release() {
    let sut = D3D8Harness::new();
    let chain = sut.create_swap_chain(32, 16);
    assert_eq!(sut.reset(32, 48), mtld3d_types::D3DERR_DEVICELOST);
    assert_eq!(sut.cooperative_level(), mtld3d_types::D3DERR_DEVICENOTRESET);
    drop(chain);
    assert_eq!(sut.reset(32, 48), 0);
    assert_eq!(sut.cooperative_level(), 0);
}
