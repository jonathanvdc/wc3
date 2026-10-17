//! Diablo III appearance import and typed native export planning.
use super::super::{Context, Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::common::Borrowed as _;
use crate::model::formats::diablo::d3::records as native;
use crate::model::formats::diablo::d3::{
    Animation, Appearance, Material as NativeMaterial, V118, V25, V260,
};
use crate::model::profiles::diablo::d3::{
    Appearance as LogicalAppearance, BoneControls, LookAtControls, LookAtTarget, MaterialAsset,
    ModelMetadata, NodeRole, Slot, D3,
};
use crate::model::{
    Bounds, ExplicitBinding, Material, MeshInstance, Model, NodeId, Quaternion, Skin, Transform,
};
use std::{array::from_fn, borrow::Cow, str::from_utf8};
#[path = "d3_animation.rs"]
mod animation;
#[path = "d3_export.rs"]
mod export;
#[path = "d3_geometry.rs"]
mod geometry;
#[path = "d3_materials.rs"]
mod materials;
#[path = "d3_shaders.rs"]
mod shaders;
pub use shaders::{import_shader, import_shader_map, plan_shader, plan_shader_map};

/// A source assembled by the caller from independently versioned native resources.
#[derive(Clone, Debug)]
pub struct Bundle<'view, 'data> {
    /// Primary appearance document.
    pub appearance: &'view Appearance<'data, V260>,
    /// Animation resources to bind by their declared bone names.
    pub animations: Vec<&'view Animation<'data, V118>>,
}
/// Native source accepted by the Diablo III semantic importer.
pub trait Source {
    /// Convert native records without opening external resources.
    fn import<'a>(&'a self, options: &ImportOptions) -> Result<Conversion<Model<'a, D3>>, Error>;
}
impl Source for Appearance<'_, V260> {
    fn import<'a>(&'a self, options: &ImportOptions) -> Result<Conversion<Model<'a, D3>>, Error> {
        import_appearance(self, options)
    }
}
impl Source for Bundle<'_, '_> {
    fn import<'a>(&'a self, options: &ImportOptions) -> Result<Conversion<Model<'a, D3>>, Error> {
        let mut converted = import_appearance(self.appearance, options)?;
        let mut context = Context::import(options);
        animation::read(&mut converted.value, &self.animations, &mut context)?;
        converted.report.issues.extend(context.report.issues);
        Ok(converted)
    }
}
/// Import an appearance or a caller-assembled appearance and animation bundle.
pub fn import<'a, S: Source>(
    source: &'a S,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, D3>>, Error> {
    source.import(options)
}
/// Native output resources with no filesystem naming or persistence policy.
#[derive(Clone, Debug)]
pub struct Output {
    /// Revision 260 appearance.
    pub appearance: Appearance<'static, V260>,
    /// Independently versioned revision 118 animation documents.
    pub animations: Vec<Animation<'static, V118>>,
}
/// Plan appearance and animation documents together, retaining clip programs.
pub fn plan_bundle(asset: &Model<'_, D3>, options: &ExportOptions) -> Result<Plan<Output>, Error> {
    let appearance = export::plan_with_animations(asset, options, true)?.execute()?;
    let mut context = Context::export(options);
    let animations = animation::write(asset, options, &mut context)?;
    let mut report = appearance.report;
    report.issues.extend(context.report.issues);
    Ok(Plan::new(
        Output {
            appearance: appearance.value,
            animations,
        },
        report,
    ))
}

