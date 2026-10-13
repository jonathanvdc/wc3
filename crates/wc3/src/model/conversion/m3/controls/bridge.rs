//! Native control graph construction and named WEM profile fields.
use super::*;
/// Native behavior arrays reconstructed from active WEM profile controls.
#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    /// Per-node view-volume records.
    pub view_volumes: Ref<nr::targets::ViewVolume>,
    /// Turret behavior entries.
    pub turret_behaviors: Ref<nr::targets::TurretBehavior>,
    /// Secondary animated behavior controls.
    pub trailing_models: Ref<nr::targets::TrailingModel>,
    /// Ik joints native records.
    pub ik_joints: Ref<nr::targets::IKJoint>,
    /// Ik two joints native records.
    pub ik_two_joints: Ref<nr::targets::IKTwoJoint>,
    /// Ik c c d native records.
    pub ik_c_c_d: Ref<nr::targets::IKCCD>,
    /// One bone solvers native records.
    pub one_bone_solvers: Ref<nr::targets::OneBoneSolver>,
    /// Billboard behaviors native records.
    pub billboard_behaviors: Ref<nr::targets::BillboardBehavior>,
    /// Shadow boxes native records.
    pub shadow_boxes: Ref<nr::targets::ShadowBox>,
    /// Bone animation sets native records.
    pub bone_animation_sets: Ref<nr::targets::BoneAnimationSet>,
    /// Trigger data native records.
    pub trigger_data: Ref<nr::targets::TriggerData>,
}
pub(in crate::model::conversion::m3) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
) -> Result<wem::Document<'static>, Error> {
    let mut fields = Fields::new();
    mesh_sections::import(source, &mut fields)?;
    let source_doc = source.document();
    native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| {
        families::import_ik_joints(source_doc, r.ik_joints, &mut fields)?;
        families::import_ik_two_joints(source_doc, r.ik_two_joints, &mut fields)?;
        families::import_ik_c_c_d(source_doc, Optional::reference(&r.ik_c_c_d), &mut fields)?;
        families::import_one_bone_solvers(source_doc, r.one_bone_solvers, &mut fields)?;
        families::import_billboard_behaviors(source_doc, r.billboard_behaviors, &mut fields)?;
        families::import_shadow_boxes(source_doc, r.shadow_boxes, &mut fields)?;
        families::import_bone_animation_sets(source_doc, r.bone_animation_sets, &mut fields)?;
        families::import_trigger_data(source_doc, r.trigger_data, &mut fields)?;
        view_volumes::import(
            source_doc,
            Optional::reference(&r.view_volumes),
            &mut fields,
        )?;
        extras::import_turret_behaviors(source_doc, r.turret_behaviors, &mut fields)?;
        extras::import_trailing_models(
            source_doc,
            Optional::reference(&r.trailing_models),
            &mut fields,
        )?;
        Ok::<(), Error>(())
    })?;
    let mut doc = document.into_raw();
    let root = doc
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::from_section(
            doc.root_reference().entries,
            doc.root_reference().index,
        ))?
        .get(0)
        .ok_or_else(|| Error::new("wem.root", "missing root"))??;
    let model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut sets = doc
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let set = sets
        .iter_mut()
        .find(|s| s.profile == wr::ProfileId::SC2)
        .ok_or_else(|| Error::new("wem.profile", "missing SC2 profile"))?;
    set.native = metadata::extend(&mut doc, &set.native, &fields.values)?;
    for (key, value) in fields.texts {
        set.native = metadata::set_text(&mut doc, &set.native, &key, &value)?;
    }
    let id = doc
        .section_id(model.profile_sets.index)
        .ok_or_else(|| Error::new("wem.profile", "missing profiles"))?;
    doc.replace(id, &sets)?;
    Ok(wem::Document::from_raw(doc)?)
}
pub(in crate::model::conversion::m3) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Controls, Error> {
    let Some(mut fields) = view(model)? else {
        return Ok(Controls::default());
    };
    let modes = super::super::animation::modes(model, context)?;
    super::super::animation::reconcile_controls(&mut fields.values, &modes);
    Ok(Controls {
        view_volumes: view_volumes::export(&fields, doc, bone_map)?,
        turret_behaviors: extras::export_turret_behaviors(&fields, doc, bone_map, context)?,
        trailing_models: extras::export_trailing_models(&fields, doc, bone_map)?,
        ik_joints: families::export_ik_joints(&fields, doc, bone_map)?,
        ik_two_joints: families::export_ik_two_joints(&fields, doc, bone_map)?,
        ik_c_c_d: families::export_ik_c_c_d(&fields, doc, bone_map)?,
        one_bone_solvers: families::export_one_bone_solvers(&fields, doc, bone_map)?,
        billboard_behaviors: families::export_billboard_behaviors(&fields, doc, bone_map)?,
        shadow_boxes: families::export_shadow_boxes(&fields, doc, bone_map)?,
        bone_animation_sets: families::export_bone_animation_sets(&fields, doc, bone_map)?,
        trigger_data: families::export_trigger_data(&fields, doc, bone_map)?,
    })
}

fn view(model: &sem::Model<'_>) -> Result<Option<View>, Error> {
    let Some(native) = &model.native else {
        return Ok(None);
    };
    let source = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = source
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let sets = source
        .resolve::<wr::ProfileMaterialSet>(record.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let Some(set) = sets.iter().find(|s| s.profile == model.profile) else {
        return Ok(None);
    };
    let mut fields = View {
        values: metadata::read(source, &set.native)?,
        texts: BTreeMap::new(),
    };
    for row in source
        .resolve::<wr::NativeEntry<wr::V2>>(set.native.entries.cast())?
        .iter()
    {
        let row = row?;
        let name = source.string(row.name)?.unwrap_or_default();
        if row.text.entries != 0 {
            fields.texts.insert(
                name.to_owned(),
                source.string(row.text)?.unwrap_or_default().to_owned(),
            );
        }
    }
    Ok(Some(fields))
}

pub(in crate::model::conversion::m3) fn restore_divisions<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    reference: Ref<nr::targets::MeshDivision>,
    map: &[u32],
    context: &mut Context,
) -> Result<Ref<nr::targets::MeshDivision>, Error> {
    let Some(mut fields) = view(model)? else {
        return Ok(reference);
    };
    let modes = super::super::animation::modes(model, context)?;
    super::super::animation::reconcile_controls(&mut fields.values, &modes);
    mesh_sections::export(&fields, doc, reference, map)
}
