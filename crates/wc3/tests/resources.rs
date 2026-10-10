use std::convert::Infallible;
use std::error::Error;
use std::io;
use wc3::model::chunks::{
    AttachmentsChunk, FaceFxChunk, ModelChunk, ModelInfoChunk, ParticleEmitters2Chunk,
    ParticleEmittersChunk, PopcornEmittersChunk, RawChunk, TexturesChunk, UnknownChunk,
};
use wc3::model::emitters::{ParticleEmitter, ParticleEmitter2, PopcornEmitter};
use wc3::model::materials::{Layer, Material, Texture};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::Write as _;
use wc3::model::resources::{
    OwnedResourceValue, ResourceEdit, ResourceField, ResourceLocation, ResourceSite, ResourceValue,
    RewriteError,
};
use wc3::model::scene::{Attachment, FaceFx, ModelInfo, Node};
use wc3::model::ConversionOptions;
use wc3::model::NoExtensions;
use wc3::model::{DynamicModel, FixedText, Model, V1800, V800};

fn fixture() -> Model<V1800> {
    let mut model = Model::new();
    let mut info = ModelInfo::new("resources").unwrap();
    info.animation_file_name
        .set_text("Anims\\External.mdx")
        .unwrap();
    model
        .chunks
        .push(ModelInfoChunk::new(info, vec![9, 0, 7]).into());
    let mut bitmap = Texture::new("Textures\\Shared.blp").unwrap();
    bitmap.replaceable_id = 31;
    model
        .chunks
        .push(TexturesChunk::new(vec![bitmap, Texture::new("").unwrap()]).into());
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk {
            tag: *b"TEST",
            data: b"hidden\\resource.mdx\0".to_vec(),
        })
        .unwrap(),
    ));
    model
        .chunks
        .push(TexturesChunk::new(vec![Texture::new("Textures\\Shared.blp").unwrap()]).into());
    model.chunks.push(
        AttachmentsChunk::new(vec![
            Attachment::new(Node::new("child", 0).unwrap(), "Units/Child.mdx", 0).unwrap(),
            Attachment::new(Node::new("empty", 1).unwrap(), "", 1).unwrap(),
        ])
        .into(),
    );
    model.chunks.push(
        ParticleEmittersChunk::new(vec![
            ParticleEmitter::new(Node::new("model particle", 2).unwrap(), "Effects\\Dust.mdl")
                .unwrap(),
            ParticleEmitter::new(Node::new("image particle", 3).unwrap(), "Effects\\Dust.tga")
                .unwrap(),
        ])
        .into(),
    );
    let mut particle = ParticleEmitter2::new(Node::new("quad", 4).unwrap());
    particle.texture_id = 72;
    let mut replaceable = ParticleEmitter2::new(Node::new("replaceable quad", 5).unwrap());
    replaceable.replaceable_id = 2;
    model
        .chunks
        .push(ParticleEmitters2Chunk::new(vec![particle, replaceable]).into());
    let mut popcorn = PopcornEmitter::new(
        Node::new("popcorn", 6).unwrap(),
        "Effects\\Blood.pkfx",
        "Looks\\LikeAPath.mdx",
    )
    .unwrap();
    popcorn.replaceable_id = 7;
    model
        .chunks
        .push(PopcornEmittersChunk::new(vec![popcorn]).into());
    model
        .chunks
        .push(FaceFxChunk::new(vec![FaceFx::new("face", "Faces\\Unit.facefx").unwrap()]).into());
    let mut material = Material::new();
    let mut layer = Layer::new();
    layer.texture_id.set_value(99);
    material.layers.push(layer);
    model.set_materials(&[material]);
    model
}

fn resource_texts(model: &Model<V1800>) -> Vec<String> {
    model
        .resources()
        .map(|reference| match reference.value {
            ResourceValue::Path(path) => path.text().into_owned(),
            ResourceValue::ReplaceableId(id) => format!("replaceable:{id}"),
        })
        .collect()
}

