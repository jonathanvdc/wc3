use wc3::model::conversion::{
    diablo::d3 as conversion, ApproximationPolicy, ExportOptions, ImportOptions, LossPolicy,
    UnknownPolicy,
};
use wc3::model::diablo::d3::records::{
    AppearanceLook, AppearanceMaterial, AppearanceRecord, BoneStructure, FatVertex, GeoSet,
    Influence, SubObject, SubObjectAppearance, Transform, VertexInfluences,
};
use wc3::model::diablo::d3::{Appearance, V260};
use zerovec::ZeroVec;
fn fixed<const N: usize>(text: &str) -> [u8; N] {
    let mut out = [0; N];
    out[..text.len()].copy_from_slice(text.as_bytes());
    out
}
fn source() -> Appearance<'static, V260> {
    let pose = Transform {
        rotation: [0., 0., 0., 1.],
        translation: [0.; 3],
        scale: 1.,
    };
    let bone = BoneStructure {
        name: fixed("root"),
        parent_index: -1,
        transform0: pose,
        transform1: pose,
        transform2: pose,
        transform3: pose,
        transform4: pose,
        particle: -1,
        ..Default::default()
    };
    let vertices = (0..3)
        .map(|i| FatVertex {
            position: match i {
                0 => [0., 0., 0.],
                1 => [1., 0., 0.],
                _ => [0., 1., 0.],
            },
            normal: 0x42ff8080,
            color: 0xff0000ff,
            aux_color: 0xffffffff,
            tex_coord0: 0x80008000,
            tex_coord1: 0x80008000,
            tangent: 0x238080ff,
            binormal: 0x1180ff80,
            cloth_vertex_index: u32::MAX,
        })
        .collect::<Vec<_>>();
    let row = VertexInfluences {
        influence0: Influence {
            bone_index: 0,
            weight: 1.,
        },
        influence1: Influence {
            bone_index: 0,
            weight: 0.,
        },
        influence2: Influence {
            bone_index: 0,
            weight: 0.,
        },
    };
    let sub = SubObject {
        vertex_count: 3,
        vertices: ZeroVec::alloc_from_slice(&vertices),
        vertex_influences: ZeroVec::alloc_from_slice(&[row; 3]),
        index_count: 3,
        indices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
        name: fixed("body"),
        material_name: fixed("body_naked_0"),
        bone_index: 0,
        surface: -1,
        ..Default::default()
    };
    let mut first = SubObjectAppearance::default();
    first.material.shader_map = 10;
    first.material_sno = 20;
    first.cloth = -1;
    first.unknown00 = 7;
    let mut second = first.clone();
    second.material.colors.diffuse = [0.2, 0.4, 0.6, 1.];
    Appearance::<V260>::new(AppearanceRecord {
        sno_id: 123,
        object_type: 5,
        bone_count: 1,
        bones: vec![bone],
        geo_set0: GeoSet {
            sub_object_count: 1,
            sub_objects: vec![sub],
            ..Default::default()
        },
        material_count: 1,
        materials: vec![AppearanceMaterial {
            name: fixed("body"),
            variants: vec![first, second],
            ..Default::default()
        }],
        look_count: 2,
        looks: ZeroVec::alloc_from_slice(&[
            AppearanceLook {
                name: fixed("base"),
            },
            AppearanceLook {
                name: fixed("blue"),
            },
        ]),
        unknown190: 123,
        unknown198: -987654321,
        unknown1_fc: 0.75,
        ..Default::default()
    })
}
#[test]
fn appearance_import_export_preserves_authored_fields() {
    let source = source();
    let imported = conversion::import(&source, &ImportOptions::default()).unwrap();
    assert!(imported.report.issues.is_empty());
    let planned = conversion::plan(&imported.value, &ExportOptions::default()).unwrap();
    let out = planned.execute().unwrap().value;
    assert_eq!(out.root, source.root);
    let bytes = out.write().unwrap();
    let reread = Appearance::<V260>::read(&bytes).unwrap();
    assert_eq!(reread.root, source.root);
}
#[test]
fn appearance_unsupported_semantics_require_explicit_loss() {
    let mut source = source();
    source.root.bone_pulses = ZeroVec::alloc_from_slice(&[1u8, 2, 3]);
    assert!(conversion::import(&source, &ImportOptions::default()).is_err());
    let imported = conversion::import(
        &source,
        &ImportOptions {
            unknown: UnknownPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "d3.bone_pulses"));
}
#[test]
fn appearance_export_honors_numeric_approximation_policy() {
    let source = source();
    let mut asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let mesh_id = asset.models[0].meshes().next().unwrap().0;
    asset
        .edit_model(0, |builder| {
            let mut mesh = builder.model().mesh(mesh_id).unwrap().clone();
            let mut streams = mesh.geometry.streams().clone();
            streams.normals = vec![[0., 0., 1.]; 3].into();
            mesh.geometry = mesh.geometry.with_streams(streams)?;
            builder.set_mesh(mesh_id, mesh)
        })
        .unwrap();
    assert!(conversion::plan(&asset, &ExportOptions::default()).is_err());
    let plan = conversion::plan(
        &asset,
        &ExportOptions {
            approximation: ApproximationPolicy::Allow { tolerance: 0.005 },
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!plan.report().issues.is_empty());
}
#[test]
fn appearance_and_animation_bundle_roundtrip_keeps_independent_declarations() {
    use wc3::model::diablo::d3::records::{
        AnimationPermutation, AnimationRecord, BoneName, Quaternion16, RotationCurve, RotationKey,
        ScaleCurve, ScaleKey, TranslationCurve, TranslationKey,
    };
    use wc3::model::diablo::d3::{Animation, V118};
    let appearance = source();
    let permutation = AnimationPermutation {
        name: fixed("walk"),
        frames_per_tick: 0.5,
        frame_count: 31,
        bone_count: 1,
        bone_names: ZeroVec::alloc_from_slice(&[BoneName {
            bone_name: fixed("root"),
        }]),
        translation_curves: vec![TranslationCurve {
            key_count: 2,
            keys: ZeroVec::alloc_from_slice(&[
                TranslationKey {
                    frame: 0,
                    position: [0.; 3],
                },
                TranslationKey {
                    frame: 30,
                    position: [1., 2., 3.],
                },
            ]),
            ..Default::default()
        }],
        rotation_curves: vec![RotationCurve {
            key_count: 1,
            keys: ZeroVec::alloc_from_slice(&[RotationKey {
                frame: 0,
                rotation: Quaternion16 {
                    x: 0,
                    y: 0,
                    z: 0,
                    w: 32767,
                },
            }]),
            ..Default::default()
        }],
        scale_curves: vec![ScaleCurve {
            key_count: 1,
            keys: ZeroVec::alloc_from_slice(&[ScaleKey {
                frame: 0,
                scale: 1.,
            }]),
            ..Default::default()
        }],
        root_motion: ZeroVec::alloc_from_slice(&[[0., 1., 2.], [3., 4., 5.]]),
        marker_frame0: 12.,
        speed_scalar: 1.25,
        ..Default::default()
    };
    let animation = Animation::<V118>::new(AnimationRecord {
        sno_id: 567,
        appearance: 123,
        name_hash: 4321,
        flags: 17,
        flags2: 19,
        permutation_count: 1,
        permutations: vec![permutation],
        ..Default::default()
    });
    let bundle = conversion::Bundle {
        appearance: &appearance,
        animations: vec![&animation],
    };
    let asset = conversion::import(&bundle, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(asset.models[0].clips().len(), 1);
    assert!(asset.models[0].clips().next().unwrap().1.channels().iter().any(|channel| matches!(channel,wc3::model::semantic::Channel::Rotation(track) if track.curve.interpolation()==wc3::model::semantic::Interpolation::Nlerp)));
    assert!(conversion::plan(&asset, &ExportOptions::default()).is_err());
    let out = conversion::plan_bundle(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(out.appearance.root, appearance.root);
    assert_eq!(out.animations.len(), 1);
    assert_eq!(out.animations[0].root, animation.root);
    let bytes = out.animations[0].write().unwrap();
    let reread = Animation::<V118>::read(&bytes).unwrap();
    assert_eq!(reread.root, animation.root);
}
#[test]
fn actual_slot_variants_survive_without_look_or_geometry_uses() {
    let mut source = source();
    let mut extra = source.root.materials[0].variants[0].clone();
    extra.unknown00 = 999;
    source.root.materials[0].variants.push(extra);
    let mut unused = source.root.materials[0].clone();
    unused.name = fixed("unplaced");
    source.root.materials.push(unused);
    source.root.material_count = 2;
    let asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(asset.models[0].appearance().slots().len(), 2);
    assert!(asset.models[0]
        .instances()
        .all(|(_, instance)| instance.draws.is_empty()));
    assert!(asset.models[0]
        .materials()
        .all(|(_, material)| material.name.is_empty()));
    let output = conversion::plan(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root, source.root);
}
#[test]
fn slot_identity_renames_native_references_without_material_label_inference() {
    let source = source();
    let mut asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let slot = asset.models[0].appearance().slots().next().unwrap().0;
    asset
        .edit_model(0, |builder| {
            let mut appearance = builder.model().appearance().clone();
            appearance.slot_mut(slot).unwrap().name = "renamed".into();
            builder.set_appearance(appearance);
            Ok(())
        })
        .unwrap();
    let output = conversion::plan(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root.materials[0].name, fixed("renamed"));
    assert_eq!(output.root.geo_set0.sub_objects[0].name, fixed("renamed"));
}
#[test]
fn native_implicit_look_ordinals_cannot_silently_reassign_actual_variants() {
    let source = source();
    let mut asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    asset
        .edit_model(0, |builder| {
            let mut appearance = builder.model().appearance().clone();
            let slot = appearance.slots().next().unwrap().0;
            let variant = appearance.slot(slot).unwrap().variants().nth(1).unwrap().0;
            let old = appearance
                .alternatives()
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            for id in old {
                appearance.remove_look(id);
            }
            appearance.add_look("only B", vec![(slot, variant)])?;
            builder.set_appearance(appearance);
            Ok(())
        })
        .unwrap();
    assert!(conversion::plan(&asset, &ExportOptions::default()).is_err());
    assert!(conversion::plan(
        &asset,
        &ExportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn foreign_slot_variant_id_is_not_resolved_by_its_display_name() {
    use wc3::model::semantic::profiles::diablo::d3::Slot;
    let source = source();
    let asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let mut appearance = asset.models[0].appearance().clone();
    let slot = appearance.slots().next().unwrap().0;
    let material = appearance.slot(slot).unwrap().variants().next().unwrap().1;
    let look = appearance.alternatives().next().unwrap().0;
    let extra = appearance.add_slot(Slot::named("extra"));
    let foreign = appearance.add_variant(extra, material).unwrap();
    assert!(appearance.bind(look, slot, foreign).is_err());
}
#[test]
fn nlerp_is_quaternion_only() {
    use wc3::model::semantic::{Curve, Quaternion, Time};
    assert!(Curve::nlerp(vec![Time::seconds(0.).unwrap()], vec![Quaternion::IDENTITY]).is_ok());
}
#[test]
fn unknown_bone_pulses_stay_inactive_after_graph_edits() {
    let mut source = source();
    source.root.bone_pulse_count = 2;
    source.root.bone_pulses = ZeroVec::alloc_from_slice(&[10u8, 20, 30, 40]);
    let mut asset = conversion::import(
        &source,
        &ImportOptions {
            unknown: UnknownPolicy::Archive,
            ..Default::default()
        },
    )
    .unwrap()
    .value;
    assert_eq!(asset.opaque.len(), 1);
    let bone = asset.models[0]
        .nodes()
        .find(|(_, node)| {
            matches!(
                node.controls,
                wc3::model::semantic::profiles::diablo::d3::NodeRole::Bone(_)
            )
        })
        .unwrap()
        .0;
    asset
        .edit_model(0, |builder| {
            let mut node = builder.model().node(bone).unwrap().clone();
            node.name = "renamed".into();
            builder.set_node(bone, node)
        })
        .unwrap();
    assert!(conversion::plan(&asset, &ExportOptions::default()).is_err());
    let out = conversion::plan(
        &asset,
        &ExportOptions {
            unknown: UnknownPolicy::Archive,
            ..Default::default()
        },
    )
    .unwrap()
    .execute()
    .unwrap()
    .value;
    assert!(out.root.bone_pulses.is_empty());
    assert_eq!(asset.opaque[0].bytes.as_ref(), &[10u8, 20, 30, 40]);
}
#[test]
fn animation_from_another_appearance_is_not_bound_by_matching_bone_names() {
    use wc3::model::diablo::d3::records::AnimationRecord;
    use wc3::model::diablo::d3::{Animation, V118};
    let appearance = source();
    let animation = Animation::<V118>::new(AnimationRecord {
        sno_id: 1,
        appearance: 999,
        ..Default::default()
    });
    let bundle = conversion::Bundle {
        appearance: &appearance,
        animations: vec![&animation],
    };
    assert!(conversion::import(&bundle, &ImportOptions::default()).is_err());
}
#[test]
fn standalone_material_resource_roundtrip_keeps_shared_shader_declarations() {
    use wc3::model::diablo::d3::records::{
        MaterialRecord, MaterialTextureAnim, MaterialTextureEntry, TagMapEntry,
    };
    use wc3::model::diablo::d3::{Material, V25};
    let mut root = MaterialRecord {
        sno_id: 42,
        unknown0_c: 7,
        ..Default::default()
    };
    root.material.shader_map = 123;
    root.material.colors.diffuse = [0.1, 0.2, 0.3, 0.4];
    root.material.colors.material_flags = 0x1234;
    root.shader_params = ZeroVec::alloc_from_slice(&[TagMapEntry {
        value_type: 99,
        tag_id: 88,
        value: 77,
    }]);
    root.material.textures = ZeroVec::alloc_from_slice(&[MaterialTextureEntry {
        texture_type: 5,
        texture: 987,
        unknown04: 4,
        uv_transform_mode: 6,
        uv_flags: 17,
        unknown9_c: 20,
        anim_u: MaterialTextureAnim {
            amount: 0.3,
            rate0: 1.2,
            rate1: 3.4,
        },
        uv_row0: [1., 2., 3., 4.],
        ..Default::default()
    }]);
    let source = Material::<V25>::new(root);
    let value = conversion::import_material(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(value.sno_id, 42);
    assert_eq!(value.uber.textures[0].texture.id, 987);
    let output = conversion::plan_material(&value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root, source.root);
    let bytes = output.write().unwrap();
    assert_eq!(Material::<V25>::read(&bytes).unwrap().root, source.root);
}
#[test]
fn standalone_material_validation_rejects_nonfinite_coefficients() {
    use wc3::model::diablo::d3::{Material, V25};
    let mut source = Material::<V25>::default();
    source.root.material.colors.shininess = f32::NAN;
    assert!(conversion::import_material(&source, &ImportOptions::default()).is_err());
    source.root.material.colors.shininess = 0.;
    let mut value = conversion::import_material(&source, &ImportOptions::default())
        .unwrap()
        .value;
    value.uber.colors.diffuse[0] = f32::INFINITY;
    assert!(conversion::plan_material(&value, &ExportOptions::default()).is_err());
}
#[test]
fn standalone_shader_passes_and_shader_map_roundtrip() {
    use wc3::model::diablo::d3::records::{
        RenderParams, RenderPass, ShaderMap, ShaderMapEntry, ShaderTagMapEntry, Shaders,
        TextureStageParams,
    };
    use wc3::model::diablo::d3::{Shader, ShaderMapDocument, V150, V26};
    let params = RenderParams {
        cull_mode: 17,
        z_write_enable: 99,
        stencil_enable: 1,
        alpha_test_enable: 1,
        b_alpha_ref: 127,
        depth_bias: 0.25,
        blend_op: 8,
        src_blend: 9,
        dest_blend: 10,
        ..Default::default()
    };
    let pass = RenderPass {
        unknown00: 1,
        unknown04: 2,
        unknown68: 3,
        render_params: params,
        texture_stage_count: 1,
        texture_stages: ZeroVec::alloc_from_slice(&[TextureStageParams {
            texture_type: 4,
            address_u: 5,
            address_v: 6,
            address_w: 7,
            filter: 8,
            mip_map_lod_bias: 0.5,
        }]),
        pass_flags: 0x12345678,
        effect_file: fixed("effect.fx"),
        vertex_shader_entry: fixed("VSMain"),
        pixel_shader_entry: fixed("PSMain"),
        shader_params: ZeroVec::alloc_from_slice(&[ShaderTagMapEntry {
            value_type: 9,
            tag_id: 10,
            value: 11,
        }]),
        ..Default::default()
    };
    let shader = Shader::<V150>::new(Shaders {
        sno_id: 42,
        shader_flags: 7,
        unknown10: 5,
        render_pass_count: 1,
        render_passes: vec![pass],
        name: [b'x'; 256],
        ..Default::default()
    });
    let value = conversion::import_shader(&shader, &ImportOptions::default())
        .unwrap()
        .value;
    let output = conversion::plan_shader(&value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root, shader.root);
    assert_eq!(
        Shader::<V150>::read(&output.write().unwrap()).unwrap().root,
        shader.root
    );
    let map = ShaderMapDocument::<V26>::new(ShaderMap {
        sno_id: 123,
        shader_map_flags: 7,
        shaders: ZeroVec::alloc_from_slice(&[ShaderMapEntry {
            value_type: 9,
            tag_id: 11,
            shader: 42,
        }]),
        ..Default::default()
    });
    let value = conversion::import_shader_map(&map, &ImportOptions::default())
        .unwrap()
        .value;
    let output = conversion::plan_shader_map(&value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root, map.root);
}
#[test]
fn duplicate_native_slot_names_cannot_hide_distinct_scoped_relationships() {
    use wc3::model::semantic::profiles::diablo::d3::Slot;
    let mut duplicate = source();
    duplicate
        .root
        .materials
        .push(duplicate.root.materials[0].clone());
    duplicate.root.material_count = 2;
    assert!(conversion::import(&duplicate, &ImportOptions::default()).is_err());
    let source = source();
    let mut asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    asset
        .edit_model(0, |builder| {
            let mut appearance = builder.model().appearance().clone();
            let material = appearance
                .slots()
                .next()
                .unwrap()
                .1
                .variants()
                .next()
                .unwrap()
                .1;
            let slot = appearance.add_slot(Slot::named("body"));
            appearance.add_variant(slot, material)?;
            builder.set_appearance(appearance);
            Ok(())
        })
        .unwrap();
    assert!(conversion::plan(&asset, &ExportOptions::default()).is_err());
}
#[test]
fn active_look_at_reference_follows_actual_bone_rename() {
    use wc3::model::semantic::profiles::diablo::d3::LookAtTarget;
    let mut source = source();
    source.root.look_at.enabled = 1;
    source.root.look_at.bone_name = fixed("root");
    let mut asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let LookAtTarget::Bone(bone) = asset.models[0].controls().look_at.target else {
        panic!("look-at reference was not resolved")
    };
    asset
        .edit_model(0, |builder| {
            let mut node = builder.model().node(bone).unwrap().clone();
            node.name = "renamed".into();
            builder.set_node(bone, node)
        })
        .unwrap();
    let output = conversion::plan(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root.look_at.bone_name, fixed("renamed"));
    assert_eq!(output.root.bones[0].name, fixed("renamed"));
}
#[test]
fn active_look_at_names_require_a_unique_actual_bone() {
    let mut source = source();
    source.root.look_at.enabled = 1;
    source.root.look_at.bone_name = fixed("missing");
    assert!(conversion::import(&source, &ImportOptions::default()).is_err());
    source.root.look_at.bone_name = fixed("root");
    source.root.bones.push(source.root.bones[0].clone());
    source.root.bone_count = 2;
    assert!(conversion::import(&source, &ImportOptions::default()).is_err());
}
#[test]
fn inactive_unresolved_look_at_spelling_stays_editable_without_claiming_a_bone() {
    use wc3::model::semantic::profiles::diablo::d3::LookAtTarget;
    let mut source = source();
    source.root.look_at.bone_name = fixed("missing");
    let mut asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        asset.models[0].controls().look_at.target,
        LookAtTarget::Unresolved("missing".to_owned())
    );
    asset
        .edit_model(0, |builder| {
            let mut controls = builder.model().controls().clone();
            controls.look_at.target = LookAtTarget::Unresolved("retained".to_owned());
            builder.set_controls(controls);
            Ok(())
        })
        .unwrap();
    let output = conversion::plan(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root.look_at.bone_name, fixed("retained"));
    assert!(asset
        .edit_model(0, |builder| {
            let mut controls = builder.model().controls().clone();
            controls.look_at.enabled = 1;
            builder.set_controls(controls);
            Ok(())
        })
        .is_err());
}
#[test]
fn look_at_typed_target_rejects_non_bones_and_ambiguous_native_export_names() {
    use wc3::model::semantic::profiles::diablo::d3::{LookAtTarget, NodeRole};
    let mut source = source();
    source.root.look_at.enabled = 1;
    source.root.look_at.bone_name = fixed("root");
    let mut asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let frame = asset.models[0]
        .nodes()
        .find(|(_, node)| matches!(node.controls, NodeRole::ModelSpace))
        .unwrap()
        .0;
    assert!(asset
        .edit_model(0, |builder| {
            let mut controls = builder.model().controls().clone();
            controls.look_at.target = LookAtTarget::Bone(frame);
            builder.set_controls(controls);
            Ok(())
        })
        .is_err());
    let LookAtTarget::Bone(bone) = asset.models[0].controls().look_at.target else {
        unreachable!()
    };
    asset
        .edit_model(0, |builder| {
            builder.add_node(builder.model().node(bone).unwrap().clone())?;
            Ok(())
        })
        .unwrap();
    assert!(conversion::plan(&asset, &ExportOptions::default()).is_err());
}
#[test]
fn disabled_ambiguous_look_at_name_remains_unresolved() {
    use wc3::model::semantic::profiles::diablo::d3::LookAtTarget;
    let mut source = source();
    source.root.look_at.bone_name = fixed("root");
    source.root.bones.push(source.root.bones[0].clone());
    source.root.bone_count = 2;
    let asset = conversion::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        asset.models[0].controls().look_at.target,
        LookAtTarget::Unresolved("root".to_owned())
    );
    let output = conversion::plan(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root.look_at.bone_name, source.root.look_at.bone_name);
}
