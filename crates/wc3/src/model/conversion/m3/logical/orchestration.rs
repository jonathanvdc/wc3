//! Direct graph orchestration; no encoded editable document participates in conversion.
use super::emitter_links::Links;
use super::{
    animation, bindings, cameras, forces, geometry, geometry_write, hierarchy, ik, lights,
    materials,
    profiles::{ExportProfile, RootFields},
    skeleton_write, write, GraphBuilder,
};
use crate::model::common::FileKey;
use crate::model::conversion::{
    Asset, Context, Conversion, EncodingHint, Error, ExportOptions, Format, ImportOptions,
    IssueKind, OpaquePayload, Plan, UnknownPolicy,
};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{profiles::m3 as surface, profiles::M3, LengthUnit};
use native::Envelope;
use std::borrow::Cow;
use std::collections::BTreeMap;

pub(in crate::model::conversion::m3) fn read<'a, E: Envelope + Send + Sync>(
    source: &'a native::Bundle<'_, E>,
    options: &ImportOptions,
) -> Result<Conversion<Asset<'a, M3>>, Error> {
    source.validate_complete().map_err(materials::read_error)?;
    let root = source.model.root_record().map_err(materials::read_error)?;
    let name = native::visit_m3_root!(root, |r| r.name);
    let mut builder = GraphBuilder::with_profile(materials::string(source.model.document(), name)?);
    let flags = native::visit_m3_root!(root, |r| r.flags.bits());
    builder.set_controls(surface::ModelControls {
        flags: surface::values::ModelFlags(flags),
        partition_control: native::visit_m3_root!(root, |r| r.animation_split_count),
    });
    let mut context = Context::import(options);
    super::qualifiers::check(source.model.document(), &mut context)?;
    super::partitions::read(
        source.model.document(),
        native::visit_m3_root!(root, |r| r.bone_animation_sets),
        &mut builder,
    )?;
    check_unresolved_source(&source.model, &mut context)?;
    let mut bindings = bindings::Bindings::default();
    let skeleton = hierarchy::read(&source.model, &mut builder, &mut bindings)?;
    super::turrets::read(&source.model, &mut builder, &skeleton.nodes, &mut context)?;
    super::volumes::read(&source.model, &skeleton.nodes, &mut builder, &mut context)?;
    super::physics::read(
        &source.model,
        &mut builder,
        &skeleton.nodes,
        &mut bindings,
        &mut context,
    )?;
    let materials = materials::read(&source.model, &mut builder, &mut context, &mut bindings)?;
    let geometry = geometry::read(&source.model, &mut builder, &skeleton, &materials)?;
    let cloth_ref = native::visit_m3_root!(root, |r| materials::Slot::<
        native::Ref<nr::targets::ClothPhysics>,
    >::value(&r.cloth_physics));
    super::cloth::read(
        source.model.document(),
        cloth_ref,
        geometry
            .regions
            .first()
            .map(Vec::as_slice)
            .unwrap_or_default(),
        &skeleton.nodes,
        &mut builder,
        &mut bindings,
        &mut context,
    )?;
    let (light_ref, camera_ref, force_ref, warp_ref) =
        native::visit_m3_root!(root, |r| (r.lights, r.cameras, r.forces, r.warps));
    lights::read(
        source.model.document(),
        light_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    cameras::read(
        source.model.document(),
        camera_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    forces::force(
        source.model.document(),
        force_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    forces::warp(
        source.model.document(),
        warp_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    let projector_ref = native::visit_m3_root!(root, |r| r.projections);
    let projectors = super::projectors::read(
        source.model.document(),
        projector_ref,
        &mut builder,
        &skeleton,
        &materials,
        &mut bindings,
    )?;
    let (ribbons, particles, copies) = native::visit_m3_root!(root, |r| (
        r.ribbon_emitters,
        r.particle_emitters,
        r.particle_emitter_copies
    ));
    let meshes = geometry
        .regions
        .first()
        .map(Vec::as_slice)
        .unwrap_or_default();
    let ribbons = super::ribbons::import(
        source.model.document(),
        ribbons,
        &mut builder,
        &Links {
            skeleton: &skeleton,
            materials: &materials,
            meshes,
            ribbons: &[],
            projectors: &[],
        },
        &mut bindings,
        &mut context,
    )?;
    super::particles::import(
        source.model.document(),
        particles,
        copies,
        &mut builder,
        &Links {
            skeleton: &skeleton,
            materials: &materials,
            meshes,
            ribbons: &ribbons,
            projectors: &projectors,
        },
        &mut bindings,
        &mut context,
    )?;
    ik::read(&source.model, &mut builder, &skeleton)?;
    super::static_scene::read(&source.model, &mut builder, &skeleton, &mut bindings)?;
    super::collision::read(&source.model, &mut builder)?;
    animation::read(
        &source.model,
        &skeleton.nodes,
        &bindings,
        &mut builder,
        &mut context,
    )?;
    for (_, file) in source.animations() {
        match file {
            native::DynamicModel::Md33(model) => {
                super::qualifiers::check(model.document(), &mut context)?;
                super::companions::check(&source.model, model, &mut context)?;
                animation::read(
                    model,
                    &skeleton.nodes,
                    &bindings,
                    &mut builder,
                    &mut context,
                )?;
            }
            native::DynamicModel::Md34(model) => {
                super::qualifiers::check(model.document(), &mut context)?;
                super::companions::check(&source.model, model, &mut context)?;
                animation::read(
                    model,
                    &skeleton.nodes,
                    &bindings,
                    &mut builder,
                    &mut context,
                )?;
            }
        }
    }
    if source.animations().next().is_some() {
        context.issue(
            "m3.animation.dependencies",
            IssueKind::Normalized,
            "external animation closure is rebuilt inline with relocated native tracks",
        );
    }
    let mut asset = Asset::new(builder.finish()?);
    asset.encoding_hints = geometry.encoding_hints;
    asset.encoding_hints.push(EncodingHint::M3VertexQualifiers {
        flags: nr::VertexFlags(native::visit_m3_root!(root, |r| r.vertex_flags.bits()) & 0x1c),
    });
    opaque(&source.model, FileKey::Root, &mut asset, &mut context)?;
    for (hash, file) in source.animations() {
        let file_key = FileKey::Root.sibling(format!(".animation-{hash:08x}.m3a"));
        match file {
            native::DynamicModel::Md33(model) => opaque(model, file_key, &mut asset, &mut context)?,
            native::DynamicModel::Md34(model) => opaque(model, file_key, &mut asset, &mut context)?,
        }
    }
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
pub(in crate::model::conversion::m3) fn plan<'view, 'source: 'view, P: ExportProfile>(
    asset: &'view Asset<'source, M3>,
    options: &ExportOptions,
) -> Result<Plan<native::Bundle<'view, P::Envelope>>, Error> {
    let graph = asset.model(options)?;
    let mut context = Context::export(options);

    if graph.units() != LengthUnit::Unspecified {
        context.unsupported("m3.units","native M3 has no authored physical-unit declaration; unit translation is a semantic operation")?;
    }
    let ids = animation::prepare(graph, &mut context)?;
    let mut document = native::Document::<P::Envelope>::new();
    let mut skeleton = skeleton_write::prepare(graph, &ids, &mut context)?;
    for (index, node) in skeleton.nodes.iter().enumerate() {
        skeleton.bones[index].name = write::string(
            &mut document,
            &graph.node(*node).expect("checked skeleton node").name,
        )?;
    }
    let bones = write::append(&mut document, &skeleton.bones)?.cast();
    let initial_reference = write::append(&mut document, &skeleton.inverse_bind)?.cast();
    let (tables, material_indices) = {
        let mut writer = materials::write::Writer {
            model: graph,
            resources: asset.resources(),
            document: &mut document,
            animation_ids: &ids,
            context: &mut context,
            materials: BTreeMap::new(),
        };
        let tables = materials::write::tables::encode(&mut writer)?;
        (tables, writer.materials)
    };
    let geometry = geometry_write::write(
        graph,
        &mut document,
        options.look,
        geometry_write::References {
            materials: &material_indices,
            bones: &skeleton.indices,
            animations: &ids,
        },
        &asset.encoding_hints,
        &mut context,
    )?;
    let vertices: Vec<_> = geometry
        .vertices
        .into_iter()
        .map(|value| nr::Byte { value })
        .collect();
    let vertices = write::append(&mut document, &vertices)?.cast();
    let programs = animation::write(graph, &mut document, &ids, &skeleton.nodes, &mut context)?;
    let lights = lights::write(graph, &mut document, &skeleton.indices, &ids)?;
    let cameras = cameras::write(graph, &mut document, &skeleton.indices, &ids)?;
    let (forces, warps) = forces::write(graph, &mut document, &skeleton.indices, &ids)?;
    let projectors = super::projectors::write(
        graph,
        &mut document,
        &skeleton.indices,
        &material_indices,
        &ids,
    )?;
    let emitters = super::particles::write(
        graph,
        &mut document,
        &skeleton.indices,
        &material_indices,
        &geometry.regions,
        &ids,
        &mut context,
    )?;
    let solvers = ik::write(graph, &mut document, &skeleton.indices)?;
    let scene = super::static_scene::write(graph, &mut document, &skeleton.indices, &ids)?;
    let turrets = super::turrets::write(graph, &mut document, &skeleton.indices)?;
    let volumes = super::volumes::write(graph, &mut document, &skeleton.indices)?;
    let bounds = graph
        .bounds()
        .map_or(nr::Extent::default(), |b| nr::Extent {
            minimum: b.minimum,
            maximum: b.maximum,
            radius: b.sphere_radius,
        });
    let collision = super::collision::write(graph, &mut document)?;
    let cloths = super::cloth::write(
        graph,
        &mut document,
        &geometry.regions,
        &skeleton.indices,
        &ids,
        &mut context,
    )?;
    let physics =
        super::physics::write(graph, &mut document, &skeleton.indices, &ids, &mut context)?;
    let root = P::root(RootFields {
        partitions: super::partitions::write(graph, &mut document)?,
        partition_control: graph.controls().partition_control,
        turrets: turrets.parts,
        turret_groups: turrets.groups,
        tight_hit: volumes.tight,
        fuzzy_hit: volumes.fuzzy,
        attachment_volumes: volumes.attachments,
        cloths,
        physics_joints: physics.joints,
        constraints: physics.constraints,
        rigid_bodies: physics.bodies,
        collision_positions: collision.positions,
        collision_faces: collision.faces,
        collision_normals: collision.normals,
        collision_bounds: collision.bounds,
        particles: emitters.particles,
        copies: emitters.copies,
        ribbons: emitters.ribbons,
        name: write::string(&mut document, graph.name())?,
        flags: nr::ModelFlags(graph.controls().flags.bits()),
        bones,
        vertices,
        vertex_flags: packed_vertex_flags(geometry.flags, &asset.encoding_hints)?,
        divisions: geometry.divisions,
        bone_lookup: geometry.bone_lookup,
        initial_reference,
        skin_bone_count: skeleton.skin_bone_count,
        bounds,
        material_maps: tables.map,
        standard: tables.standard,
        displacement: tables.displacement,
        composite: tables.composite,
        terrain: tables.terrain,
        volume: tables.volume,
        volume_noise: tables.volume_noise,
        creep: tables.creep,
        hair: tables.hair,
        splat_terrain_bake: tables.splat_terrain_bake,
        reflection: tables.reflection,
        lens_flare: tables.lens_flare,
        data_driven: tables.data_driven,
        sequences: programs.sequences,
        controllers: programs.controllers,
        groups: programs.groups,
        states: programs.states,
        lights,
        cameras,
        forces,
        warps,
        ik_joints: solvers.joint,
        ik_two_joints: solvers.two_joint,
        ik_ccd: solvers.ccd,
        one_bone: solvers.one_bone,
        projectors,
        attachments: scene.attachments,
        billboards: scene.billboards,
        shadow: scene.shadow,
        view_volumes: scene.volumes,
    })?;
    let root = write::append(&mut document, &[root])?;
    document.set_root(root);
    let model = native::Model::from_document(document).map_err(materials::read_error)?;
    Ok(Plan::new(native::Bundle::from_model(model), context.report))
}

// Each guard is removed together with its direct logical mapping. This keeps the
// active draft honest while unrelated payload families are migrated.
fn check_unresolved_source<E: Envelope>(
    model: &native::Model<'_, E>,
    context: &mut Context,
) -> Result<(), Error> {
    use materials::Slot;
    let root = model.root_record().map_err(materials::read_error)?;
    let fields = native::visit_m3_root!(root, |r| vec![
        ("attachment_addons", r.attachment_point_addons.entries),
        ("camera_addons", r.cameras_addons.entries),
        (
            "attachment_volume_addon0",
            Slot::<native::Ref<nr::targets::U16>>::value(&r.attachment_volumes_addon0).entries
        ),
        (
            "attachment_volume_addon1",
            Slot::<native::Ref<nr::targets::U16>>::value(&r.attachment_volumes_addon1).entries
        ),
    ]);
    let legacy_effect = native::visit_m3_root!(root, |r| Slot::<
        native::Ref<nr::targets::TrailingModel>,
    >::value(&r.trailing_models));
    if !legacy_effect.is_empty() {
        context.unsupported(
            "m3.legacy_model_effect",
            "the authored behavior of the obsolete TMD_ model effect has not been established",
        )?;
    }
    for (field, count) in fields {
        if count != 0 {
            context.unsupported(
                format!("m3.{field}"),
                "the purpose and logical destination of this populated addon table are unresolved",
            )?;
        }
    }
    Ok(())
}

fn opaque<'a, E: Envelope>(
    source: &'a native::Model<'_, E>,
    file: FileKey,
    asset: &mut Asset<'a, M3>,
    context: &mut Context,
) -> Result<(), Error> {
    for (_, section) in source.document().sections() {
        if section.tag() == E::MAGIC
            || nr::known_record_size::<E>(section.tag(), section.revision()).is_some()
        {
            continue;
        }
        let path = format!(
            "m3.{file:?}.opaque.{:?}.{}",
            section.tag(),
            section.revision()
        );
        match context.unknown {
            UnknownPolicy::Reject => {
                return Err(Error::new(path, "source section schema is unknown"));
            }
            UnknownPolicy::Drop => context.issue(
                path,
                IssueKind::Dropped,
                "uninterpreted source section discarded",
            ),
            UnknownPolicy::Archive => {
                asset.opaque.push(OpaquePayload {
                    format: Format::M3,
                    file: file.clone(),
                    tag: section.tag(),
                    revision: Some(section.revision()),
                    count: Some(section.count()),
                    bytes: Cow::Borrowed(section.bytes()),
                });
                context.issue(
                    path,
                    IssueKind::Archived,
                    "uninterpreted payload retained outside the active logical graph",
                );
            }
        }
    }
    Ok(())
}

fn packed_vertex_flags(
    flags: nr::VertexFlags,
    hints: &[EncodingHint<'_>],
) -> Result<nr::VertexFlags, Error> {
    let mut qualifier = None;
    for hint in hints {
        if let EncodingHint::M3VertexQualifiers { flags } = hint {
            if flags.bits() & !0x1c != 0 {
                return Err(Error::new(
                    "m3.vertex_qualifier.hint",
                    "qualifier hint cannot select geometry attributes",
                ));
            }
            if qualifier.is_some_and(|value| value != flags.bits()) {
                return Err(Error::new(
                    "m3.vertex_qualifier.hint",
                    "conflicting packed-vertex qualifiers",
                ));
            }
            qualifier = Some(flags.bits());
        }
    }
    Ok(nr::VertexFlags(
        (flags.bits() & !0x1c) | qualifier.unwrap_or(flags.bits() & 0x1c),
    ))
}
