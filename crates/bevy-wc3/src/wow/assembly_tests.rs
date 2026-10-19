use super::*;
use crate::assembly::{Atlas, PixelRect, RgbaImage, TextureLayer, TextureReplacement};
use crate::material::native_fixtures::m2::{Material as M2Material, TextureUnit};
use crate::wow::ModelPart as EquipmentPart;
use std::collections::HashMap;
use wc3::model::controls::model::BoneControls;
use wc3::model::materials::bindings::ImageSelection;
use wc3::model::materials::program::MaterialStep;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    Draw, GeometryData, GeometryInfluences, IdentityBinding, Influence, Material, MaterialSurface,
    Mesh, MeshInstance, Node, NodeControls, Skin, Texture, UniversalBinding, ValueGroups,
};
use wc3::model::{ImageResource, ResourceLocator};

fn model(name: &str, geosets: &[u16]) -> (Model<'static>, NodeId) {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder(ModelMetadata::M2(name.to_owned().into()));
    let joints: Vec<_> = (0..8)
        .map(|i| {
            builder
                .add_node(
                    Node::named(format!("matching bone{i}"), &mut slot_values).with_controls(Some(
                        NodeControls::M2(BoneControls {
                            name_crc: 100 + i,
                            ..SlotConstruct::construct(&mut slot_values)
                        }),
                    )),
                )
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
    texture.resource = builder.add_image_resource(ImageResource::replaceable(1));

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
                surface: MaterialSurface::from(M2Material {
                    textures: vec![TextureUnit {
                        image: ImageSelection {
                            texture: Some(texture),
                            ..SlotConstruct::construct(&mut slot_values)
                        },
                        contribution: slot_values.add(1.),
                        ..SlotConstruct::construct(&mut slot_values)
                    }],
                    color: slot_values.add([1.; 3]),
                    opacity: slot_values.add(1.),
                    ..SlotConstruct::construct(&mut slot_values)
                }),
            })
            .unwrap();
        let mut instance = MeshInstance::new(joint, mesh);
        instance.skin = Some(skin);
        instance.selection_group = u32::from(*geoset);
        instance.draws.push(Draw::new(
            subset,
            SlotConstruct::construct(&mut slot_values),
        ));
        builder
            .add_instance_with_materials(instance, vec![material])
            .unwrap();
    }
    (
        {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap(),
        joint,
    )
}
fn definition() -> CharacterDefinition {
    CharacterDefinition {
        bone_sets: Default::default(),
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
                resource: ImageResource::file_data_id(3),
                source: None,
                destination: PixelRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                opacity: 255,
                blend: Default::default(),
            }],
        },
    );
    definition.parts[0]
        .textures
        .insert(1, TextureReplacement::Image(ImageResource::file_data_id(3)));
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
        resource: ImageResource::file_data_id(3),
        source: None,
        destination: PixelRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        },
        opacity: 255,
        blend: Default::default(),
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
        .iter()
        .filter(|(_, instance)| instance.selection_group == 0)
        .flat_map(|(id, instance)| instance.draws.iter().map(move |draw| (id, draw.id())))
        .map(|(instance, draw)| {
            let id = body.material_for_draw(instance, draw, None).unwrap();
            (id, body.material(id).unwrap().clone())
        })
        .collect();
    body.edit(|draft| {
        let resource = draft.add_image_resource(ImageResource::replaceable(6));
        let texture = Texture::new(resource);

        for (id, mut material) in hidden_materials {
            for pass in &mut material.surface.steps {
                if let MaterialStep::M2(surface) = pass {
                    surface.inputs_mut()[0].image.texture = Some(texture);
                }
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
        if *resource == ImageResource::replaceable(6) {
            requested_hair = true;
            return Err(ModelError("missing selected hair texture".into()));
        }
        Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
    })
    .unwrap();
    assert!(!requested_hair);
    definition.geosets.visible = [0].into_iter().collect();
    let result = compile_character(&definition, &models, &mut Assets::default(), |resource| {
        if *resource == ImageResource::replaceable(6) {
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
    let mut slot_values = SlotValues::default();
    let (body, joint) = model("body", &[401]);
    let (mut collection, armor_joint) = model("collection", &[2201, 801]);
    // Retained helpers/anchors make duplicate source scopes a real collision,
    // even when the visible subsets and their body-joint matches are disjoint.
    collection
        .edit(|builder| {
            let edit_result = (|| {
                builder.add_node(
                    Node::named("collection helper", &mut slot_values).with_parent(armor_joint),
                )?;
                Ok(())
            })();
            builder.import_values(&slot_values)?;
            edit_result
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
    wrists.textures.insert(
        1,
        TextureReplacement::Image(ImageResource::file_data_id(99)),
    );
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
    let mut slot_values = SlotValues::default();
    use wc3::model::{Attachment, Socket, SocketIdentity};

    let (mut body, joint) = model("body", &[401]);
    let mut node = body.node(joint).unwrap().clone();
    node.transform.pivot = [1., 2., 3.];
    let point = body
        .edit(|builder| {
            let edit_result = (|| {
                builder.set_node(joint, node)?;
                builder.add_feature(
                    joint,
                    NodeFeature::Attachment(Attachment::Socket(Socket {
                        identity: SocketIdentity::Role(6),
                        visibility: slot_values.add(0.),
                        control: None,
                    })),
                )
            })();
            builder.import_values(&slot_values)?;
            edit_result
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

#[test]
fn selected_native_bone_resources_are_checked_and_retained_in_compiled_pose() {
    use bevy::prelude::Vec3;
    use wc3::model::systems::{
        model::{BoneOverride, System as M2System},
        System,
    };
    let (mut body, joint) = model("body", &[401]);
    let resource = ResourceLocator::FileDataId(55);
    let correction = Mat4::from_translation(Vec3::new(0.1, 0.2, 0.3));
    body.edit(|builder| {
        let first = builder.add_bone_override_resource(resource.clone());
        let second = builder.add_bone_override_resource(ResourceLocator::FileDataId(56));
        builder.add_system(System::M2(M2System::BoneOverrides(vec![
            BoneOverride {
                resource: Some(first),
                bones: vec![joint],
                transforms: [correction.to_cols_array_2d()].into_iter().collect(),
            },
            BoneOverride {
                resource: Some(second),
                bones: vec![joint],
                transforms: [correction.to_cols_array_2d()].into_iter().collect(),
            },
        ])))?;
        Ok(())
    })
    .unwrap();
    let mut definition = definition();
    definition.parts.clear();
    let models = Models::from([(definition.model.clone(), body)]);
    let mut images = Assets::default();
    definition.bone_sets.insert(resource);
    let compiled = compile_character(&definition, &models, &mut images, |_| {
        RgbaImage::new(1, 1, vec![255; 4]).map_err(|e| ModelError(e.to_string()))
    })
    .unwrap();
    assert_eq!(
        compiled
            .nodes
            .iter()
            .find(|node| node.id == joint)
            .unwrap()
            .correction,
        Some(correction)
    );
    let template = compiled
        .image_template
        .as_ref()
        .expect("rebindable appearance template");
    assert_eq!(
        template
            .nodes
            .iter()
            .find(|node| node.id == joint)
            .unwrap()
            .correction,
        Some(correction)
    );
    definition.bone_sets.insert(ResourceLocator::FileDataId(56));
    assert!(
        compile_character(&definition, &models, &mut images, |_| panic!(
            "overlap rejected before image I/O"
        ))
        .err()
        .unwrap()
        .0
        .contains("overlap")
    );
    definition.bone_sets = [ResourceLocator::FileDataId(99)].into_iter().collect();
    assert!(
        compile_character(&definition, &models, &mut images, |_| panic!(
            "unknown resource rejected before image I/O"
        ))
        .err()
        .unwrap()
        .0
        .contains("absent")
    );
    definition.bone_sets.clear();
    let compiled = compile_character(&definition, &models, &mut images, |_| {
        RgbaImage::new(1, 1, vec![255; 4]).map_err(|e| ModelError(e.to_string()))
    })
    .unwrap();
    assert!(compiled.nodes.iter().all(|node| node.correction.is_none()));
}
