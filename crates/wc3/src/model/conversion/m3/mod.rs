//! Bidirectional conversion of indexed M3 geometry, skeletons, and animation.
mod animation;
mod companions;
mod controls;
pub use controls::Controls;
mod effects;
mod geometry;
mod hierarchy;
mod hierarchy_metadata;
mod materials;
mod metadata;
use materials as material;
mod profiles;

use crate::model::m3::{self as native, records as nr};
use animation::{export_clips, import_native_clips};
pub use effects::collision::Collision;
use geometry::semantic;
use native::{Envelope, Ref};
pub use profiles::{ExportProfile, RootFields};

use crate::model::common::FileKey;
use crate::model::wem;
use std::borrow::Cow;
use std::collections::BTreeSet;

use super::{
    Asset, Context, Conversion, Error, ExportOptions, Format, ImportOptions, IssueKind,
    OpaquePayload, Plan, UnknownPolicy,
};
fn read_error(error: native::ReadError) -> Error {
    Error::new(format!("m3.bytes[{}]", error.offset), error.to_string())
}
fn write_error(error: native::WriteError) -> Error {
    Error::new("m3", error.to_string())
}
/// Imports a complete native model and its external animation closure.
pub fn import<E: Envelope>(
    source: &native::Bundle<'_, E>,
    options: &ImportOptions,
) -> Result<Conversion<Asset>, Error> {
    source.validate_complete().map_err(read_error)?;
    let mut context = Context::import(options);
    let source_metadata = source.model.root_record().map_err(read_error)?;
    let dependency = native::visit_m3_root!(source_metadata, |r| r.metadata());
    if source.animations().next().is_some()
        || dependency.m3a_anim_hash.is_some_and(|value| value != 0)
        || dependency
            .m3a_anim_hashes
            .is_some_and(|reference| reference.entries != 0)
    {
        context.issue(
            "m3.animations.dependencies",
            IssueKind::Normalized,
            "External animation tracks are materialized into editable clips; native export rebuilds inline tracks and clears companion dependency hashes",
        );
    }

    let mut model = semantic(&source.model).map_err(read_error)?;
    material::import_semantic(&source.model, &mut model, &mut context)?;
    let document = effects::attach(
        &source.model,
        material::attach(
            &source.model,
            wem::semantic::build_with_options(
                &model,
                wem::Options {
                    native_abi: Some(wem::NativeAbi::Abi64),
                },
            )?,
        )?,
    )?;
    let document = hierarchy_metadata::attach(&source.model, document)?;
    let document = geometry::attach(&source.model, document)?;
    let document = effects::physics::attach(&source.model, document, &mut context)?;
    let document = effects::cloth::attach(&source.model, document, &mut context)?;
    let mut document = metadata::decorate(&source.model, document)?;
    document = effects::collision::attach(&source.model, document, &mut context)?;
    document = controls::attach(&source.model, document)?;
    document = import_native_clips(&source.model, document, &mut context)?;
    for (hash, file) in source.animations() {
        document = match file {
            native::DynamicModel::Md33(file) => {
                companions::check(file, &source.model, hash, &mut context)?;
                import_native_clips(file, document, &mut context)?
            }
            native::DynamicModel::Md34(file) => {
                companions::check(file, &source.model, hash, &mut context)?;
                import_native_clips(file, document, &mut context)?
            }
        };
    }
    let mut raw = document.into_raw();
    raw.prune_unreachable()?;
    let mut asset = Asset::new(wem::Document::from_raw(raw)?);
    retain_unknown(&source.model, FileKey::Root, &mut asset, &mut context)?;
    for (hash, animation) in source.animations() {
        let file = FileKey::Root.sibling(format!(".animation-{hash:08x}.m3a"));
        match animation {
            native::DynamicModel::Md33(model) => {
                retain_unknown(model, file, &mut asset, &mut context)?
            }
            native::DynamicModel::Md34(model) => {
                retain_unknown(model, file, &mut asset, &mut context)?
            }
        }
    }
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}

