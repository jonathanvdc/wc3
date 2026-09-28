use wc3_mdx::chunks::ModelChunk;
use wc3_mdx::chunks::RawChunk;
use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::materials::{Texture, TextureFlags};
use wc3_mdx::mdl::{MdlRead, MdlWriter};
use wc3_mdx::Model;

#[test]
fn texture_fields_round_trip() {
    let mut texture = Texture::new("Textures\\Footman.blp").unwrap();
    texture.set_replaceable_id(1);
    texture.set_flags(TextureFlags(3));
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_textures(&[texture]);

    let decoded = Model::<wc3_mdx::V1800>::decode(&model.encode().unwrap()).unwrap();
    let texture = &decoded.textures()[0];
    assert_eq!(texture.path(), "Textures\\Footman.blp");
    assert_eq!(texture.replaceable_id(), 1);
    assert!(texture.flags().wrap_width());
    assert!(texture.flags().wrap_height());
}

#[test]
fn texture_path_padding_roundtrips_and_is_cleared_when_replacing_the_path() {
    let mut model = Model::<wc3_mdx::V800>::new();
    let mut data = Texture::new("a.blp").unwrap().encode().unwrap();
    data[260..264].copy_from_slice(&[9, 8, 7, 6]);
    model.push(ModelChunk::from_raw(RawChunk::new(*b"TEXS", data)).unwrap());
    assert_eq!(&model.encode().unwrap()[24 + 260..24 + 264], &[9, 8, 7, 6]);
    let mut textures = model.textures();
    textures[0].set_path("b.blp").unwrap();
    model.set_textures(&textures);
    let bytes = model.encode().unwrap();
    assert_eq!(&bytes[24 + 260..24 + 264], &[0; 4]);
}

#[test]
fn texture_path_uses_all_260_bytes() {
    let path = "a".repeat(259);
    let texture = Texture::new(&path).unwrap();
    assert!(Texture::new(&"a".repeat(260)).is_err());
    let bytes = texture.encode().unwrap();
    assert_eq!(bytes.len(), 268);
    assert_eq!(&bytes[4..263], path.as_bytes());
    assert_eq!(Texture::decode(&bytes).unwrap().path(), path);
    let mut writer = MdlWriter::new(Vec::new());
    writer.write(&texture).unwrap();
    let text = String::from_utf8(writer.finish().unwrap()).unwrap();
    assert_eq!(Texture::parse_mdl(&text).unwrap().encode().unwrap(), bytes);
}
