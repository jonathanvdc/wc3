use super::*;
use crate::assembly::{Atlas, PixelRect, RgbaImage, TextureLayer, TextureReplacement};
use crate::wow::ModelPart as EquipmentPart;
use std::collections::HashMap;
use wc3::model::controls::model::BoneControls;
use wc3::model::materials::bindings::ImageSelection;
use wc3::model::materials::model_shader::{ShaderPass, TextureUnit};
use wc3::model::{
    Animated, Draw, GeometryData, GeometryInfluences, IdentityBinding, Influence, Material,
    MaterialSurface, Mesh, MeshInstance, Node, NodeControls, Skin, Texture, UniversalBinding,
    ValueGroups,
};
use wc3::model::{ImageResource, ResourceLocator};

fn model(name: &str, geosets: &[u16]) -> (Model<'static>, NodeId) {
    let mut builder = Model::builder(name.to_owned());
    let joints: Vec<_> = (0..8)
        .map(|i| {
            builder
                .add_node(Node::named(format!("matching bone{i}")).with_controls(Some(
                    NodeControls::M2(BoneControls {
                        name_crc: 100 + i,
                        ..Default::default()
                    }),
                )))
                .unwrap()
        })
        .collect();
    let joint = joints[0];
    let skin = builder
        .add_skin(Skin::new(
            joints,
            UniversalBinding::Identity(IdentityBinding),
        ))
        .unwrap();
    let mut texture = Texture::new(
        builder.add_image_resource(wc3::model::ImageResource::Path("unused".as_bytes().into())),
    );
    texture.resource = builder.add_image_resource(ImageResource::Replaceable(1));
    let texture = builder.add_texture(texture).unwrap();
    for geoset in geosets {
        let mut geometry =
            GeometryData::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]])
                .unwrap();
        geometry.influences =
            GeometryInfluences::Variable(ValueGroups::from_rows((0..3).map(|_| {
                (0..8).map(|i| Influence {
                    joint: i,
                    weight: 0.125,
                })
            })));
        let mut mesh = Mesh::new(geometry);
        let subset = mesh.add_primary_subset(0, 1);
        let mesh = builder.add_mesh(mesh).unwrap();
        let material = builder
            .add_material(Material {
                name: "M2".into(),
                surface: MaterialSurface::ModelShader(Box::new(ShaderPass {
                    geoset: 17,
                    textures: vec![TextureUnit {
                        image: ImageSelection {
                            texture: Some(texture),
                            ..Default::default()
                        },
                        contribution: Animated::constant(1.),
                        ..Default::default()
                    }],
                    color: Animated::constant([1.; 3]),
                    opacity: Animated::constant(1.),
                    ..Default::default()
                })),
            })
            .unwrap();
        let mut instance = MeshInstance::new(joint, mesh);
        instance.skin = Some(skin);
        instance.selection_group = u32::from(*geoset);
        instance.draws.push(Draw::new(subset, Default::default()));
        builder
            .add_instance_with_materials(instance, vec![material])
            .unwrap();
    }
    (builder.finish().unwrap(), joint)
}
fn definition() -> CharacterDefinition {
    CharacterDefinition {
        model: ResourceLocator::FileDataId(1),
        geosets: GeosetSelection {
            visible: [401].into_iter().collect(),
        },
        textures: BTreeMap::new(),
        parts: vec![EquipmentPart {
            model: ResourceLocator::FileDataId(2),
            binding: EquipmentBinding::SharedSkeleton,
            joints: BTreeMap::new(),
            retain_unmatched_joints: false,
            clips: BTreeMap::new(),
            geosets: None,
            textures: BTreeMap::new(),
        }],
    }
}
#[test]
fn assembly_selects_geosets_and_reuses_character_joint_for_armor() {
    let (body, joint) = model("body", &[0, 401]);
    let (armor, _) = model("armor", &[2201]);
    let models = HashMap::from([
        (ResourceLocator::FileDataId(1), body),
        (ResourceLocator::FileDataId(2), armor),
    ]);
    let mut definition = definition();
    definition.textures.insert(
        1,
        TextureReplacement::Atlas {
            atlas: Atlas {
                width: 1,
                height: 1,
                clear: [0; 4],
            },
            layers: vec![TextureLayer {
                resource: ImageResource::FileDataId(3),
                source: None,
                destination: PixelRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                opacity: 255,
            }],
        },
    );
    definition.parts[0]
        .textures
        .insert(1, TextureReplacement::Image(ImageResource::FileDataId(3)));
    let mut images = Assets::default();
    let compiled = compile_character(&definition, &models, &mut images, |_| {
        Ok(RgbaImage::new(1, 1, vec![200, 100, 50, 255]).unwrap())
    })
    .unwrap();
    assert_eq!(compiled.draws.len(), 2);
    for draw in &compiled.draws {
        assert!(draw.mesh.attribute(EXTRA_JOINT_WEIGHT).is_some());
        let texture = draw.material.base.base_color_texture.as_ref().unwrap();
        assert_eq!(
            images.get(texture).unwrap().data.as_deref(),
            Some(&[200, 100, 50, 255][..])
        );
    }
    for skin in compiled.skins {
        assert_eq!(compiled.nodes[skin.joints[0]].id, joint);
    }
}
#[test]
fn unresolved_armor_cannot_silently_produce_a_partial_character() {
    let (body, _) = model("body", &[401]);
    let models = HashMap::from([(ResourceLocator::FileDataId(1), body)]);
    let result = compile_character(&definition(), &models, &mut Assets::default(), |_| {
        Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
    });
    assert!(result.err().unwrap().0.contains("missing assembly model"));
}
#[test]
fn repeated_atlas_sources_are_decoded_once() {
    let mut calls = 0;
    let mut images = Assets::default();
    let layer = TextureLayer {
        resource: ImageResource::FileDataId(3),
        source: None,
        destination: PixelRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        },
        opacity: 255,
    };
    let replacement = TextureReplacement::Atlas {
        atlas: Atlas {
            width: 1,
            height: 1,
            clear: [0; 4],
        },
        layers: vec![layer.clone(), layer],
    };
    let (body, _) = model("body", &[401]);
    let models = HashMap::from([(ResourceLocator::FileDataId(1), body)]);
    let mut definition = definition();
    definition.parts.clear();
    definition.textures.insert(1, replacement);
    let compiled = compile_character(&definition, &models, &mut images, |_| {
        calls += 1;
        Ok(RgbaImage::new(1, 1, vec![255, 0, 0, 255]).unwrap())
    })
    .unwrap();
    let handle = compiled.draws[0]
        .material
        .base
        .base_color_texture
        .as_ref()
        .unwrap();
    assert_eq!(
        images.get(handle).unwrap().data.as_deref(),
        Some(&[255, 0, 0, 255][..])
    );
    assert_eq!(calls, 1);
}

