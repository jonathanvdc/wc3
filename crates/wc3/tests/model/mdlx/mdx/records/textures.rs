use wc3::model::formats::mdlx::chunks::ModelChunk;
use wc3::model::formats::mdlx::chunks::RawChunk;
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;

use wc3::model::formats::mdlx::materials::Texture;
use wc3::model::formats::mdlx::mdl::Writer;
use wc3::model::formats::mdlx::Document;

#[test]
fn texture_path_padding_roundtrips_and_is_cleared_when_replacing_the_path() {
    let mut model = Document::<wc3::model::formats::mdlx::V800>::new();
    let mut data = Texture::new("a.blp").unwrap().encode_mdx().unwrap();
    data[260..264].copy_from_slice(&[9, 8, 7, 6]);
    model
        .chunks
        .push(ModelChunk::from_raw(RawChunk::new(*b"TEXS", data)).unwrap());
    assert_eq!(
        &model.encode_mdx().unwrap()[24 + 260..24 + 264],
        &[9, 8, 7, 6]
    );
    let mut textures = model.textures().cloned().collect::<Vec<_>>();
    textures[0].path.set_text("b.blp").unwrap();
    model.set_textures(textures);
    let bytes = model.encode_mdx().unwrap();
    assert_eq!(&bytes[24 + 260..24 + 264], &[0; 4]);
}

#[test]
fn texture_path_uses_all_260_bytes() {
    let path = "a".repeat(259);
    let texture = Texture::new(&path).unwrap();
    assert!(Texture::new(&"a".repeat(260)).is_err());
    let bytes = texture.encode_mdx().unwrap();
    assert_eq!(bytes.len(), 268);
    assert_eq!(&bytes[4..263], path.as_bytes());
    assert_eq!(Texture::decode_mdx(&bytes).unwrap().path.text(), path);
    let mut writer = Writer::new(Vec::new());
    writer.write(&texture).unwrap();
    let text = String::from_utf8(writer.finish().unwrap()).unwrap();
    assert_eq!(
        Texture::decode_mdl(&text).unwrap().encode_mdx().unwrap(),
        bytes
    );
}
