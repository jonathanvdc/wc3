//! Current authoring objects determine every destination reference.
use super::super::{
    animation::{programs, routing, targets},
    document, hierarchy,
};
use super::{emit, model_records, resources};
use crate::model::conversion::{
    Asset, Context, Error, ExportOptions, Format, IssueKind, Plan, UnknownPolicy,
};
use crate::model::semantic::profiles::Wem;
use crate::model::wem::{records, NativeAbi, Options, RawDocument, Ref};
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) fn write<'view, 'data: 'view>(
    asset: &'view Asset<'data, Wem>,
    options: &ExportOptions,
) -> Result<Plan<RawDocument<'view>>, Error> {
    asset.validate()?;
    if options.model != 0 || options.look.is_some() {
        return Err(Error::new(
            "wem.options",
            "native document export emits its complete model collection and authored appearances",
        ));
    }
    let mut context = Context::export(options);
    let mut destination = RawDocument::new_with_options(Options {
        native_abi: Some(NativeAbi::Abi64),
    });
    let clips: BTreeMap<_, _> = asset
        .models
        .iter()
        .flat_map(|model| model.clips().map(|(id, _)| id))
        .enumerate()
        .map(|(index, id)| {
            u32::try_from(index)
                .map(|index| (id, index))
                .map_err(|error| Error::new("wem.clips", error.to_string()))
        })
        .collect::<Result<_, _>>()?;
    let (mut root, sets) = document::write(&mut destination, asset.controls(), &clips)?;
    let textures = resources::write(&mut destination, asset.resources())?;
    let mut models = emit::write(
        &mut destination,
        asset,
        &textures,
        &clips,
        &sets,
        &asset.encoding_hints,
    )?;
    let mut drafts = Vec::new();
    for (index, (model, encoded)) in asset.models.iter().zip(models.iter_mut()).enumerate() {
        let route = routing::WriteContext {
            simulation: targets::WriteContext {
                nodes: &encoded.maps.nodes,
                physics: encoded.physics.as_ref().map(|(id, value)| (*id, value)),
                stages: &encoded.stages,
                pipeline: &model.controls().pose_pipeline,
            },
            model,
            meshes: &encoded.maps.meshes,
            sections: &encoded.maps.sections,
            slots: &encoded.appearance.slot_indices,
            features: &encoded.maps.feature_nodes,
        };
        let programs = programs::write_model(
            &mut destination,
            model,
            &route,
            &textures.indices,
            &mut context,
        )?;
        encoded.record.channels = programs.channels;
        encoded.record.selections = programs.selections;
        let index =
            u32::try_from(index).map_err(|error| Error::new("wem.models", error.to_string()))?;
        drafts.extend(programs.clips.into_iter().map(|clip| (index, clip)));
    }
    let (clip_reference, actual_clips) = programs::write_clips(&mut destination, drafts)?;
    if actual_clips != clips {
        return Err(Error::new(
            "wem.clips",
            "destination programme ordering differs from reserved clip identities",
        ));
    }
    let records: Vec<_> = models.into_iter().map(|value| value.record).collect();
    root.models = model_records::write(&mut destination, &records)?;
    root.textures = textures.reference;
    root.clips = clip_reference;
    let root_reference: Ref<records::DocumentRecord<records::V1>> =
        hierarchy::poses::put(&mut destination, &[root])?;
    destination.set_root(root_reference);
    for payload in &asset.opaque {
        match options.unknown {
            UnknownPolicy::Reject => {
                return Err(Error::new(
                    "wem.opaque",
                    "uninterpreted source payload requires an explicit export policy",
                ));
            }
            UnknownPolicy::Drop => context.issue(
                "wem.opaque",
                IssueKind::Dropped,
                "uninterpreted source section omitted",
            ),
            UnknownPolicy::Archive if payload.format == Format::Wem => {
                destination
                    .append_data(
                        payload.tag,
                        payload.revision.unwrap_or(0),
                        payload.count.unwrap_or(0),
                        payload.bytes.clone(),
                    )
                    .map_err(|error| Error::new("wem.opaque", error.to_string()))?;
                context.issue(
                    "wem.opaque",
                    IssueKind::Archived,
                    "uninterpreted payload retained as a separate native section",
                );
            }
            UnknownPolicy::Archive => {
                return Err(Error::new(
                    "wem.opaque",
                    "foreign format payload cannot be inserted into this document",
                ));
            }
        }
    }
    destination
        .validate()
        .map_err(|error| Error::new("wem", error.to_string()))?;
    Ok(Plan::new(destination, context.report))
}
