use wc3_mdx::chunks::ModelChunk;
use wc3_mdx::chunks::RawChunk;
use wc3_mdx::io::{Encodable, Readable};
use wc3_mdx::materials::{Texture, TextureFlags};
use wc3_mdx::Model;

#[test]
fn texture_fields_round_trip() {
    let mut texture = Texture::new("Textures\\Footman.blp").unwrap();
    texture.set_replaceable_id(1);
    texture.set_flags(TextureFlags::from_bits(3));
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_textures(&[texture]);

    let decoded = Model::<wc3_mdx::V1800>::decode(&model.encode().unwrap()).unwrap();
    let texture = &decoded.textures()[0];
    assert_eq!(texture.path(), "Textures\\Footman.blp");
    assert_eq!(texture.replaceable_id(), 1);
    assert!(texture.flags().contains(TextureFlags::WRAP_WIDTH));
    assert!(texture.flags().contains(TextureFlags::WRAP_HEIGHT));
}

#[test]
fn texture_reserved_bytes_are_preserved() {
    let mut model = Model::<wc3_mdx::V800>::new();
    let mut data = Texture::new("a.blp").unwrap().encode().unwrap();
    data[260..264].copy_from_slice(&[9, 8, 7, 6]);
    model.push(ModelChunk::from_raw(RawChunk::new(*b"TEXS", data)).unwrap());
    let mut textures = model.textures();
    textures[0].set_path("b.blp").unwrap();
    model.set_textures(&textures);
    let bytes = model.encode().unwrap();
    assert_eq!(&bytes[24 + 260..24 + 264], &[9, 8, 7, 6]);
}
