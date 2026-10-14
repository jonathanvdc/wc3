//! Authored family alternatives resolve material slots through scoped identities.
use super::{document, material_records, metadata, text};
use crate::model::conversion::{EncodingHint, Error};
use crate::model::semantic::profiles::wem::{Appearance, AppearanceLook, AppearanceSet};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{MaterialSlotId, Model, ModelBuilder, TextureId};
use crate::model::wem::{
    self, records, InlineList, RawDocument, Record, RecordLayout, Ref, Target,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

pub(super) struct Decoded<'a> {
    pub appearance: Appearance<'a>,
    pub slots: Vec<MaterialSlotId>,
}
fn rows<T, U: Target>(document: &RawDocument<'_>, reference: Ref<U>) -> Result<Vec<T>, Error>
where
    T: Record + RecordLayout + for<'record> wem::Read<'record>,
{
    document
        .resolve(reference.cast::<T>())
        .map_err(|error| Error::new("wem.appearance", error.to_string()))?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| Error::new("wem.appearance", error.to_string()))
}
fn put<T: Record + wem::Write, U: Target>(
    document: &mut RawDocument<'_>,
    values: &[T],
) -> Result<Ref<U>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len())
        .map_err(|_| Error::new("wem.appearance", "record count exceeds u32"))?;
    let section = document
        .append(values)
        .map_err(|error| Error::new("wem.appearance", error.to_string()))?;
    Ok(Ref::from_section(count, section.index()))
}