/// Import a revision 260 appearance without filesystem or archive assumptions.
/// External material and shader resources remain explicit SNO references.
fn import_appearance<'a>(
    source: &'a Appearance<'_, V260>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, D3>>, Error> {
    let root = &source.root;
    let mut context = Context::import(options);
    unsupported(root, &mut context)?;
    let mut builder = Model::<D3>::builder("");
    builder.set_bounds(Some(bounds(root.bounds)));
    let model_frame = {
        let definition = builder.new_node("").with_controls(NodeRole::ModelSpace);
        builder.add_node(definition)
    }?;
    let bone_ids: Vec<_> = root.bones.iter().map(|_| builder.reserve_node()).collect();
    builder.set_metadata(read_controls(root, &bone_ids)?);
    for (i, bone) in root.bones.iter().enumerate() {
        if !bone.collision_shapes.is_empty() || !bone.constraints.is_empty() {
            context.unsupported(
                format!("d3.bones[{i}].physics"),
                "bone collision shapes and constraints have no logical physics representation",
            )?;
        }
        let mut node = builder
            .new_node(text(&bone.name)?)
            .with_controls(NodeRole::Bone(Box::new(BoneControls {
                bind_pose: transform(bone.transform0),
                inverse_bind_pose: transform(bone.transform1),
                alternate_bind_pose: transform(bone.transform3),
                inverse_alternate_bind_pose: transform(bone.transform4),
                bounds: bounds(bone.bounds),
                sphere_center: bone.sphere.center,
                sphere_radius: bone.sphere.radius,
                particle: bone.particle,
            })))
            .with_transform(transform(bone.transform2), builder.values_mut())?;
        node.parent = parent(bone.parent_index, &bone_ids)?;
        builder.set_node(bone_ids[i], node)?;
    }
    let skin = if bone_ids.is_empty() {
        None
    } else {
        Some(
            builder.add_skin(Skin::new(
                bone_ids.clone(),
                ExplicitBinding::new(
                    root.bones
                        .iter()
                        .map(|b| matrix(transform(b.transform4)))
                        .collect::<Vec<_>>(),
                )?,
            ))?,
        )
    };
    for hardpoint in root.hardpoints.iter() {
        let mut node = builder
            .new_node(text_owned(&hardpoint.name)?)
            .with_controls(NodeRole::Hardpoint {
                name: Cow::Owned(text_owned(&hardpoint.name)?),
            })
            .with_transform(
                Transform {
                    translation: hardpoint.transform.translation,
                    rotation: Quaternion(hardpoint.transform.rotation),
                    ..Default::default()
                },
                builder.values_mut(),
            )?;
        node.parent = parent(hardpoint.bone_index, &bone_ids)?;
        builder.add_node(node)?;
    }
    let mut appearance = LogicalAppearance::default();
    let mut slot_ids = Vec::new();
    for slot in &root.materials {
        let slot_name = text(&slot.name)?;
        if appearance.slots().any(|(_, slot)| slot.name == slot_name) {
            return Err(Error::new(
                "d3.materials.name",
                "native material-slot names must be unique",
            ));
        }
        let slot_id = appearance.add_slot(Slot::named(slot_name));
        for variant in &slot.variants {
            let (selection, surface) = materials::read(variant);
            let material = builder.add_material(Material {
                name: Cow::Borrowed(""),
                surface,
            })?;
            appearance.add_variant_with_selection(slot_id, material, selection)?;
        }
        slot_ids.push(slot_id);
    }
    for (lod, geoset) in [&root.geo_set0, &root.geo_set1].into_iter().enumerate() {
        for (i, sub) in geoset.sub_objects.iter().enumerate() {
            if !sub.cloth_data.is_empty() || !sub.collision_shapes.is_empty() {
                context.unsupported(
                    format!("d3.geometry[{lod}][{i}].physics"),
                    "cloth and collision shapes have no logical physics representation",
                )?;
            }
            let slot_name = text(&sub.name)?;
            let slot = appearance
                .slots()
                .find(|(_, slot)| slot.name == slot_name)
                .map(|(id, _)| id)
                .ok_or_else(|| {
                    Error::new(
                        "d3.geometry.material",
                        "subobject names an undeclared native material slot",
                    )
                })?;
            let mesh = geometry::read(sub, &bone_ids, Some(slot))?;
            let mesh_id = builder.add_mesh(mesh)?;
            let mut instance = MeshInstance::new(model_frame, mesh_id);
            instance.skin = if sub.vertex_influences.is_empty() {
                None
            } else {
                skin
            };
            instance.lod = Some(lod as u32);
            builder.add_instance(instance)?;
        }
    }
    for (look_index, look) in root.looks.iter().enumerate() {
        let bindings = slot_ids
            .iter()
            .map(|slot| {
                let variant = appearance
                    .slot(*slot)
                    .unwrap()
                    .variants()
                    .nth(look_index)
                    .map(|(id, _)| id)
                    .ok_or_else(|| {
                        Error::new("d3.looks", "native look selects a missing slot variant")
                    })?;
                Ok((*slot, variant))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        appearance.add_look(text_owned(&look.name)?, bindings)?;
    }
    builder.set_appearance(appearance);
    let asset = builder.finish()?;
    if !root.bone_pulses.is_empty() || root.bone_pulse_count != 0 {
        context.unknown_data(
            "d3.bone_pulses",
            "uninterpreted bone-pulse payload has no semantic representation",
        )?;
    }
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
/// Plan revision 260 appearance output from the Diablo III authoring vocabulary.
pub fn plan(
    asset: &Model<'_, D3>,
    options: &ExportOptions,
) -> Result<Plan<Appearance<'static, V260>>, Error> {
    export::plan(asset, options)
}
fn i32_count(value: usize) -> Result<i32, Error> {
    value
        .try_into()
        .map_err(|_| Error::new("d3.count", "count exceeds i32"))
}
fn text(bytes: &[u8]) -> Result<&str, Error> {
    from_utf8(&bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())])
        .map_err(|_| Error::new("d3.name", "native name is not UTF-8"))
}
fn text_owned(bytes: &[u8]) -> Result<String, Error> {
    Ok(text(bytes)?.to_owned())
}
fn name<const N: usize>(value: &str) -> Result<[u8; N], Error> {
    if value.len() > N || value.as_bytes().contains(&0) {
        return Err(Error::new(
            "d3.name",
            "name does not fit its native fixed string",
        ));
    }
    let mut out = [0; N];
    out[..value.len()].copy_from_slice(value.as_bytes());
    Ok(out)
}
fn parent(index: i32, bones: &[NodeId]) -> Result<Option<NodeId>, Error> {
    if index == -1 {
        return Ok(None);
    }
    if index < -1 {
        return Err(Error::new(
            "d3.bone",
            "unestablished negative hierarchy sentinel",
        ));
    }
    bones
        .get(index as usize)
        .copied()
        .map(Some)
        .ok_or_else(|| Error::new("d3.bone", "bone index is out of range"))
}
fn bounds(v: native::Bounds) -> Bounds {
    Bounds {
        minimum: from_fn(|i| v.center[i] - v.half_extent[i]),
        maximum: from_fn(|i| v.center[i] + v.half_extent[i]),
        sphere_radius: 0.,
    }
}
fn native_bounds(v: Bounds) -> native::Bounds {
    native::Bounds {
        center: from_fn(|i| (v.minimum[i] + v.maximum[i]) * 0.5),
        half_extent: from_fn(|i| (v.maximum[i] - v.minimum[i]) * 0.5),
    }
}
fn transform(v: native::Transform) -> Transform {
    Transform {
        translation: v.translation,
        rotation: Quaternion(v.rotation),
        scale: [v.scale; 3],
        pivot: [0.; 3],
    }
}
fn native_transform(v: Transform) -> Result<native::Transform, Error> {
    if v.scale[0] != v.scale[1] || v.scale[0] != v.scale[2] || v.pivot != [0.; 3] {
        return Err(Error::new(
            "d3.transform",
            "native transform requires uniform scale and no pivot",
        ));
    }
    Ok(native::Transform {
        translation: v.translation,
        rotation: v.rotation.0,
        scale: v.scale[0],
    })
}
fn matrix(t: Transform) -> [[f32; 4]; 4] {
    let [x, y, z, w] = t.rotation.0;
    let inv = (x * x + y * y + z * z + w * w).recip();
    let s = t.scale[0];
    [
        [
            s * (1. - 2. * (y * y + z * z) * inv),
            s * 2. * (x * y + z * w) * inv,
            s * 2. * (x * z - y * w) * inv,
            0.,
        ],
        [
            s * 2. * (x * y - z * w) * inv,
            s * (1. - 2. * (x * x + z * z) * inv),
            s * 2. * (y * z + x * w) * inv,
            0.,
        ],
        [
            s * 2. * (x * z + y * w) * inv,
            s * 2. * (y * z - x * w) * inv,
            s * (1. - 2. * (x * x + y * y) * inv),
            0.,
        ],
        [t.translation[0], t.translation[1], t.translation[2], 1.],
    ]
}
fn read_controls(
    v: &native::AppearanceRecord<'_>,
    bone_ids: &[NodeId],
) -> Result<ModelMetadata, Error> {
    let authored = text(&v.look_at.bone_name)?;
    let mut matches = Vec::new();
    for (index, bone) in v.bones.iter().enumerate() {
        if text(&bone.name)? == authored {
            matches.push(bone_ids[index]);
        }
    }
    let target = if matches.len() == 1 {
        LookAtTarget::Bone(matches[0])
    } else if v.look_at.enabled == 0 {
        LookAtTarget::Unresolved(authored.to_owned())
    } else {
        return Err(Error::new(
            "d3.look_at.target",
            "active look-at target does not identify one unique skeleton bone",
        ));
    };
    Ok(ModelMetadata {
        sno_id: v.sno_id,
        object_type: v.object_type,
        collision_center: v.collision_sphere.center,
        collision_radius: v.collision_sphere.radius,
        look_at: LookAtControls {
            enabled: v.look_at.enabled,
            target,
            yaw: [v.look_at.yaw_min, v.look_at.yaw_max],
            pitch: [v.look_at.pitch_min, v.look_at.pitch_max],
            angular_velocity: v.look_at.angular_velocity,
            unclassified_scalar: v.look_at.unknown58,
        },
        unclassified_vector: v.unknown108,
        unclassified_integers: [
            v.unknown190,
            v.unknown1_e0,
            v.unknown1_e4,
            v.unknown1_e8,
            v.unknown1_ec,
            v.unknown1_f0,
            v.unknown1_f4,
            v.unknown1_f8,
            v.unknown200,
            v.unknown204,
            v.unknown210,
        ],
        unclassified_wide_integers: [v.unknown198, v.unknown208],
        unclassified_scalar: v.unknown1_fc,
    })
}
fn unsupported(v: &native::AppearanceRecord<'_>, ctx: &mut Context) -> Result<(), Error> {
    for (present, path, description) in [
        (
            !v.collision_capsules.is_empty() || !v.constraints.is_empty(),
            "d3.physics",
            "appearance collision capsules and constraints are unavailable",
        ),
        (
            v.octree.unknown00 != 0
                || v.octree.unknown04 != 0
                || v.octree.unknown08 != 0
                || !v.octree.nodes.is_empty()
                || !v.octree.leaves.is_empty()
                || !v.octree.primitives.is_empty(),
            "d3.octree",
            "native octree semantics are unavailable",
        ),
        (
            !v.static_lights.is_empty(),
            "d3.lights",
            "static light semantics are unavailable",
        ),
    ] {
        if present {
            ctx.unsupported(path, description)?;
        }
    }
    Ok(())
}

