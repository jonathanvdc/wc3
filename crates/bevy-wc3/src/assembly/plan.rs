use super::{Models, PendingAssembly, PendingPart, Source};
use crate::{texture_requests, DrawSelection, ModelError};
use std::collections::{HashMap, HashSet};
use wc3::model::{MaterialSelection, Model};

fn source_model<'view, 'data>(
    source: &Source,
    models: &Models<'view, 'data>,
) -> Result<&'view Model<'data>, ModelError> {
    models
        .get(&source.model)
        .copied()
        .ok_or_else(|| ModelError::new(format!("missing assembly model {:?}", source.model)))
}
fn validate_source(source: &Source, model: &Model<'_>) -> Result<(), ModelError> {
    if matches!(source.material, MaterialSelection::Draw(look) if model.look(look).is_none()) {
        return Err(ModelError::new(
            "assembly selection references an absent or foreign material look",
        ));
    }
    let draws: HashSet<_> = model
        .instances()
        .iter()
        .flat_map(|(_, instance)| instance.draws.iter().map(|d| d.id()))
        .collect();
    if source
        .draws
        .draws
        .as_ref()
        .is_some_and(|selected| !selected.is_subset(&draws))
    {
        return Err(ModelError::new(
            "assembly selection references an absent draw",
        ));
    }
    for &id in &source.draws.hidden_instances {
        if model.instance(id).is_none() {
            return Err(ModelError::new(
                "assembly selection references an absent instance",
            ));
        }
    }
    for (id, replacement) in &source.images {
        if model.image_resource(*id).is_none() {
            return Err(ModelError::new(
                "assembly replacement references an absent image resource",
            ));
        }
        replacement.validate()?;
    }
    let requests = texture_requests(model);
    for (input, replacement) in &source.inputs {
        if !requests.iter().any(|request| request.input == Some(*input)) {
            return Err(ModelError::new(
                "assembly replacement references an absent image input",
            ));
        }
        replacement.validate()?;
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
pub(super) fn plan<B: Clone + PartialEq>(
    assembly: &PendingAssembly<B>,
    models: &Models<'_, '_>,
    is_skinned: impl Fn(&B) -> bool,
) -> Result<Vec<PendingPart<B>>, ModelError> {
    validate_source(&assembly.root, source_model(&assembly.root, models)?)?;
    // Validate every original request before a union can subsume it.
    for part in &assembly.parts {
        let model = source_model(&part.source, models)?;
        validate_source(&part.source, model)?;
        for (host, child) in &part.clips {
            if source_model(&assembly.root, models)?.clip(*host).is_none()
                || model.clip(*child).is_none()
            {
                return Err(ModelError::new(
                    "assembly clip mapping references an absent clip",
                ));
            }
        }
    }
    let mut output: Vec<PendingPart<B>> = Vec::with_capacity(assembly.parts.len());
    let mut shared = HashMap::new();
    for part in &assembly.parts {
        let mut part = part.clone();
        if part.source.material == MaterialSelection::Default {
            part.source.material = source_model(&part.source, models)?
                .default_look()
                .map_or(MaterialSelection::Default, MaterialSelection::Draw);
        }
        if !is_skinned(&part.binding) {
            output.push(part);
            continue;
        }
        if let Some(&index) = shared.get(&part.source.model) {
            let existing: &mut PendingPart<B> = &mut output[index];
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
                return Err(ModelError::new(format!(
                    "shared assembly resource {:?} has incompatible {conflict}",
                    part.source.model
                )));
            }
            existing.source.draws = union(
                &existing.source.draws,
                &part.source.draws,
                source_model(&part.source, models)?,
            );
        } else {
            shared.insert(part.source.model.clone(), output.len());
            output.push(part);
        }
    }
    Ok(output)
}
