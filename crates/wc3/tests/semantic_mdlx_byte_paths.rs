use mdx::{Read as _, Write as _};
use native::emitters::{ParticleEmitter, PopcornEmitter};
use native::geometry::Geoset;
use native::scene::{Attachment, FaceFx, Node};
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::mdlx::{self as native, mdx, FixedText};
use wc3::model::{io, PathResource};

#[test]
fn semantic_roundtrip_preserves_non_utf8_effect_and_attachment_paths() {
    let mut source = native::Document::<native::V1000>::new();
    let path = b"Units\\\xffmodel.mdx";
    let mut particle = ParticleEmitter::new(Node::new("particle", 0).unwrap(), "").unwrap();
    particle.path.set_bytes(path).unwrap();
    source.set_particle_emitters([particle]);
    let mut popcorn = PopcornEmitter::new(Node::new("popcorn", 1).unwrap(), "", "").unwrap();
    popcorn.path.set_bytes(path).unwrap();
    source.set_popcorn_emitters([popcorn]);
    let mut attachment = Attachment::new(Node::new("attachment", 2).unwrap(), "", 7).unwrap();
    // A full native field need not have a trailing NUL.
    let full_path = [0xff; 260];
    attachment.path.set_bytes(&full_path).unwrap();
    source.set_attachments([attachment]);
    let mut face = FaceFx::new("face", "").unwrap();
    face.path.set_bytes(path).unwrap();
    source.try_set_face_fx([face]).unwrap();
    let mut model = io::mdlx::import_owned(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let face = &model.metadata().face_effects[0];
    assert_eq!(face.name, "face");
    let face_resource = face.resource;
    let PathResource::Path(value) = model.effect_resource(face_resource).unwrap();
    assert_eq!(value.as_ref(), path);
    // Resource ownership participates in the ordinary checked graph transaction.
    assert!(model
        .edit(|draft| {
            draft.remove_effect_resource(face_resource);
            Ok(())
        })
        .is_err());
    assert!(model.effect_resource(face_resource).is_some());
    let output = io::mdlx::plan::<native::V1000>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let bytes = output.encode_mdx().unwrap();
    let output = native::Document::<native::V1000>::decode_mdx(&bytes).unwrap();
    assert_eq!(
        &output.particle_emitters().next().unwrap().path.as_bytes()[..path.len()],
        path
    );
    assert_eq!(
        &output.popcorn_emitters().next().unwrap().path.as_bytes()[..path.len()],
        path
    );
    assert_eq!(
        output.attachments().next().unwrap().path.as_bytes(),
        &full_path
    );
    assert_eq!(
        &output
            .try_face_fx()
            .unwrap()
            .next()
            .unwrap()
            .path
            .as_bytes()[..path.len()],
        path
    );
}

#[test]
fn byte_text_capacity_is_checked_without_replacing_invalid_encoding() {
    let mut field = FixedText::<3>::default();
    field.set_bytes(&[0xff, 0xfe, 0xfd]).unwrap();
    assert_eq!(field.as_bytes(), &[0xff, 0xfe, 0xfd]);
    let original = field;
    assert!(field.set_bytes(&[1, 0, 2]).is_err());
    assert_eq!(field, original);
    assert!(field.set_bytes(&[1, 2, 3, 4]).is_err());
    assert_eq!(field, original);
    assert!(field.set_text("abc").is_err());
    field.set_bytes(&[0xff]).unwrap();
    assert_eq!(field.as_bytes(), &[0xff, 0, 0]);
}

#[test]
fn mismatched_native_primitive_tables_are_semantically_invalid() {
    use native::mdl::Read as _;
    let mut source =
        native::Document::<native::V800>::decode_mdl(include_str!("fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let mut bytes = source.geosets().next().unwrap().encode_mdx().unwrap();
    let position = bytes.windows(4).position(|tag| tag == b"PTYP").unwrap();
    assert_eq!(
        u32::from_le_bytes(bytes[position + 4..position + 8].try_into().unwrap()),
        1
    );
    // Add a second type without adding a corresponding count. This remains
    // structurally decodable transport, but lacks a complete primitive declaration.
    bytes[position + 4..position + 8].copy_from_slice(&2u32.to_le_bytes());
    bytes.splice(position + 12..position + 12, 4u32.to_le_bytes());
    let size = u32::from_le_bytes(bytes[..4].try_into().unwrap()) + 4;
    bytes[..4].copy_from_slice(&size.to_le_bytes());
    let geoset = Geoset::<native::V800>::decode_mdx(&bytes).unwrap();
    source.set_geosets([geoset]);
    let error = io::mdlx::import(&source, &ImportOptions::default()).unwrap_err();
    assert!(
        matches!(error, wc3::model::conversion::Error::Requirement { requirement, .. } if requirement.contains("matching lengths"))
    );
}