fn retain_unknown<E: Envelope>(
    source: &native::Model<'_, E>,
    file: FileKey,
    asset: &mut Asset,
    context: &mut Context,
) -> Result<(), Error> {
    let document = source.document();
    let mut pending = vec![document.root_reference().index];
    let mut reachable = BTreeSet::new();
    while let Some(index) = pending.pop() {
        if !reachable.insert(index) {
            continue;
        }
        let section = document
            .section_id(index)
            .and_then(|id| document.section(id))
            .ok_or_else(|| Error::new("m3.sections", "missing referenced section"))?;
        nr::validate_known_section::<E>(
            section.tag(),
            section.revision(),
            section.bytes(),
            section.count(),
            &mut |reference| {
                if reference.entries != 0 {
                    pending.push(reference.index);
                }
                Ok(())
            },
        )
        .map_err(read_error)?;
    }
    for (id, section) in document.sections() {
        if section.tag() != E::MAGIC
            && section.count() != 0
            && nr::known_record_size::<E>(section.tag(), section.revision()).is_some()
            && !reachable.contains(&id.index())
        {
            context.issue(
                format!("m3.{file:?}.sections[{}]", id.index()),
                IssueKind::Normalized,
                "Unreachable known section storage is discarded when rebuilding the active model graph",
            );
        }
    }
    for (id, section) in source.document().sections() {
        if section.tag() != E::MAGIC
            && native::records::known_record_size::<E>(section.tag(), section.revision()).is_none()
        {
            match context.unknown {
                UnknownPolicy::Reject => {
                    return Err(Error::new(
                        format!("m3.sections[{}]", id.index()),
                        "unrecognized native section",
                    ));
                }
                UnknownPolicy::Drop => context.issue(
                    format!("m3.sections[{}]", id.index()),
                    IssueKind::Dropped,
                    "unknown section discarded",
                ),
                UnknownPolicy::Archive => {
                    asset.opaque.push(OpaquePayload {
                        format: Format::M3,
                        file: file.clone(),
                        tag: section.tag(),
                        revision: Some(section.revision()),
                        count: Some(section.count()),
                        bytes: Cow::Owned(section.bytes().to_vec()),
                    });
                    context.issue(
                        format!("m3.sections[{}]", id.index()),
                        IssueKind::Archived,
                        "unknown section retained as inactive opaque payload",
                    )
                }
            }
        }
    }
    Ok(())
}

