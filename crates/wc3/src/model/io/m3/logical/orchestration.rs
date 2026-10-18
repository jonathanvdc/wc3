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
    Context, Conversion, Error, ExportOptions, ImportOptions, IssueKind, Plan,
};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::Model;
use crate::model::{profiles::m3 as surface, profiles::M3};
use native::Envelope;
use std::collections::BTreeMap;

pub(in crate::model::io::m3) fn read<'a, E: Envelope + Send + Sync>(
    source: &'a native::Bundle<'_, E>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, M3>>, Error> {
    source.validate_complete().map_err(materials::read_error)?;
    let root = source.model.root_record().map_err(materials::read_error)?;
    let name = native::visit_m3_root!(root, |r| r.name);
    let mut builder = GraphBuilder::with_profile(surface::ModelMetadata::default());
    let flags = native::visit_m3_root!(root, |r| r.flags.bits());
    builder.set_metadata(surface::ModelMetadata {
        name: materials::string(source.model.container(), name)?,
        bounds: Default::default(),
        flags: surface::values::ModelFlags(flags),
        partition_control: native::visit_m3_root!(root, |r| r.animation_split_count),
    });
    let mut context = Context::import(options);
    super::qualifiers::check(source.model.container(), &mut context)?;
    super::partitions::read(
        source.model.container(),
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
        source.model.container(),
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
        source.model.container(),
        light_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    cameras::read(
        source.model.container(),
        camera_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    forces::force(
        source.model.container(),
        force_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    forces::warp(
        source.model.container(),
        warp_ref,
        &mut builder,
        &skeleton,
        &mut bindings,
    )?;
    let projector_ref = native::visit_m3_root!(root, |r| r.projections);
    let projectors = super::projectors::read(
        source.model.container(),
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
        source.model.container(),
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
        source.model.container(),
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
            native::DynamicDocument::Md33(model) => {
                super::qualifiers::check(model.container(), &mut context)?;
                super::companions::check(&source.model, model, &mut context)?;
                animation::read(
                    model,
                    &skeleton.nodes,
                    &bindings,
                    &mut builder,
                    &mut context,
                )?;
            }
            native::DynamicDocument::Md34(model) => {
                super::qualifiers::check(model.container(), &mut context)?;
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
    let asset = builder.finish()?;
    if native::visit_m3_root!(root, |r| r.vertex_flags.bits()) & 0x1c != 0 {
        context.unknown_data(
            "m3.vertex_qualifier",
            "packed vertex qualifier bits have no semantic representation",
        )?;
    }
    check_unknown_sections(&source.model, FileKey::Root, &mut context)?;
    for (hash, file) in source.animations() {
        let file_key = FileKey::Root.sibling(format!(".animation-{hash:08x}.m3a"));
        match file {
            native::DynamicDocument::Md33(model) => {
                check_unknown_sections(model, file_key, &mut context)?
            }
            native::DynamicDocument::Md34(model) => {
                check_unknown_sections(model, file_key, &mut context)?
            }
        }
    }
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
pub(in crate::model::io::m3) fn plan<'view, 'source: 'view, P: ExportProfile>(
    asset: &'view Model<'source, M3>,
    options: &ExportOptions,
) -> Result<Plan<native::Bundle<'view, P::Envelope>>, Error> {
    asset.validate()?;
    let graph = asset;
    let mut context = Context::export(options);

    let ids = animation::prepare(graph, &mut context)?;
    let mut document = native::Container::<P::Envelope>::new();
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
    let authored_bounds = graph.bounds();
    let bounds = nr::Extent {
        minimum: authored_bounds.minimum,
        maximum: authored_bounds.maximum,
        radius: authored_bounds.sphere_radius,
    };
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
        partition_control: graph.metadata().partition_control,
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
        flags: nr::ModelFlags(graph.metadata().flags.bits()),
        bones,
        vertices,
        vertex_flags: geometry.flags,
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
    let model = native::Document::from_container(document).map_err(materials::read_error)?;
    Ok(Plan::new(native::Bundle::from_model(model), context.report))
}

// Each guard is removed together with its direct logical mapping. This keeps the
// active draft honest while unrelated payload families are migrated.
fn check_unresolved_source<E: Envelope>(
    model: &native::Document<'_, E>,
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

fn check_unknown_sections<E: Envelope>(
    source: &native::Document<'_, E>,
    file: FileKey,
    context: &mut Context,
) -> Result<(), Error> {
    for (_, section) in source.container().sections() {
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
        context.unknown_data(path, "source section schema is unknown")?;
    }
    Ok(())
}
