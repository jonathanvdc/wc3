use super::{Models, PendingAssembly, PendingPart, SelectionError, Source};
use crate::{
    texture_requests, DrawSelection, ModelError, PreparationContext, PreparationOperation,
};
use std::collections::{HashMap, HashSet};
use wc3::model::{MaterialSelection, Model};

fn source_model<'view, 'data>(
    source: &Source,
    models: &Models<'view, 'data>,
) -> Result<&'view Model<'data>, ModelError> {
    models
        .get(&source.model)
        .copied()
        .ok_or_else(|| ModelError::from_source(SelectionError::MissingModel))
}
fn validate_source(source: &Source, model: &Model<'_>) -> Result<(), ModelError> {
    if let MaterialSelection::Draw(look) = source.material {
        if model.look(look).is_none() {
            return Err(ModelError::from_source(SelectionError::MaterialLook(look)));
        }
    }
    let draws: HashSet<_> = model
        .instances()
        .iter()
        .flat_map(|(_, instance)| instance.draws.iter().map(|d| d.id()))
        .collect();
    if let Some(id) = source
        .draws
        .draws
        .as_ref()
        .and_then(|selected| selected.difference(&draws).next())
    {
        return Err(ModelError::from_source(SelectionError::Draw(*id)));
    }
    for &id in &source.draws.hidden_instances {
        if model.instance(id).is_none() {
            return Err(ModelError::from_source(SelectionError::Instance(id)));
        }
    }
    for (id, replacement) in &source.images {
        if model.image_resource(*id).is_none() {
            return Err(ModelError::from_source(SelectionError::ImageResource(*id)));
        }
        replacement
            .validate()
            .map_err(|error| error.with_context(PreparationContext::ImageDependency(*id)))?;
    }
    let requests = texture_requests(model);
    for (input, replacement) in &source.inputs {
        if !requests.iter().any(|request| request.input == Some(*input)) {
            return Err(ModelError::from_source(SelectionError::ImageInput(*input)));
        }
        replacement
            .validate()
            .map_err(|error| error.with_context(PreparationContext::ImageInput(*input)))?;
    }
    Ok(())
}
fn union(a: &DrawSelection, b: &DrawSelection, model: &Model<'_>) -> DrawSelection {
    // Draw identities belong to their instance. Normalize before union so a draw
    // hidden by one request cannot appear merely because another request unmasks
    // that instance. Whole-instance native geometry is selected by visibility.
    let draws = model
        .instances()
        .iter()
        .flat_map(|(id, instance)| {
            instance
                .draws
                .iter()
                .filter(move |draw| {
                    (!a.hidden_instances.contains(&id) && a.includes(draw.id()))
                        || (!b.hidden_instances.contains(&id) && b.includes(draw.id()))
                })
                .map(|d| d.id())
        })
        .collect();
    DrawSelection {
        draws: Some(draws),
        hidden_instances: a
            .hidden_instances
            .intersection(&b.hidden_instances)
            .copied()
            .collect(),
    }
}
pub(super) struct PlannedPart<B> {
    pub(super) index: usize,
    pub(super) part: PendingPart<B>,
}

pub(super) fn plan<B: Clone + PartialEq>(
    assembly: &PendingAssembly<B>,
    models: &Models<'_, '_>,
    is_skinned: impl Fn(&B) -> bool,
) -> Result<Vec<PlannedPart<B>>, ModelError> {
    let validate_root = |error: ModelError| {
        error
            .with_context(PreparationContext::Operation(
                PreparationOperation::Validate,
            ))
            .with_context(PreparationContext::Model(assembly.root.model.clone()))
    };
    let root = source_model(&assembly.root, models).map_err(validate_root)?;
    validate_source(&assembly.root, root).map_err(validate_root)?;
    // Validate every original request before a union can subsume it.
    for (index, part) in assembly.parts.iter().enumerate() {
        let context = |error: ModelError| part_error(error, index, &part.source);
        let model = source_model(&part.source, models).map_err(context)?;
        validate_source(&part.source, model).map_err(context)?;
        for (&host, &child) in &part.clips {
            if root.clip(host).is_none() || model.clip(child).is_none() {
                return Err(context(ModelError::from_source(
                    SelectionError::ClipMapping { host, child },
                )));
            }
        }
    }
    let mut output: Vec<PlannedPart<B>> = Vec::with_capacity(assembly.parts.len());
    let mut shared = HashMap::new();
    for (original_index, part) in assembly.parts.iter().enumerate() {
        let mut part = part.clone();
        if part.source.material == MaterialSelection::Default {
            part.source.material = source_model(&part.source, models)?
                .default_look()
                .map_or(MaterialSelection::Default, MaterialSelection::Draw);
        }
        if !is_skinned(&part.binding) {
            output.push(PlannedPart {
                index: original_index,
                part,
            });
            continue;
        }
        if let Some(&index) = shared.get(&part.source.model) {
            let existing: &mut PlannedPart<B> = &mut output[index];
            let previous_index = existing.index;
            let existing = &mut existing.part;
            let conflict = if existing.binding != part.binding {
                Some("skeleton bindings")
            } else if existing.clips != part.clips {
                Some("clip mappings")
            } else if existing.source.material != part.source.material {
                Some("material selections")
            } else if existing.source.images != part.source.images
                || existing.source.inputs != part.source.inputs
            {
                Some("replacement textures")
            } else {
                None
            };
            if let Some(conflict) = conflict {
                return Err(part_error(ModelError::new(format!(
                    "shared assembly resource has incompatible {conflict} with part {previous_index}",
                )), original_index, &part.source));
            }
            existing.source.draws = union(
                &existing.source.draws,
                &part.source.draws,
                source_model(&part.source, models)?,
            );
        } else {
            shared.insert(part.source.model.clone(), output.len());
            output.push(PlannedPart {
                index: original_index,
                part,
            });
        }
    }
    Ok(output)
}

fn part_error(error: ModelError, index: usize, source: &Source) -> ModelError {
    error
        .with_context(PreparationContext::Operation(
            PreparationOperation::Validate,
        ))
        .with_context(PreparationContext::Part {
            index,
            model: source.model.clone(),
        })
}