/// Import a standalone revision 25 material resource into the shared Diablo vocabulary.
/// Shader-map and texture dependencies remain explicit SNO references.
pub fn import_material(
    source: &NativeMaterial<'_, V25>,
    _options: &ImportOptions,
) -> Result<Conversion<MaterialAsset>, Error> {
    let root = &source.root;
    let (_, projected) = materials::read(&native::SubObjectAppearance {
        material: root.material.as_borrowed(),
        shader_params: root.shader_params.as_borrowed(),
        ..Default::default()
    });
    let value = MaterialAsset {
        sno_id: root.sno_id,
        unclassified_control: root.unknown0_c,
        shader_params: projected.shader_params,
        uber: projected.uber,
    };
    value.validate()?;
    Ok(Conversion {
        value,
        report: Default::default(),
    })
}
/// Plan a standalone revision 25 material from independently authored declarations.
pub fn plan_material(
    value: &MaterialAsset,
    _options: &ExportOptions,
) -> Result<Plan<NativeMaterial<'static, V25>>, Error> {
    value.validate()?;
    let root = native::MaterialRecord {
        sno_id: value.sno_id,
        unknown0_c: value.unclassified_control,
        shader_params: materials::write_tags(&value.shader_params),
        material: materials::write_uber(&value.uber)?,
        ..Default::default()
    };
    Ok(Plan::new(
        NativeMaterial::<V25>::new(root),
        Default::default(),
    ))
}