pub(super) fn read<'view>(
    document: &'view RawDocument<'_>,
    slots: &InlineList<Ref<records::targets::Character>>,
    sets: Ref<records::targets::ProfileMaterialSet>,
    textures: &[TextureId],
    texture_resolver: &mut dyn FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
    builder: &mut ModelBuilder<'view, Wem>,
    hints: &mut Vec<EncodingHint<'view>>,
) -> Result<Decoded<'view>, Error> {
    let mut appearance = Appearance::new();
    let slots = slots
        .0
        .iter()
        .map(|reference| text::read(document, *reference).map(|name| appearance.add_slot(name)))
        .collect::<Result<Vec<_>, _>>()?;
    for source in rows::<records::ProfileMaterialSet, _>(document, sets)? {
        let mut set = AppearanceSet::new(document::read_family(source.profile));
        set.metadata = metadata::read(document, &source.native)?;
        for source in rows::<records::MaterialRecord, _>(document, source.materials)? {
            let (value, revision) =
                material_records::read(document, &source, textures, texture_resolver)?;
            let material = builder.add_material(value)?;
            if let Some((source, revision)) = revision {
                hints.push(EncodingHint::NativeShaderRevision {
                    material,
                    source,
                    revision,
                });
            }
            set.materials.push(material);
        }
        let looks = rows::<records::Look, _>(document, source.looks.looks)?
            .into_iter()
            .map(|look| {
                let mut value = AppearanceLook::named(text::read(document, look.name)?);
                value.weight = look.weight;
                Ok(set.add_look(value))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        if source.default_look != u32::MAX {
            set.set_default_look(Some(*looks.get(source.default_look as usize).ok_or_else(
                || Error::new("wem.appearance.default", "default alternative is absent"),
            )?))?;
        }
        let materials = set.materials.clone();
        let id = appearance.add_set(set)?;
        let bindings = rows::<records::SlotBinding, _>(document, source.slot_bindings)?;
        if bindings.len() != slots.len() {
            return Err(Error::new(
                "wem.appearance.bindings",
                "slot binding count differs from the owning slot vocabulary",
            ));
        }
        for (slot, source) in slots.iter().zip(bindings) {
            let indices = rows::<records::U32, _>(document, source.by_look)?;
            if indices.len() != looks.len() {
                return Err(Error::new(
                    "wem.appearance.bindings",
                    "alternative binding count differs from the owning look vocabulary",
                ));
            }
            for (look, index) in looks.iter().zip(indices) {
                let material = if index.value == u32::MAX {
                    None
                } else {
                    Some(*materials.get(index.value as usize).ok_or_else(|| {
                        Error::new(
                            "wem.appearance.material",
                            "binding selects an absent material",
                        )
                    })?)
                };
                appearance.bind(id, *look, *slot, material)?;
            }
        }
    }
    Ok(Decoded { appearance, slots })
}

pub(super) struct Encoded {
    pub slots: InlineList<Ref<records::targets::Character>>,
    pub sets: Ref<records::targets::ProfileMaterialSet>,
    pub slot_indices: BTreeMap<MaterialSlotId, u32>,
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    model: &Model<'_, Wem>,
    textures: &BTreeMap<TextureId, u32>,
    hints: &[EncodingHint<'_>],
    filename_resolver: &dyn Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<Encoded, Error> {
    let appearance = model.appearance();
    let mut names = Vec::new();
    let mut slot_indices = BTreeMap::new();
    for (id, name) in appearance.slots() {
        slot_indices.insert(
            id,
            u32::try_from(names.len())
                .map_err(|_| Error::new("wem.appearance", "slot count exceeds u32"))?,
        );
        names.push(text::write_bytes(document, name.as_bytes())?);
    }
    let mut sets = Vec::new();
    for (_, set) in appearance.sets() {
        let mut material_indices = BTreeMap::new();
        let mut materials = Vec::new();
        for id in &set.materials {
            let material = model
                .material(*id)
                .ok_or_else(|| Error::new("wem.appearance", "material belongs to another model"))?;
            let revision = hints
                .iter()
                .find_map(|hint| match hint {
                    EncodingHint::NativeShaderRevision {
                        material, revision, ..
                    } if material == id => Some(*revision),
                    _ => None,
                })
                .unwrap_or(0);
            material_indices.insert(
                *id,
                u32::try_from(materials.len())
                    .map_err(|_| Error::new("wem.appearance", "material count exceeds u32"))?,
            );
            materials.push(material_records::write(
                document,
                material,
                textures,
                revision,
                filename_resolver,
            )?);
        }
        let mut looks = Vec::new();
        let mut look_ids = Vec::new();
        for (id, look) in set.looks() {
            look_ids.push(id);
            looks.push(records::Look {
                name: text::write_bytes(document, look.name.as_bytes())?,
                weight: look.weight,
            });
        }
        let default_look = match set.default_look() {
            None => u32::MAX,
            Some(id) => {
                u32::try_from(look_ids.iter().position(|key| *key == id).ok_or_else(|| {
                    Error::new("wem.appearance.default", "default alternative is absent")
                })?)
                .map_err(|_| Error::new("wem.appearance", "look index exceeds u32"))?
            }
        };
        let mut bindings = Vec::new();
        for (slot, _) in appearance.slots() {
            let mut indices = Vec::new();
            for (_, look) in set.looks() {
                let material = look
                    .bindings()
                    .find(|(key, _)| *key == slot)
                    .and_then(|(_, value)| value);
                let value = match material {
                    None => u32::MAX,
                    Some(id) => *material_indices.get(&id).ok_or_else(|| {
                        Error::new(
                            "wem.appearance.binding",
                            "selected material is absent from this family set",
                        )
                    })?,
                };
                indices.push(records::U32 { value });
            }
            bindings.push(records::SlotBinding {
                by_look: put(document, &indices)?,
            });
        }
        sets.push(records::ProfileMaterialSet {
            profile: document::write_family(set.family),
            looks: records::LookTable {
                looks: put(document, &looks)?,
            },
            default_look,
            materials: put(document, &materials)?,
            slot_bindings: put(document, &bindings)?,
            native: metadata::write(document, &set.metadata)?,
        });
    }
    Ok(Encoded {
        slots: InlineList(names),
        sets: put(document, &sets)?,
        slot_indices,
    })
}