#[test]
fn enumeration_preserves_occurrences_empty_fields_and_exact_source_order() {
    let model = fixture();
    let references: Vec<_> = model.resources().collect();
    assert_eq!(
        resource_texts(&model),
        [
            "Anims\\External.mdx",
            "Textures\\Shared.blp",
            "replaceable:31",
            "",
            "Textures\\Shared.blp",
            "Units/Child.mdx",
            "",
            "Effects\\Dust.mdl",
            "Effects\\Dust.tga",
            "replaceable:2",
            "Effects\\Blood.pkfx",
            "replaceable:7",
            "Faces\\Unit.facefx",
        ]
    );
    let path = ResourceField::Path;
    let id = ResourceField::ReplaceableId;
    let expected = [
        (1, ResourceSite::ModelInfo, path),
        (2, ResourceSite::Bitmap(0), path),
        (2, ResourceSite::Bitmap(0), id),
        (2, ResourceSite::Bitmap(1), path),
        (4, ResourceSite::Bitmap(0), path),
        (5, ResourceSite::Attachment(0), path),
        (5, ResourceSite::Attachment(1), path),
        (6, ResourceSite::ParticleEmitter(0), path),
        (6, ResourceSite::ParticleEmitter(1), path),
        (7, ResourceSite::ParticleEmitter2(1), id),
        (8, ResourceSite::PopcornEmitter(0), path),
        (8, ResourceSite::PopcornEmitter(0), id),
        (9, ResourceSite::FaceFx(0), path),
    ];
    assert_eq!(
        references
            .iter()
            .map(|reference| {
                let location = reference.location;
                (location.chunk, location.site, location.field)
            })
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        references[0].location.to_string(),
        "chunk[1].animation_file_name"
    );
    assert_eq!(
        references[2].location.to_string(),
        "chunk[2].textures[0].replaceable_id"
    );
    let mut iterator = model.resources();
    while iterator.next().is_some() {}
    assert!(iterator.next().is_none());
    assert!(iterator.next().is_none());
}

#[test]
fn callback_failure_after_proposed_changes_is_atomic_and_keeps_its_error() {
    let mut model = fixture();
    let original = model.encode_mdx().unwrap();
    let error = model
        .rewrite_resources(|reference| {
            if reference.location.site == ResourceSite::Attachment(0) {
                Err(io::Error::other("caller resolution failed"))
            } else {
                Ok(match reference.value {
                    ResourceValue::Path(_) => ResourceEdit::SetPath("changed.mdx".into()),
                    ResourceValue::ReplaceableId(_) => ResourceEdit::SetReplaceableId(42),
                })
            }
        })
        .unwrap_err();
    assert_eq!(
        error.location(),
        ResourceLocation {
            chunk: 5,
            site: ResourceSite::Attachment(0),
            field: ResourceField::Path
        }
    );
    assert!(
        matches!(&error, RewriteError::Callback { error, .. } if error.downcast_ref::<io::Error>().unwrap().kind() == io::ErrorKind::Other)
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "caller resolution failed"
    );
    assert_eq!(model.encode_mdx().unwrap(), original);
}

#[test]
fn invalid_text_and_wrong_edit_kinds_roll_back_every_proposal() {
    for invalid in [
        "x".repeat(260),
        format!("{}é", "x".repeat(258)),
        "bad\0path".into(),
    ] {
        let mut model = fixture();
        let original = model.encode_mdx().unwrap();
        let error = model
            .rewrite_resources(|reference| {
                Ok::<_, Infallible>(match reference.value {
                    ResourceValue::Path(_) => {
                        ResourceEdit::SetPath(if reference.location.chunk == 4 {
                            invalid.clone()
                        } else {
                            "changed.mdx".into()
                        })
                    }
                    ResourceValue::ReplaceableId(_) => ResourceEdit::Keep,
                })
            })
            .unwrap_err();
        assert!(matches!(error, RewriteError::InvalidPath { .. }));
        assert_eq!(error.location().chunk, 4);
        assert_eq!(model.encode_mdx().unwrap(), original);
    }
    for wrong in [
        ResourceEdit::SetPath("bad.blp".into()),
        ResourceEdit::SetReplaceableId(42),
    ] {
        let mut model = fixture();
        let original = model.encode_mdx().unwrap();
        let error = model
            .rewrite_resources(|reference| {
                let mismatch = match (&wrong, reference.value) {
                    (ResourceEdit::SetPath(_), ResourceValue::ReplaceableId(_)) => true,
                    (ResourceEdit::SetReplaceableId(_), ResourceValue::Path(_)) => {
                        reference.location.chunk == 4
                    }
                    _ => false,
                };
                Ok::<_, Infallible>(if mismatch {
                    wrong.clone()
                } else {
                    match reference.value {
                        ResourceValue::Path(_) => ResourceEdit::SetPath("changed.mdx".into()),
                        ResourceValue::ReplaceableId(_) => ResourceEdit::Keep,
                    }
                })
            })
            .unwrap_err();
        assert!(matches!(error, RewriteError::IncompatibleEdit { .. }));
        assert_eq!(model.encode_mdx().unwrap(), original);
    }
}

