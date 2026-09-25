use wc3_mdx::{Chunk, Error, Model, Texture, TextureFlags};

#[test]
fn texture_fields_round_trip() {
    let mut texture = Texture::new("Textures\\Footman.blp").unwrap();
    texture.set_replaceable_id(1);
    texture.set_flags(TextureFlags::from_bits(3));
    let mut model = Model::new(1800);
    model.set_textures(&[texture]).unwrap();

    let decoded = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let texture = &decoded.textures().unwrap()[0];
    assert_eq!(texture.path(), "Textures\\Footman.blp");
    assert_eq!(texture.replaceable_id(), 1);
    assert!(texture.flags().contains(TextureFlags::WRAP_WIDTH));
    assert!(texture.flags().contains(TextureFlags::WRAP_HEIGHT));
}

#[test]
fn texture_reserved_bytes_are_preserved() {
    let mut model = Model::new(800);
    model
        .set_textures(&[Texture::new("a.blp").unwrap()])
        .unwrap();
    model.chunk_mut(*b"TEXS").unwrap().data[260..264].copy_from_slice(&[9, 8, 7, 6]);
    let mut textures = model.textures().unwrap();
    textures[0].set_path("b.blp").unwrap();
    model.set_textures(&textures).unwrap();
    assert_eq!(
        &model.chunk(*b"TEXS").unwrap().data[260..264],
        &[9, 8, 7, 6]
    );
}

#[test]
fn malformed_texture_chunk_is_reported() {
    let mut model = Model::new(800);
    model.push(Chunk::new(*b"TEXS", vec![0; 267]));
    assert_eq!(
        model.textures(),
        Err(Error::MalformedChunk {
            tag: *b"TEXS",
            size: 267,
            expected: 268,
        })
    );
}
