//! Document declarations and selection programs resolve references against one collection.
use super::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{
    AnimationSet, AnimationSetId, CoordinateConvention, DocumentControls, Family,
};
use crate::model::semantic::{Bounds, ClipId};
use crate::model::wem::{self, records, InlineList, RawDocument, Record, Ref};
use std::collections::BTreeMap;

pub(super) fn read_family(value: records::ProfileId) -> Family {
    match value {
        records::ProfileId::Generic => Family::Common,
        records::ProfileId::Wc3Classic => Family::WarcraftClassic,
        records::ProfileId::Wc3Reforged => Family::WarcraftReforged,
        records::ProfileId::Wc3Definitive => Family::WarcraftDefinitive,
        records::ProfileId::Wow => Family::WarcraftOnline,
        records::ProfileId::Sc2 => Family::StarCraft,
        records::ProfileId::Heroes => Family::Heroes,
        records::ProfileId::Diablo3 => Family::Diablo,
        records::ProfileId::Count => Family::Unknown(8),
        records::ProfileId::Unknown(value) => Family::Unknown(value),
    }
}
pub(super) fn write_family(value: Family) -> records::ProfileId {
    match value {
        Family::Common => records::ProfileId::Generic,
        Family::WarcraftClassic => records::ProfileId::Wc3Classic,
        Family::WarcraftReforged => records::ProfileId::Wc3Reforged,
        Family::WarcraftDefinitive => records::ProfileId::Wc3Definitive,
        Family::WarcraftOnline => records::ProfileId::Wow,
        Family::StarCraft => records::ProfileId::Sc2,
        Family::Heroes => records::ProfileId::Heroes,
        Family::Diablo => records::ProfileId::Diablo3,
        Family::Unknown(value) => records::ProfileId::Unknown(value),
    }
}
fn error(value: impl std::fmt::Display) -> Error {
    Error::new("wem.document", value.to_string())
}
fn append<T: Record + wem::Write>(
    document: &mut RawDocument<'_>,
    values: &[T],
) -> Result<Ref<T>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len()).map_err(error)?;
    let section = document.append(values).map_err(error)?;
    Ok(Ref::from_section(count, section.index()))
}
pub(super) fn read<'a>(
    document: &'a RawDocument<'_>,
    root: &records::DocumentRecord<records::V1>,
    clips: &[ClipId],
) -> Result<DocumentControls<'a>, Error> {
    let mut result = DocumentControls::default();
    result.name = text::read(document, root.name)?;
    result.families = root.profiles.0.iter().copied().map(read_family).collect();
    result.default_family = read_family(root.default_profile);
    result.convention = match root.space {
        records::CoordSpace::Blizzard => CoordinateConvention::Blizzard,
        records::CoordSpace::Sc2 => CoordinateConvention::StarCraft,
        records::CoordSpace::Unknown(value) => CoordinateConvention::Unknown(value),
    };
    result.unit_multiplier = root.unit_scale;
    result.bounds = Bounds {
        minimum: root.bounds.minimum,
        maximum: root.bounds.maximum,
        sphere_radius: root.bounds.sphere_radius,
    };
    let rows = document
        .resolve::<records::AnimSet>(root.anim_sets.cast())
        .map_err(error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(error)?;
    let ids: Vec<_> = rows
        .iter()
        .map(|_| result.reserve_animation_set())
        .collect();
    for (id, row) in ids.iter().zip(rows) {
        let tags = document
            .resolve::<records::AnimTag>(row.by_tag.cast())
            .map_err(error)?
            .iter()
            .map(|tag| {
                let tag = tag.map_err(error)?;
                let clip = clips
                    .get(tag.clip as usize)
                    .copied()
                    .ok_or_else(|| error("animation tag selects an absent clip"))?;
                Ok((tag.tag_id, clip))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let fallback = if row.base_anim_set == u32::MAX {
            None
        } else {
            Some(
                *ids.get(row.base_anim_set as usize)
                    .ok_or_else(|| error("fallback selects an absent animation set"))?,
            )
        };
        result
            .set_animation_set(
                *id,
                AnimationSet {
                    name: text::read(document, row.name)?,
                    tags,
                    fallback,
                },
            )
            .map_err(error)?;
    }
    result.validate().map_err(error)?;
    Ok(result)
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    controls: &DocumentControls<'_>,
    clips: &BTreeMap<ClipId, u32>,
) -> Result<
    (
        records::DocumentRecord<records::V1>,
        BTreeMap<AnimationSetId, u32>,
    ),
    Error,
> {
    controls.validate().map_err(error)?;
    let ids = controls
        .animation_sets()
        .enumerate()
        .map(|(index, (id, _))| u32::try_from(index).map(|index| (id, index)).map_err(error))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut sets = Vec::new();
    for (_, set) in controls.animation_sets() {
        let tags = set
            .tags
            .iter()
            .map(|(tag, clip)| {
                Ok(records::AnimTag {
                    tag_id: *tag,
                    clip: *clips
                        .get(clip)
                        .ok_or_else(|| error("animation tag has no destination clip"))?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        sets.push(records::AnimSet {
            name: text::write_bytes(document, set.name.as_bytes())?,
            by_tag: append(document, &tags)?.cast(),
            base_anim_set: match set.fallback {
                None => u32::MAX,
                Some(id) => *ids
                    .get(&id)
                    .ok_or_else(|| error("fallback belongs to another document"))?,
            },
        });
    }
    Ok((
        records::DocumentRecord {
            profiles: InlineList(
                controls
                    .families
                    .iter()
                    .copied()
                    .map(write_family)
                    .collect(),
            ),
            default_profile: write_family(controls.default_family),
            space: match controls.convention {
                CoordinateConvention::Blizzard => records::CoordSpace::Blizzard,
                CoordinateConvention::StarCraft => records::CoordSpace::Sc2,
                CoordinateConvention::Unknown(value) => records::CoordSpace::Unknown(value),
            },
            unit_scale: controls.unit_multiplier,
            name: text::write_bytes(document, controls.name.as_bytes())?,
            bounds: records::Extent {
                minimum: controls.bounds.minimum,
                maximum: controls.bounds.maximum,
                sphere_radius: controls.bounds.sphere_radius,
            },
            anim_sets: append(document, &sets)?.cast(),
            models: Ref::default(),
            textures: Ref::default(),
            clips: Ref::default(),
        },
        ids,
    ))
}

#[cfg(test)]
mod tests {
    use super::{read, write};
    use crate::model::semantic::profiles::wem::{
        AnimationSet, CoordinateConvention, DocumentControls, Family,
    };
    use crate::model::semantic::Scope;
    use crate::model::wem::{records, RawDocument};
    use std::borrow::Cow;
    use std::collections::BTreeMap;

    #[test]
    fn collection_selection_programs_relocate_clips_and_borrow_names() {
        let clip = Scope::default().allocate();
        let mut controls = DocumentControls::default();
        controls.name = Cow::Borrowed("collection");
        controls.families = vec![Family::StarCraft, Family::Unknown(123)];
        controls.default_family = Family::StarCraft;
        controls.convention = CoordinateConvention::Unknown(29);
        let primary = controls.reserve_animation_set();
        let fallback = controls.reserve_animation_set();
        controls
            .set_animation_set(
                primary,
                AnimationSet {
                    name: Cow::Borrowed("combat"),
                    tags: vec![(17, clip)],
                    fallback: Some(fallback),
                },
            )
            .unwrap();
        controls
            .set_animation_set(
                fallback,
                AnimationSet {
                    name: Cow::Borrowed("idle"),
                    tags: vec![],
                    fallback: None,
                },
            )
            .unwrap();
        let mut output = RawDocument::new();
        let (root, sets) = write(&mut output, &controls, &BTreeMap::from([(clip, 0)])).unwrap();
        assert_eq!(sets[&fallback], 1);
        let result = read(&output, &root, &[clip]).unwrap();
        assert!(matches!(result.name, Cow::Borrowed("collection")));
        assert_eq!(result.convention, controls.convention);
        assert_eq!(result.families, controls.families);
        let sets: Vec<_> = result.animation_sets().collect();
        assert_eq!(sets[0].1.tags, vec![(17, clip)]);
        assert_eq!(sets[0].1.fallback, Some(sets[1].0));
        assert!(read(&output, &root, &[]).is_err());
        let rows = output
            .resolve::<records::AnimSet>(root.anim_sets.cast())
            .unwrap();
        assert_eq!(rows.get(0).unwrap().unwrap().base_anim_set, 1);
    }

    #[test]
    fn fallback_cycles_are_rejected_before_publication_or_encoding() {
        let mut controls = DocumentControls::default();
        let id = controls.reserve_animation_set();
        controls
            .set_animation_set(
                id,
                AnimationSet {
                    name: Cow::Borrowed("cycle"),
                    tags: vec![],
                    fallback: Some(id),
                },
            )
            .unwrap();
        assert!(controls.validate().is_err());
        assert!(write(&mut RawDocument::new(), &controls, &BTreeMap::new()).is_err());
    }
}