#[test]
fn hidden_hair_texture_is_optional_but_selected_missing_texture_is_an_error() {
    let (mut body, _) = model("body", &[0, 401]);
    let hidden_materials: Vec<_> = body
        .instances()
        .filter(|(_, instance)| instance.selection_group == 0)
        .flat_map(|(id, instance)| instance.draws.iter().map(move |draw| (id, draw.id())))
        .map(|(instance, draw)| {
            let id = body.material_for_draw(instance, draw, None).unwrap();
            (id, body.material(id).unwrap().clone())
        })
        .collect();
    body.edit(|draft| {
        let resource = draft.add_image_resource(ImageResource::Replaceable(6));
        let texture = Texture::new(resource);
        let texture = draft.add_texture(texture)?;
        for (id, mut material) in hidden_materials {
            if let MaterialSurface::ModelShader(surface) = &mut material.surface {
                surface.textures[0].image.texture = Some(texture);
            }
            draft.set_material(id, material)?;
        }
        Ok(())
    })
    .unwrap();
    let models = HashMap::from([(ResourceLocator::FileDataId(1), body)]);
    let mut definition = definition();
    definition.parts.clear();
    let mut requested_hair = false;
    compile_character(&definition, &models, &mut Assets::default(), |resource| {
        if *resource == ImageResource::Replaceable(6) {
            requested_hair = true;
            return Err(ModelError("missing selected hair texture".into()));
        }
        Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
    })
    .unwrap();
    assert!(!requested_hair);
    definition.geosets.visible = [0].into_iter().collect();
    let result = compile_character(&definition, &models, &mut Assets::default(), |resource| {
        if *resource == ImageResource::Replaceable(6) {
            return Err(ModelError("missing selected hair texture".into()));
        }
        Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
    });
    assert!(result
        .err()
        .unwrap()
        .0
        .contains("missing selected hair texture"));
}