#[test]
fn noops_retain_nonzero_padding_and_reports_keep_lossless_before_values() {
    let mut model = fixture();
    let ModelChunk::Textures(chunk) = &mut model.chunks[2] else {
        unreachable!()
    };
    let mut bytes = *chunk.records[0].path.as_bytes();
    bytes[259] = 0xff;
    chunk.records[0].path = FixedText::from_bytes(bytes);
    let original = model.encode_mdx().unwrap();
    let report = model
        .rewrite_resources(|reference| {
            Ok::<_, Infallible>(match reference.value {
                ResourceValue::Path(path) => ResourceEdit::SetPath(path.text().into_owned()),
                ResourceValue::ReplaceableId(id) => ResourceEdit::SetReplaceableId(id),
            })
        })
        .unwrap();
    assert!(report.changes.is_empty());
    assert_eq!(model.encode_mdx().unwrap(), original);
    let tags: Vec<_> = model.chunks.iter().map(ModelChunk::tag).collect();
    let report = model
        .rewrite_resources(|reference| {
            Ok::<_, Infallible>(
                if reference.location
                    == (ResourceLocation {
                        chunk: 2,
                        site: ResourceSite::Bitmap(0),
                        field: ResourceField::Path,
                    })
                {
                    ResourceEdit::SetPath("new/path.dds".into())
                } else {
                    ResourceEdit::Keep
                },
            )
        })
        .unwrap();
    assert_eq!(report.changes.len(), 1);
    let change = &report.changes[0];
    let OwnedResourceValue::Path(before) = &change.before else {
        unreachable!()
    };
    assert_eq!(before.as_bytes(), &bytes);
    let OwnedResourceValue::Path(after) = &change.after else {
        unreachable!()
    };
    assert_eq!(after.text(), "new/path.dds");
    assert_eq!(after.as_bytes()[259], 0);
    assert_eq!(
        model.chunks.iter().map(ModelChunk::tag).collect::<Vec<_>>(),
        tags
    );
    assert_eq!(model.textures()[2].path.text(), "Textures\\Shared.blp");
    let ModelChunk::Unknown(opaque) = &model.chunks[3] else {
        unreachable!()
    };
    assert_eq!(opaque.raw().data, b"hidden\\resource.mdx\0");
    let ModelChunk::ModelInfo(info) = &model.chunks[1] else {
        unreachable!()
    };
    assert_eq!(info.extension, [9, 0, 7]);
}

#[test]
fn malformed_source_text_is_kept_unless_explicitly_replaced() {
    let mut model = Model::<V800>::new();
    let mut bitmap = Texture::new("").unwrap();
    let mut bytes = [0; 260];
    bytes[..4].copy_from_slice(&[b'a', 0xff, b'b', 0]);
    bytes[240] = 42;
    bitmap.path = FixedText::from_bytes(bytes);
    model.set_textures(&[bitmap]);
    let before = model.encode_mdx().unwrap();
    let report = model
        .rewrite_resources(|_| Ok::<_, Infallible>(ResourceEdit::Keep))
        .unwrap();
    assert!(report.changes.is_empty());
    assert_eq!(model.encode_mdx().unwrap(), before);
    let report = model
        .rewrite_resources(|reference| {
            let ResourceValue::Path(path) = reference.value else {
                unreachable!()
            };
            Ok::<_, Infallible>(ResourceEdit::SetPath(path.text().into_owned()))
        })
        .unwrap();
    assert_eq!(report.changes.len(), 1);
    assert_eq!(model.textures()[0].path.text(), "a�b");
    assert_ne!(model.textures()[0].path.as_bytes(), &bytes);
}

#[test]
fn path_and_replaceable_id_are_independent_and_can_be_cleared() {
    let mut model = Model::<V800>::new();
    let mut texture = Texture::new("body.blp").unwrap();
    texture.replaceable_id = 31;
    model.set_textures(&[texture]);
    let report = model
        .rewrite_resources(|reference| {
            Ok::<_, Infallible>(match reference.value {
                ResourceValue::Path(_) => ResourceEdit::Keep,
                ResourceValue::ReplaceableId(_) => ResourceEdit::SetReplaceableId(0),
            })
        })
        .unwrap();
    assert_eq!(report.changes.len(), 1);
    assert_eq!(model.textures()[0].path.text(), "body.blp");
    assert_eq!(model.resources().count(), 1);
    let report = model
        .rewrite_resources(|_| Ok::<_, Infallible>(ResourceEdit::SetPath(String::new())))
        .unwrap();
    assert_eq!(report.changes.len(), 1);
    assert_eq!(model.resources().count(), 1);
    assert!(model.textures()[0].path.text().is_empty());
    let maximum = format!("{}é", "a".repeat(257));
    model
        .rewrite_resources(|_| Ok::<_, Infallible>(ResourceEdit::SetPath(maximum.clone())))
        .unwrap();
    assert_eq!(model.textures()[0].path.text(), maximum);
    assert_eq!(model.textures()[0].path.as_bytes()[259], 0);
}

