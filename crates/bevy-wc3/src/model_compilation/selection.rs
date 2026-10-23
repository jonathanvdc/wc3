//! Resource closure for the application's authored draw selection.
use super::CompileOptions;
use crate::ModelError;
use std::collections::HashSet;
use wc3::model::profiles::Universal;
use wc3::model::{Link, MaterialId, Model, Texture, ValidateLinks, ValueSlot};

pub(super) fn materials(
    model: &Model<'_, Universal>,
    options: &CompileOptions,
) -> Result<Option<HashSet<MaterialId>>, ModelError> {
    Ok(
        if options.draws.draws.is_some() || !options.draws.hidden_instances.is_empty() {
            let mut selected = HashSet::new();
            for (id, instance) in model.instances() {
                if options.draws.hidden_instances.contains(&id) {
                    continue;
                }
                for draw in instance
                    .draws
                    .iter()
                    .filter(|draw| options.draws.includes(draw.id()))
                {
                    selected.insert(
                        model
                            .resolve_material(id, draw.id(), options.material)
                            .map_err(|e| ModelError(e.to_string()))?,
                    );
                }
            }
            for (_, feature) in model.features() {
                feature.payload.visit_links(&mut |link| {
                    if let Link::Material(id) = link {
                        selected.insert(id);
                    }
                });
            }
            for (_, system) in model.systems() {
                system.visit_links(&mut |link| {
                    if let Link::Material(id) = link {
                        selected.insert(id);
                    }
                });
            }
            Some(selected)
        } else {
            None
        },
    )
}

pub(super) fn textures(
    model: &Model<'_, Universal>,
    selected_materials: &Option<HashSet<MaterialId>>,
) -> Option<HashSet<Texture>> {
    selected_materials
        .as_ref()
        .map(|materials: &HashSet<MaterialId>| {
            let mut selected = HashSet::<Texture>::new();
            let mut animated = false;
            let mut visit = |link| match link {
                Link::SampledImage(id) => {
                    selected.insert(id);
                }
                Link::ImageResource(id) => {
                    selected.insert(Texture::new(id));
                }
                Link::Value(ValueSlot::Texture(_) | ValueSlot::OptionalTexture(_)) => {
                    animated = true;
                }
                _ => {}
            };
            for (_, feature) in model.features() {
                feature.payload.visit_links(&mut visit);
            }
            for (_, system) in model.systems() {
                system.visit_links(&mut visit);
            }
            for (id, material) in model.materials() {
                if materials.contains(&id) {
                    material.surface.visit_links(&mut visit);
                }
            }
            if animated {
                selected.extend(model.sampled_images());
            }
            selected
        })
}