#[test]
fn collection_shared_by_chest_and_wrists_compiles_one_rig_with_union_of_geosets() {
    let (body, joint) = model("body", &[401]);
    let (mut collection, armor_joint) = model("collection", &[2201, 801]);
    // Retained helpers/anchors make duplicate source scopes a real collision,
    // even when the visible subsets and their body-joint matches are disjoint.
    collection
        .edit(|builder| {
            builder.add_node(Node::named("collection helper").with_parent(armor_joint))?;
            Ok(())
        })
        .unwrap();
    let models = HashMap::from([
        (ResourceLocator::FileDataId(1), body),
        (ResourceLocator::FileDataId(2), collection),
    ]);
    let mut definition = definition();
    definition.parts[0].geosets = Some(GeosetSelection {
        visible: [2201].into_iter().collect(),
    });
    let mut wrist = definition.parts[0].clone();
    wrist.geosets = Some(GeosetSelection {
        visible: [801].into_iter().collect(),
    });
    definition.parts.push(wrist);
    let compiled = compile_character(&definition, &models, &mut Assets::default(), |_| {
        Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
    })
    .unwrap();
    assert_eq!(compiled.draws.len(), 3);
    assert_eq!(
        compiled.skins.len(),
        2,
        "collection was compiled once per equipment slot"
    );
    for skin in &compiled.skins {
        assert_eq!(compiled.nodes[skin.joints[0]].id, joint);
    }
    let ids: HashSet<_> = compiled.nodes.iter().map(|node| node.id).collect();
    assert_eq!(ids.len(), compiled.nodes.len());
}

#[test]
fn collection_policy_conflicts_fail_before_images_are_resolved() {
    let mut definition = definition();
    let mut wrists = definition.parts[0].clone();
    wrists
        .textures
        .insert(1, TextureReplacement::Image(ImageResource::FileDataId(99)));
    definition.parts.push(wrists);
    let (body, _) = model("body", &[401]);
    let (part, _) = model("part", &[2201]);
    let models = HashMap::from([
        (ResourceLocator::FileDataId(1), body),
        (ResourceLocator::FileDataId(2), part),
    ]);
    let mut calls = 0;
    let error = compile_character(&definition, &models, &mut Assets::default(), |_| {
        calls += 1;
        Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
    })
    .err()
    .unwrap();
    assert_eq!(calls, 0);
    assert!(error.0.contains("incompatible"));
}

#[test]
fn rigid_equipment_mounts_at_the_attachment_feature_and_preserves_its_pivot() {
    use wc3::model::{Attachment, Socket, SocketIdentity};

    let (mut body, joint) = model("body", &[401]);
    let mut node = body.node(joint).unwrap().clone();
    node.transform.pivot = [1., 2., 3.];
    let point = body
        .edit(|builder| {
            builder.set_node(joint, node)?;
            builder.add_feature(
                joint,
                NodeFeature::Attachment(Attachment::Socket(Socket {
                    identity: SocketIdentity::Role(6),
                    visibility: Animated::constant(0.),
                    control: None,
                })),
            )
        })
        .unwrap();
    let (shoulder, _) = model("shoulder", &[2201]);
    let models = HashMap::from([
        (ResourceLocator::FileDataId(1), body),
        (ResourceLocator::FileDataId(2), shoulder),
    ]);
    let mut recipe = definition();
    recipe.parts[0].binding = EquipmentBinding::Attachment(SocketIdentity::Role(6));
    let compiled = compile_character(&recipe, &models, &mut Assets::default(), |_| {
        Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
    })
    .unwrap();
    assert_eq!(compiled.mounts.len(), 1);
    assert_eq!(compiled.mounts[0].socket, Some(point));
    assert_eq!(
        compiled.mounts[0].transform.translation.to_array(),
        [1., 2., 3.]
    );
}