#[test]
fn dynamic_models_keep_source_versions_and_duplicate_chunks_for_every_layout() {
    for version in [800, 900, 1000, 1100, 1200, 1300, 1400, 1600, 1800] {
        let text = format!("Version {{ FormatVersion {version}, }} Model \"resources\" {{}} Textures 1 {{ Bitmap {{ Image \"old.blp\", }} }}");
        let mut model = DynamicModel::<NoExtensions>::decode_mdl(&text).unwrap();
        let report = model
            .rewrite_resources(|reference| {
                Ok::<_, Infallible>(match reference.value {
                    ResourceValue::Path(path) if path.text() == "old.blp" => {
                        ResourceEdit::SetPath("new.blp".into())
                    }
                    _ => ResourceEdit::Keep,
                })
            })
            .unwrap();
        assert_eq!(report.changes.len(), 1);
        let binary = model.encode_mdx().unwrap();
        let decoded = DynamicModel::<NoExtensions>::decode_mdx(&binary, 800).unwrap();
        assert_eq!(decoded.version(), version);
        assert!(decoded.resources().any(|reference| matches!(reference.value, ResourceValue::Path(path) if path.text() == "new.blp")));
        let mdl = model.encode_mdl().unwrap();
        let decoded = DynamicModel::<NoExtensions>::decode_mdl(&mdl).unwrap();
        assert_eq!(decoded.version(), version);
        assert!(decoded.resources().any(|reference| matches!(reference.value, ResourceValue::Path(path) if path.text() == "new.blp")));
    }
    let typed = fixture();
    let mut dynamic = DynamicModel::<NoExtensions>::V1800(
        typed
            .clone()
            .convert(&ConversionOptions::strict())
            .unwrap()
            .model,
    );
    assert_eq!(
        typed.resources().collect::<Vec<_>>(),
        dynamic.resources().collect::<Vec<_>>()
    );
    dynamic
        .rewrite_resources(|reference| {
            Ok::<_, Infallible>(match reference.value {
                ResourceValue::Path(path) if path.text() == "Textures\\Shared.blp" => {
                    ResourceEdit::SetPath("both.blp".into())
                }
                _ => ResourceEdit::Keep,
            })
        })
        .unwrap();
    assert_eq!(dynamic.resources().filter(|reference| matches!(reference.value, ResourceValue::Path(path) if path.text() == "both.blp")).count(), 2);
}

#[test]
fn empty_models_and_collections_terminate_without_fabricating_references() {
    let mut model = Model::<V800>::new();
    model.chunks.push(TexturesChunk::new(Vec::new()).into());
    model.chunks.push(AttachmentsChunk::new(Vec::new()).into());
    assert_eq!(model.resources().count(), 0);
    let report = model
        .rewrite_resources(|_| -> Result<ResourceEdit, Infallible> { panic!("no resources") })
        .unwrap();
    assert!(report.changes.is_empty());
}

#[test]
fn every_resource_site_can_be_rewritten_without_reenumerating_cleared_ids() {
    let mut model = fixture();
    let original: Vec<_> = model
        .resources()
        .map(|reference| {
            (
                reference.location,
                match reference.value {
                    ResourceValue::Path(path) => Some(format!(
                        "mapped/{}/{}",
                        reference.location.chunk,
                        path.text()
                    )),
                    ResourceValue::ReplaceableId(_) => None,
                },
            )
        })
        .collect();
    let report = model
        .rewrite_resources(|reference| {
            Ok::<_, Infallible>(match reference.value {
                ResourceValue::Path(path) => ResourceEdit::SetPath(format!(
                    "mapped/{}/{}",
                    reference.location.chunk,
                    path.text()
                )),
                ResourceValue::ReplaceableId(_) => ResourceEdit::SetReplaceableId(0),
            })
        })
        .unwrap();
    assert_eq!(
        report
            .changes
            .iter()
            .map(|change| change.location)
            .collect::<Vec<_>>(),
        original
            .iter()
            .map(|(location, _)| *location)
            .collect::<Vec<_>>()
    );
    let expected: Vec<_> = original
        .into_iter()
        .filter_map(|(location, value)| value.map(|value| (location, value)))
        .collect();
    let actual: Vec<_> = model
        .resources()
        .map(|reference| {
            let ResourceValue::Path(path) = reference.value else {
                panic!("ID was not cleared")
            };
            (reference.location, path.text().into_owned())
        })
        .collect();
    assert_eq!(actual, expected);
    let decoded =
        DynamicModel::<NoExtensions>::decode_mdx(&model.encode_mdx().unwrap(), 800).unwrap();
    assert_eq!(decoded.resources().count(), actual.len());
}
