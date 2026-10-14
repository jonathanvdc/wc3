//! All dependencies are resolved before the collection is published.
use super::super::{
    animation::{programs, routing, targets},
    document,
};
use super::{models, prepare};
use crate::model::conversion::{Asset, Conversion, Error, ImportOptions};
use crate::model::semantic::profiles::Wem;
use crate::model::wem::RawDocument;

pub(in crate::model::conversion::wem) fn read<'view>(
    source: &'view RawDocument<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Asset<'view, Wem>>, Error> {
    let mut prepared = prepare::read(source, options)?;
    let programs = programs::prepare_clips(
        source,
        prepared.root.clips,
        &mut prepared.builder,
        &mut prepared.context,
    )?;
    let mut hints = Vec::new();
    let decoded = models::read(source, &mut prepared, &programs.clip_ids, &mut hints)?;
    let controls = document::read(source, &prepared.root, &programs.clip_ids)?;
    let sets: Vec<_> = controls.animation_sets().map(|(id, _)| id).collect();
    for (index, value) in decoded.iter().enumerate() {
        let native = &prepared.models[index];
        let builder = prepared
            .builder
            .model_mut(index)
            .ok_or_else(|| Error::new("wem.models", "model draft is absent"))?;
        let mut model_controls = builder.model().controls().clone();
        model_controls.animation_set =
            if native.animation_set == u32::MAX {
                None
            } else {
                Some(*sets.get(native.animation_set as usize).ok_or_else(|| {
                    Error::new("wem.animation_set", "default selection is absent")
                })?)
            };
        builder.set_controls(model_controls);
        let route = routing::ReadContext {
            simulation: targets::ReadContext {
                nodes: &value.nodes,
                physics: value.physics.as_ref().map(|(id, physics)| (*id, physics)),
                stages: &value.stages,
            },
            model: builder.model(),
            meshes: &value.meshes,
            sections: &value.sections,
            slots: &value.appearance.slots,
        };
        let programme = programs::decode_model(
            source,
            &programs,
            programs::SourceTables {
                model: index,
                channels: native.channels,
                selections: native.selections.unwrap_or_default(),
            },
            &route,
            &prepared.textures,
            &mut prepared.context,
        )?;
        hints.extend(programs::install(programme, builder)?);
    }
    prepared.builder.set_collection_controls(controls);
    let asset = Asset {
        models: prepared.builder.finish()?,
        opaque: prepared.opaque,
        encoding_hints: hints,
    };
    asset.validate()?;
    Ok(Conversion {
        value: asset,
        report: prepared.context.report,
    })
}