fn append<E: Envelope, T: native::Record + native::Write<E> + native::RecordLayout<E>>(
    document: &mut native::Document<'static, E>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let id = document.append(rows).map_err(write_error)?;
    Ok(Ref::from_section(rows.len() as u32, id.index()))
}
fn append_string<E: Envelope>(
    document: &mut native::Document<'static, E>,
    text: &str,
) -> Result<Ref<nr::targets::Character>, Error> {
    let mut bytes = text
        .as_bytes()
        .iter()
        .map(|value| nr::Character { value: *value })
        .collect::<Vec<_>>();
    bytes.push(nr::Character { value: 0 });
    Ok(append(document, &bytes)?.cast())
}
fn quantize(
    value: f32,
    scale: f32,
    min: f32,
    max: f32,
    path: &str,
    context: &mut Context,
) -> Result<f32, Error> {
    if !value.is_finite() {
        return Err(Error::new(path, "non-finite packed value"));
    }
    let quantized = (value * scale).round().clamp(min, max) / scale;
    context.approximate(
        path,
        (quantized - value).abs(),
        "value requires quantization to the native packed representation",
    )?;
    Ok(quantized)
}
fn signed_unorm(value: f32, path: &str, context: &mut Context) -> Result<f32, Error> {
    if !value.is_finite() {
        return Err(Error::new(path, "non-finite packed direction"));
    }
    let encoded = ((value + 1.) * 127.5).round().clamp(0., 255.);
    let decoded = encoded / 255. * 2. - 1.;
    context.approximate(
        path,
        (decoded - value).abs(),
        "direction requires quantization to the native packed representation",
    )?;
    Ok(encoded / 255.)
}
/// Inspects and constructs a complete native model bundle under a creation profile.
pub fn plan<P: ExportProfile>(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<Plan<native::Bundle<'static, P::Envelope>>, Error> {
    let mut context = Context::export(options);
    let model = asset.model(options)?;
    if model.space != wem::records::CoordSpace::SC2 {
        context.issue("document.space",IssueKind::Normalized,"M3 uses the SC2 authoring-space tag; authored numeric coordinates are preserved and no basis transform is applied");
    }
    if model.unit_scale != 1. {
        context.unsupported("document.unit_scale","M3 has no document unit-scale metadata; authored component values are preserved without rescaling")?;
    }
    if model.nodes.len() > u16::MAX as usize {
        return Err(Error::new("nodes", "M3 bone index capacity exceeded"));
    }
    let mut document = native::Document::<P::Envelope>::new();
    let mut geometry = geometry::encode(&model, &mut document, &mut context)?;
    geometry.divisions = controls::restore_divisions(
        &model,
        &mut document,
        geometry.divisions,
        &hierarchy::mapping(&model)?,
        &mut context,
    )?;
    let (mut bones, initial) = hierarchy::encode(&model, &mut document, &mut context)?;
    let animation = export_clips(&model, &mut document, &mut bones, &mut context)?;
    let bone_ref = hierarchy_metadata::append_bones(&model, &mut document, &bones)?;
    let initial_ref = append(&mut document, &initial)?.cast();
    let materials = material::export(
        &model,
        &mut document,
        &mut context,
        <P::Root as native::Record>::VERSION,
    )?;
    let name = append_string(&mut document, &model.name)?;
    let collision = effects::collision::export(
        &model,
        &mut document,
        &hierarchy::mapping(&model)?,
        &mut context,
    )?;
    let effects = effects::export(&model, &mut document, &mut context)?;
    let metadata = metadata::values(&model)?;
    let cloth = effects::cloth::export(
        &model,
        &mut document,
        &hierarchy::mapping(&model)?,
        &mut context,
    )?;
    if <P::Root as native::Record>::VERSION < 28 && cloth.entries != 0 {
        context.unsupported(
            "physics.cloth",
            "selected native root revision omits cloth simulation",
        )?;
    }
    let root = P::root(RootFields {
        cloth,
        projections: effects.projections,
        controls: controls::export(
            &model,
            &mut document,
            &hierarchy::mapping(&model)?,
            &mut context,
        )?,
        animation_split_count: metadata
            .get("m3.model.animation_split_count")
            .copied()
            .unwrap_or(0) as u32,
        flags: metadata.get("m3.model.flags").copied().unwrap_or(0) as u32,
        rigid_bodies: effects.rigid_bodies,
        physics_joints: effects.physics_joints,
        collision,
        particles: effects.particles,
        particle_copies: effects.particle_copies,
        ribbons: effects.ribbons,
        attachments: effects.attachments,
        cameras: effects.cameras,
        lights: effects.lights,
        forces: effects.forces,
        warps: effects.warps,
        name,
        bones: bone_ref,
        vertices: geometry.vertices,
        vertex_flags: geometry.flags,
        divisions: geometry.divisions,
        bone_lookup: geometry.palette,
        initial_reference: initial_ref,
        sequences: animation.0,
        animation_groups: animation.1,
        sub_track_collections: animation.2,
        animation_states: animation.3,
        skin_bone_count: metadata
            .get("m3.model.skin_bone_count")
            .copied()
            .map(|v| v as u32)
            .unwrap_or_else(|| model.nodes.iter().filter(|n| n.bone).count() as u32),
        bounds: nr::Extent {
            minimum: model.bounds.minimum,
            maximum: model.bounds.maximum,
            radius: model.bounds.sphere_radius,
        },
        ..materials
    });
    let root_ref = append(&mut document, &[root])?;
    document.set_root(root_ref);
    let bundle =
        native::Bundle::from_model(native::Model::from_document(document).map_err(read_error)?);
    bundle.write().map_err(write_error)?;
    Ok(Plan::new(bundle, context.report))
}
fn revision<E: Envelope, T: native::Target>(
    document: &native::Document<'_, E>,
    reference: Ref<T>,
) -> Result<u32, native::ReadError> {
    document
        .section_id(reference.index)
        .and_then(|id| document.section(id))
        .map(|section| section.revision())
        .ok_or_else(|| native::ReadError::invalid(0, "missing referenced section"))
}

fn string<E: Envelope>(
    document: &native::Document<'_, E>,
    reference: Ref<nr::targets::Character>,
) -> Result<String, native::ReadError> {
    Ok(document.string(reference)?.unwrap_or_default().to_owned())
}
